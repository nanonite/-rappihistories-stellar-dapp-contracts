#![cfg(test)]

extern crate std;

use super::*;
use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, BytesN as _, Events as _, Ledger as _,
    },
    Address, Bytes, BytesN, Env, Symbol, Vec,
};

const PRESENCE_SIGNING_KEY: [u8; 32] = [7; 32];

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
fn critical_broker_state_is_renewed_on_write() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let contract_id = env.register(AccessBrokerContract, ());

    env.as_contract(&contract_id, || {
        let admin = Address::generate(&env);
        let issuer_root = Address::generate(&env);
        let patient = Address::generate(&env);
        let grantee = Address::generate(&env);
        let record_id = BytesN::random(&env);
        let grant_id = BytesN::random(&env);
        let token_pubkey = BytesN::random(&env);
        let category = Symbol::new(&env, "cardiology");
        let record = RecordMeta {
            owner: patient.clone(),
            tier: Tier::FullHistory,
            category: category.clone(),
            sensitive: false,
            commitment: BytesN::random(&env),
            locator: Bytes::from_slice(&env, &[1, 2, 3]),
        };
        let normal_grant = grant(
            &env,
            &record_id,
            &grantee,
            GrantType::Normal,
            Symbol::new(&env, "treatment"),
            category,
        );

        storage::set_admin(&env, &admin);
        storage::set_issuer_root(&env, &issuer_root);
        storage::set_record(&env, &record_id, &record);
        storage::set_patient_token(&env, &patient, &token_pubkey);
        storage::set_grant(&env, &grant_id, &normal_grant);

        assert!(env.storage().instance().get_ttl() >= storage::CRITICAL_STATE_TTL_THRESHOLD);
        assert!(
            env.storage()
                .persistent()
                .get_ttl(&storage::DataKey::Record(record_id))
                >= storage::CRITICAL_STATE_TTL_THRESHOLD
        );
        assert!(
            env.storage()
                .persistent()
                .get_ttl(&storage::DataKey::PatientToken(patient))
                >= storage::CRITICAL_STATE_TTL_THRESHOLD
        );
        assert!(
            env.storage()
                .persistent()
                .get_ttl(&storage::DataKey::Grant(grant_id))
                >= storage::CRITICAL_STATE_TTL_THRESHOLD
        );
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
fn initialize_records_broker_admin_once() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_client(&env);
    let next_admin = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(client.admin(), admin);
    assert_eq!(
        client.try_initialize(&next_admin),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn configure_issuer_root_is_admin_only() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_client(&env);
    let non_admin = Address::generate(&env);
    let issuer_root = Address::generate(&env);
    let next_issuer_root = Address::generate(&env);

    client.initialize(&admin);
    client.configure_issuer_root(&admin, &issuer_root);

    assert_eq!(client.issuer_root(), issuer_root);
    assert_eq!(
        client.try_configure_issuer_root(&non_admin, &next_issuer_root),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn configure_issuer_root_requires_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_client(&env);
    let issuer_root = Address::generate(&env);

    assert_eq!(
        client.try_configure_issuer_root(&admin, &issuer_root),
        Err(Ok(Error::NotInitialized))
    );
}

#[test]
fn renew_critical_state_is_admin_only() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_client(&env);
    let non_admin = Address::generate(&env);
    let record_ids: Vec<BytesN<32>> = Vec::new(&env);
    let patients: Vec<Address> = Vec::new(&env);
    let grant_ids: Vec<BytesN<32>> = Vec::new(&env);

    assert_eq!(
        client.try_renew_critical_state(&admin, &record_ids, &patients, &grant_ids),
        Err(Ok(Error::NotInitialized))
    );

    client.initialize(&admin);
    assert_eq!(
        client.try_renew_critical_state(&non_admin, &record_ids, &patients, &grant_ids),
        Err(Ok(Error::Unauthorized))
    );
    client.renew_critical_state(&admin, &record_ids, &patients, &grant_ids);
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
fn create_normal_grant_rejects_duplicate_grant_id() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let grantee = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    let expires_at = env.ledger().timestamp() + 3_600;
    register_full_history_record(&env, &client, &patient, &record_id, &category);

    client.create_normal_grant(
        &patient,
        &grantee,
        &record_id,
        &purpose,
        &category,
        &expires_at,
    );

    assert_eq!(
        client.try_create_normal_grant(
            &patient,
            &grantee,
            &record_id,
            &purpose,
            &category,
            &expires_at,
        ),
        Err(Ok(Error::GrantAlreadyExists))
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

#[test]
fn request_access_stores_grant_emits_audit_and_returns_capability() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    let (locator, commitment) = register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    let capability = client.request_access(&requester, &record_id, &purpose, &cred, &presence);

    assert_eq!(env.events().all().len(), 1);
    assert_eq!(capability.locator, locator);
    assert_eq!(capability.commitment, commitment);

    let grant = client.get_grant(&capability.grant_id);
    assert_eq!(grant.record, record_id);
    assert_eq!(grant.grantee, requester);
    assert_eq!(grant.gtype, GrantType::Normal);
    assert_eq!(grant.purpose, purpose);
    assert_eq!(grant.scope_category, category);
    assert!(!grant.revoked);
    assert!(!grant.vetoed);
}

#[test]
fn request_access_simulation_preparation_does_not_commit_or_emit() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id, patient) = setup_client_with_id(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    env.as_contract(&contract_id, || {
        let prepared =
            prepare_request_access(&env, &requester, &record_id, &purpose, &cred, &presence)
                .unwrap();
        assert!(!storage::has_grant(&env, &prepared.grant_id));
        assert!(!storage::has_spent_nonce(&env, &presence.nonce));
    });

    assert!(env.events().all().is_empty());
}

#[test]
fn request_access_rejects_missing_record() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let purpose = Symbol::new(&env, "treatment");
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::NoSuchRecord))
    );
}

#[test]
fn request_access_rejects_bad_credential() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    let cred = CredentialProof {
        cred_id: BytesN::from_array(&env, &[0; 32]),
        role: Symbol::new(&env, "clinician"),
        subject: requester.clone(),
    };
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::BadCredential))
    );
}

#[test]
fn request_access_rejects_credential_for_different_caller() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_client(&env);
    let requester = Address::generate(&env);
    let other_subject = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let purpose = Symbol::new(&env, "treatment");
    let cred = CredentialProof {
        cred_id: BytesN::random(&env),
        role: Symbol::new(&env, "clinician"),
        subject: other_subject,
    };
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::CredentialNotForCaller))
    );
}

#[test]
fn request_access_rejects_missing_patient_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::NoTokenRegistered))
    );
}

#[test]
fn request_access_rejects_wrong_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    client.register_patient_token(&patient, &BytesN::random(&env));
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::WrongToken))
    );
}

#[test]
fn request_access_rejects_stale_presence() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp());

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::StalePresence))
    );
}

#[test]
fn request_access_rejects_bad_presence_signature() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let mut presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);
    presence.signature = BytesN::from_array(&env, &[0; 64]);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::BadPresenceSig))
    );
}

#[test]
fn request_access_rejects_nonce_replay() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);
    client.request_access(&requester, &record_id, &purpose, &cred, &presence);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::NonceReplayed))
    );
}

#[test]
fn request_access_rejects_offline_tier() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::OfflineCard,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::OfflineTierNotBrokered))
    );
}

#[test]
fn request_access_rejects_sensitive_without_explicit_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, patient) = setup_client(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        true,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);

    assert_eq!(
        client.try_request_access(&requester, &record_id, &category, &cred, &presence),
        Err(Ok(Error::SensitiveNeedsExplicitGrant))
    );
}

#[test]
fn request_access_rejects_sensitive_scope_mismatch() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id, patient) = setup_client_with_id(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        true,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);
    store_request_grant(
        &env,
        &contract_id,
        &requester,
        &record_id,
        &purpose,
        &cred,
        Symbol::new(&env, "allergy"),
        false,
        env.ledger().timestamp() + 300,
    );

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::ScopeMismatch))
    );
}

#[test]
fn request_access_rejects_expired_existing_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id, patient) = setup_client_with_id(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);
    store_request_grant(
        &env,
        &contract_id,
        &requester,
        &record_id,
        &purpose,
        &cred,
        category,
        false,
        env.ledger().timestamp(),
    );

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::GrantExpired))
    );
}

#[test]
fn request_access_rejects_revoked_existing_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id, patient) = setup_client_with_id(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "cardiology");
    let purpose = Symbol::new(&env, "treatment");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::FullHistory,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "clinician");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);
    store_request_grant(
        &env,
        &contract_id,
        &requester,
        &record_id,
        &purpose,
        &cred,
        category,
        true,
        env.ledger().timestamp() + 300,
    );

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::GrantRevoked))
    );
}

#[test]
fn request_access_rejects_existing_grant_before_reveal_at() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id, patient) = setup_client_with_id(&env);
    let requester = Address::generate(&env);
    let record_id = BytesN::random(&env);
    let category = Symbol::new(&env, "emergency");
    let purpose = Symbol::new(&env, "emergency");
    register_broker_record(
        &env,
        &client,
        &patient,
        &record_id,
        Tier::EmergencyBundle,
        &category,
        false,
    );
    register_patient_token(&env, &client, &patient);
    let cred = credential(&env, &requester, "responder");
    let presence = presence(&env, &requester, &record_id, env.ledger().timestamp() + 300);
    store_request_grant_with_controls(
        &env,
        &contract_id,
        &requester,
        &record_id,
        &purpose,
        &cred,
        category,
        GrantType::BreakGlass,
        false,
        false,
        env.ledger().timestamp() + 30,
        env.ledger().timestamp() + 300,
    );

    assert_eq!(
        client.try_request_access(&requester, &record_id, &purpose, &cred, &presence),
        Err(Ok(Error::NoGrant))
    );
}

fn setup_client(env: &Env) -> (AccessBrokerContractClient<'_>, Address) {
    let (client, _, owner) = setup_client_with_id(env);
    (client, owner)
}

fn setup_client_with_id(env: &Env) -> (AccessBrokerContractClient<'_>, Address, Address) {
    let contract_id = env.register(AccessBrokerContract, ());
    let client = AccessBrokerContractClient::new(env, &contract_id);
    let owner = Address::generate(env);

    (client, contract_id, owner)
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

fn register_broker_record(
    env: &Env,
    client: &AccessBrokerContractClient<'_>,
    patient: &Address,
    record_id: &BytesN<32>,
    tier: Tier,
    category: &Symbol,
    sensitive: bool,
) -> (Bytes, BytesN<32>) {
    let locator = Bytes::from_slice(env, &[4, 3, 2, 1]);
    let commitment = BytesN::random(env);

    client.register_record(
        patient,
        record_id,
        &tier,
        category,
        &sensitive,
        &locator,
        &commitment,
    );

    (locator, commitment)
}

fn register_patient_token(env: &Env, client: &AccessBrokerContractClient<'_>, patient: &Address) {
    let signing_key = presence_signing_key();
    let token_pubkey = BytesN::from_array(env, &signing_key.verifying_key().to_bytes());
    client.register_patient_token(patient, &token_pubkey);
}

fn credential(env: &Env, subject: &Address, role: &str) -> CredentialProof {
    CredentialProof {
        cred_id: BytesN::random(env),
        role: Symbol::new(env, role),
        subject: subject.clone(),
    }
}

fn presence(
    env: &Env,
    requester: &Address,
    record_id: &BytesN<32>,
    expires_at: u64,
) -> PresenceProof {
    let signing_key = presence_signing_key();
    let token_pubkey = BytesN::from_array(env, &signing_key.verifying_key().to_bytes());
    let nonce = BytesN::random(env);
    let mut proof = PresenceProof {
        token_pubkey,
        nonce,
        expires_at,
        signature: BytesN::from_array(env, &[0; 64]),
    };
    let message = presence_message(env, requester, record_id, &proof);
    let mut message_bytes = std::vec![0; message.len() as usize];
    message.copy_into_slice(&mut message_bytes);
    let signature = signing_key.sign(&message_bytes);
    proof.signature = BytesN::from_array(env, &signature.to_bytes());

    proof
}

#[allow(clippy::too_many_arguments)]
fn store_request_grant(
    env: &Env,
    contract_id: &Address,
    requester: &Address,
    record_id: &BytesN<32>,
    purpose: &Symbol,
    cred: &CredentialProof,
    scope_category: Symbol,
    revoked: bool,
    expires_at: u64,
) {
    store_request_grant_with_controls(
        env,
        contract_id,
        requester,
        record_id,
        purpose,
        cred,
        scope_category,
        GrantType::Normal,
        revoked,
        false,
        0,
        expires_at,
    );
}

#[allow(clippy::too_many_arguments)]
fn store_request_grant_with_controls(
    env: &Env,
    contract_id: &Address,
    requester: &Address,
    record_id: &BytesN<32>,
    purpose: &Symbol,
    cred: &CredentialProof,
    scope_category: Symbol,
    gtype: GrantType,
    revoked: bool,
    vetoed: bool,
    reveal_at: u64,
    expires_at: u64,
) {
    env.as_contract(contract_id, || {
        let grant_id = request_access_grant_id(env, requester, record_id, purpose, &cred.cred_id);
        let grant = Grant {
            record: record_id.clone(),
            grantee: requester.clone(),
            gtype,
            purpose: purpose.clone(),
            scope_category,
            expires_at,
            reveal_at,
            revoked,
            vetoed,
        };
        storage::set_grant(env, &grant_id, &grant);
    });
}

fn presence_signing_key() -> SigningKey {
    SigningKey::from_bytes(&PRESENCE_SIGNING_KEY)
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
