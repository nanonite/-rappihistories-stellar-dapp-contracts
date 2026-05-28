use soroban_sdk::{contracttype, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MarkerStatus {
    Recorded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnitStatus {
    Available,
    Reserved,
    Dispensed,
    Quarantined,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Unit {
    pub batch_id: BytesN<32>,
    pub reservation_ref: BytesN<32>,
    pub status: UnitStatus,
}
