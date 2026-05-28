use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    IdentityContractId,
    AccessBrokerContractId,
    SupplychainContractId,
    Marker(BytesN<32>),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_dependency_ids(
    env: &Env,
    identity_contract_id: &Address,
    access_broker_contract_id: &Address,
    supplychain_contract_id: &Address,
) {
    env.storage()
        .instance()
        .set(&DataKey::IdentityContractId, identity_contract_id);
    env.storage()
        .instance()
        .set(&DataKey::AccessBrokerContractId, access_broker_contract_id);
    env.storage()
        .instance()
        .set(&DataKey::SupplychainContractId, supplychain_contract_id);
}

pub fn get_identity_contract_id(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::IdentityContractId)
}

pub fn get_access_broker_contract_id(env: &Env) -> Option<Address> {
    env.storage()
        .instance()
        .get(&DataKey::AccessBrokerContractId)
}

pub fn get_supplychain_contract_id(env: &Env) -> Option<Address> {
    env.storage()
        .instance()
        .get(&DataKey::SupplychainContractId)
}

pub fn put_marker(env: &Env, marker_id: &BytesN<32>) {
    env.storage()
        .instance()
        .set(&DataKey::Marker(marker_id.clone()), &true);
}

pub fn has_marker(env: &Env, marker_id: &BytesN<32>) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Marker(marker_id.clone()))
        .unwrap_or(false)
}
