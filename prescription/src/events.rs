use soroban_sdk::{symbol_short, Address, BytesN, Env};

pub fn publish_marker(env: &Env, marker_id: &BytesN<32>) {
    env.events()
        .publish((symbol_short!("marker"),), marker_id.clone());
}

pub fn publish_prescription_issued(
    env: &Env,
    patient: &Address,
    clinician: &Address,
    prescription_id: &BytesN<32>,
    diagnosis_record_id: &BytesN<32>,
    prescription_commitment: &BytesN<32>,
) {
    env.events().publish(
        (symbol_short!("rx_issue"), patient.clone(), clinician.clone()),
        (
            prescription_id.clone(),
            diagnosis_record_id.clone(),
            prescription_commitment.clone(),
        ),
    );
}

pub fn publish_prescription_reserved(
    env: &Env,
    patient: &Address,
    pharmacy: &Address,
    prescription_id: &BytesN<32>,
    unit_id: &BytesN<32>,
    reservation_ref: &BytesN<32>,
) {
    env.events().publish(
        (symbol_short!("rx_res"), prescription_id.clone(), pharmacy.clone()),
        (patient.clone(), unit_id.clone(), reservation_ref.clone()),
    );
}

pub fn publish_prescription_dispensed(
    env: &Env,
    patient: &Address,
    pharmacy: &Address,
    prescription_id: &BytesN<32>,
    unit_id: &BytesN<32>,
    receipt_record_id: &BytesN<32>,
) {
    env.events().publish(
        (symbol_short!("rx_disp"), prescription_id.clone(), pharmacy.clone()),
        (patient.clone(), unit_id.clone(), receipt_record_id.clone()),
    );
}
