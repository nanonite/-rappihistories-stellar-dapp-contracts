#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env,
};

#[test]
fn identity_schema_version_is_exposed() {
    let env = Env::default();
    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    assert_eq!(client.schema_version(), 1);
}

#[test]
fn role_codes_are_stable_for_events() {
    assert_eq!(Role::Patient.code(), 1);
    assert_eq!(Role::Clinician.code(), 2);
    assert_eq!(Role::Institution.code(), 3);
    assert_eq!(Role::Pharmacy.code(), 4);
    assert_eq!(Role::Distributor.code(), 5);
    assert_eq!(Role::Manufacturer.code(), 6);
    assert_eq!(Role::Responder.code(), 7);
    assert_eq!(Role::Admin.code(), 8);
}

#[test]
fn storage_uses_instance_admin_and_persistent_identity_records() {
    let env = Env::default();
    let contract_id = env.register(IdentityContract, ());

    env.as_contract(&contract_id, || {
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let subject = Address::generate(&env);
        let cred_id = BytesN::random(&env);

        storage::set_admin(&env, &admin);
        assert_eq!(storage::get_admin(&env), Some(admin.clone()));

        let issuer_record = IssuerRecord {
            issuer: issuer.clone(),
            registered_at: env.ledger().timestamp(),
            active: true,
        };
        storage::set_issuer(&env, &issuer, &issuer_record);
        assert!(storage::has_issuer(&env, &issuer));
        assert_eq!(storage::get_issuer(&env, &issuer), Some(issuer_record));

        let credential = CredentialRef {
            subject: subject.clone(),
            role: Role::Clinician,
            issuer,
            expires_at: env.ledger().timestamp() + 86_400,
            status: CredentialStatus::Active,
        };
        storage::set_credential(&env, &cred_id, &credential);
        assert!(storage::has_credential(&env, &cred_id));
        assert_eq!(storage::get_credential(&env, &cred_id), Some(credential));

        storage::add_subject_credential(&env, &subject, &cred_id);
        let subject_credentials = storage::get_subject_creds(&env, &subject);
        assert_eq!(subject_credentials.len(), 1);
        assert_eq!(subject_credentials.first(), Some(cred_id));
    });
}
