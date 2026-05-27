#![no_std]

pub mod errors;
pub mod storage;
pub mod types;

pub use errors::IdentityError;
pub use types::{CredentialRef, CredentialStatus, IssuerRecord, Role};

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct IdentityContract;

#[contractimpl]
impl IdentityContract {
    pub fn schema_version() -> u32 {
        1
    }
}

#[cfg(test)]
mod test;
