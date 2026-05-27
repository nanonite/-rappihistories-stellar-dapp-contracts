use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NoSuchRecord = 1,
    BadCredential = 2,
    CredentialNotForCaller = 3,
    NoGrant = 4,
    GrantExpired = 5,
    GrantRevoked = 6,
    ScopeMismatch = 7,
    SensitiveNeedsExplicitGrant = 8,
    OfflineTierNotBrokered = 9,
    StalePresence = 10,
    WrongToken = 11,
    NoTokenRegistered = 12,
    NonceReplayed = 13,
    BadPresenceSig = 14,
    FallbackNeedsDualSign = 15,
    Unauthorized = 16,
    AlreadyInitialized = 17,
    NotInitialized = 18,
    RecordAlreadyExists = 19,
    GrantAlreadyExists = 20,
    PatientTokenAlreadyRegistered = 21,
    InvalidExpiration = 22,
    InvalidRevealWindow = 23,
    IssuerRootNotConfigured = 24,
}
