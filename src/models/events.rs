use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified event structure for all ERC-8004 events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Option<i64>,
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
    /// Number of blocks to look back
    pub blocks: Option<u64>,

    /// Hours to look back
    pub hours: Option<f64>,

    /// Filter by contract address
    pub contract: Option<String>,

    /// Filter by event type
    pub event_type: Option<String>,

    /// Limit number of results
    pub limit: Option<i64>,
}

impl Default for EventQuery {
    fn default() -> Self {
        Self {
            blocks: Some(100),
            hours: None,
            contract: None,
            event_type: None,
            limit: Some(1000),
        }
    }
}
