use soroban_sdk::{symbol_short, BytesN, Env};

pub fn publish_marker(env: &Env, marker_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("marker"),), marker_id.clone());
}
