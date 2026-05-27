#![no_std]

pub mod errors;
mod events;
pub mod storage;
pub mod types;

pub use errors::Error;
pub use types::{Capability, CredentialProof, Grant, GrantType, PresenceProof, RecordMeta, Tier};

use soroban_sdk::{contract, contractimpl, xdr::ToXdr, Address, Bytes, BytesN, Env, Symbol};

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

    pub fn create_normal_grant(
        env: Env,
        patient: Address,
        grantee: Address,
        record_id: BytesN<32>,
        purpose: Symbol,
        scope_category: Symbol,
        expires_at: u64,
    ) -> Result<BytesN<32>, Error> {
        patient.require_auth();

        let record = storage::get_record(&env, &record_id).ok_or(Error::NoSuchRecord)?;
        if record.owner != patient {
            return Err(Error::Unauthorized);
        }

        let now = env.ledger().timestamp();
        if expires_at <= now {
            return Err(Error::InvalidExpiration);
        }

        let preimage = (grantee.clone(), record_id.clone(), now).to_xdr(&env);
        let grant_id = env.crypto().sha256(&preimage).to_bytes();
        if storage::has_grant(&env, &grant_id) {
            return Err(Error::GrantAlreadyExists);
        }

        let grant = Grant {
            record: record_id.clone(),
            grantee: grantee.clone(),
            gtype: GrantType::Normal,
            purpose,
            scope_category,
            expires_at,
            reveal_at: 0,
            revoked: false,
            vetoed: false,
        };

        storage::set_normal_grant(&env, &grant_id, &grant);
        events::publish_grant_created(&env, &patient, &grant_id, &record_id, &grantee, expires_at);
        Ok(grant_id)
    }

    pub fn get_grant(env: Env, grant_id: BytesN<32>) -> Result<Grant, Error> {
        storage::get_grant(&env, &grant_id).ok_or(Error::NoGrant)
    }

    pub fn revoke(env: Env, owner: Address, grant_id: BytesN<32>) -> Result<(), Error> {
        owner.require_auth();

        let mut grant = storage::get_grant(&env, &grant_id).ok_or(Error::NoGrant)?;
        let record = storage::get_record(&env, &grant.record).ok_or(Error::NoSuchRecord)?;
        if record.owner != owner {
            return Err(Error::Unauthorized);
        }

        grant.revoked = true;
        storage::set_grant(&env, &grant_id, &grant);
        events::publish_grant_revoked(&env, &owner, &grant_id);
        Ok(())
    }
}

#[cfg(test)]
mod test;
