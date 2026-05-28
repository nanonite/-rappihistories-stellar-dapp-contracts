use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, Symbol};

use crate::types::GrantType;

pub fn publish_record_registered(
    env: &Env,
    owner: &Address,
    record_id: &BytesN<32>,
    tier_code: u32,
    category: &Symbol,
    locator: &Bytes,
    commitment: &BytesN<32>,
) {
    env.events().publish(
        (symbol_short!("rec_reg"), owner.clone()),
        (
            record_id.clone(),
            tier_code,
            category.clone(),
            locator.clone(),
            commitment.clone(),
        ),
    );
}

pub fn publish_patient_token_registered(env: &Env, patient: &Address) {
    env.events()
        .publish((symbol_short!("pt_token"), patient.clone()), ());
}

pub fn publish_grant_created(
    env: &Env,
    patient: &Address,
    grant_id: &BytesN<32>,
    record_id: &BytesN<32>,
    grantee: &Address,
    grant_type: &GrantType,
    purpose: &Symbol,
    scope_category: &Symbol,
    expires_at: u64,
    reveal_at: u64,
) {
    env.events().publish(
        (symbol_short!("grant_cr"), patient.clone(), grantee.clone()),
        (
            grant_id.clone(),
            record_id.clone(),
            expires_at,
            reveal_at,
            purpose.clone(),
            scope_category.clone(),
            grant_type.code(),
        ),
    );
}

pub fn publish_grant_revoked(env: &Env, owner: &Address, grant_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("grant_rv"), owner.clone()), grant_id.clone());
}

pub fn publish_grant_vetoed(env: &Env, patient: &Address, grant_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("veto"), patient.clone()), grant_id.clone());
}

pub fn publish_access_requested(
    env: &Env,
    requester: &Address,
    grant_id: &BytesN<32>,
    record_id: &BytesN<32>,
    purpose: &Symbol,
    tier_code: u32,
) {
    env.events().publish(
        (symbol_short!("acc_req"), requester.clone()),
        (
            grant_id.clone(),
            record_id.clone(),
            purpose.clone(),
            tier_code,
        ),
    );
}
