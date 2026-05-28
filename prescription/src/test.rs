#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, Env,
};

#[test]
fn marker_smoke_test() {
    let env = Env::default();
    let contract_id = env.register(PrescriptionContract, ());
    let client = PrescriptionContractClient::new(&env, &contract_id);
    let marker_id = BytesN::random(&env);

    assert!(!client.has_marker(&marker_id));
    assert_eq!(client.mark(&marker_id), MarkerStatus::Recorded);
    assert!(client.has_marker(&marker_id));
}

#[test]
fn initialize_records_prescription_admin_once() {
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
fn configure_dependencies_is_admin_only() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client, admin) = setup_client(&env);
    let non_admin = Address::generate(&env);
    let identity = Address::generate(&env);
    let broker = Address::generate(&env);
    let supplychain = Address::generate(&env);

    client.initialize(&admin);
    client.configure_dependencies(&admin, &identity, &broker, &supplychain);

    assert_eq!(client.identity_contract_id(), identity);
    assert_eq!(client.access_broker_contract_id(), broker);
    assert_eq!(client.supplychain_contract_id(), supplychain);
    assert_eq!(
        client.try_configure_dependencies(
            &non_admin,
            &Address::generate(&env),
            &Address::generate(&env),
            &Address::generate(&env),
        ),
        Err(Ok(ContractError::Unauthorized))
    );
}

#[test]
fn configure_dependencies_requires_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client, admin) = setup_client(&env);

    assert_eq!(
        client.try_configure_dependencies(
            &admin,
            &Address::generate(&env),
            &Address::generate(&env),
            &Address::generate(&env),
        ),
        Err(Ok(ContractError::NotInitialized))
    );
}

fn setup_client(env: &Env) -> (Address, PrescriptionContractClient<'_>, Address) {
    let contract_id = env.register(PrescriptionContract, ());
    let client = PrescriptionContractClient::new(env, &contract_id);
    let admin = Address::generate(env);

    (contract_id, client, admin)
}
