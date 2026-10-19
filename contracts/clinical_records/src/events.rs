#![allow(deprecated)] // events().publish migration tracked separately

use soroban_sdk::{contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EyeExaminationAddedEvent {
    pub record_id: u64,
    pub caller: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrescriptionPreparedEvent {
    pub rx_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrescriptionCommittedEvent {
    pub rx_id: u64,
    pub caller: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrescriptionRolledBackEvent {
    pub rx_id: u64,
    pub caller: Address,
    pub timestamp: u64,
}

pub fn publish_eye_examination_added(env: &Env, record_id: u64, caller: Address) {
    let topics = (symbol_short!("EXAM_ADD"), record_id, caller.clone());
    let data = EyeExaminationAddedEvent {
        record_id,
        caller,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(topics, data);
}

pub fn publish_prescription_prepared(env: &Env, rx_id: u64, patient: Address, provider: Address) {
    let topics = (
        symbol_short!("RX_PREP"),
        rx_id,
        patient.clone(),
        provider.clone(),
    );
    let data = PrescriptionPreparedEvent {
        rx_id,
        patient,
        provider,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(topics, data);
}

pub fn publish_prescription_committed(env: &Env, rx_id: u64, caller: Address) {
    let topics = (symbol_short!("RX_CMT"), rx_id, caller.clone());
    let data = PrescriptionCommittedEvent {
        rx_id,
        caller,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(topics, data);
}

pub fn publish_prescription_rolled_back(env: &Env, rx_id: u64, caller: Address) {
    let topics = (symbol_short!("RX_RBK"), rx_id, caller.clone());
    let data = PrescriptionRolledBackEvent {
        rx_id,
        caller,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(topics, data);
}
