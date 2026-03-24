//! The Escrow contract handles scheduled payments between vaults.
//! This implementation focuses on security, identity commitment, and host-level authentication.

#![no_std]

pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use crate::errors::EscrowError;
use crate::events::Events;
use crate::storage::{
    increment_payment_id, read_registration_contract, read_vault_config, read_vault_state,
    write_registration_contract, write_scheduled_payment, write_vault_config, write_vault_state,
};
use crate::types::{DataKey, ScheduledPayment, VaultConfig, VaultState};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, vec, Address, BytesN, Env, IntoVal, Symbol,
};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initializes the contract by storing the Registration contract address.
    ///
    /// ### Arguments
    /// - `registration_contract`: The address of the deployed Registration contract.
    ///
    /// ### Errors
    /// - `AlreadyInitialized`: If the Registration contract address is already set.
    pub fn initialize(env: Env, registration_contract: Address) {
        if read_registration_contract(&env).is_some() {
            panic_with_error!(&env, EscrowError::AlreadyInitialized);
        }
        write_registration_contract(&env, &registration_contract);
    }

    /// Creates a new vault for a registered commitment.
    ///
    /// The owner is resolved by calling `get_owner` on the Registration contract.
    /// The caller must be the registered owner of the commitment.
    ///
    /// ### Arguments
    /// - `commitment`: The BytesN<32> identity commitment (Poseidon hash of username).
    /// - `token`: The Stellar asset address this vault will hold.
    ///
    /// ### Errors
    /// - `CommitmentNotRegistered`: If no owner is found for the commitment.
    /// - `VaultAlreadyExists`: If a vault already exists for this commitment.
    pub fn create_vault(env: Env, commitment: BytesN<32>, token: Address) {
        // 1. Load Registration contract address (must be initialized first).
        let registration = read_registration_contract(&env)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::CommitmentNotRegistered));

        // 2. Cross-contract call: resolve owner from Registration contract.
        let owner: Option<Address> = env.invoke_contract(
            &registration,
            &Symbol::new(&env, "get_owner"),
            vec![&env, commitment.clone().into_val(&env)],
        );
        let owner =
            owner.unwrap_or_else(|| panic_with_error!(&env, EscrowError::CommitmentNotRegistered));

        // 3. Authenticate: the resolved owner must sign this transaction.
        owner.require_auth();

        // 4. Existence guard: reject if a vault already exists for this commitment.
        if read_vault_config(&env, &commitment).is_some() {
            panic_with_error!(&env, EscrowError::VaultAlreadyExists);
        }

        // 5. Store immutable vault configuration.
        write_vault_config(
            &env,
            &commitment,
            &VaultConfig {
                owner: owner.clone(),
                token: token.clone(),
                created_at: env.ledger().timestamp(),
            },
        );

        // 6. Store initial mutable vault state.
        write_vault_state(
            &env,
            &commitment,
            &VaultState {
                balance: 0,
                is_active: true,
            },
        );

        // 7. Emit VAULT_CRT event with fields in order: (commitment, token, owner).
        Events::vault_crt(&env, commitment, token, owner);
    }

    /// Schedules a payment from one vault to another.
    ///
    /// Funds are reserved in the source vault immediately upon scheduling.
    /// The payment can be executed at or after the `release_at` timestamp.
    ///
    /// ### Arguments
    /// - `from`: The commitment ID of the source vault.
    /// - `to`: The commitment ID of the destination vault.
    /// - `amount`: The amount of tokens to schedule. Must be > 0.
    /// - `release_at`: The ledger timestamp (u64) for release. Must be > current time.
    ///
    /// ### Returns
    /// - `u32`: The unique payment ID assigned to this schedule.
    ///
    /// ### Errors
    /// - `VaultNotFound`: If the `from` vault does not exist.
    /// - `InvalidAmount`: If `amount <= 0`.
    /// - `InsufficientBalance`: If the vault has less than `amount`.
    /// - `PastReleaseTime`: If `release_at` is not in the future.
    /// - `PaymentCounterOverflow`: If the global ID counter overflows.
    pub fn schedule_payment(
        env: Env,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        release_at: u64,
    ) -> Result<u32, EscrowError> {
        // 1. Validate Input
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        if release_at <= env.ledger().timestamp() {
            return Err(EscrowError::PastReleaseTime);
        }

        // 2. Read Vault (config + state separately)
        let config = read_vault_config(&env, &from).ok_or(EscrowError::VaultNotFound)?;
        let mut state = read_vault_state(&env, &from).ok_or(EscrowError::VaultNotFound)?;

        // 3. Authenticate caller as owner of from vault
        // Host-level authentication. Panics with host error if unauthorized.
        config.owner.require_auth();

        // 4. Reject if vault is inactive
        if !state.is_active {
            return Err(EscrowError::VaultInactive);
        }

        // 5. Validate Balance
        if state.balance < amount {
            return Err(EscrowError::InsufficientBalance);
        }

        // 6. Reserve Funds
        state.balance -= amount;
        write_vault_state(&env, &from, &state);

        // 7. Generate Payment ID
        let payment_id = increment_payment_id(&env)?;

        // 8. Store Scheduled Payment
        let payment = ScheduledPayment {
            from,
            to,
            token: config.token.clone(),
            amount,
            release_at,
            executed: false,
        };
        write_scheduled_payment(&env, payment_id, &payment);

        // 9. Emit Event
        Events::schedule_pay(
            &env,
            payment_id,
            payment.from,
            payment.to,
            payment.amount,
            payment.release_at,
        );

        Ok(payment_id)
    }

    pub fn execute_scheduled(env: Env, payment_id: u32) {
        let key = DataKey::ScheduledPayment(payment_id);
        let mut payment: ScheduledPayment = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, EscrowError::PaymentNotFound));

        if payment.executed {
            panic_with_error!(&env, EscrowError::PaymentAlreadyExecuted);
        }

        if env.ledger().timestamp() < payment.release_at {
            panic_with_error!(&env, EscrowError::PaymentNotYetDue);
        }

        let recipient = resolve(&env, &payment.to);
        let token_client = token::Client::new(&env, &payment.token);
        token_client.transfer(&env.current_contract_address(), &recipient, &payment.amount);

        payment.executed = true;
        write_scheduled_payment(&env, payment_id, &payment);

        Events::pay_exec(&env, payment_id, payment.from, payment.to, payment.amount);
    }
}

fn resolve(env: &Env, commitment: &BytesN<32>) -> Address {
    let config = read_vault_config(env, commitment)
        .unwrap_or_else(|| panic_with_error!(env, EscrowError::VaultNotFound));
    config.owner
}
