#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};

use errors::VaultError;
use types::Position;

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn initialize(env: Env, admin: Address, _oracle: Address) {
        admin.require_auth();
        if storage::get_admin(&env).is_some() {
            panic!("already initialized");
        }
        storage::set_admin(&env, &admin);
        storage::set_paused(&env, false);
    }

    pub fn set_paused(env: Env, paused: bool) {
        let admin = storage::get_admin(&env).expect("not initialized");
        admin.require_auth();
        storage::set_paused(&env, paused);
    }

    pub fn add_supported_asset(env: Env, asset: Address) {
        let admin = storage::get_admin(&env).expect("not initialized");
        admin.require_auth();

        if storage::is_supported_asset(&env, &asset) {
            soroban_sdk::panic_with_error!(&env, VaultError::AlreadySupported);
        }

        storage::add_supported_asset(&env, &asset);

        events::AssetAdded { asset }.publish(&env);
    }

    pub fn is_supported_asset(env: Env, asset: Address) -> bool {
        storage::is_supported_asset(&env, &asset)
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        storage::get_admin(&env)
    }

    pub fn get_position_balance(env: Env, user: Address, asset: Address) -> i128 {
        storage::get_position_balance(&env, &user, &asset)
    }

    pub fn get_position_index(env: Env) -> soroban_sdk::Vec<Address> {
        storage::get_position_index(&env)
    }

    pub fn deposit(env: Env, user: Address, asset: Address, amount: i128) {
        user.require_auth();

        if amount <= 0 {
            soroban_sdk::panic_with_error!(&env, VaultError::InvalidInputs);
        }

        if storage::is_paused(&env) {
            soroban_sdk::panic_with_error!(&env, VaultError::VaultPaused);
        }

        if !storage::is_supported_asset(&env, &asset) {
            soroban_sdk::panic_with_error!(&env, VaultError::UnsupportedAsset);
        }

        let token_client = token::Client::new(&env, &asset);
        token_client.transfer(&user, env.current_contract_address(), &amount);

        let balance = storage::get_position_balance(&env, &user, &asset);
        let new_balance = balance + amount;
        storage::set_position_balance(&env, &user, &asset, new_balance);

        // Track this asset for the user (used to build Position)
        storage::add_user_asset(&env, &user, &asset);
        // Add user to the global position index if not already present
        storage::add_to_position_index(&env, &user);

        events::Deposited {
            user,
            asset,
            amount,
        }
        .publish(&env);
    }

    pub fn withdraw(env: Env, receiver: Address, asset: Address, amount: i128) {
        receiver.require_auth();

        if amount <= 0 {
            soroban_sdk::panic_with_error!(&env, VaultError::InvalidInputs);
        }

        if storage::is_paused(&env) {
            soroban_sdk::panic_with_error!(&env, VaultError::VaultPaused);
        }

        let balance = storage::get_position_balance(&env, &receiver, &asset);
        if amount > balance {
            soroban_sdk::panic_with_error!(&env, VaultError::InvalidInputs);
        }

        let new_balance = balance - amount;
        storage::set_position_balance(&env, &receiver, &asset, new_balance);

        // If the user has no remaining balance across any asset, remove from index
        let position = storage::get_position(&env, &receiver);
        if position.assets.is_empty() {
            storage::remove_from_position_index(&env, &receiver);
        }

        let token_client = token::Client::new(&env, &asset);
        token_client.transfer(&env.current_contract_address(), &receiver, &amount);
    }

    pub fn get_all_positions(env: Env) -> Vec<Position> {
        storage::get_all_positions(&env)
    }

    pub fn seize_collateral(_env: Env, _user: Address, _asset: Address, _amount: i128) {}

    pub fn is_withdrawal_safe(_env: &Env, _user: Address, _amount: i128) {}

    pub fn get_position(_env: &Env, _user: Address) {}

    pub fn get_collateral_value(_env: &Env, _user: Address) {}
}

mod errors;
mod events;
mod storage;
mod test;
mod types;
