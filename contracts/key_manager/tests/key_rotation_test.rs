//! Key rotation edge-case tests (issue #260).
//!
//! Covers: master rotation preserving derived keys, rotation during active
//! sessions, immediate revocation denial + revocation cascade semantics,
//! attestation across rotations, and sequential (concurrent-admin equivalent)
//! rotations.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use key_manager::{
    ContractError, KeyLevel, KeyManagerContract, KeyManagerContractClient, KeyPolicy, KeyStatus,
    KeyType,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, Vec,
};

fn setup() -> (Env, KeyManagerContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(KeyManagerContract, ());
    let client = KeyManagerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let identity = Address::generate(&env);
    client.initialize(&admin, &identity);
    (env, client, admin)
}

fn unrestricted_policy(env: &Env) -> KeyPolicy {
    KeyPolicy {
        max_uses: 0,
        not_before: 0,
        not_after: 0,
        allowed_ops: Vec::new(env),
    }
}

fn make_master(
    env: &Env,
    client: &KeyManagerContractClient,
    admin: &Address,
    rotation_interval: u64,
    seed: u8,
) -> BytesN<32> {
    let key_bytes = BytesN::from_array(env, &[seed; 32]);
    client.create_master_key(
        admin,
        &KeyType::Encryption,
        &unrestricted_policy(env),
        &rotation_interval,
        &key_bytes,
    )
}

#[test]
fn test_rotate_master_preserves_derived_keys() {
    let (env, client, admin) = setup();
    let master = make_master(&env, &client, &admin, 0, 7);

    // Build a derivation chain before rotation.
    let policy = unrestricted_policy(&env);
    let contract_key = client.derive_key(
        &admin,
        &master,
        &KeyLevel::Contract,
        &0u32,
        &false,
        &KeyType::Encryption,
        &policy,
        &0u64,
    );
    let op_key = client.derive_key(
        &admin,
        &contract_key,
        &KeyLevel::Operation,
        &0u32,
        &false,
        &KeyType::Signing,
        &policy,
        &0u64,
    );

    let pre_v1 = client.derive_record_key(&master, &1u64);
    assert_eq!(pre_v1.version, 1);

    // Rotate master: version bumps, material changes.
    let v2 = client.rotate_key(&admin, &master);
    assert_eq!(v2, 2);
    let post = client.derive_record_key(&master, &1u64);
    assert_eq!(post.version, 2);
    assert_ne!(pre_v1.key, post.key);

    // Old version is still retrievable for decrypting historical records.
    let historic = client.derive_record_key_with_version(&master, &1u64, &1u32);
    assert_eq!(historic.key, pre_v1.key);
    assert_eq!(historic.version, 1);

    // Previously derived children keep working after parent rotation.
    let used = client.use_key(&admin, &op_key, &symbol_short!("SIGN"));
    assert_eq!(used.to_array().len(), 32);
    let contract_rec = client.get_key_record(&contract_key).unwrap();
    assert_eq!(contract_rec.status, KeyStatus::Active);
}

#[test]
fn test_rotation_during_active_session() {
    let (env, client, admin) = setup();
    let master = make_master(&env, &client, &admin, 0, 21);

    // Active session: repeated uses before rotation.
    for _ in 0..3 {
        assert!(client
            .try_use_key(&admin, &master, &symbol_short!("ENC"))
            .is_ok());
    }
    let uses_before = client.get_key_record(&master).unwrap().uses;

    // Rotate mid-session; the same key id stays usable with a new version.
    let v2 = client.rotate_key(&admin, &master);
    assert_eq!(v2, 2);
    let after = client.use_key(&admin, &master, &symbol_short!("ENC"));
    assert_eq!(after.to_array().len(), 32);

    let record = client.get_key_record(&master).unwrap();
    assert_eq!(record.current_version, 2);
    assert!(record.uses > uses_before);
    assert!(record.last_rotated >= env.ledger().timestamp());
}

#[test]
fn test_revocation_denies_immediately_and_blocks_cascade() {
    let (env, client, admin) = setup();
    let master = make_master(&env, &client, &admin, 0, 33);

    let policy = unrestricted_policy(&env);
    let child = client.derive_key(
        &admin,
        &master,
        &KeyLevel::Contract,
        &1u32,
        &false,
        &KeyType::Encryption,
        &policy,
        &0u64,
    );

    // Revoke master: immediate denial for use, rotate, and new derivations.
    client.revoke_key(&admin, &master);
    assert_eq!(
        client.try_use_key(&admin, &master, &symbol_short!("ENC")),
        Err(Ok(ContractError::KeyRevoked))
    );
    assert_eq!(
        client.try_rotate_key(&admin, &master),
        Err(Ok(ContractError::KeyRevoked))
    );
    assert_eq!(
        client.try_derive_key(
            &admin,
            &master,
            &KeyLevel::Contract,
            &9u32,
            &false,
            &KeyType::Encryption,
            &policy,
            &0u64,
        ),
        Err(Ok(ContractError::KeyRevoked))
    );
    assert_eq!(
        client.get_key_record(&master).unwrap().status,
        KeyStatus::Revoked
    );

    // Existing child is not auto-revoked (no silent cascade), but revoking it
    // denies immediately too.
    assert!(client
        .try_use_key(&admin, &child, &symbol_short!("ENC"))
        .is_ok());
    client.revoke_key(&admin, &child);
    assert_eq!(
        client.try_use_key(&admin, &child, &symbol_short!("ENC")),
        Err(Ok(ContractError::KeyRevoked))
    );
}

#[test]
fn test_attestation_changes_across_rotation_but_history_verifiable() {
    let (env, client, admin) = setup();
    let master = make_master(&env, &client, &admin, 0, 44);

    let attest_v1 = client.attest_key(&master);
    client.rotate_key(&admin, &master);
    let attest_v2 = client.attest_key(&master);
    assert_ne!(attest_v1, attest_v2);

    // Both versions remain retrievable so historical attestations verify.
    assert!(client.get_key_version(&master, &1u32).is_some());
    assert!(client.get_key_version(&master, &2u32).is_some());
    assert_ne!(
        client.get_key_version(&master, &1u32).unwrap().key_bytes,
        client.get_key_version(&master, &2u32).unwrap().key_bytes
    );
}

#[test]
fn test_sequential_rotations_by_owner_and_admin_are_consistent() {
    let (env, client, admin) = setup();
    // Soroban executes sequentially, so rapid back-to-back rotations are the
    // concurrent-admin equivalent: no lost updates, monotonic versions.
    let master = make_master(&env, &client, &admin, 0, 55);

    let v2 = client.rotate_key(&admin, &master);
    let v3 = client.rotate_key(&admin, &master);
    assert_eq!((v2, v3), (2, 3));

    let record = client.get_key_record(&master).unwrap();
    assert_eq!(record.current_version, 3);

    // A stranger (neither owner nor admin) still cannot rotate.
    let stranger = Address::generate(&env);
    assert_eq!(
        client.try_rotate_key(&stranger, &master),
        Err(Ok(ContractError::Unauthorized))
    );
}

#[test]
fn test_rotation_interval_enforced_then_due_after_time() {
    let (env, client, admin) = setup();
    let master = make_master(&env, &client, &admin, 3600, 66);

    // Immediate rotation is not due.
    assert_eq!(
        client.try_rotate_key(&admin, &master),
        Err(Ok(ContractError::RotationNotDue))
    );

    // Advance past the interval: rotation succeeds.
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + 3601);
    assert_eq!(client.rotate_key(&admin, &master), 2);
}
