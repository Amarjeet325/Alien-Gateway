use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

pub mod auth;

// Storage Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Username,
    Owner,
    CreatedAt,
}

// Event
const INIT_EVENT: Symbol = symbol_short!("INIT");

pub struct CoreContract;

impl CoreContract {
    /// Initialize the contract. Owner is the address that will have write access (auth).
    pub fn init(env: Env, username: Symbol, owner: Address) {
        // Prevent re-init
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("Contract already initialized");
        }

        // Validate username: must not be empty (symbol_short!("") is the empty symbol)
        if username == symbol_short!("") {
            panic!("Username cannot be empty");
        }

        // Store values
        env.storage().instance().set(&DataKey::Username, &username);
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&DataKey::CreatedAt, &env.ledger().timestamp());

        // Emit event
        #[allow(deprecated)]
        env.events().publish((INIT_EVENT,), (username, owner));
    }

    // Getters
    pub fn get_username(env: Env) -> Symbol {
        env.storage().instance().get(&DataKey::Username).unwrap()
    }

    pub fn get_owner(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Owner).unwrap()
    }

    pub fn get_created_at(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::CreatedAt).unwrap()
    }

    /// Transfer ownership to a new address. Caller must be current owner.
    pub fn transfer_ownership(env: Env, new_owner: Address) {
        auth::require_owner(&env);
        env.storage().instance().set(&DataKey::Owner, &new_owner);
    }
}
