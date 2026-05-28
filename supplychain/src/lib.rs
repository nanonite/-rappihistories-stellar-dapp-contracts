#![no_std]

mod errors;
mod events;
mod storage;
mod types;

use errors::ContractError;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};
use types::{MarkerStatus, Unit, UnitStatus};

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

    pub fn configure_prescription_contract(
        env: Env,
        admin: Address,
        prescription_contract: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;

        storage::set_prescription_contract(&env, &prescription_contract);
        Ok(())
    }

    pub fn prescription_contract(env: Env) -> Result<Address, ContractError> {
        storage::get_prescription_contract(&env).ok_or(ContractError::NotInitialized)
    }

    pub fn register_unit(
        env: Env,
        oracle: Address,
        unit_id: BytesN<32>,
        batch_id: BytesN<32>,
    ) -> Result<(), ContractError> {
        oracle.require_auth();
        if !storage::is_oracle(&env, &oracle) {
            return Err(ContractError::Unauthorized);
        }
        if storage::has_unit(&env, &unit_id) {
            return Err(ContractError::UnitAlreadyExists);
        }

        let unit = Unit {
            batch_id: batch_id.clone(),
            reservation_ref: BytesN::from_array(&env, &[0; 32]),
            status: UnitStatus::Available,
        };
        storage::set_unit(&env, &unit_id, &unit);
        events::publish_unit_registered(&env, &unit_id, &batch_id);
        Ok(())
    }

    pub fn quarantine_batch(
        env: Env,
        oracle: Address,
        batch_id: BytesN<32>,
    ) -> Result<(), ContractError> {
        oracle.require_auth();
        if !storage::is_oracle(&env, &oracle) {
            return Err(ContractError::Unauthorized);
        }

        storage::set_batch_quarantined(&env, &batch_id);
        events::publish_batch_quarantined(&env, &batch_id);
        Ok(())
    }

    pub fn reserve_unit(
        env: Env,
        prescription_contract: Address,
        unit_id: BytesN<32>,
        reservation_ref: BytesN<32>,
    ) -> Result<(), ContractError> {
        prescription_contract.require_auth();
        require_prescription_contract(&env, &prescription_contract)?;

        let mut unit = storage::get_unit(&env, &unit_id).ok_or(ContractError::NoSuchUnit)?;
        if storage::is_batch_quarantined(&env, &unit.batch_id) {
            return Err(ContractError::BatchQuarantined);
        }
        if unit.status != UnitStatus::Available {
            return Err(ContractError::UnitUnavailable);
        }

        unit.status = UnitStatus::Reserved;
        unit.reservation_ref = reservation_ref.clone();
        storage::set_unit(&env, &unit_id, &unit);
        events::publish_unit_reserved(&env, &unit_id, &reservation_ref);
        Ok(())
    }

    pub fn dispense_unit(
        env: Env,
        prescription_contract: Address,
        unit_id: BytesN<32>,
    ) -> Result<(), ContractError> {
        prescription_contract.require_auth();
        require_prescription_contract(&env, &prescription_contract)?;

        let mut unit = storage::get_unit(&env, &unit_id).ok_or(ContractError::NoSuchUnit)?;
        if unit.status != UnitStatus::Reserved {
            return Err(ContractError::UnitUnavailable);
        }

        unit.status = UnitStatus::Dispensed;
        storage::set_unit(&env, &unit_id, &unit);
        events::publish_unit_dispensed(&env, &unit_id);
        Ok(())
    }

    pub fn release_reservation(
        env: Env,
        prescription_contract: Address,
        unit_id: BytesN<32>,
    ) -> Result<(), ContractError> {
        prescription_contract.require_auth();
        require_prescription_contract(&env, &prescription_contract)?;

        let mut unit = storage::get_unit(&env, &unit_id).ok_or(ContractError::NoSuchUnit)?;
        if unit.status != UnitStatus::Reserved {
            return Err(ContractError::UnitUnavailable);
        }

        unit.status = UnitStatus::Available;
        unit.reservation_ref = BytesN::from_array(&env, &[0; 32]);
        storage::set_unit(&env, &unit_id, &unit);
        Ok(())
    }

    pub fn get_unit(env: Env, unit_id: BytesN<32>) -> Result<Unit, ContractError> {
        storage::get_unit(&env, &unit_id).ok_or(ContractError::NoSuchUnit)
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

fn require_prescription_contract(env: &Env, caller: &Address) -> Result<(), ContractError> {
    let configured =
        storage::get_prescription_contract(env).ok_or(ContractError::NotInitialized)?;
    if configured != *caller {
        return Err(ContractError::WrongPrescriptionContract);
    }

    Ok(())
}

#[cfg(test)]
mod test;
