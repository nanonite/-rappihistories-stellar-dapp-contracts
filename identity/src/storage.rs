use soroban_sdk::{contracttype, Address, BytesN, Env, Vec};

use crate::types::{CredentialRef, IssuerRecord};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,                  // instance storage: Address, always alive
    Issuer(Address),        // persistent storage: IssuerRecord
    Credential(BytesN<32>), // persistent storage: CredentialRef
    SubjectCreds(Address),  // persistent storage: Vec<BytesN<32>>
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_issuer(env: &Env, issuer: &Address, record: &IssuerRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Issuer(issuer.clone()), record);
}

pub fn get_issuer(env: &Env, issuer: &Address) -> Option<IssuerRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Issuer(issuer.clone()))
}

pub fn has_issuer(env: &Env, issuer: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Issuer(issuer.clone()))
}

pub fn set_credential(env: &Env, cred_id: &BytesN<32>, credential: &CredentialRef) {
    env.storage()
        .persistent()
        .set(&DataKey::Credential(cred_id.clone()), credential);
}

pub fn get_credential(env: &Env, cred_id: &BytesN<32>) -> Option<CredentialRef> {
    env.storage()
        .persistent()
        .get(&DataKey::Credential(cred_id.clone()))
}

pub fn has_credential(env: &Env, cred_id: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Credential(cred_id.clone()))
}

pub fn get_subject_creds(env: &Env, subject: &Address) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::SubjectCreds(subject.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_subject_credential(env: &Env, subject: &Address, cred_id: &BytesN<32>) {
    let mut credentials = get_subject_creds(env, subject);
    credentials.push_back(cred_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::SubjectCreds(subject.clone()), &credentials);
}
