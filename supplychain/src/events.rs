use soroban_sdk::{symbol_short, BytesN, Env};

pub fn publish_marker(env: &Env, marker_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("marker"),), marker_id.clone());
}

pub fn publish_unit_registered(env: &Env, unit_id: &BytesN<32>, batch_id: &BytesN<32>) {
    env.events().publish(
        (symbol_short!("unit_reg"), unit_id.clone()),
        batch_id.clone(),
    );
}

pub fn publish_unit_reserved(env: &Env, unit_id: &BytesN<32>, reservation_ref: &BytesN<32>) {
    env.events().publish(
        (symbol_short!("unit_res"), unit_id.clone()),
        reservation_ref.clone(),
    );
}

pub fn publish_unit_dispensed(env: &Env, unit_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("unit_dis"),), unit_id.clone());
}

pub fn publish_batch_quarantined(env: &Env, batch_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("batch_q"),), batch_id.clone());
}
