#![no_std]

pub mod errors;
mod events;
pub mod storage;
pub mod types;

pub use errors::Error;
pub use types::{Capability, CredentialProof, Grant, GrantType, PresenceProof, RecordMeta, Tier};

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, Symbol};

#[contract]
pub struct AccessBrokerContract;

#[contractimpl]
impl AccessBrokerContract {
    pub fn schema_version() -> u32 {
        1
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_record(
        env: Env,
        owner: Address,
        record_id: BytesN<32>,
        tier: Tier,
        category: Symbol,
        sensitive: bool,
        locator_bytes: Bytes,
        commitment: BytesN<32>,
    ) -> Result<(), Error> {
        owner.require_auth();

        if storage::has_record(&env, &record_id) {
            return Err(Error::RecordAlreadyExists);
        }

        let meta = RecordMeta {
            owner: owner.clone(),
            tier: tier.clone(),
            category: category.clone(),
            sensitive,
            commitment,
            locator: locator_bytes,
        };

        storage::set_record(&env, &record_id, &meta);
        events::publish_record_registered(&env, &owner, &record_id, tier.code(), &category);
        Ok(())
    }

    pub fn get_record(env: Env, record_id: BytesN<32>) -> Result<RecordMeta, Error> {
        storage::get_record(&env, &record_id).ok_or(Error::NoSuchRecord)
    }

    pub fn register_patient_token(
        env: Env,
        patient: Address,
        token_pubkey: BytesN<32>,
    ) -> Result<(), Error> {
        patient.require_auth();

        if storage::has_patient_token(&env, &patient) {
            return Err(Error::PatientTokenAlreadyRegistered);
        }

        storage::set_patient_token(&env, &patient, &token_pubkey);
        events::publish_patient_token_registered(&env, &patient);
        Ok(())
    }

    pub fn get_patient_token(env: Env, patient: Address) -> Result<BytesN<32>, Error> {
        storage::get_patient_token(&env, &patient).ok_or(Error::NoTokenRegistered)
    }
}

#[cfg(test)]
mod test;
