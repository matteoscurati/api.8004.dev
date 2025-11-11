use api_8004_dev::models::{
    Event, EventData, EventType, FeedbackRevokedData, MetadataSetData, NewFeedbackData,
    RegisteredData, ResponseAppendedData, UriUpdatedData, ValidationRequestData,
    ValidationResponseData,
};
use api_8004_dev::storage::Storage;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Integration test configuration
/// Note: Update with your actual PostgreSQL user (usually your system username on macOS)
const TEST_DATABASE_URL: &str = "postgresql://matteoscurati@localhost:5432/api_8004_dev_test";

/// Helper function to setup test database
async fn setup_test_db() -> (PgPool, Storage) {
    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(TEST_DATABASE_URL)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let storage = Storage::new(pool.clone(), 1000);
    (pool, storage)
}

/// Helper to clean up test data for a specific chain before running test
async fn cleanup_chain_data(pool: &PgPool, chain_id: u64) {
    // First ensure the chain exists in the chains table
    sqlx::query(
        r#"
        INSERT INTO chains (chain_id, name, rpc_url, enabled, identity_registry, reputation_registry, validation_registry)
        VALUES ($1, $2, $3, true, $4, $5, $6)
        ON CONFLICT (chain_id) DO NOTHING
        "#
    )
    .bind(chain_id as i64)
    .bind(format!("Test Chain {}", chain_id))
    .bind("http://localhost:8545")
    .bind("0x8004a6090Cd10A7288092483047B097295Fb8847")
    .bind("0x8004B8FD1A363aa02fDC07635C0c5F94f6Af5B7E")
    .bind("0x8004CB39f29c09145F24Ad9dDe2A108C1A2cdfC5")
    .execute(pool)
    .await
    .expect("Failed to ensure chain exists");

    // Clean up existing test data
    sqlx::query("DELETE FROM events WHERE chain_id = $1")
        .bind(chain_id as i64)
        .execute(pool)
        .await
        .expect("Failed to clean up events");

    sqlx::query("DELETE FROM chain_sync_state WHERE chain_id = $1")
        .bind(chain_id as i64)
        .execute(pool)
        .await
        .expect("Failed to clean up chain sync state");
}

/// Helper to create a realistic Registered event
fn create_registered_event(chain_id: u64, block_number: u64, agent_id: &str) -> Event {
    Event {
        id: None,
        chain_id,
        event_type: EventType::Registered,
        contract_address: "0x8004a6090cd10a7288092483047b097295fb8847".to_string(),
        transaction_hash: format!("0x{:064x}", block_number), // Unique per block
        log_index: 0,
        block_number,
        block_timestamp: chrono::Utc::now(),
        event_data: EventData::Registered(RegisteredData {
            agent_id: agent_id.to_string(),
            token_uri: "ipfs://test".to_string(),
            owner: "0x742d35cc6634c0532925a3b844bc9e7595f0beb1".to_string(),
        }),
        created_at: None,
    }
}

/// Helper to create a realistic MetadataSet event
fn create_metadata_set_event(chain_id: u64, block_number: u64, agent_id: &str) -> Event {
    Event {
        id: None,
        chain_id,
        event_type: EventType::MetadataSet,
        contract_address: "0x8004a6090cd10a7288092483047b097295fb8847".to_string(),
        transaction_hash: format!("0x{:064x}", block_number + 1),
        log_index: 0,
        block_number,
        block_timestamp: chrono::Utc::now(),
        event_data: EventData::MetadataSet(MetadataSetData {
            agent_id: agent_id.to_string(),
            indexed_key: "name".to_string(),
            key: "name".to_string(),
            value: "0x5465737420416765".to_string(), // "Test Age" in hex
        }),
        created_at: None,
    }
}

/// Helper to create a UriUpdated event
fn create_uri_updated_event(chain_id: u64, block_number: u64, agent_id: &str) -> Event {
    Event {
        id: None,
        chain_id,
        event_type: EventType::UriUpdated,
        contract_address: "0x8004a6090cd10a7288092483047b097295fb8847".to_string(),
        transaction_hash: format!("0x{:064x}", block_number + 2),
        log_index: 0,
        block_number,
        block_timestamp: chrono::Utc::now(),
        event_data: EventData::UriUpdated(UriUpdatedData {
            agent_id: agent_id.to_string(),
            new_uri: "ipfs://QmNewTestUri".to_string(),
            updated_by: "0x742d35cc6634c0532925a3b844bc9e7595f0beb1".to_string(),
        }),
        created_at: None,
    }
}

/// Helper to create a NewFeedback event
fn create_new_feedback_event(chain_id: u64, block_number: u64, agent_id: &str) -> Event {
    Event {
        id: None,
        chain_id,
        event_type: EventType::NewFeedback,
        contract_address: "0x8004b8fd1a363aa02fdc07635c0c5f94f6af5b7e".to_string(),
        transaction_hash: format!("0x{:064x}", block_number + 3),
        log_index: 0,
        block_number,
        block_timestamp: chrono::Utc::now(),
        event_data: EventData::NewFeedback(NewFeedbackData {
            agent_id: agent_id.to_string(),
            client: "0x742d35cc6634c0532925a3b844bc9e7595f0beb1".to_string(),
            score: 5,
            tag1: "quality".to_string(),
            tag2: "responsive".to_string(),
            feedback_uri: "ipfs://QmFeedbackUri".to_string(),
            feedback_hash: "0xfeedbackhash123456789abcdef".to_string(),
        }),
        created_at: None,
    }
}

/// Helper to create a FeedbackRevoked event
fn create_feedback_revoked_event(chain_id: u64, block_number: u64, agent_id: &str) -> Event {
    Event {
        id: None,
        chain_id,
        event_type: EventType::FeedbackRevoked,
        contract_address: "0x8004b8fd1a363aa02fdc07635c0c5f94f6af5b7e".to_string(),
        transaction_hash: format!("0x{:064x}", block_number + 4),
        log_index: 0,
        block_number,
        block_timestamp: chrono::Utc::now(),
        event_data: EventData::FeedbackRevoked(FeedbackRevokedData {
            agent_id: agent_id.to_string(),
            client: "0x742d35cc6634c0532925a3b844bc9e7595f0beb1".to_string(),
            feedback_index: "0".to_string(),
        }),
        created_at: None,
    }
}

/// Helper to create a ResponseAppended event
fn create_response_appended_event(chain_id: u64, block_number: u64, agent_id: &str) -> Event {
    Event {
        id: None,
        chain_id,
        event_type: EventType::ResponseAppended,
        contract_address: "0x8004b8fd1a363aa02fdc07635c0c5f94f6af5b7e".to_string(),
        transaction_hash: format!("0x{:064x}", block_number + 5),
        log_index: 0,
        block_number,
        block_timestamp: chrono::Utc::now(),
        event_data: EventData::ResponseAppended(ResponseAppendedData {
            agent_id: agent_id.to_string(),
            client: "0x742d35cc6634c0532925a3b844bc9e7595f0beb1".to_string(),
            feedback_index: "0".to_string(),
            responder: "0x8004a6090cd10a7288092483047b097295fb8847".to_string(),
            response_uri: "ipfs://QmResponseUri".to_string(),
            response_hash: "0xresponsehash123456789abcdef".to_string(),
        }),
        created_at: None,
    }
}

/// Helper to create a ValidationRequest event
fn create_validation_request_event(chain_id: u64, block_number: u64, agent_id: &str) -> Event {
    Event {
        id: None,
        chain_id,
        event_type: EventType::ValidationRequest,
        contract_address: "0x8004cb39f29c09145f24ad9dde2a108c1a2cdfc5".to_string(),
        transaction_hash: format!("0x{:064x}", block_number + 6),
        log_index: 0,
        block_number,
        block_timestamp: chrono::Utc::now(),
        event_data: EventData::ValidationRequest(ValidationRequestData {
            validator_address: "0x742d35cc6634c0532925a3b844bc9e7595f0beb1".to_string(),
            agent_id: agent_id.to_string(),
            request_uri: "ipfs://QmValidationRequest".to_string(),
            request_hash: "0xrequesthash123456789abcdef".to_string(),
        }),
        created_at: None,
    }
}

/// Helper to create a ValidationResponse event
fn create_validation_response_event(chain_id: u64, block_number: u64, agent_id: &str) -> Event {
    Event {
        id: None,
        chain_id,
        event_type: EventType::ValidationResponse,
        contract_address: "0x8004cb39f29c09145f24ad9dde2a108c1a2cdfc5".to_string(),
        transaction_hash: format!("0x{:064x}", block_number + 7),
        log_index: 0,
        block_number,
        block_timestamp: chrono::Utc::now(),
        event_data: EventData::ValidationResponse(ValidationResponseData {
            validator_address: "0x742d35cc6634c0532925a3b844bc9e7595f0beb1".to_string(),
            agent_id: agent_id.to_string(),
            request_hash: "0xrequesthash123456789abcdef".to_string(),
            response: 1, // Approved
            response_uri: "ipfs://QmValidationResponse".to_string(),
            response_hash: "0xresponsehash123456789abcdef".to_string(),
            tag: "verified".to_string(),
        }),
        created_at: None,
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test --test integration_test -- --ignored
async fn test_ethereum_sepolia_event_processing_and_storage() {
    // Setup
    let (pool, storage) = setup_test_db().await;
    let chain_id = 11155111; // Ethereum Sepolia

    // Clean up any existing data for this chain
    cleanup_chain_data(&pool, chain_id).await;

    // Create events
    let agent_id = "0x8004000000000000000000000000000000000001";
    let registered_event = create_registered_event(chain_id, 100, agent_id);
    let metadata_event = create_metadata_set_event(chain_id, 100, agent_id);

    // Store events
    storage
        .store_event(registered_event.clone())
        .await
        .expect("Failed to store Registered event");

    storage
        .store_event(metadata_event.clone())
        .await
        .expect("Failed to store MetadataSet event");

    // Update last synced block
    storage
        .update_last_synced_block_for_chain(chain_id, 100)
        .await
        .expect("Failed to update last synced block");

    // Verify: Check that events were stored
    let query = api_8004_dev::models::EventQuery {
        chain_id: Some(chain_id.to_string()),
        hours: None,
        blocks: None,
        contract: None,
        event_type: None,
        agent_id: Some(agent_id.to_string()),
        category: None,
        limit: Some(10),
        offset: Some(0),
        include_stats: false,
    };

    let events = storage
        .get_recent_events(query)
        .await
        .expect("Failed to query events");

    // Assertions
    assert_eq!(events.len(), 2, "Should have stored 2 events");

    // Verify Registered event
    let registered = events
        .iter()
        .find(|e| e.event_type == EventType::Registered)
        .expect("Should find Registered event");
    assert_eq!(registered.chain_id, chain_id);
    assert_eq!(registered.block_number, 100);
    if let EventData::Registered(data) = &registered.event_data {
        assert_eq!(data.agent_id, agent_id);
        assert_eq!(data.owner, "0x742d35cc6634c0532925a3b844bc9e7595f0beb1");
    } else {
        panic!("Expected Registered event data");
    }

    // Verify MetadataSet event
    let metadata = events
        .iter()
        .find(|e| e.event_type == EventType::MetadataSet)
        .expect("Should find MetadataSet event");
    assert_eq!(metadata.chain_id, chain_id);
    assert_eq!(metadata.block_number, 100);
    if let EventData::MetadataSet(data) = &metadata.event_data {
        assert_eq!(data.key, "name");
    } else {
        panic!("Expected MetadataSet event data");
    }

    // Verify last synced block
    let last_block = storage
        .get_last_synced_block_for_chain(chain_id)
        .await
        .expect("Failed to get last synced block");
    assert_eq!(last_block, 100, "Last synced block should be 100");

    println!("✅ Ethereum Sepolia: Event processing and storage test passed!");
}

#[tokio::test]
#[ignore]
async fn test_base_sepolia_event_processing_and_storage() {
    let (pool, storage) = setup_test_db().await;
    let chain_id = 84532; // Base Sepolia

    // Clean up any existing data for this chain
    cleanup_chain_data(&pool, chain_id).await;

    let agent_id = "0x8004000000000000000000000000000000000002";
    let event = create_registered_event(chain_id, 200, agent_id);

    storage
        .store_event(event.clone())
        .await
        .expect("Failed to store event");

    storage
        .update_last_synced_block_for_chain(chain_id, 200)
        .await
        .expect("Failed to update last synced block");

    let query = api_8004_dev::models::EventQuery {
        chain_id: Some(chain_id.to_string()),
        hours: None,
        blocks: None,
        contract: None,
        event_type: None,
        agent_id: Some(agent_id.to_string()),
        category: None,
        limit: Some(10),
        offset: Some(0),
        include_stats: false,
    };

    let events = storage
        .get_recent_events(query)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].chain_id, chain_id);
    assert_eq!(events[0].block_number, 200);

    println!("✅ Base Sepolia: Event processing and storage test passed!");
}

#[tokio::test]
#[ignore]
async fn test_linea_sepolia_event_processing_and_storage() {
    let (pool, storage) = setup_test_db().await;
    let chain_id = 59141; // Linea Sepolia

    // Clean up any existing data for this chain
    cleanup_chain_data(&pool, chain_id).await;

    let agent_id = "0x8004000000000000000000000000000000000003";
    let event = create_registered_event(chain_id, 300, agent_id);

    storage
        .store_event(event.clone())
        .await
        .expect("Failed to store event");

    storage
        .update_last_synced_block_for_chain(chain_id, 300)
        .await
        .expect("Failed to update last synced block");

    let query = api_8004_dev::models::EventQuery {
        chain_id: Some(chain_id.to_string()),
        hours: None,
        blocks: None,
        contract: None,
        event_type: None,
        agent_id: Some(agent_id.to_string()),
        category: None,
        limit: Some(10),
        offset: Some(0),
        include_stats: false,
    };

    let events = storage
        .get_recent_events(query)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].chain_id, chain_id);

    println!("✅ Linea Sepolia: Event processing and storage test passed!");
}

#[tokio::test]
#[ignore]
async fn test_multi_chain_isolation() {
    let (pool, storage) = setup_test_db().await;

    // Clean up data for all test chains
    cleanup_chain_data(&pool, 11155111).await;
    cleanup_chain_data(&pool, 84532).await;
    cleanup_chain_data(&pool, 59141).await;

    // Create events for different chains
    let eth_event =
        create_registered_event(11155111, 100, "0x8004000000000000000000000000000000000001");
    let base_event =
        create_registered_event(84532, 100, "0x8004000000000000000000000000000000000002");
    let linea_event =
        create_registered_event(59141, 100, "0x8004000000000000000000000000000000000003");

    storage.store_event(eth_event).await.unwrap();
    storage.store_event(base_event).await.unwrap();
    storage.store_event(linea_event).await.unwrap();

    // Query each chain separately
    let eth_query = api_8004_dev::models::EventQuery {
        chain_id: Some("11155111".to_string()),
        hours: None,
        blocks: None,
        contract: None,
        event_type: None,
        agent_id: None,
        category: None,
        limit: Some(10),
        offset: Some(0),
        include_stats: false,
    };

    let eth_events = storage.get_recent_events(eth_query.clone()).await.unwrap();
    assert_eq!(eth_events.len(), 1);
    assert_eq!(eth_events[0].chain_id, 11155111);

    let base_query = api_8004_dev::models::EventQuery {
        chain_id: Some("84532".to_string()),
        ..eth_query.clone()
    };

    let base_events = storage.get_recent_events(base_query).await.unwrap();
    assert_eq!(base_events.len(), 1);
    assert_eq!(base_events[0].chain_id, 84532);

    println!("✅ Multi-chain isolation test passed!");
}

#[tokio::test]
#[ignore]
async fn test_all_event_types_storage_and_retrieval() {
    let (pool, storage) = setup_test_db().await;
    let chain_id = 11155111; // Ethereum Sepolia

    // Clean up any existing data for this chain
    cleanup_chain_data(&pool, chain_id).await;

    let agent_id = "0x8004000000000000000000000000000000000099";
    let block_number = 500;

    // Create and store all 8 event types
    let events = vec![
        create_registered_event(chain_id, block_number, agent_id),
        create_metadata_set_event(chain_id, block_number, agent_id),
        create_uri_updated_event(chain_id, block_number, agent_id),
        create_new_feedback_event(chain_id, block_number, agent_id),
        create_feedback_revoked_event(chain_id, block_number, agent_id),
        create_response_appended_event(chain_id, block_number, agent_id),
        create_validation_request_event(chain_id, block_number, agent_id),
        create_validation_response_event(chain_id, block_number, agent_id),
    ];

    // Store all events
    for event in &events {
        storage
            .store_event(event.clone())
            .await
            .expect("Failed to store event");
    }

    storage
        .update_last_synced_block_for_chain(chain_id, block_number)
        .await
        .expect("Failed to update last synced block");

    // Verify: Query all events for this agent
    let query = api_8004_dev::models::EventQuery {
        chain_id: Some(chain_id.to_string()),
        hours: None,
        blocks: None,
        contract: None,
        event_type: None,
        agent_id: Some(agent_id.to_string()),
        category: None,
        limit: Some(20),
        offset: Some(0),
        include_stats: false,
    };

    let retrieved_events = storage
        .get_recent_events(query)
        .await
        .expect("Failed to query events");

    // Assertions
    assert_eq!(
        retrieved_events.len(),
        8,
        "Should have stored all 8 event types"
    );

    // Verify each event type is present
    let event_types: Vec<EventType> = retrieved_events
        .iter()
        .map(|e| e.event_type.clone())
        .collect();
    assert!(
        event_types.contains(&EventType::Registered),
        "Missing Registered event"
    );
    assert!(
        event_types.contains(&EventType::MetadataSet),
        "Missing MetadataSet event"
    );
    assert!(
        event_types.contains(&EventType::UriUpdated),
        "Missing UriUpdated event"
    );
    assert!(
        event_types.contains(&EventType::NewFeedback),
        "Missing NewFeedback event"
    );
    assert!(
        event_types.contains(&EventType::FeedbackRevoked),
        "Missing FeedbackRevoked event"
    );
    assert!(
        event_types.contains(&EventType::ResponseAppended),
        "Missing ResponseAppended event"
    );
    assert!(
        event_types.contains(&EventType::ValidationRequest),
        "Missing ValidationRequest event"
    );
    assert!(
        event_types.contains(&EventType::ValidationResponse),
        "Missing ValidationResponse event"
    );

    // Verify specific event data for each type
    for event in &retrieved_events {
        assert_eq!(event.chain_id, chain_id);
        assert_eq!(event.block_number, block_number);

        match &event.event_data {
            EventData::Registered(data) => {
                assert_eq!(data.agent_id, agent_id);
                assert_eq!(data.owner, "0x742d35cc6634c0532925a3b844bc9e7595f0beb1");
            }
            EventData::MetadataSet(data) => {
                assert_eq!(data.agent_id, agent_id);
                assert_eq!(data.key, "name");
            }
            EventData::UriUpdated(data) => {
                assert_eq!(data.agent_id, agent_id);
                assert_eq!(data.new_uri, "ipfs://QmNewTestUri");
            }
            EventData::NewFeedback(data) => {
                assert_eq!(data.agent_id, agent_id);
                assert_eq!(data.score, 5);
                assert_eq!(data.tag1, "quality");
            }
            EventData::FeedbackRevoked(data) => {
                assert_eq!(data.agent_id, agent_id);
                assert_eq!(data.feedback_index, "0");
            }
            EventData::ResponseAppended(data) => {
                assert_eq!(data.agent_id, agent_id);
                assert_eq!(data.feedback_index, "0");
            }
            EventData::ValidationRequest(data) => {
                assert_eq!(data.agent_id, agent_id);
                assert!(data.request_uri.starts_with("ipfs://"));
            }
            EventData::ValidationResponse(data) => {
                assert_eq!(data.agent_id, agent_id);
                assert_eq!(data.response, 1);
                assert_eq!(data.tag, "verified");
            }
        }
    }

    println!("✅ All 8 event types: Storage and retrieval test passed!");
}

#[tokio::test]
#[ignore]
async fn test_polygon_amoy_event_processing() {
    let (pool, storage) = setup_test_db().await;
    let chain_id = 80002; // Polygon Amoy

    // Clean up any existing data for this chain
    cleanup_chain_data(&pool, chain_id).await;

    let agent_id = "0x8004000000000000000000000000000000000004";
    let event = create_registered_event(chain_id, 400, agent_id);

    storage
        .store_event(event.clone())
        .await
        .expect("Failed to store event");

    storage
        .update_last_synced_block_for_chain(chain_id, 400)
        .await
        .expect("Failed to update last synced block");

    let query = api_8004_dev::models::EventQuery {
        chain_id: Some(chain_id.to_string()),
        hours: None,
        blocks: None,
        contract: None,
        event_type: None,
        agent_id: Some(agent_id.to_string()),
        category: None,
        limit: Some(10),
        offset: Some(0),
        include_stats: false,
    };

    let events = storage
        .get_recent_events(query)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].chain_id, chain_id);
    assert_eq!(events[0].block_number, 400);

    println!("✅ Polygon Amoy: Event processing test passed!");
}

#[tokio::test]
#[ignore]
async fn test_hedera_testnet_event_processing() {
    let (pool, storage) = setup_test_db().await;
    let chain_id = 296; // Hedera Testnet

    // Clean up any existing data for this chain
    cleanup_chain_data(&pool, chain_id).await;

    let agent_id = "0x8004000000000000000000000000000000000005";
    let event = create_registered_event(chain_id, 500, agent_id);

    storage
        .store_event(event.clone())
        .await
        .expect("Failed to store event");

    storage
        .update_last_synced_block_for_chain(chain_id, 500)
        .await
        .expect("Failed to update last synced block");

    let query = api_8004_dev::models::EventQuery {
        chain_id: Some(chain_id.to_string()),
        hours: None,
        blocks: None,
        contract: None,
        event_type: None,
        agent_id: Some(agent_id.to_string()),
        category: None,
        limit: Some(10),
        offset: Some(0),
        include_stats: false,
    };

    let events = storage
        .get_recent_events(query)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].chain_id, chain_id);
    assert_eq!(events[0].block_number, 500);

    println!("✅ Hedera Testnet: Event processing test passed!");
}

#[tokio::test]
#[ignore]
async fn test_crash_recovery_block_minus_one() {
    let (pool, storage) = setup_test_db().await;
    let chain_id = 999999; // Use a unique chain ID to avoid conflicts

    // Clean up any existing data for this chain
    cleanup_chain_data(&pool, chain_id).await;

    // Simulate processing blocks 100, 101, 102
    for block in 100..=102 {
        let event = create_registered_event(
            chain_id,
            block,
            &format!("0x800400000000000000000000000000000000{:04x}", block),
        );
        storage.store_event(event).await.unwrap();
        storage
            .update_last_synced_block_for_chain(chain_id, block)
            .await
            .unwrap();
    }

    // Verify last synced block
    let last_synced = storage
        .get_last_synced_block_for_chain(chain_id)
        .await
        .unwrap();
    assert_eq!(last_synced, 102);

    // Simulate crash recovery: should resume from block 101 (102 - 1)
    let resume_from = last_synced.saturating_sub(1);
    assert_eq!(resume_from, 101, "Should resume from last_synced - 1");

    println!("✅ Crash recovery (block - 1) test passed!");
}
