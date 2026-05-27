use soroban_sdk::{contracttype, Address, Bytes, BytesN, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tier {
    OfflineCard,
    EmergencyBundle,
    FullHistory,
}

impl Tier {
    pub fn code(&self) -> u32 {
        match self {
            Self::OfflineCard => 1,
            Self::EmergencyBundle => 2,
            Self::FullHistory => 3,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GrantType {
    Normal,
    BreakGlass,
    TokenlessFallback,
}

impl GrantType {
    pub fn code(&self) -> u32 {
        match self {
            Self::Normal => 1,
            Self::BreakGlass => 2,
            Self::TokenlessFallback => 3,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordMeta {
    pub owner: Address,
    pub tier: Tier,
    pub category: Symbol,
    pub sensitive: bool,
    pub commitment: BytesN<32>,
    pub locator: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grant {
    pub record: BytesN<32>,
    pub grantee: Address,
    pub gtype: GrantType,
    pub purpose: Symbol,
    pub scope_category: Symbol,
    pub expires_at: u64,
    pub reveal_at: u64,
    pub revoked: bool,
    pub vetoed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresenceProof {
    pub token_pubkey: BytesN<32>,
    pub nonce: BytesN<32>,
    pub expires_at: u64,
    pub signature: BytesN<64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialProof {
    pub cred_id: BytesN<32>,
    pub role: Symbol,
    pub subject: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capability {
    pub grant_id: BytesN<32>,
    pub locator: Bytes,
    pub commitment: BytesN<32>,
}
