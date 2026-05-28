#![no_std]

pub mod errors;
mod events;
pub mod storage;
pub mod types;

pub use errors::Error;
pub use types::{Capability, CredentialProof, Grant, GrantType, PresenceProof, RecordMeta, Tier};

use soroban_sdk::{contract, contractimpl, xdr::ToXdr, Address, Bytes, BytesN, Env, Symbol, Vec};

const REQUEST_GRANT_SECONDS: u64 = 300;

#[contract]
pub struct AccessBrokerContract;

#[contractimpl]
impl AccessBrokerContract {
    pub fn schema_version() -> u32 {
        1
    }

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if storage::get_admin(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }

        storage::set_admin(&env, &admin);
        Ok(())
    }

    pub fn admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    pub fn configure_issuer_root(
        env: Env,
        admin: Address,
        issuer_root: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        require_admin(&env, &admin)?;

        storage::set_issuer_root(&env, &issuer_root);
        Ok(())
    }

    pub fn issuer_root(env: Env) -> Result<Address, Error> {
        storage::get_issuer_root(&env).ok_or(Error::IssuerRootNotConfigured)
    }

    pub fn renew_critical_state(
        env: Env,
        admin: Address,
        record_ids: Vec<BytesN<32>>,
        patients: Vec<Address>,
        grant_ids: Vec<BytesN<32>>,
    ) -> Result<(), Error> {
        admin.require_auth();
        require_admin(&env, &admin)?;

        if storage::get_issuer_root(&env).is_some() {
            storage::renew_instance_ttl(&env);
        }

        for record_id in record_ids {
            storage::renew_record(&env, &record_id);
        }
        for patient in patients {
            storage::renew_patient_token(&env, &patient);
        }
        for grant_id in grant_ids {
            if let Some(grant) = storage::get_grant(&env, &grant_id) {
                storage::renew_active_normal_grant(&env, &grant_id, &grant);
            }
        }

        Ok(())
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
            commitment: commitment.clone(),
            locator: locator_bytes.clone(),
        };

        storage::set_record(&env, &record_id, &meta);
        events::publish_record_registered(
            &env,
            &owner,
            &record_id,
            tier.code(),
            &category,
            &locator_bytes,
            &commitment,
        );
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

        let preimage = (
            patient.clone(),
            grantee.clone(),
            record_id.clone(),
            purpose.clone(),
            scope_category.clone(),
            expires_at,
        )
            .to_xdr(&env);
        let grant_id = env.crypto().sha256(&preimage).to_bytes();
        if storage::has_grant(&env, &grant_id) {
            return Err(Error::GrantAlreadyExists);
        }

        let grant = Grant {
            record: record_id.clone(),
            grantee: grantee.clone(),
            gtype: GrantType::Normal,
            purpose: purpose.clone(),
            scope_category: scope_category.clone(),
            expires_at,
            reveal_at: 0,
            revoked: false,
            vetoed: false,
        };

        storage::set_normal_grant(&env, &grant_id, &grant);
        events::publish_grant_created(
            &env,
            &patient,
            &grant_id,
            &record_id,
            &grantee,
            &grant.gtype,
            &purpose,
            &scope_category,
            expires_at,
            grant.reveal_at,
        );
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

    pub fn open_break_glass(
        env: Env,
        responder: Address,
        patient: Address,
        record_id: BytesN<32>,
        purpose: Symbol,
        reveal_at: u64,
        expires_at: u64,
    ) -> Result<BytesN<32>, Error> {
        responder.require_auth();

        let record = storage::get_record(&env, &record_id).ok_or(Error::NoSuchRecord)?;
        if record.owner != patient {
            return Err(Error::Unauthorized);
        }
        if record.tier != Tier::EmergencyBundle {
            return Err(Error::BadCredential);
        }

        let now = env.ledger().timestamp();
        if reveal_at <= now {
            return Err(Error::InvalidRevealWindow);
        }
        if expires_at <= reveal_at {
            return Err(Error::InvalidExpiration);
        }

        let grant_id = explicit_grant_id(
            &env,
            &patient,
            &responder,
            &record_id,
            &purpose,
            GrantType::BreakGlass,
        );
        if storage::has_grant(&env, &grant_id) {
            return Err(Error::GrantAlreadyExists);
        }

        let grant = Grant {
            record: record_id.clone(),
            grantee: responder.clone(),
            gtype: GrantType::BreakGlass,
            purpose: purpose.clone(),
            scope_category: record.category.clone(),
            expires_at,
            reveal_at,
            revoked: false,
            vetoed: false,
        };

        storage::set_grant(&env, &grant_id, &grant);
        events::publish_grant_created(
            &env,
            &patient,
            &grant_id,
            &record_id,
            &responder,
            &grant.gtype,
            &purpose,
            &grant.scope_category,
            expires_at,
            reveal_at,
        );
        Ok(grant_id)
    }

    pub fn create_tokenless_fallback_grant(
        env: Env,
        requester: Address,
        cosigner: Address,
        patient: Address,
        record_id: BytesN<32>,
        purpose: Symbol,
        expires_at: u64,
    ) -> Result<BytesN<32>, Error> {
        if requester == cosigner {
            return Err(Error::FallbackNeedsDualSign);
        }

        // The requester still signs the KMS release request. The on-chain
        // safeguard here is the independent emergency cosigner.
        cosigner.require_auth();

        let record = storage::get_record(&env, &record_id).ok_or(Error::NoSuchRecord)?;
        if record.owner != patient {
            return Err(Error::Unauthorized);
        }
        if record.tier != Tier::EmergencyBundle {
            return Err(Error::BadCredential);
        }
        if expires_at <= env.ledger().timestamp() {
            return Err(Error::InvalidExpiration);
        }

        let grant_id = explicit_grant_id(
            &env,
            &patient,
            &requester,
            &record_id,
            &purpose,
            GrantType::TokenlessFallback,
        );
        if storage::has_grant(&env, &grant_id) {
            return Err(Error::GrantAlreadyExists);
        }

        let grant = Grant {
            record: record_id.clone(),
            grantee: requester.clone(),
            gtype: GrantType::TokenlessFallback,
            purpose: purpose.clone(),
            scope_category: record.category.clone(),
            expires_at,
            reveal_at: 0,
            revoked: false,
            vetoed: false,
        };

        storage::set_grant(&env, &grant_id, &grant);
        events::publish_grant_created(
            &env,
            &patient,
            &grant_id,
            &record_id,
            &requester,
            &grant.gtype,
            &purpose,
            &grant.scope_category,
            expires_at,
            grant.reveal_at,
        );
        Ok(grant_id)
    }

    pub fn veto(env: Env, patient: Address, grant_id: BytesN<32>) -> Result<(), Error> {
        patient.require_auth();

        let mut grant = storage::get_grant(&env, &grant_id).ok_or(Error::NoGrant)?;
        let record = storage::get_record(&env, &grant.record).ok_or(Error::NoSuchRecord)?;
        if record.owner != patient {
            return Err(Error::Unauthorized);
        }
        if grant.gtype != GrantType::BreakGlass && grant.gtype != GrantType::TokenlessFallback {
            return Err(Error::BadCredential);
        }

        grant.vetoed = true;
        storage::set_grant(&env, &grant_id, &grant);
        events::publish_grant_vetoed(&env, &patient, &grant_id);
        Ok(())
    }

    pub fn request_access(
        env: Env,
        requester: Address,
        record_id: BytesN<32>,
        purpose: Symbol,
        cred: CredentialProof,
        presence: PresenceProof,
    ) -> Result<Capability, Error> {
        requester.require_auth();

        let prepared =
            prepare_request_access(&env, &requester, &record_id, &purpose, &cred, &presence)?;

        events::publish_access_requested(
            &env,
            &requester,
            &prepared.grant_id,
            &record_id,
            &purpose,
            prepared.record.tier.code(),
        );
        storage::mark_nonce_spent(&env, &presence.nonce);
        if !prepared.existing_grant {
            storage::set_grant(&env, &prepared.grant_id, &prepared.grant);
        }

        Ok(Capability {
            grant_id: prepared.grant_id,
            locator: prepared.record.locator,
            commitment: prepared.record.commitment,
        })
    }
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = storage::get_admin(env).ok_or(Error::NotInitialized)?;
    if admin != *caller {
        return Err(Error::Unauthorized);
    }

    Ok(())
}

struct PreparedAccess {
    grant_id: BytesN<32>,
    grant: Grant,
    record: RecordMeta,
    existing_grant: bool,
}

fn prepare_request_access(
    env: &Env,
    requester: &Address,
    record_id: &BytesN<32>,
    purpose: &Symbol,
    cred: &CredentialProof,
    presence: &PresenceProof,
) -> Result<PreparedAccess, Error> {
    verify_credential(env, requester, cred)?;
    let record = storage::get_record(env, record_id).ok_or(Error::NoSuchRecord)?;
    verify_presence(env, requester, record_id, &record.owner, presence)?;

    let grant_id = request_access_grant_id(env, requester, record_id, purpose, &cred.cred_id);
    if let Some(grant) = storage::get_grant(env, &grant_id) {
        validate_grant(env, requester, record_id, purpose, &record, &grant)?;
        return Ok(PreparedAccess {
            grant_id,
            grant,
            record,
            existing_grant: true,
        });
    }

    let gtype = authorize_new_grant(env, &record, purpose, cred)?;
    let grant = Grant {
        record: record_id.clone(),
        grantee: requester.clone(),
        gtype,
        purpose: purpose.clone(),
        scope_category: record.category.clone(),
        expires_at: env.ledger().timestamp() + REQUEST_GRANT_SECONDS,
        reveal_at: 0,
        revoked: false,
        vetoed: false,
    };

    Ok(PreparedAccess {
        grant_id,
        grant,
        record,
        existing_grant: false,
    })
}

fn request_access_grant_id(
    env: &Env,
    requester: &Address,
    record_id: &BytesN<32>,
    purpose: &Symbol,
    cred_id: &BytesN<32>,
) -> BytesN<32> {
    let preimage = (
        requester.clone(),
        record_id.clone(),
        purpose.clone(),
        cred_id.clone(),
    )
        .to_xdr(env);
    env.crypto().sha256(&preimage).to_bytes()
}

fn explicit_grant_id(
    env: &Env,
    patient: &Address,
    grantee: &Address,
    record_id: &BytesN<32>,
    purpose: &Symbol,
    grant_type: GrantType,
) -> BytesN<32> {
    let preimage = (
        patient.clone(),
        grantee.clone(),
        record_id.clone(),
        purpose.clone(),
        grant_type.code(),
    )
        .to_xdr(env);
    env.crypto().sha256(&preimage).to_bytes()
}

fn verify_credential(env: &Env, requester: &Address, cred: &CredentialProof) -> Result<(), Error> {
    if cred.subject != *requester {
        return Err(Error::CredentialNotForCaller);
    }

    if cred.cred_id == BytesN::from_array(env, &[0; 32]) || !is_known_role(env, &cred.role) {
        return Err(Error::BadCredential);
    }

    Ok(())
}

fn authorize_new_grant(
    env: &Env,
    record: &RecordMeta,
    purpose: &Symbol,
    cred: &CredentialProof,
) -> Result<GrantType, Error> {
    match record.tier {
        Tier::OfflineCard => Err(Error::OfflineTierNotBrokered),
        Tier::EmergencyBundle => {
            if is_responder_role(env, &cred.role) || is_clinician_role(env, &cred.role) {
                Ok(GrantType::BreakGlass)
            } else {
                Err(Error::BadCredential)
            }
        }
        Tier::FullHistory => {
            if !is_clinician_role(env, &cred.role) && !is_institution_role(env, &cred.role) {
                return Err(Error::BadCredential);
            }
            if record.sensitive && *purpose != record.category {
                return Err(Error::ScopeMismatch);
            }
            if record.sensitive {
                return Err(Error::SensitiveNeedsExplicitGrant);
            }
            Ok(GrantType::Normal)
        }
    }
}

fn validate_grant(
    env: &Env,
    requester: &Address,
    record_id: &BytesN<32>,
    purpose: &Symbol,
    record: &RecordMeta,
    grant: &Grant,
) -> Result<(), Error> {
    if grant.record != *record_id || grant.grantee != *requester || grant.purpose != *purpose {
        return Err(Error::NoGrant);
    }
    if grant.revoked || grant.vetoed {
        return Err(Error::GrantRevoked);
    }
    if env.ledger().timestamp() >= grant.expires_at {
        return Err(Error::GrantExpired);
    }
    if env.ledger().timestamp() < grant.reveal_at {
        return Err(Error::NoGrant);
    }
    if record.sensitive && grant.scope_category != record.category {
        return Err(Error::ScopeMismatch);
    }

    Ok(())
}

fn verify_presence(
    env: &Env,
    requester: &Address,
    record_id: &BytesN<32>,
    owner: &Address,
    presence: &PresenceProof,
) -> Result<(), Error> {
    let registered_token =
        storage::get_patient_token(env, owner).ok_or(Error::NoTokenRegistered)?;
    if registered_token != presence.token_pubkey {
        return Err(Error::WrongToken);
    }
    if env.ledger().timestamp() >= presence.expires_at {
        return Err(Error::StalePresence);
    }
    if storage::has_spent_nonce(env, &presence.nonce) {
        return Err(Error::NonceReplayed);
    }
    if presence.signature == BytesN::from_array(env, &[0; 64]) {
        return Err(Error::BadPresenceSig);
    }

    let message = presence_message(env, requester, record_id, presence);
    env.crypto()
        .ed25519_verify(&presence.token_pubkey, &message, &presence.signature);
    Ok(())
}

fn presence_message(
    env: &Env,
    requester: &Address,
    record_id: &BytesN<32>,
    presence: &PresenceProof,
) -> Bytes {
    let domain = Bytes::from_slice(env, b"hcstellar:presence:v1");
    (
        domain,
        requester.clone(),
        record_id.clone(),
        presence.nonce.clone(),
        presence.expires_at,
    )
        .to_xdr(env)
}

fn is_known_role(env: &Env, role: &Symbol) -> bool {
    is_clinician_role(env, role)
        || is_institution_role(env, role)
        || is_responder_role(env, role)
        || *role == Symbol::new(env, "patient")
}

fn is_clinician_role(env: &Env, role: &Symbol) -> bool {
    *role == Symbol::new(env, "clinician")
}

fn is_institution_role(env: &Env, role: &Symbol) -> bool {
    *role == Symbol::new(env, "institution")
}

fn is_responder_role(env: &Env, role: &Symbol) -> bool {
    *role == Symbol::new(env, "responder")
}

#[cfg(test)]
mod test;
