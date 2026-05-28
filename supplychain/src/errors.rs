use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    Placeholder = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    Unauthorized = 4,
    UnitAlreadyExists = 5,
    NoSuchUnit = 6,
    UnitUnavailable = 7,
    BatchQuarantined = 8,
    WrongPrescriptionContract = 9,
}
