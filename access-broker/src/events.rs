use soroban_sdk::{symbol_short, Address, BytesN, Env, Symbol};

pub fn publish_record_registered(
    env: &Env,
    owner: &Address,
    record_id: &BytesN<32>,
    tier_code: u32,
    category: &Symbol,
) {
    let category_code = category_code(env, category);
    env.events().publish(
        (symbol_short!("rec_reg"), owner.clone()),
        (record_id.clone(), tier_code, category_code),
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
    expires_at: u64,
) {
    env.events().publish(
        (symbol_short!("grant_cr"), patient.clone(), grantee.clone()),
        (grant_id.clone(), record_id.clone(), expires_at),
    );
}

pub fn publish_grant_revoked(env: &Env, owner: &Address, grant_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("grant_rv"), owner.clone()), grant_id.clone());
}

fn category_code(env: &Env, category: &Symbol) -> u32 {
    if *category == Symbol::new(env, "cardiology") {
        1
    } else if *category == Symbol::new(env, "medication") {
        2
    } else if *category == Symbol::new(env, "allergy") {
        3
    } else if *category == Symbol::new(env, "behavioral_health") {
        4
    } else if *category == Symbol::new(env, "emergency") {
        5
    } else {
        0
    }
}
