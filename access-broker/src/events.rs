use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, Symbol};

use crate::types::GrantType;

pub fn publish_record_registered(
    env: &Env,
    subject: &Address,
    record_id: &BytesN<32>,
    tier_code: u32,
    category: &Symbol,
    locator: &Bytes,
    commitment: &BytesN<32>,
    created_at: u64,
) {
    env.events().publish(
        (symbol_short!("rec_reg"), subject.clone()),
        (
            record_id.clone(),
            tier_code,
            category.clone(),
            locator.clone(),
            commitment.clone(),
            created_at,
        ),
    );
}

pub fn publish_record_appended(
    env: &Env,
    subject: &Address,
    author: &Address,
    write_grant_id: &BytesN<32>,
    record_id: &BytesN<32>,
    tier_code: u32,
    category: &Symbol,
    locator: &Bytes,
    commitment: &BytesN<32>,
    created_at: u64,
) {
    env.events().publish(
        (symbol_short!("rec_app"), subject.clone(), author.clone()),
        (
            record_id.clone(),
            write_grant_id.clone(),
            tier_code,
            category.clone(),
            locator.clone(),
            commitment.clone(),
            created_at,
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

pub fn publish_write_grant_created(
    env: &Env,
    subject: &Address,
    grantee: &Address,
    grant_id: &BytesN<32>,
    scope_category: &Symbol,
    expires_at: u64,
    created_at: u64,
) {
    env.events().publish(
        (symbol_short!("write_gr"), subject.clone(), grantee.clone()),
        (
            grant_id.clone(),
            scope_category.clone(),
            expires_at,
            created_at,
        ),
    );
}

pub fn publish_write_grant_revoked(env: &Env, subject: &Address, grant_id: &BytesN<32>) {
    env.events().publish(
        (symbol_short!("wrgr_rv"), subject.clone()),
        grant_id.clone(),
    );
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
