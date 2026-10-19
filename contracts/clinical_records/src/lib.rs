#![no_std]

pub mod admin;
pub mod errors;
pub mod examination;
pub mod prescription;
pub mod types;

#[cfg(test)]
mod test;

pub use errors::ContractError;
pub use types::*;

use soroban_sdk::{contract, contractimpl, Address, Env, String};

#[contract]
pub struct ClinicalRecordsContract;

#[contractimpl]
impl ClinicalRecordsContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        vision_records: Address,
        access_control: Option<Address>,
    ) -> Result<(), ContractError> {
        admin::initialize(&env, admin, vision_records, access_control)
    }

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        admin::get_admin(&env)
    }

    pub fn set_vision_records_contract(
        env: Env,
        admin: Address,
        vision_records: Address,
    ) -> Result<(), ContractError> {
        admin::set_vision_records_contract(&env, admin, vision_records)
    }

    pub fn get_vision_records_contract(env: Env) -> Option<Address> {
        admin::get_vision_records_contract(&env)
    }

    pub fn set_access_control_contract(
        env: Env,
        admin: Address,
        access_control: Address,
    ) -> Result<(), ContractError> {
        admin::set_access_control_contract(&env, admin, access_control)
    }

    pub fn get_access_control_contract(env: Env) -> Option<Address> {
        admin::get_access_control_contract(&env)
    }

    pub fn add_eye_examination(
        env: Env,
        caller: Address,
        record_id: u64,
        visual_acuity: VisualAcuity,
        iop: IntraocularPressure,
        slit_lamp: SlitLampFindings,
        visual_field: OptVisualField,
        retina_imaging: OptRetinalImaging,
        fundus_photo: OptFundusPhotography,
        clinical_notes: String,
    ) -> Result<(), ContractError> {
        examination::add_eye_examination(
            &env,
            caller,
            record_id,
            visual_acuity,
            iop,
            slit_lamp,
            visual_field,
            retina_imaging,
            fundus_photo,
            clinical_notes,
        )
    }

    pub fn set_examination_patient(
        env: Env,
        caller: Address,
        record_id: u64,
        patient: Address,
    ) -> Result<(), ContractError> {
        examination::set_examination_patient(&env, caller, record_id, patient)
    }

    pub fn get_eye_examination(
        env: Env,
        caller: Address,
        record_id: u64,
    ) -> Result<EyeExamination, ContractError> {
        examination::get_eye_examination(&env, caller, record_id)
    }

    pub fn prepare_add_prescription(
        env: Env,
        caller: Address,
        patient: Address,
        provider: Address,
        prescription_data: PrescriptionData,
    ) -> Result<u64, ContractError> {
        prescription::prepare_add_prescription(&env, caller, patient, provider, prescription_data)
    }

    pub fn commit_add_prescription(
        env: Env,
        caller: Address,
        rx_id: u64,
    ) -> Result<(), ContractError> {
        prescription::commit_add_prescription(&env, caller, rx_id)
    }

    pub fn rollback_add_prescription(
        env: Env,
        caller: Address,
        rx_id: u64,
    ) -> Result<(), ContractError> {
        prescription::rollback_add_prescription(&env, caller, rx_id)
    }

    pub fn get_prescription_count(env: Env) -> u64 {
        prescription::get_prescription_count(&env)
    }

    pub fn get_prescription(env: Env, caller: Address, id: u64) -> Result<Prescription, ContractError> {
        prescription::get_prescription(&env, caller, id)
    }
}
