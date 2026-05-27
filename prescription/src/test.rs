#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::BytesN as _, Env};

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
