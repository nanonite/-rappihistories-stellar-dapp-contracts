#![no_std]

mod errors;
mod events;
mod storage;
mod types;

use errors::ContractError;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};
use types::MarkerStatus;

#[contract]
pub struct PrescriptionContract;

#[contractimpl]
impl PrescriptionContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();

        if storage::get_admin(&env).is_some() {
            return Err(ContractError::AlreadyInitialized);
        }

        storage::set_admin(&env, &admin);
        Ok(())
    }

    pub fn admin(env: Env) -> Result<Address, ContractError> {
        storage::get_admin(&env).ok_or(ContractError::NotInitialized)
    }

    pub fn configure_dependencies(
        env: Env,
        admin: Address,
        identity_contract_id: Address,
        access_broker_contract_id: Address,
        supplychain_contract_id: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;

        storage::set_dependency_ids(
            &env,
            &identity_contract_id,
            &access_broker_contract_id,
            &supplychain_contract_id,
        );
        Ok(())
    }

    pub fn identity_contract_id(env: Env) -> Result<Address, ContractError> {
        storage::get_identity_contract_id(&env).ok_or(ContractError::NotInitialized)
    }

    pub fn access_broker_contract_id(env: Env) -> Result<Address, ContractError> {
        storage::get_access_broker_contract_id(&env).ok_or(ContractError::NotInitialized)
    }

    pub fn supplychain_contract_id(env: Env) -> Result<Address, ContractError> {
        storage::get_supplychain_contract_id(&env).ok_or(ContractError::NotInitialized)
    }

    pub fn mark(env: Env, marker_id: BytesN<32>) -> Result<MarkerStatus, ContractError> {
        storage::put_marker(&env, &marker_id);
        events::publish_marker(&env, &marker_id);
        Ok(MarkerStatus::Recorded)
    }

    pub fn has_marker(env: Env, marker_id: BytesN<32>) -> bool {
        storage::has_marker(&env, &marker_id)
    }
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), ContractError> {
    let admin = storage::get_admin(env).ok_or(ContractError::NotInitialized)?;
    if admin != *caller {
        return Err(ContractError::Unauthorized);
    }

    Ok(())
}

#[cfg(test)]
mod test;
