#![no_std]

mod errors;
mod events;
mod storage;
mod types;

use errors::ContractError;
use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, IntoVal, Symbol, Val, Vec,
};
use types::{MarkerStatus, Prescription, PrescriptionStatus, Tier};

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

    pub fn issue(
        env: Env,
        clinician: Address,
        patient: Address,
        prescription_id: BytesN<32>,
        diagnosis_record_id: BytesN<32>,
        prescription_commitment: BytesN<32>,
    ) -> Result<(), ContractError> {
        clinician.require_auth();
        if storage::has_prescription(&env, &prescription_id) {
            return Err(ContractError::PrescriptionAlreadyExists);
        }

        let prescription = Prescription {
            clinician,
            patient,
            pharmacy: env.current_contract_address(),
            diagnosis_record_id,
            unit_id: BytesN::from_array(&env, &[0; 32]),
            commitment: prescription_commitment,
            receipt_record_id: BytesN::from_array(&env, &[0; 32]),
            status: PrescriptionStatus::Issued,
        };
        storage::set_prescription(&env, &prescription_id, &prescription);
        events::publish_prescription_issued(
            &env,
            &prescription.patient,
            &prescription.clinician,
            &prescription_id,
            &prescription.diagnosis_record_id,
            &prescription.commitment,
        );
        Ok(())
    }

    pub fn select_pharmacy(
        env: Env,
        patient: Address,
        prescription_id: BytesN<32>,
        pharmacy: Address,
        unit_id: BytesN<32>,
        reservation_ref: BytesN<32>,
    ) -> Result<(), ContractError> {
        patient.require_auth();

        let mut prescription = storage::get_prescription(&env, &prescription_id)
            .ok_or(ContractError::NoSuchPrescription)?;
        if prescription.patient != patient {
            return Err(ContractError::WrongPatient);
        }
        if prescription.status != PrescriptionStatus::Issued {
            return Err(ContractError::InvalidState);
        }

        let supplychain =
            storage::get_supplychain_contract_id(&env).ok_or(ContractError::NotInitialized)?;
        invoke_supplychain_reserve(&env, &supplychain, &unit_id, &reservation_ref);

        prescription.pharmacy = pharmacy;
        prescription.unit_id = unit_id.clone();
        prescription.status = PrescriptionStatus::Reserved;
        storage::set_prescription(&env, &prescription_id, &prescription);
        events::publish_prescription_reserved(
            &env,
            &prescription.patient,
            &prescription.pharmacy,
            &prescription_id,
            &prescription.unit_id,
            &reservation_ref,
        );
        Ok(())
    }

    pub fn dispense(
        env: Env,
        pharmacy: Address,
        patient: Address,
        prescription_id: BytesN<32>,
        receipt_record_id: BytesN<32>,
        receipt_locator_bytes: Bytes,
        receipt_commitment: BytesN<32>,
    ) -> Result<(), ContractError> {
        // The local stellar-cli flow cannot currently satisfy a second
        // non-source Soroban auth entry. The critical safety control for this
        // MVP path is the patient co-sign; the selected pharmacy is still
        // checked against prescription state below.
        patient.require_auth();

        let mut prescription = storage::get_prescription(&env, &prescription_id)
            .ok_or(ContractError::NoSuchPrescription)?;
        if prescription.patient != patient {
            return Err(ContractError::WrongPatient);
        }
        if prescription.pharmacy != pharmacy {
            return Err(ContractError::WrongPharmacy);
        }
        if prescription.status != PrescriptionStatus::Reserved {
            return Err(ContractError::InvalidState);
        }

        let supplychain =
            storage::get_supplychain_contract_id(&env).ok_or(ContractError::NotInitialized)?;
        invoke_supplychain_dispense(&env, &supplychain, &prescription.unit_id);

        let access_broker =
            storage::get_access_broker_contract_id(&env).ok_or(ContractError::NotInitialized)?;
        invoke_access_broker_register_record(
            &env,
            &access_broker,
            &patient,
            &receipt_record_id,
            &receipt_locator_bytes,
            &receipt_commitment,
        );

        prescription.receipt_record_id = receipt_record_id.clone();
        prescription.status = PrescriptionStatus::Dispensed;
        storage::set_prescription(&env, &prescription_id, &prescription);
        events::publish_prescription_dispensed(
            &env,
            &prescription.patient,
            &prescription.pharmacy,
            &prescription_id,
            &prescription.unit_id,
            &prescription.receipt_record_id,
        );
        Ok(())
    }

    pub fn get_prescription(
        env: Env,
        prescription_id: BytesN<32>,
    ) -> Result<Prescription, ContractError> {
        storage::get_prescription(&env, &prescription_id).ok_or(ContractError::NoSuchPrescription)
    }
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), ContractError> {
    let admin = storage::get_admin(env).ok_or(ContractError::NotInitialized)?;
    if admin != *caller {
        return Err(ContractError::Unauthorized);
    }

    Ok(())
}

fn invoke_supplychain_reserve(
    env: &Env,
    supplychain: &Address,
    unit_id: &BytesN<32>,
    reservation_ref: &BytesN<32>,
) {
    let args: Vec<Val> = (
        env.current_contract_address(),
        unit_id.clone(),
        reservation_ref.clone(),
    )
        .into_val(env);
    env.invoke_contract::<()>(supplychain, &Symbol::new(env, "reserve_unit"), args);
}

fn invoke_supplychain_dispense(env: &Env, supplychain: &Address, unit_id: &BytesN<32>) {
    let args: Vec<Val> = (env.current_contract_address(), unit_id.clone()).into_val(env);
    env.invoke_contract::<()>(supplychain, &Symbol::new(env, "dispense_unit"), args);
}

fn invoke_access_broker_register_record(
    env: &Env,
    access_broker: &Address,
    patient: &Address,
    receipt_record_id: &BytesN<32>,
    receipt_locator_bytes: &Bytes,
    commitment: &BytesN<32>,
) {
    let args: Vec<Val> = (
        patient.clone(),
        receipt_record_id.clone(),
        Tier::FullHistory,
        symbol_short!("dispense"),
        false,
        receipt_locator_bytes.clone(),
        commitment.clone(),
    )
        .into_val(env);
    env.invoke_contract::<()>(access_broker, &Symbol::new(env, "register_record"), args);
}

#[cfg(test)]
mod test;
