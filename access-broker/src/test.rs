#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, Bytes, BytesN, Env, Symbol,
};

#[test]
fn broker_schema_version_is_exposed() {
    let env = Env::default();
    let contract_id = env.register(AccessBrokerContract, ());
    let client = AccessBrokerContractClient::new(&env, &contract_id);

    assert_eq!(client.schema_version(), 1);
}

#[test]
fn tier_and_grant_codes_are_stable_for_events() {
    assert_eq!(Tier::OfflineCard.code(), 1);
    assert_eq!(Tier::EmergencyBundle.code(), 2);
    assert_eq!(Tier::FullHistory.code(), 3);

    assert_eq!(GrantType::Normal.code(), 1);
    assert_eq!(GrantType::BreakGlass.code(), 2);
    assert_eq!(GrantType::TokenlessFallback.code(), 3);
}

#[test]
fn error_codes_match_design_catalog() {
    assert_eq!(Error::NoSuchRecord as u32, 1);
    assert_eq!(Error::BadCredential as u32, 2);
    assert_eq!(Error::CredentialNotForCaller as u32, 3);
    assert_eq!(Error::NoGrant as u32, 4);
    assert_eq!(Error::GrantExpired as u32, 5);
    assert_eq!(Error::GrantRevoked as u32, 6);
    assert_eq!(Error::ScopeMismatch as u32, 7);
    assert_eq!(Error::SensitiveNeedsExplicitGrant as u32, 8);
    assert_eq!(Error::OfflineTierNotBrokered as u32, 9);
    assert_eq!(Error::StalePresence as u32, 10);
    assert_eq!(Error::WrongToken as u32, 11);
    assert_eq!(Error::NoTokenRegistered as u32, 12);
    assert_eq!(Error::NonceReplayed as u32, 13);
    assert_eq!(Error::BadPresenceSig as u32, 14);
    assert_eq!(Error::FallbackNeedsDualSign as u32, 15);
}

#[test]
fn storage_uses_explicit_broker_storage_classes() {
    let env = Env::default();
    let contract_id = env.register(AccessBrokerContract, ());

    env.as_contract(&contract_id, || {
        let admin = Address::generate(&env);
        let issuer_root = Address::generate(&env);
        let owner = Address::generate(&env);
        let grantee = Address::generate(&env);
        let record_id = BytesN::random(&env);
        let normal_grant_id = BytesN::random(&env);
        let break_glass_grant_id = BytesN::random(&env);
        let nonce = BytesN::random(&env);
        let token_pubkey = BytesN::random(&env);

        storage::set_admin(&env, &admin);
        storage::set_issuer_root(&env, &issuer_root);
        assert_eq!(storage::get_admin(&env), Some(admin));
        assert_eq!(storage::get_issuer_root(&env), Some(issuer_root));

        let record = RecordMeta {
            owner: owner.clone(),
            tier: Tier::FullHistory,
            category: Symbol::new(&env, "cardiology"),
            sensitive: true,
            commitment: BytesN::random(&env),
            locator: Bytes::from_slice(&env, &[1, 2, 3, 4]),
        };
        storage::set_record(&env, &record_id, &record);
        assert!(storage::has_record(&env, &record_id));
        assert_eq!(storage::get_record(&env, &record_id), Some(record.clone()));

        let normal_grant = grant(
            &env,
            &record_id,
            &grantee,
            GrantType::Normal,
            Symbol::new(&env, "treatment"),
            record.category.clone(),
        );
        storage::set_grant(&env, &normal_grant_id, &normal_grant);
        assert!(storage::has_grant(&env, &normal_grant_id));
        assert_eq!(
            storage::get_grant(&env, &normal_grant_id),
            Some(normal_grant)
        );

        let break_glass_grant = grant(
            &env,
            &record_id,
            &grantee,
            GrantType::BreakGlass,
            Symbol::new(&env, "emergency"),
            record.category,
        );
        storage::set_grant(&env, &break_glass_grant_id, &break_glass_grant);
        assert!(storage::has_grant(&env, &break_glass_grant_id));
        assert_eq!(
            storage::get_grant(&env, &break_glass_grant_id),
            Some(break_glass_grant)
        );

        storage::set_patient_token(&env, &owner, &token_pubkey);
        assert!(storage::has_patient_token(&env, &owner));
        assert_eq!(storage::get_patient_token(&env, &owner), Some(token_pubkey));

        storage::mark_nonce_spent(&env, &nonce);
        assert!(storage::has_spent_nonce(&env, &nonce));
    });
}

#[test]
fn broker_proof_and_capability_types_match_contract_boundary() {
    let env = Env::default();
    let subject = Address::generate(&env);
    let cred_id = BytesN::random(&env);
    let role = Symbol::new(&env, "clinician");

    let credential = CredentialProof {
        cred_id: cred_id.clone(),
        role: role.clone(),
        subject: subject.clone(),
    };
    assert_eq!(credential.cred_id, cred_id);
    assert_eq!(credential.role, role);
    assert_eq!(credential.subject, subject);

    let capability = Capability {
        grant_id: BytesN::random(&env),
        locator: Bytes::from_slice(&env, &[9, 8, 7]),
        commitment: BytesN::random(&env),
    };
    assert_eq!(capability.locator, Bytes::from_slice(&env, &[9, 8, 7]));
}

#[test]
fn register_record_stores_and_returns_record_meta() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, owner) = setup_client(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let locator = Bytes::from_slice(&env, &[4, 3, 2, 1]);
    let commitment = BytesN::random(&env);

    client.register_record(
        &owner,
        &record_id,
        &Tier::FullHistory,
        &category,
        &true,
        &locator,
        &commitment,
    );

    let meta = client.get_record(&record_id);
    assert_eq!(meta.owner, owner);
    assert_eq!(meta.tier, Tier::FullHistory);
    assert_eq!(meta.category, category);
    assert!(meta.sensitive);
    assert_eq!(meta.locator, locator);
    assert_eq!(meta.commitment, commitment);
}

#[test]
fn register_record_rejects_duplicate_record_id() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, owner) = setup_client(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let locator = Bytes::from_slice(&env, &[4, 3, 2, 1]);
    let commitment = BytesN::random(&env);

    client.register_record(
        &owner,
        &record_id,
        &Tier::FullHistory,
        &category,
        &false,
        &locator,
        &commitment,
    );

    assert_eq!(
        client.try_register_record(
            &owner,
            &record_id,
            &Tier::FullHistory,
            &category,
            &false,
            &locator,
            &commitment,
        ),
        Err(Ok(Error::RecordAlreadyExists))
    );
}

#[test]
fn get_record_returns_error_for_missing_record() {
    let env = Env::default();
    let (client, _) = setup_client(&env);
    let record_id = BytesN::random(&env);

    assert_eq!(
        client.try_get_record(&record_id),
        Err(Ok(Error::NoSuchRecord))
    );
}

#[test]
fn register_patient_token_stores_and_returns_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let token_pubkey = BytesN::random(&env);

    client.register_patient_token(&patient, &token_pubkey);

    assert_eq!(client.get_patient_token(&patient), token_pubkey);
}

#[test]
fn register_patient_token_rejects_duplicate_patient_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let token_pubkey = BytesN::random(&env);
    let next_token_pubkey = BytesN::random(&env);

    client.register_patient_token(&patient, &token_pubkey);

    assert_eq!(
        client.try_register_patient_token(&patient, &next_token_pubkey),
        Err(Ok(Error::PatientTokenAlreadyRegistered))
    );
}

#[test]
fn get_patient_token_returns_error_for_missing_token() {
    let env = Env::default();
    let (client, patient) = setup_client(&env);

    assert_eq!(
        client.try_get_patient_token(&patient),
        Err(Ok(Error::NoTokenRegistered))
    );
}

#[test]
fn create_normal_grant_stores_grant_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let grantee = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    let expires_at = env.ledger().timestamp() + 3_600;
    register_full_history_record(&env, &client, &patient, &record_id, &category);

    let grant_id = client.create_normal_grant(
        &patient,
        &grantee,
        &record_id,
        &purpose,
        &category,
        &expires_at,
    );

    let grant = client.get_grant(&grant_id);
    assert_eq!(grant.record, record_id);
    assert_eq!(grant.grantee, grantee);
    assert_eq!(grant.gtype, GrantType::Normal);
    assert_eq!(grant.purpose, purpose);
    assert_eq!(grant.scope_category, category);
    assert_eq!(grant.expires_at, expires_at);
    assert_eq!(grant.reveal_at, 0);
    assert!(!grant.revoked);
    assert!(!grant.vetoed);
}

#[test]
fn create_normal_grant_requires_existing_record() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let grantee = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    let expires_at = env.ledger().timestamp() + 3_600;

    assert_eq!(
        client.try_create_normal_grant(
            &patient,
            &grantee,
            &record_id,
            &purpose,
            &category,
            &expires_at,
        ),
        Err(Ok(Error::NoSuchRecord))
    );
}

#[test]
fn create_normal_grant_requires_record_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let other_patient = Address::generate(&env);
    let grantee = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    let expires_at = env.ledger().timestamp() + 3_600;
    register_full_history_record(&env, &client, &patient, &record_id, &category);

    assert_eq!(
        client.try_create_normal_grant(
            &other_patient,
            &grantee,
            &record_id,
            &purpose,
            &category,
            &expires_at,
        ),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn create_normal_grant_rejects_expired_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let grantee = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    let expires_at = env.ledger().timestamp();
    register_full_history_record(&env, &client, &patient, &record_id, &category);

    assert_eq!(
        client.try_create_normal_grant(
            &patient,
            &grantee,
            &record_id,
            &purpose,
            &category,
            &expires_at,
        ),
        Err(Ok(Error::InvalidExpiration))
    );
}

#[test]
fn revoke_marks_normal_grant_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let grantee = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    let expires_at = env.ledger().timestamp() + 3_600;
    register_full_history_record(&env, &client, &patient, &record_id, &category);
    let grant_id = client.create_normal_grant(
        &patient,
        &grantee,
        &record_id,
        &purpose,
        &category,
        &expires_at,
    );

    client.revoke(&patient, &grant_id);

    let grant = client.get_grant(&grant_id);
    assert!(grant.revoked);
}

#[test]
fn revoke_requires_record_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let other_owner = Address::generate(&env);
    let grantee = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    let expires_at = env.ledger().timestamp() + 3_600;
    register_full_history_record(&env, &client, &patient, &record_id, &category);
    let grant_id = client.create_normal_grant(
        &patient,
        &grantee,
        &record_id,
        &purpose,
        &category,
        &expires_at,
    );

    assert_eq!(
        client.try_revoke(&other_owner, &grant_id),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn get_grant_returns_error_for_missing_grant() {
    let env = Env::default();
    let (client, _) = setup_client(&env);
    let grant_id = BytesN::random(&env);

    assert_eq!(client.try_get_grant(&grant_id), Err(Ok(Error::NoGrant)));
}

fn setup_client(env: &Env) -> (AccessBrokerContractClient<'_>, Address) {
    let contract_id = env.register(AccessBrokerContract, ());
    let client = AccessBrokerContractClient::new(env, &contract_id);
    let owner = Address::generate(env);

    (client, owner)
}

fn register_full_history_record(
    env: &Env,
    client: &AccessBrokerContractClient<'_>,
    patient: &Address,
    record_id: &BytesN<32>,
    category: &Symbol,
) {
    let locator = Bytes::from_slice(env, &[4, 3, 2, 1]);
    let commitment = BytesN::random(env);

    client.register_record(
        patient,
        record_id,
        &Tier::FullHistory,
        category,
        &true,
        &locator,
        &commitment,
    );
}

fn grant(
    env: &Env,
    record_id: &BytesN<32>,
    grantee: &Address,
    gtype: GrantType,
    purpose: Symbol,
    scope_category: Symbol,
) -> Grant {
    Grant {
        record: record_id.clone(),
        grantee: grantee.clone(),
        gtype,
        purpose,
        scope_category,
        expires_at: env.ledger().timestamp() + 3_600,
        reveal_at: 0,
        revoked: false,
        vetoed: false,
    }
}
