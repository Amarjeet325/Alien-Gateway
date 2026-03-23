use soroban_sdk::{contracttype, symbol_short, BytesN, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedPayEvent {
    pub payment_id: u32,
    pub from: BytesN<32>,
    pub to: BytesN<32>,
    pub amount: i128,
    pub release_at: u64,
}

pub struct EscrowEvents;

impl EscrowEvents {
    pub fn emit_sched_pay(
        env: &Env,
        payment_id: u32,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        release_at: u64,
    ) {
        let topics = (symbol_short!("SCHED_PAY"), payment_id);
        #[allow(deprecated)]
        env.events().publish(
            topics,
            SchedPayEvent {
                payment_id,
                from,
                to,
                amount,
                release_at,
            },
        );
    }
}
