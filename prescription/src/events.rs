use soroban_sdk::{symbol_short, BytesN, Env};

pub fn publish_marker(env: &Env, marker_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("marker"),), marker_id.clone());
}

pub fn publish_prescription_issued(env: &Env, prescription_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("rx_issue"),), prescription_id.clone());
}

pub fn publish_prescription_reserved(
    env: &Env,
    prescription_id: &BytesN<32>,
    unit_id: &BytesN<32>,
) {
    env.events().publish(
        (symbol_short!("rx_res"), prescription_id.clone()),
        unit_id.clone(),
    );
}

pub fn publish_prescription_dispensed(
    env: &Env,
    prescription_id: &BytesN<32>,
    receipt_record_id: &BytesN<32>,
) {
    env.events().publish(
        (symbol_short!("rx_disp"), prescription_id.clone()),
        receipt_record_id.clone(),
    );
}
