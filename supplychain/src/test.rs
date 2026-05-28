#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env,
};

#[test]
fn marker_smoke_test() {
    let env = Env::default();
    let contract_id = env.register(SupplychainContract, ());
    let client = SupplychainContractClient::new(&env, &contract_id);
    let marker_id = BytesN::random(&env);

    assert!(!client.has_marker(&marker_id));
    assert_eq!(client.mark(&marker_id), MarkerStatus::Recorded);
    assert!(client.has_marker(&marker_id));
}

#[test]
fn initialize_records_supplychain_admin_once() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client, admin) = setup_client(&env);
    let next_admin = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(client.admin(), admin);
    assert_eq!(
        client.try_initialize(&next_admin),
        Err(Ok(ContractError::AlreadyInitialized))
    );
}

#[test]
fn register_oracle_is_admin_only() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client, admin) = setup_client(&env);
    let non_admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let other_oracle = Address::generate(&env);

    client.initialize(&admin);
    client.register_oracle(&admin, &oracle);

    assert!(client.is_oracle(&oracle));
    assert_eq!(
        client.try_register_oracle(&non_admin, &other_oracle),
        Err(Ok(ContractError::Unauthorized))
    );
    assert!(!client.is_oracle(&other_oracle));
}

#[test]
fn register_attester_is_admin_only() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client, admin) = setup_client(&env);
    let non_admin = Address::generate(&env);
    let attester = BytesN::random(&env);
    let other_attester = BytesN::random(&env);

    client.initialize(&admin);
    client.register_attester(&admin, &attester);

    assert!(client.is_attester(&attester));
    assert_eq!(
        client.try_register_attester(&non_admin, &other_attester),
        Err(Ok(ContractError::Unauthorized))
    );
    assert!(!client.is_attester(&other_attester));
}

#[test]
fn register_supplychain_authorities_requires_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client, admin) = setup_client(&env);

    assert_eq!(
        client.try_register_oracle(&admin, &Address::generate(&env)),
        Err(Ok(ContractError::NotInitialized))
    );
    assert_eq!(
        client.try_register_attester(&admin, &BytesN::random(&env)),
        Err(Ok(ContractError::NotInitialized))
    );
}

fn setup_client(env: &Env) -> (Address, SupplychainContractClient<'_>, Address) {
    let contract_id = env.register(SupplychainContract, ());
    let client = SupplychainContractClient::new(env, &contract_id);
    let admin = Address::generate(env);

    (contract_id, client, admin)
}
