use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    Oracle(Address),
    Attester(BytesN<32>),
    Marker(BytesN<32>),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_oracle(env: &Env, oracle: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::Oracle(oracle.clone()), &true);
}

pub fn is_oracle(env: &Env, oracle: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Oracle(oracle.clone()))
        .unwrap_or(false)
}

pub fn set_attester(env: &Env, attester_pubkey: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::Attester(attester_pubkey.clone()), &true);
}

pub fn is_attester(env: &Env, attester_pubkey: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Attester(attester_pubkey.clone()))
        .unwrap_or(false)
}

pub fn put_marker(env: &Env, marker_id: &BytesN<32>) {
    env.storage()
        .instance()
        .set(&DataKey::Marker(marker_id.clone()), &true);
}

pub fn has_marker(env: &Env, marker_id: &BytesN<32>) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Marker(marker_id.clone()))
        .unwrap_or(false)
}
