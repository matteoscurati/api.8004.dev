# API Examples - Category Filtering

## Authentication

First, get a JWT token:

```bash
curl -X POST "https://api-8004-dev.fly.dev/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'
```

Response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_at": "2025-11-08T16:00:00Z"
}
```

## Category Filtering Examples

### 1. All Events (No Category Filter)

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&limit=20"
```

**Returns:** All event types (Registered, MetadataSet, UriUpdated, ValidationRequest, ValidationResponse, NewFeedback, FeedbackRevoked, ResponseAppended)

**Use Case:** Dashboard overview, complete event history

---

### 2. Agents Category

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=agents&limit=20"
```

**Filters for:**
- `Registered` - Agent registration events

**Use Case:** Agent onboarding tracking, new agent discovery

**Example Response:**
```json
{
  "count": 20,
  "total": 1086,
  "stats": {
    "all": 3649,
    "agents": 1086,
    "metadata": 2336,
    "validation": 99,
    "feedback": 128,
    "capabilities": 0,
    "payments": 0
  },
  "events": [
    {
      "event_type": {"type": "Registered"},
      "event_data": {
        "agent_id": "765",
        "owner": "0xcc44d0e7f25403ecbe2702cae2daffe1b510c1ad",
        "token_uri": "ipfs://..."
      },
      "block_number": 9534993,
      "transaction_hash": "0x0e8b43d86e5b95180e832829f665d84be1f3f2ff...",
      ...
    }
  ]
}
```

---

### 3. Metadata Category

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=metadata&limit=20"
```

**Filters for:**
- `MetadataSet` - Metadata updates (name, description, etc.)
- `UriUpdated` - URI/endpoint changes

**Use Case:** Agent profile updates, configuration changes tracking

**Example Response:**
```json
{
  "count": 20,
  "total": 2336,
  "stats": {
    "all": 3649,
    "agents": 1086,
    "metadata": 2336,
    "validation": 99,
    "feedback": 128,
    "capabilities": 0,
    "payments": 0
  },
  "events": [
    {
      "event_type": {"type": "MetadataSet"},
      "event_data": {
        "agent_id": "765",
        "indexed_key": "0xbeac84f150cf983e3e9740d2cef38aa9c331bfec...",
        "key": "agentName",
        "value": "0x66697273742d6c6976652d736974652d6167656e74..."
      },
      "block_number": 9534993,
      ...
    },
    {
      "event_type": {"type": "UriUpdated"},
      "event_data": {
        "agent_id": "100",
        "new_uri": "https://api.example.com/agent/100",
        "updated_by": "0x..."
      },
      ...
    }
  ]
}
```

---

### 4. Validation Category

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=validation&limit=20"
```

**Filters for:**
- `ValidationRequest` - Validation requests submitted
- `ValidationResponse` - Validation results

**Use Case:** Compliance tracking, quality assurance monitoring

**Example Response:**
```json
{
  "count": 20,
  "total": 99,
  "stats": {
    "all": 3649,
    "agents": 1086,
    "metadata": 2336,
    "validation": 99,
    "feedback": 128,
    "capabilities": 0,
    "payments": 0
  },
  "events": [
    {
      "event_type": {"type": "ValidationRequest"},
      "event_data": {
        "validator_address": "0x992b8382305a1d6c53713c39c2f4d2b0fb34bb7c",
        "agent_id": "766",
        "request_uri": "ipfs://QmecrMsp3jd7YiWqMuAjWbPbhu314M1KPpTXW8eDbCPz3u",
        "request_hash": "0xd4740b8d3a7a086848140f7f517b25d862da61a01f..."
      },
      "block_number": 9535655,
      ...
    },
    {
      "event_type": {"type": "ValidationResponse"},
      "event_data": {
        "validator_address": "0x...",
        "agent_id": "766",
        "request_hash": "0x...",
        "response": 1,
        "response_uri": "ipfs://...",
        "response_hash": "0x...",
        "tag": "verified"
      },
      ...
    }
  ]
}
```

---

### 5. Feedback Category

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=feedback&limit=20"
```

**Filters for:**
- `NewFeedback` - New feedback/reviews submitted
- `FeedbackRevoked` - Feedback removal/revocation
- `ResponseAppended` - Agent responses to feedback

**Use Case:** Reputation monitoring, review system tracking

**Example Response:**
```json
{
  "count": 20,
  "total": 128,
  "stats": {
    "all": 3649,
    "agents": 1086,
    "metadata": 2336,
    "validation": 99,
    "feedback": 128,
    "capabilities": 0,
    "payments": 0
  },
  "events": [
    {
      "event_type": {"type": "NewFeedback"},
      "event_data": {
        "agent_id": "500",
        "client": "0x...",
        "score": 5,
        "tag1": "quality",
        "tag2": "fast",
        "feedback_uri": "ipfs://...",
        "feedback_hash": "0x..."
      },
      "block_number": 9530000,
      ...
    },
    {
      "event_type": {"type": "ResponseAppended"},
      "event_data": {
        "agent_id": "500",
        "client": "0x...",
        "feedback_index": "0",
        "responder": "0x...",
        "response_uri": "ipfs://...",
        "response_hash": "0x..."
      },
      ...
    }
  ]
}
```

---

### 6. Capabilities Category

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=capabilities&limit=20"
```

**Filters for:**
- `CapabilityAdded` - (Not yet implemented)

**Use Case:** Feature announcements, capability updates

**Current Response:**
```json
{
  "count": 0,
  "total": 0,
  "stats": {
    "all": 3649,
    "agents": 1086,
    "metadata": 2336,
    "validation": 99,
    "feedback": 128,
    "capabilities": 0,  // Not implemented yet
    "payments": 0
  },
  "events": []
}
```

---

### 7. Payments Category

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=payments&limit=20"
```

**Filters for:**
- `X402Enabled` - (Not yet implemented)

**Use Case:** Payment tracking, HTTP 402 monetization

**Current Response:**
```json
{
  "count": 0,
  "total": 0,
  "stats": {
    "all": 3649,
    "agents": 1086,
    "metadata": 2336,
    "validation": 99,
    "feedback": 128,
    "capabilities": 0,
    "payments": 0  // Not implemented yet
  },
  "events": []
}
```

---

### 8. Category "all" (Explicit)

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=all&limit=20"
```

**Returns:** Same as no category filter - all event types

---

## Combined Filters

### Agent-specific Events by Category

```bash
# Get all metadata changes for agent 765
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=metadata&agent_id=765&limit=50"

# Get all feedback for agent 500
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=feedback&agent_id=500&limit=50"

# Get validation events for agent 766
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=validation&agent_id=766&limit=50"
```

### Pagination with Category

```bash
# Page 1 - First 20 agents
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=agents&limit=20&offset=0"

# Page 2 - Next 20 agents
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=agents&limit=20&offset=20"

# Page 3 - Next 20 agents
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=agents&limit=20&offset=40"
```

Use `pagination.has_more` and `pagination.next_offset` to navigate pages automatically:

```json
{
  "pagination": {
    "offset": 0,
    "limit": 20,
    "has_more": true,
    "next_offset": 20  // Use this for next page
  }
}
```

### Time-based Filters with Category

```bash
# Recent validation events (last 24 hours)
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=validation&hours=24"

# Recent agents (last 100 blocks)
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=agents&blocks=100"
```

---

## Statistics Response

Every response includes global stats regardless of filters:

```json
{
  "stats": {
    "all": 3649,        // Total events across all categories
    "agents": 1086,     // 30% - Agent registrations
    "metadata": 2336,   // 64% - Metadata updates
    "validation": 99,   // 3% - Validation events
    "feedback": 128,    // 4% - Feedback/reviews
    "capabilities": 0,  // Not implemented
    "payments": 0       // Not implemented
  }
}
```

**Note:** `total` in the response shows count for current filter, while `stats` always shows global counts.

---

## Error Handling

### Missing chain_id

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?category=agents"
```

Response:
```json
{
  "error": "Failed to deserialize query string: missing field `chain_id`"
}
```

### Invalid category

Unknown categories are treated as "all" (no filter applied):

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?chain_id=11155111&category=unknown&limit=10"
```

Returns all event types.

---

## Complete URL Structure

```
https://api-8004-dev.fly.dev/events
  ?chain_id={required}          // Chain ID (e.g., 11155111 for Sepolia)
  &category={optional}           // agents|metadata|validation|feedback|capabilities|payments|all
  &agent_id={optional}           // Filter by specific agent
  &contract={optional}           // Filter by contract address
  &event_type={optional}         // Filter by specific event type
  &hours={optional}              // Time range in hours
  &blocks={optional}             // Block range
  &limit={optional}              // Results per page (default: 1000)
  &offset={optional}             // Pagination offset (default: 0)
```

---

## Database Statistics (Sepolia - as of 2025-11-07)

| Category | Count | Percentage |
|----------|-------|------------|
| **Total** | 3,649 | 100% |
| Agents | 1,086 | 29.8% |
| Metadata | 2,336 | 64.0% |
| Validation | 99 | 2.7% |
| Feedback | 128 | 3.5% |
| Capabilities | 0 | 0% (not implemented) |
| Payments | 0 | 0% (not implemented) |

---

## Category Mapping Reference

| Category | Event Types | Description |
|----------|-------------|-------------|
| `agents` | Registered | Agent registration/creation |
| `metadata` | MetadataSet, UriUpdated | Profile updates, configuration changes |
| `validation` | ValidationRequest, ValidationResponse | Compliance, quality checks |
| `feedback` | NewFeedback, FeedbackRevoked, ResponseAppended | Reviews, ratings, responses |
| `capabilities` | CapabilityAdded | Feature announcements (not yet implemented) |
| `payments` | X402Enabled | Payment events (not yet implemented) |
| `all` or null | All types | No filtering |
