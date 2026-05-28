#![no_std]

mod errors;
mod events;
mod storage;
mod types;

use errors::ContractError;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};
use types::MarkerStatus;

#[contract]
pub struct SupplychainContract;

#[contractimpl]
impl SupplychainContract {
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

    pub fn register_oracle(env: Env, admin: Address, oracle: Address) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;

        storage::set_oracle(&env, &oracle);
        Ok(())
    }

    pub fn is_oracle(env: Env, oracle: Address) -> bool {
        storage::is_oracle(&env, &oracle)
    }

    pub fn register_attester(
        env: Env,
        admin: Address,
        attester_pubkey: BytesN<32>,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;

        storage::set_attester(&env, &attester_pubkey);
        Ok(())
    }

    pub fn is_attester(env: Env, attester_pubkey: BytesN<32>) -> bool {
        storage::is_attester(&env, &attester_pubkey)
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
