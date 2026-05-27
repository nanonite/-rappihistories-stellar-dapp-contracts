#![no_std]

mod errors;
mod events;
mod storage;
mod types;

use errors::ContractError;
use soroban_sdk::{contract, contractimpl, BytesN, Env};
use types::MarkerStatus;

#[contract]
pub struct AccessBrokerContract;

#[contractimpl]
impl AccessBrokerContract {
    pub fn mark(env: Env, marker_id: BytesN<32>) -> Result<MarkerStatus, ContractError> {
        storage::put_marker(&env, &marker_id);
        events::publish_marker(&env, &marker_id);
        Ok(MarkerStatus::Recorded)
    }

    pub fn has_marker(env: Env, marker_id: BytesN<32>) -> bool {
        storage::has_marker(&env, &marker_id)
    }
}

#[cfg(test)]
mod test;
