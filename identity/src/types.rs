use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Patient,
    Clinician,
    Institution,
    Pharmacy,
    Distributor,
    Manufacturer,
    Responder,
    Admin,
}

impl Role {
    pub fn code(&self) -> u32 {
        match self {
            Self::Patient => 1,
            Self::Clinician => 2,
            Self::Institution => 3,
            Self::Pharmacy => 4,
            Self::Distributor => 5,
            Self::Manufacturer => 6,
            Self::Responder => 7,
            Self::Admin => 8,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialStatus {
    Active,
    Revoked,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialRef {
    pub subject: Address,
    pub role: Role,
    pub issuer: Address,
    pub expires_at: u64,
    pub status: CredentialStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerRecord {
    pub issuer: Address,
    pub registered_at: u64,
    pub active: bool,
}
