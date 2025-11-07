use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified event structure for all ERC-8004 events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Option<i64>,
    pub chain_id: u64,
    pub block_number: u64,
    pub block_timestamp: DateTime<Utc>,
    pub transaction_hash: String,
    pub log_index: u32,
    pub contract_address: String,
    pub event_type: EventType,
    pub event_data: EventData,
    pub created_at: Option<DateTime<Utc>>,
}

/// All possible event types from the three registries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EventType {
    // IdentityRegistry events
    Registered,
    MetadataSet,
    UriUpdated,

    // ReputationRegistry events
    NewFeedback,
    FeedbackRevoked,
    ResponseAppended,

    // ValidationRegistry events
    ValidationRequest,
    ValidationResponse,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Registered => "Registered",
            EventType::MetadataSet => "MetadataSet",
            EventType::UriUpdated => "UriUpdated",
            EventType::NewFeedback => "NewFeedback",
            EventType::FeedbackRevoked => "FeedbackRevoked",
            EventType::ResponseAppended => "ResponseAppended",
            EventType::ValidationRequest => "ValidationRequest",
            EventType::ValidationResponse => "ValidationResponse",
        }
    }
}

/// Event-specific data for each event type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    Registered(RegisteredData),
    MetadataSet(MetadataSetData),
    UriUpdated(UriUpdatedData),
    NewFeedback(NewFeedbackData),
    FeedbackRevoked(FeedbackRevokedData),
    ResponseAppended(ResponseAppendedData),
    ValidationRequest(ValidationRequestData),
    ValidationResponse(ValidationResponseData),
}

// IdentityRegistry events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredData {
    pub agent_id: String,
    pub token_uri: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSetData {
    pub agent_id: String,
    pub indexed_key: String,
    pub key: String,
    pub value: String, // hex encoded bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UriUpdatedData {
    pub agent_id: String,
    pub new_uri: String,
    pub updated_by: String,
}

// ReputationRegistry events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFeedbackData {
    pub agent_id: String,
    pub client: String,
    pub score: u8,
    pub tag1: String,
    pub tag2: String,
    pub feedback_uri: String,
    pub feedback_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRevokedData {
    pub agent_id: String,
    pub client: String,
    pub feedback_index: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseAppendedData {
    pub agent_id: String,
    pub client: String,
    pub feedback_index: String,
    pub responder: String,
    pub response_uri: String,
    pub response_hash: String,
}

// ValidationRegistry events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequestData {
    pub validator_address: String,
    pub agent_id: String,
    pub request_uri: String,
    pub request_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponseData {
    pub validator_address: String,
    pub agent_id: String,
    pub request_hash: String,
    pub response: u8,
    pub response_uri: String,
    pub response_hash: String,
    pub tag: String,
}

/// Query parameters for filtering events
#[derive(Debug, Clone, Deserialize)]
pub struct EventQuery {
    /// Filter by chain ID (REQUIRED for multi-chain support)
    pub chain_id: u64,

    /// Number of blocks to look back
    pub blocks: Option<u64>,

    /// Hours to look back
    pub hours: Option<f64>,

    /// Filter by contract address
    pub contract: Option<String>,

    /// Filter by event type
    pub event_type: Option<String>,

    /// Filter by agent ID
    pub agent_id: Option<String>,

    /// Filter by category (agents, metadata, validation, feedback, all)
    pub category: Option<String>,

    /// Offset for pagination (number of records to skip)
    pub offset: Option<i64>,

    /// Limit number of results
    pub limit: Option<i64>,
}

impl EventQuery {
    /// Get event types for a given category
    /// Returns None if category is "all" or not specified
    pub fn event_types_for_category(&self) -> Option<Vec<&'static str>> {
        match self.category.as_deref() {
            Some("agents") => Some(vec!["Registered"]),
            Some("metadata") => Some(vec!["MetadataSet", "UriUpdated"]),
            Some("validation") => Some(vec!["ValidationRequest", "ValidationResponse"]),
            Some("feedback") => Some(vec!["NewFeedback", "FeedbackRevoked", "ResponseAppended"]),
            Some("all") | None => None, // No filter
            _ => None, // Unknown category, no filter
        }
    }
}

impl Default for EventQuery {
    fn default() -> Self {
        Self {
            chain_id: 11155111, // Sepolia as default
            blocks: Some(100),
            hours: None,
            contract: None,
            event_type: None,
            agent_id: None,
            category: None,
            offset: None,
            limit: Some(1000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::Registered.as_str(), "Registered");
        assert_eq!(EventType::MetadataSet.as_str(), "MetadataSet");
        assert_eq!(EventType::UriUpdated.as_str(), "UriUpdated");
        assert_eq!(EventType::NewFeedback.as_str(), "NewFeedback");
        assert_eq!(EventType::FeedbackRevoked.as_str(), "FeedbackRevoked");
        assert_eq!(EventType::ResponseAppended.as_str(), "ResponseAppended");
        assert_eq!(EventType::ValidationRequest.as_str(), "ValidationRequest");
        assert_eq!(EventType::ValidationResponse.as_str(), "ValidationResponse");
    }

    #[test]
    fn test_event_query_default_values() {
        let query = EventQuery::default();
        assert_eq!(query.chain_id, 11155111);
        assert_eq!(query.blocks, Some(100));
        assert_eq!(query.hours, None);
        assert_eq!(query.contract, None);
        assert_eq!(query.event_type, None);
        assert_eq!(query.agent_id, None);
        assert_eq!(query.offset, None);
        assert_eq!(query.limit, Some(1000));
    }

    #[test]
    fn test_event_query_deserialize_chain_id_required() {
        use serde_urlencoded;

        // Should succeed with chain_id
        let query_string = "chain_id=11155111&limit=10";
        let result: Result<EventQuery, _> = serde_urlencoded::from_str(query_string);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(query.chain_id, 11155111);
        assert_eq!(query.limit, Some(10));

        // Should fail without chain_id
        let query_string = "limit=10&offset=0";
        let result: Result<EventQuery, _> = serde_urlencoded::from_str(query_string);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_query_deserialize_pagination() {
        use serde_urlencoded;

        let query_string = "chain_id=11155111&limit=50&offset=100";
        let query: EventQuery = serde_urlencoded::from_str(query_string).unwrap();

        assert_eq!(query.chain_id, 11155111);
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.offset, Some(100));
    }

    #[test]
    fn test_event_query_deserialize_all_filters() {
        use serde_urlencoded;

        let query_string = "chain_id=1&blocks=200&contract=0x1234&event_type=Registered&agent_id=42&limit=25&offset=50";
        let query: EventQuery = serde_urlencoded::from_str(query_string).unwrap();

        assert_eq!(query.chain_id, 1);
        assert_eq!(query.blocks, Some(200));
        assert_eq!(query.contract, Some("0x1234".to_string()));
        assert_eq!(query.event_type, Some("Registered".to_string()));
        assert_eq!(query.agent_id, Some("42".to_string()));
        assert_eq!(query.limit, Some(25));
        assert_eq!(query.offset, Some(50));
    }

    #[test]
    fn test_event_serialization() {
        use chrono::Utc;

        let event = Event {
            id: Some(1),
            chain_id: 11155111,
            block_number: 12345,
            block_timestamp: Utc::now(),
            transaction_hash: "0xabcd".to_string(),
            log_index: 0,
            contract_address: "0x1234".to_string(),
            event_type: EventType::Registered,
            event_data: EventData::Registered(RegisteredData {
                agent_id: "1".to_string(),
                token_uri: "https://example.com".to_string(),
                owner: "0x5678".to_string(),
            }),
            created_at: Some(Utc::now()),
        };

        // Test serialization
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"chain_id\":11155111"));
        assert!(json.contains("\"block_number\":12345"));
        assert!(json.contains("\"agent_id\":\"1\""));

        // Test deserialization
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.chain_id, 11155111);
        assert_eq!(deserialized.block_number, 12345);
    }

    #[test]
    fn test_registered_data_serialization() {
        let data = RegisteredData {
            agent_id: "123".to_string(),
            token_uri: "https://example.com/token".to_string(),
            owner: "0xowner".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: RegisteredData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.agent_id, "123");
        assert_eq!(deserialized.token_uri, "https://example.com/token");
        assert_eq!(deserialized.owner, "0xowner");
    }

    #[test]
    fn test_metadata_set_data_serialization() {
        let data = MetadataSetData {
            agent_id: "456".to_string(),
            indexed_key: "0xkey".to_string(),
            key: "name".to_string(),
            value: "0xvalue".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: MetadataSetData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.agent_id, "456");
        assert_eq!(deserialized.key, "name");
    }

    #[test]
    fn test_category_mapping_agents() {
        let query = EventQuery {
            chain_id: 11155111,
            category: Some("agents".to_string()),
            blocks: None,
            hours: None,
            contract: None,
            event_type: None,
            agent_id: None,
            offset: None,
            limit: None,
        };

        let event_types = query.event_types_for_category();
        assert_eq!(event_types, Some(vec!["Registered"]));
    }

    #[test]
    fn test_category_mapping_metadata() {
        let query = EventQuery {
            chain_id: 11155111,
            category: Some("metadata".to_string()),
            blocks: None,
            hours: None,
            contract: None,
            event_type: None,
            agent_id: None,
            offset: None,
            limit: None,
        };

        let event_types = query.event_types_for_category();
        assert_eq!(event_types, Some(vec!["MetadataSet", "UriUpdated"]));
    }

    #[test]
    fn test_category_mapping_validation() {
        let query = EventQuery {
            chain_id: 11155111,
            category: Some("validation".to_string()),
            blocks: None,
            hours: None,
            contract: None,
            event_type: None,
            agent_id: None,
            offset: None,
            limit: None,
        };

        let event_types = query.event_types_for_category();
        assert_eq!(event_types, Some(vec!["ValidationRequest", "ValidationResponse"]));
    }

    #[test]
    fn test_category_mapping_feedback() {
        let query = EventQuery {
            chain_id: 11155111,
            category: Some("feedback".to_string()),
            blocks: None,
            hours: None,
            contract: None,
            event_type: None,
            agent_id: None,
            offset: None,
            limit: None,
        };

        let event_types = query.event_types_for_category();
        assert_eq!(event_types, Some(vec!["NewFeedback", "FeedbackRevoked", "ResponseAppended"]));
    }

    #[test]
    fn test_category_mapping_all() {
        let query = EventQuery {
            chain_id: 11155111,
            category: Some("all".to_string()),
            blocks: None,
            hours: None,
            contract: None,
            event_type: None,
            agent_id: None,
            offset: None,
            limit: None,
        };

        let event_types = query.event_types_for_category();
        assert_eq!(event_types, None); // No filter for "all"
    }

    #[test]
    fn test_category_mapping_none() {
        let query = EventQuery {
            chain_id: 11155111,
            category: None,
            blocks: None,
            hours: None,
            contract: None,
            event_type: None,
            agent_id: None,
            offset: None,
            limit: None,
        };

        let event_types = query.event_types_for_category();
        assert_eq!(event_types, None); // No filter when category is None
    }
}
