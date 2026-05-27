#![no_std]

pub mod errors;
pub mod storage;
pub mod types;

pub use errors::Error;
pub use types::{Capability, CredentialProof, Grant, GrantType, PresenceProof, RecordMeta, Tier};

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct AccessBrokerContract;

#[contractimpl]
impl AccessBrokerContract {
    pub fn schema_version() -> u32 {
        1
    }
}

#[cfg(test)]
mod test;
