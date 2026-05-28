use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MarkerStatus {
    Recorded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrescriptionStatus {
    Issued,
    Reserved,
    Dispensed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tier {
    OfflineCard,
    EmergencyBundle,
    FullHistory,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prescription {
    pub clinician: Address,
    pub patient: Address,
    pub pharmacy: Address,
    pub diagnosis_record_id: BytesN<32>,
    pub unit_id: BytesN<32>,
    pub commitment: BytesN<32>,
    pub receipt_record_id: BytesN<32>,
    pub status: PrescriptionStatus,
}
