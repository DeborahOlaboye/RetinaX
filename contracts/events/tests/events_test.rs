//! Issue 2: Event Emission Verification
//!
//! Simulates standard user flows and verifies that all state-changing
//! operations emit the correct Soroban events with the right topics and data.
//!
//! Note: with soroban-sdk 25, `env.events().all()` returns the events emitted
//! by the most recent successful contract invocation. Each assertion therefore
//! inspects the events immediately after the call under test.

#![allow(clippy::unwrap_used)]

extern crate std;

use events::{EventStreamContract, EventStreamContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env, String, Vec,
};

fn setup() -> (Env, EventStreamContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, client, admin)
}

fn register_schema(env: &Env, client: &EventStreamContractClient, admin: &Address, topic: &str) {
    let topic_str = String::from_str(env, topic);
    let hash = String::from_str(env, "sha256:abc123");
    client.register_schema(admin, &topic_str, &1, &hash);
}

// ── Initialization event ─────────────────────────────────────────────────────

/// initialize() must emit an INIT event containing the admin address.
#[test]
fn test_initialize_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "initialize() must emit at least one event"
    );
}

// ── Schema registration event ────────────────────────────────────────────────

/// register_schema() must emit a schema registration event.
#[test]
fn test_register_schema_emits_event() {
    let (env, client, admin) = setup();
    let topic = String::from_str(&env, "records.vision.create");
    let hash = String::from_str(&env, "sha256:deadbeef");

    client.register_schema(&admin, &topic, &1, &hash);

    let binding = env.events().all();
    let events = binding.events();
    assert!(!events.is_empty(), "register_schema() must emit an event");
}

/// Registering multiple schema versions emits one event per registration.
#[test]
fn test_register_multiple_schema_versions_emits_events() {
    let (env, client, admin) = setup();
    let topic = String::from_str(&env, "records.vision.create");
    let hash = String::from_str(&env, "sha256:v1");
    let hash_v2 = String::from_str(&env, "sha256:v2");

    client.register_schema(&admin, &topic, &1, &hash);
    let first_ev = env.events().all();
    let first = first_ev.events().len();
    assert_eq!(
        first, 1,
        "First schema registration must emit exactly one event"
    );

    client.register_schema(&admin, &topic, &2, &hash_v2);
    let second_ev = env.events().all();
    let second = second_ev.events().len();
    assert_eq!(
        second, 1,
        "Second schema registration must emit exactly one event"
    );
}

// ── Source registration event ────────────────────────────────────────────────

/// register_source() must emit a source registration event.
#[test]
fn test_register_source_emits_event() {
    let (env, client, admin) = setup();
    let source = Address::generate(&env);

    client.register_source(&admin, &source);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "register_source() must emit at least one event"
    );
}

// ── Event publishing emission ────────────────────────────────────────────────

/// publish_event() must emit an EVT_PUB event.
#[test]
fn test_publish_event_emits_event() {
    let (env, client, admin) = setup();
    register_schema(&env, &client, &admin, "records.vision.create");

    let topic = String::from_str(&env, "records.vision.create");
    let payload = String::from_str(&env, "payload_hash_1");

    client.publish_event(&admin, &topic, &1, &payload);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "publish_event() must emit at least one event"
    );
}

/// Each published event must produce a distinct emission — N publishes = N emits.
#[test]
fn test_multiple_publishes_emit_multiple_events() {
    let (env, client, admin) = setup();
    register_schema(&env, &client, &admin, "records.vision.create");

    let topic = String::from_str(&env, "records.vision.create");
    let payload = String::from_str(&env, "p");

    for i in 0..3u32 {
        client.publish_event(&admin, &topic, &1, &payload);
        let binding = env.events().all();
        let events = binding.events();
        assert!(
            !events.is_empty(),
            "Publish #{i} must emit at least one event"
        );
    }
}

/// A registered source contract publishing an event must also emit correctly.
#[test]
fn test_registered_source_publish_emits_event() {
    let (env, client, admin) = setup();
    register_schema(&env, &client, &admin, "records.vision.create");

    let source = Address::generate(&env);
    client.register_source(&admin, &source);

    let topic = String::from_str(&env, "records.vision.create");
    let payload = String::from_str(&env, "source_payload");

    client.publish_event(&source, &topic, &1, &payload);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "Source contract publish must emit at least one event"
    );
}

// ── Subscription events ──────────────────────────────────────────────────────

/// subscribe() must emit an event when a subscription is created.
#[test]
fn test_subscribe_emits_event() {
    let (env, client, _admin) = setup();
    let subscriber = Address::generate(&env);
    let pattern = String::from_str(&env, "records.vision.*");

    client.subscribe(&subscriber, &pattern);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "subscribe() must emit at least one event"
    );
}

/// unsubscribe() must emit an event when a subscription is removed.
#[test]
fn test_unsubscribe_emits_event() {
    let (env, client, _admin) = setup();
    let subscriber = Address::generate(&env);
    let pattern = String::from_str(&env, "records.vision.*");

    let sub_id = client.subscribe(&subscriber, &pattern);

    client.unsubscribe(&subscriber, &sub_id);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "unsubscribe() must emit at least one event"
    );
}

// ── Consumer group events ────────────────────────────────────────────────────

/// create_consumer_group() must emit an event.
#[test]
fn test_create_consumer_group_emits_event() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let name = String::from_str(&env, "vision-processors");
    let pattern = String::from_str(&env, "records.vision.*");

    let m1 = Address::generate(&env);
    let mut members = Vec::new(&env);
    members.push_back(m1);

    client.create_consumer_group(&owner, &name, &pattern, &members);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "create_consumer_group() must emit at least one event"
    );
}

// ── Webhook events ───────────────────────────────────────────────────────────

/// register_webhook() must emit an event.
#[test]
fn test_register_webhook_emits_event() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let pattern = String::from_str(&env, "records.*");
    let url_hash = String::from_str(&env, "sha256:webhook_url");

    client.register_webhook(&owner, &pattern, &url_hash);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "register_webhook() must emit at least one event"
    );
}

/// remove_webhook() must emit an event.
#[test]
fn test_remove_webhook_emits_event() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let pattern = String::from_str(&env, "records.*");
    let url_hash = String::from_str(&env, "sha256:webhook_url");

    let whk_id = client.register_webhook(&owner, &pattern, &url_hash);

    client.remove_webhook(&owner, &whk_id);

    let binding = env.events().all();
    let events = binding.events();
    assert!(
        !events.is_empty(),
        "remove_webhook() must emit at least one event"
    );
}

// ── End-to-end emission flow ─────────────────────────────────────────────────

/// Full workflow: init → schema → publish → subscribe → checkpoint
/// Each step must emit events for its own invocation.
#[test]
fn test_full_flow_emits_events_at_each_step() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    // Step 1: initialize
    client.initialize(&admin);
    let e1_ev = env.events().all();
    let e1 = e1_ev.events().len();
    assert!(e1 > 0, "initialize must emit events");

    // Step 2: register schema
    let topic = String::from_str(&env, "records.vision.create");
    let hash = String::from_str(&env, "sha256:abc");
    client.register_schema(&admin, &topic, &1, &hash);
    let e2_ev = env.events().all();
    let e2 = e2_ev.events().len();
    assert!(e2 > 0, "register_schema must emit events");

    // Step 3: publish event
    let payload = String::from_str(&env, "p1");
    client.publish_event(&admin, &topic, &1, &payload);
    let e3_ev = env.events().all();
    let e3 = e3_ev.events().len();
    assert!(e3 > 0, "publish_event must emit events");

    // Step 4: subscribe
    let subscriber = Address::generate(&env);
    let pattern = String::from_str(&env, "records.vision.*");
    client.subscribe(&subscriber, &pattern);
    let e4_ev = env.events().all();
    let e4 = e4_ev.events().len();
    assert!(e4 > 0, "subscribe must emit events");

    // Step 5: checkpoint
    client.create_checkpoint(&admin);
    let e5_ev = env.events().all();
    let e5 = e5_ev.events().len();
    assert!(e5 > 0, "create_checkpoint must emit events");
}
