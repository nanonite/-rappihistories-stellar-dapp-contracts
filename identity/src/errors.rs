use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum IdentityError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    IssuerAlreadyRegistered = 4,
    IssuerNotRegistered = 5,
    CredentialAlreadyExists = 6,
    CredentialNotFound = 7,
    CredentialRevoked = 8,
    CredentialExpired = 9,
    CredentialSubjectMismatch = 10,
    CredentialRoleMismatch = 11,
    InvalidExpiration = 12,
}
