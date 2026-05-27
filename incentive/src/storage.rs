use soroban_sdk::{contracttype, BytesN, Env};

#[contracttype]
pub enum DataKey {
    Marker(BytesN<32>),
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
