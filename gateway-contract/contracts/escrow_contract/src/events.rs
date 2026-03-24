use soroban_sdk::{contractevent, Address, BytesN, Env};

/// Event emitted when a new payment is scheduled.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulePayEvent {
    /// The unique identifier assigned to this payment.
    #[topic]
    pub payment_id: u32,
    /// The commitment identifier of the source vault.
    pub from: BytesN<32>,
    /// The commitment identifier of the intended recipient.
    pub to: BytesN<32>,
    /// The amount of tokens to be transferred.
    pub amount: i128,
    /// The timestamp at or after which the payment can be executed.
    pub release_at: u64,
}

/// Event emitted when a scheduled payment is executed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayExecEvent {
    /// The unique identifier assigned to this payment.
    #[topic]
    pub payment_id: u32,
    /// The commitment identifier of the source vault.
    pub from: BytesN<32>,
    /// The commitment identifier of the intended recipient.
    pub to: BytesN<32>,
    /// The amount of tokens transferred.
    pub amount: i128,
}

/// Event emitted when a new vault is created.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultCrtEvent {
    /// The commitment identifier of the vault being created.
    #[topic]
    pub commitment: BytesN<32>,
    /// The asset token associated with this vault.
    pub token: Address,
    /// The owner address of this vault.
    pub owner: Address,
}

/// Helper for emitting contract events.
pub struct Events;

impl Events {
    /// Emits a `SchedulePayEvent` to the host.
    pub fn schedule_pay(
        env: &Env,
        payment_id: u32,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        release_at: u64,
    ) {
        SchedulePayEvent {
            payment_id,
            from,
            to,
            amount,
            release_at,
        }
        .publish(env);
    }

    /// Emits a `PayExecEvent` to the host.
    pub fn pay_exec(env: &Env, payment_id: u32, from: BytesN<32>, to: BytesN<32>, amount: i128) {
        PayExecEvent {
            payment_id,
            from,
            to,
            amount,
        }
        .publish(env);
    }

    /// Emits a `VaultCrtEvent` to the host.
    pub fn vault_crt(env: &Env, commitment: BytesN<32>, token: Address, owner: Address) {
        VaultCrtEvent {
            commitment,
            token,
            owner,
        }
        .publish(env);
    }
}
