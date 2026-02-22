use alien_gateway::{AddressManager, Contract};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_master_assignment() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::register_address(env.clone(), user.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::set_master_stellar_address(env.clone(), user.clone());
    });

    let master = env
        .as_contract(&contract_id, || AddressManager::get_master(env.clone()))
        .unwrap();
    assert_eq!(master, user);
}

#[test]
fn test_switch_master() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::register_address(env.clone(), user1.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::register_address(env.clone(), user2.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::set_master_stellar_address(env.clone(), user1.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::set_master_stellar_address(env.clone(), user2.clone());
    });

    let master = env
        .as_contract(&contract_id, || AddressManager::get_master(env.clone()))
        .unwrap();
    assert_eq!(master, user2);
}

#[test]
#[should_panic(expected = "Address does not exist")]
fn test_non_existent_address_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::set_master_stellar_address(env.clone(), user.clone());
    });
}

#[test]
#[should_panic]
fn test_unauthorized_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });

    env.as_contract(&contract_id, || {
        owner.require_auth();
        AddressManager::register_address(env.clone(), user.clone());
    });

    env.as_contract(&contract_id, || {
        attacker.require_auth();
        AddressManager::set_master_stellar_address(env.clone(), user.clone());
    });
}
