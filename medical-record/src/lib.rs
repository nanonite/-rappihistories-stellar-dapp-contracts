#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, Map, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Record {
    pub doctor: Address,
    pub timestamp: u64,
    pub data_hash: Bytes,
    pub record_type: String,
    pub notes: String,
}

#[contracttype]
#[derive(Clone)]
pub struct Patient {
    pub authorized_doctors: Map<Address, bool>,
    pub records: Vec<Record>,
    pub owner: Address,
}

#[contracttype]
pub enum DataKey {
    Patient(Address),
}

#[contract]
pub struct MedicalRecordContract;

#[contractimpl]
impl MedicalRecordContract {
    pub fn init(env: Env, patient: Address) {
        let key = DataKey::Patient(patient.clone());
        env.storage().persistent().set(
            &key,
            &Patient {
                authorized_doctors: Map::new(&env),
                records: Vec::new(&env),
                owner: patient,
            },
        );
    }

    pub fn authorize_doctor(env: Env, patient: Address, doctor: Address) {
        patient.require_auth();
        let key = DataKey::Patient(patient.clone());
        let mut state: Patient = env.storage().persistent().get(&key).unwrap();
        state.authorized_doctors.set(doctor, true);
        env.storage().persistent().set(&key, &state);
    }

    pub fn revoke_doctor(env: Env, patient: Address, doctor: Address) {
        patient.require_auth();
        let key = DataKey::Patient(patient.clone());
        let mut state: Patient = env.storage().persistent().get(&key).unwrap();
        state.authorized_doctors.remove(doctor);
        env.storage().persistent().set(&key, &state);
    }

    pub fn append_record(
        env: Env,
        patient: Address,
        doctor: Address,
        data_hash: Bytes,
        record_type: String,
        notes: String,
    ) {
        doctor.require_auth();
        let key = DataKey::Patient(patient.clone());
        let mut state: Patient = env.storage().persistent().get(&key).unwrap();

        let authorized = state
            .authorized_doctors
            .get(doctor.clone())
            .unwrap_or(false);
        assert!(authorized, "Doctor not authorized for this patient");

        state.records.push_back(Record {
            doctor: doctor.clone(),
            timestamp: env.ledger().timestamp(),
            data_hash,
            record_type,
            notes,
        });
        env.storage().persistent().set(&key, &state);
    }

    pub fn get_records(env: Env, patient: Address) -> Vec<Record> {
        let key = DataKey::Patient(patient);
        let state: Patient = env.storage().persistent().get(&key).unwrap();
        state.records
    }

    pub fn get_authorized_doctors(env: Env, patient: Address) -> Vec<Address> {
        let key = DataKey::Patient(patient);
        let state: Patient = env.storage().persistent().get(&key).unwrap();
        state.authorized_doctors.keys()
    }

    pub fn is_doctor_authorized(env: Env, patient: Address, doctor: Address) -> bool {
        let key = DataKey::Patient(patient);
        let state: Patient = env.storage().persistent().get(&key).unwrap();
        state.authorized_doctors.get(doctor).unwrap_or(false)
    }
}

#[cfg(test)]
mod test;
