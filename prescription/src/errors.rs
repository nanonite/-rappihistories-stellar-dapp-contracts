use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    Placeholder = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    Unauthorized = 4,
    PrescriptionAlreadyExists = 5,
    NoSuchPrescription = 6,
    InvalidState = 7,
    WrongPatient = 8,
    WrongPharmacy = 9,
}
