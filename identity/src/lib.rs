#![no_std]

pub mod errors;
mod events;
pub mod storage;
pub mod types;

pub use errors::IdentityError;
pub use types::{CredentialRef, CredentialStatus, IssuerRecord, Role};

use soroban_sdk::{contract, contractimpl, xdr::ToXdr, Address, BytesN, Env};

#[contract]
pub struct IdentityContract;

#[contractimpl]
impl IdentityContract {
    pub fn schema_version() -> u32 {
        1
    }

    pub fn register_issuer(
        env: Env,
        admin: Address,
        issuer_address: Address,
    ) -> Result<(), IdentityError> {
        admin.require_auth();
        require_admin_or_bootstrap(&env, &admin)?;

        if storage::has_issuer(&env, &issuer_address) {
            return Err(IdentityError::IssuerAlreadyRegistered);
        }

        let issuer_record = IssuerRecord {
            issuer: issuer_address.clone(),
            registered_at: env.ledger().timestamp(),
            active: true,
        };

        storage::set_issuer(&env, &issuer_address, &issuer_record);
        events::publish_issuer_registered(&env, &admin, &issuer_address);
        Ok(())
    }

    pub fn issue_credential(
        env: Env,
        issuer: Address,
        subject: Address,
        role: Role,
        expires_at: u64,
    ) -> Result<BytesN<32>, IdentityError> {
        issuer.require_auth();

        let issuer_record =
            storage::get_issuer(&env, &issuer).ok_or(IdentityError::IssuerNotRegistered)?;
        if !issuer_record.active {
            return Err(IdentityError::IssuerNotRegistered);
        }

        let now = env.ledger().timestamp();
        if expires_at <= now {
            return Err(IdentityError::InvalidExpiration);
        }

        let subject_creds = storage::get_subject_creds(&env, &subject);
        let nonce = subject_creds.len();
        let preimage = (
            issuer.clone(),
            subject.clone(),
            role.clone(),
            expires_at,
            now,
            nonce,
        )
            .to_xdr(&env);
        let cred_id = env.crypto().sha256(&preimage).to_bytes();

        if storage::has_credential(&env, &cred_id) {
            return Err(IdentityError::CredentialAlreadyExists);
        }

        let credential = CredentialRef {
            subject: subject.clone(),
            role: role.clone(),
            issuer: issuer.clone(),
            expires_at,
            status: CredentialStatus::Active,
        };

        storage::set_credential(&env, &cred_id, &credential);
        storage::add_subject_credential(&env, &subject, &cred_id);
        events::publish_credential_issued(&env, &issuer, &cred_id, &subject, role.code());
        Ok(cred_id)
    }

    pub fn revoke_credential(
        env: Env,
        issuer_or_admin: Address,
        cred_id: BytesN<32>,
    ) -> Result<(), IdentityError> {
        issuer_or_admin.require_auth();

        let mut credential =
            storage::get_credential(&env, &cred_id).ok_or(IdentityError::CredentialNotFound)?;
        if credential.issuer != issuer_or_admin {
            require_admin(&env, &issuer_or_admin)?;
        }

        credential.status = CredentialStatus::Revoked;
        storage::set_credential(&env, &cred_id, &credential);
        events::publish_credential_revoked(&env, &issuer_or_admin, &cred_id);
        Ok(())
    }

    pub fn verify_credential(
        env: Env,
        cred_id: BytesN<32>,
        expected_subject: Address,
        expected_role: Role,
    ) -> bool {
        let Some(credential) = storage::get_credential(&env, &cred_id) else {
            return false;
        };

        credential.status == CredentialStatus::Active
            && credential.expires_at > env.ledger().timestamp()
            && credential.subject == expected_subject
            && credential.role == expected_role
    }
}

fn require_admin_or_bootstrap(env: &Env, admin: &Address) -> Result<(), IdentityError> {
    if let Some(stored_admin) = storage::get_admin(env) {
        if stored_admin != *admin {
            return Err(IdentityError::Unauthorized);
        }
    } else {
        storage::set_admin(env, admin);
    }

    Ok(())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), IdentityError> {
    let admin = storage::get_admin(env).ok_or(IdentityError::NotInitialized)?;
    if admin != *caller {
        return Err(IdentityError::Unauthorized);
    }

    Ok(())
}

#[cfg(test)]
mod test;
