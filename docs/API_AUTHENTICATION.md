# JWT Authentication Guide

The ERC-8004 indexer API uses JWT (JSON Web Token) authentication to secure all endpoints except `/health` and `/login`.

## Authentication Flow

1. **Login** to get a JWT token
2. **Include the token** in the `Authorization` header for all subsequent requests
3. **Token expires** after 24 hours (configurable)

## Configuration

Add these variables to your `.env` file:

```env
# JWT Authentication
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRATION_HOURS=24
AUTH_USERNAME=admin
AUTH_PASSWORD=changeme
```

**IMPORTANT**: Change `JWT_SECRET` and `AUTH_PASSWORD` in production!

## API Endpoints

### Public Endpoints (No Authentication)

- `GET /health` - Health check
- `POST /login` - Get JWT token

### Protected Endpoints (Require Authentication)

- `GET /events` - Get recent events
- `GET /stats` - Get indexer statistics
- `GET /ws` - WebSocket connection

## Usage Examples

### 1. Login

```bash
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"changeme"}'
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_at": "2025-11-06T16:54:41.319260+00:00"
}
```

### 2. Use Token for Authenticated Requests

```bash
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

curl http://localhost:8080/events?limit=10 \
  -H "Authorization: Bearer $TOKEN"
```

### 3. Access Without Token (Returns Error)

```bash
curl http://localhost:8080/events
```

**Response:**
```json
{
  "error": "Missing authentication token"
}
```

## Frontend Integration

### JavaScript / TypeScript

```typescript
// Login and get token
async function login(username: string, password: string) {
  const response = await fetch('http://localhost:8080/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ username, password }),
  });

  const data = await response.json();

  if (response.ok) {
    // Store token (localStorage, sessionStorage, or memory)
    localStorage.setItem('jwt_token', data.token);
    localStorage.setItem('jwt_expires', data.expires_at);
    return data.token;
  } else {
    throw new Error('Login failed');
  }
}

// Make authenticated request
async function getEvents(limit = 100) {
  const token = localStorage.getItem('jwt_token');

  if (!token) {
    throw new Error('Not authenticated');
  }

  const response = await fetch(`http://localhost:8080/events?limit=${limit}`, {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });

  if (response.status === 401) {
    // Token expired or invalid, need to re-login
    localStorage.removeItem('jwt_token');
    throw new Error('Authentication expired');
  }

  return await response.json();
}

// Usage
try {
  await login('admin', 'changeme');
  const events = await getEvents(10);
  console.log(events);
} catch (error) {
  console.error('Error:', error);
}
```

### React Example with Axios

```typescript
import axios from 'axios';

const API_URL = 'http://localhost:8080';

// Create axios instance
const api = axios.create({
  baseURL: API_URL,
});

// Add token to all requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('jwt_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle 401 errors (token expired)
api.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('jwt_token');
      // Redirect to login page
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Login function
export async function login(username: string, password: string) {
  const { data } = await axios.post(`${API_URL}/login`, {
    username,
    password,
  });

  localStorage.setItem('jwt_token', data.token);
  localStorage.setItem('jwt_expires', data.expires_at);

  return data;
}

// Get events
export async function getEvents(params?: {
  blocks?: number;
  hours?: number;
  contract?: string;
  event_type?: string;
  limit?: number;
}) {
  const { data } = await api.get('/events', { params });
  return data;
}

// Get stats
export async function getStats() {
  const { data } = await api.get('/stats');
  return data;
}
```

### WebSocket with Authentication

```typescript
function connectWebSocket(token: string) {
  // Note: WebSocket doesn't support custom headers in browser
  // Option 1: Send token in URL query parameter (if your backend supports it)
  const ws = new WebSocket(`ws://localhost:8080/ws?token=${token}`);

  // Option 2: Send token in first message after connection
  const ws = new WebSocket('ws://localhost:8080/ws');

  ws.onopen = () => {
    // Send authentication message
    ws.send(JSON.stringify({
      type: 'auth',
      token: token
    }));
  };

  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    if (data.type === 'connected') {
      console.log('Connected to event stream');
    } else if (data.type === 'event') {
      console.log('New event:', data.data);
    }
  };

  ws.onerror = (error) => {
    console.error('WebSocket error:', error);
  };

  ws.onclose = () => {
    console.log('WebSocket closed');
  };

  return ws;
}

// Usage
const token = localStorage.getItem('jwt_token');
if (token) {
  const ws = connectWebSocket(token);
}
```

**Note**: The current WebSocket implementation expects the token in the `Authorization` header during the upgrade handshake. For browser-based clients, you may need to modify the WebSocket handler to accept the token via query parameter or initial message.

### Python Example

```python
import requests
from datetime import datetime

class ERC8004Client:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url
        self.token = None
        self.expires_at = None

    def login(self, username, password):
        """Login and get JWT token"""
        response = requests.post(
            f"{self.base_url}/login",
            json={"username": username, "password": password}
        )
        response.raise_for_status()

        data = response.json()
        self.token = data["token"]
        self.expires_at = data["expires_at"]

        return data

    def _headers(self):
        """Get authorization headers"""
        if not self.token:
            raise Exception("Not authenticated. Call login() first.")

        return {"Authorization": f"Bearer {self.token}"}

    def get_events(self, limit=100, blocks=None, hours=None,
                   contract=None, event_type=None):
        """Get recent events"""
        params = {"limit": limit}
        if blocks:
            params["blocks"] = blocks
        if hours:
            params["hours"] = hours
        if contract:
            params["contract"] = contract
        if event_type:
            params["event_type"] = event_type

        response = requests.get(
            f"{self.base_url}/events",
            params=params,
            headers=self._headers()
        )
        response.raise_for_status()
        return response.json()

    def get_stats(self):
        """Get indexer statistics"""
        response = requests.get(
            f"{self.base_url}/stats",
            headers=self._headers()
        )
        response.raise_for_status()
        return response.json()


# Usage
client = ERC8004Client()

# Login
client.login("admin", "changeme")

# Get recent events
events = client.get_events(limit=10)
print(f"Found {events['count']} events")

# Get events from last 24 hours
recent_events = client.get_events(hours=24)

# Get stats
stats = client.get_stats()
print(f"Last synced block: {stats['last_synced_block']}")
```

## Error Responses

### 401 Unauthorized - Missing Token
```json
{
  "error": "Missing authentication token"
}
```

### 401 Unauthorized - Invalid Token
```json
{
  "error": "Invalid token"
}
```

### 401 Unauthorized - Expired Token
```json
{
  "error": "Token has expired"
}
```

### 401 Unauthorized - Wrong Credentials
```json
{
  "error": "Wrong credentials"
}
```

## Security Best Practices

1. **HTTPS in Production**: Always use HTTPS in production to prevent token interception
2. **Strong JWT Secret**: Use a long, random string for `JWT_SECRET` (at least 32 characters)
3. **Secure Password**: Change the default `AUTH_PASSWORD`
4. **Token Storage**:
   - Use `httpOnly` cookies if possible
   - If using localStorage, be aware of XSS risks
   - Consider using sessionStorage for shorter-lived sessions
5. **Token Refresh**: Implement token refresh logic before expiration
6. **CORS**: Configure CORS properly for your frontend domain

## Token Expiration

Tokens expire after 24 hours by default (configured via `JWT_EXPIRATION_HOURS`). When a token expires:

1. API returns `401 Unauthorized` with error "Token has expired"
2. Frontend should:
   - Clear the stored token
   - Redirect user to login page
   - Prompt user to re-authenticate

## Future Enhancements

Potential improvements for production use:

- Refresh token mechanism
- Multiple user support with database
- Password hashing (bcrypt/argon2)
- Rate limiting per user
- Audit logging
- Role-based access control (RBAC)
