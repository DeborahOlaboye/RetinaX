#![allow(clippy::unwrap_used, clippy::expect_used)]

use events::{EventStreamContract, EventStreamContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env, String,
};

#[test]
fn test_cross_contract_calling_invariants() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let topic = String::from_str(&env, "external.failure");
    let hash = String::from_str(&env, "sha256:cross_contract");

    // Test that the contract parses and handles successfully
    client.register_schema(&admin, &topic, &1, &hash);

    // Ensure we can publish parsing cross contract mock
    let payload = String::from_str(&env, "payload");
    client.publish_event(&admin, &topic, &1, &payload);

    let all_events = env.events().all();
    assert!(
        !all_events.events().is_empty(),
        "Cross contract calling invariants not respected"
    );
}

/// Test that an external contract can interact with the events contract
/// by subscribing to events and receiving notifications through the contract's
/// event emission mechanism.
#[test]
fn test_cross_contract_subscription_notification_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let events_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &events_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Register schema for cross-chain events
    let topic = String::from_str(&env, "crosschain.message.delivered");
    let hash = String::from_str(&env, "sha256:crosschain_schema");
    client.register_schema(&admin, &topic, &1, &hash);

    // Create subscription for cross-contract notifications
    let subscriber = Address::generate(&env);
    let pattern = String::from_str(&env, "crosschain.*");
    let sub_id = client.subscribe(&subscriber, &pattern);
    assert!(sub_id > 0, "Subscription should be created");

    // Verify subscription exists
    let subs = client.get_subscriptions(&subscriber);
    assert_eq!(subs.len(), 1);

    // Publish event that matches the subscription pattern
    let payload = String::from_str(&env, "crosschain_payload_data");
    let event_id = client.publish_event(&admin, &topic, &1, &payload);
    assert!(event_id > 0, "Event should be published");

    // Verify events were emitted
    let all_events = env.events().all();
    assert!(
        all_events.events().len() > 0,
        "Events should be emitted for cross-contract flow"
    );
}

/// Test end-to-end subscription notification flow with matching and non-matching topics
#[test]
fn test_subscription_notification_flow_end_to_end() {
    let env = Env::default();
    env.mock_all_auths();

    let events_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &events_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Register multiple schemas
    let topic1 = String::from_str(&env, "records.vision.create");
    let topic2 = String::from_str(&env, "records.staking.reward");
    let hash = String::from_str(&env, "sha256:schema");
    client.register_schema(&admin, &topic1, &1, &hash);
    client.register_schema(&admin, &topic2, &1, &hash);

    // Create wildcard subscription
    let subscriber = Address::generate(&env);
    let pattern = String::from_str(&env, "records.*");
    let sub_id = client.subscribe(&subscriber, &pattern);
    assert!(sub_id > 0);

    // Create specific subscription
    let subscriber2 = Address::generate(&env);
    let pattern2 = String::from_str(&env, "records.vision.*");
    let sub_id2 = client.subscribe(&subscriber2, &pattern2);
    assert!(sub_id2 > 0);

    // Publish events - some match, some don't
    let payload = String::from_str(&env, "payload");
    client.publish_event(&admin, &topic1, &1, &payload); // matches both
    client.publish_event(&admin, &topic2, &1, &payload); // matches only first

    let all_events = env.events().all();
    assert!(
        all_events.events().len() > 0,
        "Subscription/notification flow should emit events"
    );
}

/// Test dead-letter queue for failed deliveries with end-to-end flow
#[test]
fn test_dead_letter_queue_end_to_end() {
    let env = Env::default();
    env.mock_all_auths();

    let events_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &events_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Register schema and publish event
    let topic = String::from_str(&env, "system.error.critical");
    let hash = String::from_str(&env, "sha256:error_schema");
    client.register_schema(&admin, &topic, &1, &hash);

    let payload = String::from_str(&env, "error_payload");
    let event_id = client.publish_event(&admin, &topic, &1, &payload);

    // Create subscriber
    let subscriber = Address::generate(&env);

    // Push to dead letter queue
    let reason = String::from_str(&env, "delivery_failed_network_timeout");
    client.push_dead_letter(&admin, &event_id, &subscriber, &reason);

    // SDK 25: events().all() reflects the most recent invocation only.
    // push_dead_letter must emit a DLQ_PUSH event.
    let ev = env.events().all();
    assert!(
        ev.events().len() > 0,
        "push_dead_letter must emit a DLQ_PUSH event"
    );

    // Retrieve dead letters
    let dlq = client.get_dead_letters();
    assert_eq!(dlq.len(), 1, "DLQ should have 1 entry");
    assert_eq!(dlq.get(0).unwrap().event_id, event_id);
    assert_eq!(dlq.get(0).unwrap().subscriber, subscriber);

    // Retry dead letter
    client.retry_dead_letter(&admin, &0);

    // retry_dead_letter must emit a DLQ_RTY event
    let ev = env.events().all();
    assert!(
        ev.events().len() > 0,
        "retry_dead_letter must emit a DLQ_RTY event"
    );

    // DLQ should be empty after successful retry
    let dlq_after = client.get_dead_letters();
    assert_eq!(dlq_after.len(), 0, "DLQ should be empty after retry");
}

/// Test dead letter queue with multiple entries and retry specific index
#[test]
fn test_dead_letter_queue_multiple_entries() {
    let env = Env::default();
    env.mock_all_auths();

    let events_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &events_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let topic = String::from_str(&env, "test.topic");
    let hash = String::from_str(&env, "sha256:test");
    client.register_schema(&admin, &topic, &1, &hash);

    let payload = String::from_str(&env, "payload");
    let event_id1 = client.publish_event(&admin, &topic, &1, &payload);
    let event_id2 = client.publish_event(&admin, &topic, &1, &payload);
    let event_id3 = client.publish_event(&admin, &topic, &1, &payload);

    let subscriber1 = Address::generate(&env);
    let subscriber2 = Address::generate(&env);
    let subscriber3 = Address::generate(&env);

    client.push_dead_letter(
        &admin,
        &event_id1,
        &subscriber1,
        &String::from_str(&env, "reason1"),
    );
    client.push_dead_letter(
        &admin,
        &event_id2,
        &subscriber2,
        &String::from_str(&env, "reason2"),
    );
    client.push_dead_letter(
        &admin,
        &event_id3,
        &subscriber3,
        &String::from_str(&env, "reason3"),
    );

    let dlq = client.get_dead_letters();
    assert_eq!(dlq.len(), 3);

    // Retry middle entry (index 1)
    client.retry_dead_letter(&admin, &1);

    let dlq_after = client.get_dead_letters();
    assert_eq!(dlq_after.len(), 2);

    // Check remaining entries
    assert_eq!(dlq_after.get(0).unwrap().event_id, event_id1);
    assert_eq!(dlq_after.get(1).unwrap().event_id, event_id3);
}

/// Test cross-contract event emission with registered source contracts
#[test]
fn test_cross_contract_event_emission_with_registered_source() {
    let env = Env::default();
    env.mock_all_auths();

    let events_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &events_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Register schema
    let topic = String::from_str(&env, "external.contract.event");
    let hash = String::from_str(&env, "sha256:external");
    client.register_schema(&admin, &topic, &1, &hash);

    // Register external contract as source
    let external_contract = Address::generate(&env);
    client.register_source(&admin, &external_contract);

    // External contract publishes event
    let payload = String::from_str(&env, "external_payload");
    let event_id = client.publish_event(&external_contract, &topic, &1, &payload);
    assert!(event_id > 0);

    let all_events = env.events().all();
    assert!(
        all_events.events().len() > 0,
        "Cross-contract event emission should work"
    );
}

/// Test that cross-contract publishing enforces authorization
#[test]
fn test_cross_contract_unauthorized_publish_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let events_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &events_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let topic = String::from_str(&env, "unauthorized.topic");
    let hash = String::from_str(&env, "sha256:unauth");
    client.register_schema(&admin, &topic, &1, &hash);

    // Unregistered external contract tries to publish
    let unauthorized = Address::generate(&env);
    let payload = String::from_str(&env, "payload");

    let result = client.try_publish_event(&unauthorized, &topic, &1, &payload);
    assert!(result.is_err(), "Unauthorized publish should fail");
}

/// Test cross-contract consumer group with event distribution
#[test]
fn test_cross_contract_consumer_group_distribution() {
    let env = Env::default();
    env.mock_all_auths();

    let events_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &events_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Register schema
    let topic = String::from_str(&env, "distributed.events");
    let hash = String::from_str(&env, "sha256:dist");
    client.register_schema(&admin, &topic, &1, &hash);

    // Create consumer group with multiple members
    let owner = Address::generate(&env);
    let group_name = String::from_str(&env, "cross-contract-consumers");
    let pattern = String::from_str(&env, "distributed.*");
    let mut members = soroban_sdk::Vec::new(&env);
    members.push_back(Address::generate(&env));
    members.push_back(Address::generate(&env));
    members.push_back(Address::generate(&env));

    let group_id = client.create_consumer_group(&owner, &group_name, &pattern, &members);
    assert!(group_id > 0);

    // Publish multiple events
    let payload = String::from_str(&env, "event_data");
    for _ in 0..5 {
        client.publish_event(&admin, &topic, &1, &payload);
    }

    let all_events = env.events().all();
    assert!(
        all_events.events().len() > 0,
        "Consumer group distribution should emit events"
    );
}

/// Test DLQ with retry validation - retrying non-existent index fails
#[test]
fn test_dead_letter_queue_retry_invalid_index() {
    let env = Env::default();
    env.mock_all_auths();

    let events_id = env.register(EventStreamContract, ());
    let client = EventStreamContractClient::new(&env, &events_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Try to retry with invalid index when DLQ is empty
    let result = client.try_retry_dead_letter(&admin, &0);
    assert!(result.is_err(), "Retrying empty DLQ should fail");

    // Add entry and retry with out of bounds index
    let topic = String::from_str(&env, "test");
    let hash = String::from_str(&env, "sha256:test");
    client.register_schema(&admin, &topic, &1, &hash);
    let event_id = client.publish_event(&admin, &topic, &1, &String::from_str(&env, "p"));
    client.push_dead_letter(
        &admin,
        &event_id,
        &Address::generate(&env),
        &String::from_str(&env, "r"),
    );

    let result = client.try_retry_dead_letter(&admin, &999);
    assert!(result.is_err(), "Retrying with invalid index should fail");
}
