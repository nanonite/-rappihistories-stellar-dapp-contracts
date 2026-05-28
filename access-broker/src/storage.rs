use soroban_sdk::{contracttype, Address, BytesN, Env};

use crate::types::{Grant, GrantType, RecordMeta};

pub const MAX_PRESENCE_WINDOW: u32 = 300;
pub const CRITICAL_STATE_TTL_THRESHOLD: u32 = 10_000;
pub const CRITICAL_STATE_TTL_EXTEND_TO: u32 = 120_960;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,                  // instance storage: Address, always alive
    IssuerRoot,             // instance storage: Address, always alive
    Record(BytesN<32>),     // persistent storage: RecordMeta
    Grant(BytesN<32>),      // persistent for Normal; temporary for BreakGlass/TokenlessFallback
    PatientToken(Address),  // persistent storage: BytesN<32>
    SpentNonce(BytesN<32>), // temporary storage: bool, replay guard
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_issuer_root(env: &Env, issuer_root: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::IssuerRoot, issuer_root);
    renew_instance_ttl(env);
}

pub fn get_issuer_root(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::IssuerRoot)
}

pub fn set_record(env: &Env, record_id: &BytesN<32>, meta: &RecordMeta) {
    env.storage()
        .persistent()
        .set(&DataKey::Record(record_id.clone()), meta);
    renew_record(env, record_id);
}

pub fn get_record(env: &Env, record_id: &BytesN<32>) -> Option<RecordMeta> {
    env.storage()
        .persistent()
        .get(&DataKey::Record(record_id.clone()))
}

pub fn has_record(env: &Env, record_id: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Record(record_id.clone()))
}

pub fn set_grant(env: &Env, grant_id: &BytesN<32>, grant: &Grant) {
    match grant.gtype {
        GrantType::Normal => set_normal_grant(env, grant_id, grant),
        GrantType::BreakGlass | GrantType::TokenlessFallback => {
            set_temporary_grant(env, grant_id, grant)
        }
    }
}

pub fn set_normal_grant(env: &Env, grant_id: &BytesN<32>, grant: &Grant) {
    env.storage()
        .persistent()
        .set(&DataKey::Grant(grant_id.clone()), grant);
    renew_active_normal_grant(env, grant_id, grant);
}

pub fn set_temporary_grant(env: &Env, grant_id: &BytesN<32>, grant: &Grant) {
    let key = DataKey::Grant(grant_id.clone());
    env.storage().temporary().set(&key, grant);
}

pub fn get_grant(env: &Env, grant_id: &BytesN<32>) -> Option<Grant> {
    let key = DataKey::Grant(grant_id.clone());
    env.storage()
        .persistent()
        .get(&key)
        .or_else(|| env.storage().temporary().get(&key))
}

pub fn has_grant(env: &Env, grant_id: &BytesN<32>) -> bool {
    let key = DataKey::Grant(grant_id.clone());
    env.storage().persistent().has(&key) || env.storage().temporary().has(&key)
}

pub fn set_patient_token(env: &Env, patient: &Address, token_pubkey: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::PatientToken(patient.clone()), token_pubkey);
    renew_patient_token(env, patient);
}

pub fn get_patient_token(env: &Env, patient: &Address) -> Option<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::PatientToken(patient.clone()))
}

pub fn has_patient_token(env: &Env, patient: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::PatientToken(patient.clone()))
}

pub fn mark_nonce_spent(env: &Env, nonce: &BytesN<32>) {
    let key = DataKey::SpentNonce(nonce.clone());
    env.storage().temporary().set(&key, &true);
    env.storage()
        .temporary()
        .extend_ttl(&key, MAX_PRESENCE_WINDOW, MAX_PRESENCE_WINDOW);
}

pub fn has_spent_nonce(env: &Env, nonce: &BytesN<32>) -> bool {
    env.storage()
        .temporary()
        .get(&DataKey::SpentNonce(nonce.clone()))
        .unwrap_or(false)
}

pub fn renew_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(CRITICAL_STATE_TTL_THRESHOLD, CRITICAL_STATE_TTL_EXTEND_TO);
}

pub fn renew_record(env: &Env, record_id: &BytesN<32>) {
    let key = DataKey::Record(record_id.clone());
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            CRITICAL_STATE_TTL_THRESHOLD,
            CRITICAL_STATE_TTL_EXTEND_TO,
        );
    }
}

pub fn renew_patient_token(env: &Env, patient: &Address) {
    let key = DataKey::PatientToken(patient.clone());
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            CRITICAL_STATE_TTL_THRESHOLD,
            CRITICAL_STATE_TTL_EXTEND_TO,
        );
    }
}

pub fn renew_active_normal_grant(env: &Env, grant_id: &BytesN<32>, grant: &Grant) {
    if grant.gtype != GrantType::Normal
        || grant.revoked
        || grant.vetoed
        || grant.expires_at <= env.ledger().timestamp()
    {
        return;
    }

    let key = DataKey::Grant(grant_id.clone());
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            CRITICAL_STATE_TTL_THRESHOLD,
            CRITICAL_STATE_TTL_EXTEND_TO,
        );
    }
}
