#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Bytes, Env, String};

#[test]
fn test_full_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MedicalRecordContract, ());
    let client = MedicalRecordContractClient::new(&env, &contract_id);

    let patient = Address::generate(&env);
    let doctor = Address::generate(&env);
    client.init(&patient);

    client.authorize_doctor(&patient.clone(), &doctor);

    let data_hash = Bytes::from_slice(&env, &[1u8; 32]);
    client.append_record(
        &patient,
        &doctor,
        &data_hash,
        &String::from_str(&env, "placeholder_type"),
        &String::from_str(&env, "placeholder_note"),
    );

    let records = client.get_records(&patient);
    assert_eq!(records.len(), 1);
    assert_eq!(
        records.first().unwrap().record_type,
        String::from_str(&env, "placeholder_type")
    );
    assert_eq!(
        records.first().unwrap().notes,
        String::from_str(&env, "placeholder_note")
    );

    let doctors = client.get_authorized_doctors(&patient);
    assert_eq!(doctors.len(), 1);
    assert_eq!(doctors.first().unwrap(), doctor);

    let is_auth = client.is_doctor_authorized(&patient, &doctor);
    assert!(is_auth);
}
