# Integrazione API ERC-8004 con Svelte

Guida completa per integrare le API del **ERC-8004 Indexer** in un'applicazione web Svelte.

---

## 📋 Indice

- [Panoramica](#panoramica)
- [Setup Iniziale](#setup-iniziale)
- [Autenticazione](#autenticazione)
- [API Client](#api-client)
- [Svelte Store](#svelte-store)
- [Componenti Example](#componenti-example)
- [WebSocket Real-time](#websocket-real-time)
- [Best Practices](#best-practices)

---

## 🎯 Panoramica

### Endpoint API

**Base URL:** `https://api-8004-dev.fly.dev`

### Endpoint Pubblici (no auth)
- `GET /health` - Health check
- `GET /metrics` - Metriche Prometheus
- `POST /login` - Autenticazione

### Endpoint Protetti (require JWT)
- `GET /events` - Lista eventi con filtri e paginazione
- `GET /stats` - Statistiche indexer
- `GET /ws` - WebSocket per eventi real-time

---

## 🚀 Setup Iniziale

### 1. Installa dipendenze

```bash
npm install
```

### 2. Configura variabili d'ambiente

Crea `.env` nella root del progetto Svelte:

```env
PUBLIC_API_URL=https://api-8004-dev.fly.dev
PUBLIC_WS_URL=wss://api-8004-dev.fly.dev/ws
```

### 3. TypeScript Types

Crea `src/lib/types/api.ts`:

```typescript
export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  expires_at: string;
}

export interface Event {
  id: number;
  block_number: number;
  block_timestamp: string;
  transaction_hash: string;
  log_index: number;
  contract_address: string;
  event_type: string;
  event_data: Record<string, any>;
  created_at: string;
}

export interface EventsResponse {
  events: Event[];
  total: number;
  limit: number;
  offset: number;
}

export interface Stats {
  last_synced_block: number;
  last_synced_at: string;
  total_events: number;
  events_by_type: Record<string, number>;
  events_by_contract: Record<string, number>;
}

export interface ApiError {
  error: string;
  details?: string;
}
```

---

## 🔐 Autenticazione

### API Client con Auth

Crea `src/lib/api/client.ts`:

```typescript
import { PUBLIC_API_URL } from '$env/static/public';
import type { LoginRequest, LoginResponse, ApiError } from '$lib/types/api';

class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
    // Carica token da localStorage se disponibile
    if (typeof window !== 'undefined') {
      this.token = localStorage.getItem('jwt_token');
    }
  }

  // Salva token
  setToken(token: string) {
    this.token = token;
    if (typeof window !== 'undefined') {
      localStorage.setItem('jwt_token', token);
    }
  }

  // Rimuovi token
  clearToken() {
    this.token = null;
    if (typeof window !== 'undefined') {
      localStorage.removeItem('jwt_token');
    }
  }

  // Ottieni token corrente
  getToken(): string | null {
    return this.token;
  }

  // Helper per fare richieste HTTP
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;

    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    // Aggiungi token se presente
    if (this.token && !endpoint.includes('/login')) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (!response.ok) {
      if (response.status === 401) {
        // Token scaduto o invalido
        this.clearToken();
        throw new Error('Unauthorized: Please login again');
      }

      const error: ApiError = await response.json().catch(() => ({
        error: `HTTP ${response.status}: ${response.statusText}`,
      }));
      throw new Error(error.error || 'API request failed');
    }

    return response.json();
  }

  // Login
  async login(credentials: LoginRequest): Promise<LoginResponse> {
    const response = await this.request<LoginResponse>('/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    });

    this.setToken(response.token);
    return response;
  }

  // Logout
  logout() {
    this.clearToken();
  }

  // Verifica se utente è autenticato
  isAuthenticated(): boolean {
    return this.token !== null;
  }
}

// Export singleton instance
export const apiClient = new ApiClient(PUBLIC_API_URL);
```

---

## 📊 API Methods

Estendi `src/lib/api/client.ts` con i metodi per gli eventi:

```typescript
import type { Event, EventsResponse, Stats } from '$lib/types/api';

// ... codice precedente ...

class ApiClient {
  // ... metodi di autenticazione ...

  // Ottieni eventi con filtri
  async getEvents(params?: {
    limit?: number;
    offset?: number;
    contract?: string;
    event_type?: string;
    from_block?: number;
    to_block?: number;
  }): Promise<EventsResponse> {
    const searchParams = new URLSearchParams();

    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.offset) searchParams.set('offset', params.offset.toString());
    if (params?.contract) searchParams.set('contract', params.contract);
    if (params?.event_type) searchParams.set('event_type', params.event_type);
    if (params?.from_block) searchParams.set('from_block', params.from_block.toString());
    if (params?.to_block) searchParams.set('to_block', params.to_block.toString());

    const query = searchParams.toString();
    const endpoint = query ? `/events?${query}` : '/events';

    return this.request<EventsResponse>(endpoint);
  }

  // Ottieni statistiche
  async getStats(): Promise<Stats> {
    return this.request<Stats>('/stats');
  }

  // Health check
  async healthCheck(): Promise<{ service: string; status: string }> {
    return this.request('/health');
  }
}
```

---

## 🗄️ Svelte Store

Crea `src/lib/stores/auth.ts`:

```typescript
import { writable, derived } from 'svelte/store';
import { apiClient } from '$lib/api/client';
import type { LoginRequest } from '$lib/types/api';

interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    isAuthenticated: apiClient.isAuthenticated(),
    isLoading: false,
    error: null,
  });

  return {
    subscribe,

    login: async (credentials: LoginRequest) => {
      update(state => ({ ...state, isLoading: true, error: null }));

      try {
        await apiClient.login(credentials);
        set({ isAuthenticated: true, isLoading: false, error: null });
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Login failed';
        set({ isAuthenticated: false, isLoading: false, error: message });
        throw error;
      }
    },

    logout: () => {
      apiClient.logout();
      set({ isAuthenticated: false, isLoading: false, error: null });
    },

    clearError: () => {
      update(state => ({ ...state, error: null }));
    },
  };
}

export const authStore = createAuthStore();
export const isAuthenticated = derived(authStore, $auth => $auth.isAuthenticated);
```

Crea `src/lib/stores/events.ts`:

```typescript
import { writable } from 'svelte/store';
import { apiClient } from '$lib/api/client';
import type { Event, Stats } from '$lib/types/api';

interface EventsState {
  events: Event[];
  stats: Stats | null;
  isLoading: boolean;
  error: string | null;
  total: number;
  hasMore: boolean;
}

function createEventsStore() {
  const { subscribe, set, update } = writable<EventsState>({
    events: [],
    stats: null,
    isLoading: false,
    error: null,
    total: 0,
    hasMore: true,
  });

  return {
    subscribe,

    // Carica eventi
    loadEvents: async (params?: {
      limit?: number;
      offset?: number;
      contract?: string;
      event_type?: string;
    }) => {
      update(state => ({ ...state, isLoading: true, error: null }));

      try {
        const response = await apiClient.getEvents(params);
        update(state => ({
          ...state,
          events: params?.offset ? [...state.events, ...response.events] : response.events,
          total: response.total,
          hasMore: response.events.length === (params?.limit || 10),
          isLoading: false,
        }));
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to load events';
        update(state => ({ ...state, error: message, isLoading: false }));
      }
    },

    // Carica statistiche
    loadStats: async () => {
      try {
        const stats = await apiClient.getStats();
        update(state => ({ ...state, stats }));
      } catch (error) {
        console.error('Failed to load stats:', error);
      }
    },

    // Reset
    reset: () => {
      set({
        events: [],
        stats: null,
        isLoading: false,
        error: null,
        total: 0,
        hasMore: true,
      });
    },
  };
}

export const eventsStore = createEventsStore();
```

---

## 🎨 Componenti Example

### Login Form

Crea `src/lib/components/LoginForm.svelte`:

```svelte
<script lang="ts">
  import { authStore } from '$lib/stores/auth';

  let username = 'admin';
  let password = '';

  async function handleSubmit() {
    try {
      await authStore.login({ username, password });
    } catch (error) {
      // Errore gestito nello store
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="login-form">
  <h2>Login</h2>

  {#if $authStore.error}
    <div class="error">{$authStore.error}</div>
  {/if}

  <input
    type="text"
    placeholder="Username"
    bind:value={username}
    required
  />

  <input
    type="password"
    placeholder="Password"
    bind:value={password}
    required
  />

  <button type="submit" disabled={$authStore.isLoading}>
    {$authStore.isLoading ? 'Logging in...' : 'Login'}
  </button>
</form>

<style>
  .login-form {
    max-width: 400px;
    margin: 2rem auto;
    padding: 2rem;
    border: 1px solid #ddd;
    border-radius: 8px;
  }

  input {
    width: 100%;
    padding: 0.5rem;
    margin: 0.5rem 0;
    border: 1px solid #ccc;
    border-radius: 4px;
  }

  button {
    width: 100%;
    padding: 0.75rem;
    margin-top: 1rem;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  button:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  .error {
    padding: 0.5rem;
    margin-bottom: 1rem;
    background: #fee;
    color: #c00;
    border-radius: 4px;
  }
</style>
```

### Events List

Crea `src/lib/components/EventsList.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { eventsStore } from '$lib/stores/events';

  let limit = 20;
  let offset = 0;

  onMount(() => {
    eventsStore.loadEvents({ limit, offset });
    eventsStore.loadStats();
  });

  function loadMore() {
    offset += limit;
    eventsStore.loadEvents({ limit, offset });
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleString();
  }

  function formatAddress(address: string): string {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  }
</script>

<div class="events-container">
  <header>
    <h2>ERC-8004 Events</h2>

    {#if $eventsStore.stats}
      <div class="stats">
        <span>Block: {$eventsStore.stats.last_synced_block}</span>
        <span>Total Events: {$eventsStore.stats.total_events}</span>
      </div>
    {/if}
  </header>

  {#if $eventsStore.isLoading && $eventsStore.events.length === 0}
    <div class="loading">Loading events...</div>
  {:else if $eventsStore.error}
    <div class="error">{$eventsStore.error}</div>
  {:else if $eventsStore.events.length === 0}
    <div class="empty">No events found</div>
  {:else}
    <div class="events-list">
      {#each $eventsStore.events as event (event.id)}
        <div class="event-card">
          <div class="event-header">
            <span class="event-type">{event.event_type}</span>
            <span class="block-number">Block #{event.block_number}</span>
          </div>

          <div class="event-details">
            <div class="detail">
              <span class="label">Contract:</span>
              <span class="value">{formatAddress(event.contract_address)}</span>
            </div>

            <div class="detail">
              <span class="label">TX:</span>
              <a
                href="https://sepolia.etherscan.io/tx/{event.transaction_hash}"
                target="_blank"
                rel="noopener noreferrer"
              >
                {formatAddress(event.transaction_hash)}
              </a>
            </div>

            <div class="detail">
              <span class="label">Time:</span>
              <span class="value">{formatDate(event.block_timestamp)}</span>
            </div>
          </div>

          <details>
            <summary>Event Data</summary>
            <pre>{JSON.stringify(event.event_data, null, 2)}</pre>
          </details>
        </div>
      {/each}
    </div>

    {#if $eventsStore.hasMore}
      <button
        class="load-more"
        on:click={loadMore}
        disabled={$eventsStore.isLoading}
      >
        {$eventsStore.isLoading ? 'Loading...' : 'Load More'}
      </button>
    {/if}
  {/if}
</div>

<style>
  .events-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  .stats {
    display: flex;
    gap: 1rem;
    font-size: 0.9rem;
    color: #666;
  }

  .events-list {
    display: grid;
    gap: 1rem;
  }

  .event-card {
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 1rem;
    background: white;
  }

  .event-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #eee;
  }

  .event-type {
    font-weight: bold;
    color: #007bff;
  }

  .block-number {
    color: #666;
    font-size: 0.9rem;
  }

  .event-details {
    display: grid;
    gap: 0.5rem;
  }

  .detail {
    display: flex;
    gap: 0.5rem;
  }

  .label {
    font-weight: 500;
    color: #666;
  }

  .value {
    font-family: monospace;
  }

  a {
    color: #007bff;
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }

  details {
    margin-top: 1rem;
  }

  summary {
    cursor: pointer;
    color: #007bff;
  }

  pre {
    margin-top: 0.5rem;
    padding: 1rem;
    background: #f5f5f5;
    border-radius: 4px;
    overflow-x: auto;
    font-size: 0.85rem;
  }

  .load-more {
    width: 100%;
    padding: 1rem;
    margin-top: 1rem;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .load-more:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  .loading, .error, .empty {
    text-align: center;
    padding: 2rem;
    color: #666;
  }

  .error {
    color: #c00;
    background: #fee;
    border-radius: 4px;
  }
</style>
```

---

## 🔌 WebSocket Real-time

Crea `src/lib/api/websocket.ts`:

```typescript
import { PUBLIC_WS_URL } from '$env/static/public';
import { apiClient } from './client';
import type { Event } from '$lib/types/api';

type EventCallback = (event: Event) => void;
type ErrorCallback = (error: Error) => void;

class WebSocketClient {
  private ws: WebSocket | null = null;
  private eventCallbacks: EventCallback[] = [];
  private errorCallbacks: ErrorCallback[] = [];
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  connect() {
    const token = apiClient.getToken();

    if (!token) {
      throw new Error('Not authenticated. Please login first.');
    }

    const wsUrl = `${PUBLIC_WS_URL}?token=${token}`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = (event) => {
      try {
        const data: Event = JSON.parse(event.data);
        this.eventCallbacks.forEach(cb => cb(data));
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      const err = new Error('WebSocket connection error');
      this.errorCallbacks.forEach(cb => cb(err));
    };

    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      this.attemptReconnect();
    };
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);

      setTimeout(() => {
        this.connect();
      }, this.reconnectDelay * this.reconnectAttempts);
    } else {
      console.error('Max reconnect attempts reached');
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  onEvent(callback: EventCallback) {
    this.eventCallbacks.push(callback);
  }

  onError(callback: ErrorCallback) {
    this.errorCallbacks.push(callback);
  }

  removeEventListener(callback: EventCallback) {
    this.eventCallbacks = this.eventCallbacks.filter(cb => cb !== callback);
  }

  removeErrorListener(callback: ErrorCallback) {
    this.errorCallbacks = this.errorCallbacks.filter(cb => cb !== callback);
  }
}

export const wsClient = new WebSocketClient();
```

### Uso in un componente

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { wsClient } from '$lib/api/websocket';
  import { eventsStore } from '$lib/stores/events';
  import type { Event } from '$lib/types/api';

  let realtimeEvents: Event[] = [];

  function handleNewEvent(event: Event) {
    realtimeEvents = [event, ...realtimeEvents].slice(0, 10);
  }

  onMount(() => {
    wsClient.connect();
    wsClient.onEvent(handleNewEvent);
  });

  onDestroy(() => {
    wsClient.removeEventListener(handleNewEvent);
    wsClient.disconnect();
  });
</script>

<div class="realtime-events">
  <h3>Real-time Events</h3>
  {#each realtimeEvents as event (event.id)}
    <div class="event-notification">
      New {event.event_type} event in block {event.block_number}
    </div>
  {/each}
</div>
```

---

## ⚡ Best Practices

### 1. **Gestione Errori**

```typescript
try {
  const events = await apiClient.getEvents({ limit: 20 });
} catch (error) {
  if (error.message.includes('Unauthorized')) {
    // Redirect al login
    authStore.logout();
  } else {
    // Mostra errore all'utente
    console.error('API Error:', error);
  }
}
```

### 2. **Caching**

Usa una libreria come `@tanstack/svelte-query` per caching automatico:

```bash
npm install @tanstack/svelte-query
```

```typescript
import { createQuery } from '@tanstack/svelte-query';
import { apiClient } from '$lib/api/client';

export const eventsQuery = createQuery({
  queryKey: ['events'],
  queryFn: () => apiClient.getEvents({ limit: 20 }),
  staleTime: 30000, // 30 secondi
});
```

### 3. **Paginazione Infinita**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { eventsStore } from '$lib/stores/events';

  let observerTarget: HTMLElement;

  onMount(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !$eventsStore.isLoading) {
          eventsStore.loadEvents({
            limit: 20,
            offset: $eventsStore.events.length
          });
        }
      },
      { threshold: 0.5 }
    );

    if (observerTarget) {
      observer.observe(observerTarget);
    }

    return () => observer.disconnect();
  });
</script>

<!-- Eventi qui -->

{#if $eventsStore.hasMore}
  <div bind:this={observerTarget} class="load-trigger">
    Loading more...
  </div>
{/if}
```

### 4. **Gestione Token Scaduto**

```typescript
// Aggiungi in api/client.ts
private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  try {
    // ... codice esistente ...
  } catch (error) {
    if (error.message.includes('Unauthorized')) {
      // Tenta refresh o logout
      this.clearToken();
      window.location.href = '/login';
    }
    throw error;
  }
}
```

### 5. **Rate Limiting**

Implementa debouncing per le ricerche:

```typescript
import { debounce } from 'lodash-es';

const debouncedSearch = debounce(async (query: string) => {
  await eventsStore.loadEvents({ event_type: query });
}, 300);
```

---

## 📝 Example Routes

### `src/routes/+page.svelte` (Home con login)

```svelte
<script lang="ts">
  import { isAuthenticated } from '$lib/stores/auth';
  import LoginForm from '$lib/components/LoginForm.svelte';
  import EventsList from '$lib/components/EventsList.svelte';
</script>

{#if $isAuthenticated}
  <EventsList />
{:else}
  <LoginForm />
{/if}
```

### `src/routes/events/+page.svelte` (Pagina eventi protetta)

```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { isAuthenticated } from '$lib/stores/auth';
  import EventsList from '$lib/components/EventsList.svelte';

  onMount(() => {
    if (!$isAuthenticated) {
      goto('/');
    }
  });
</script>

<EventsList />
```

---

## 🔗 Link Utili

- **API Health:** https://api-8004-dev.fly.dev/health
- **Metrics:** https://api-8004-dev.fly.dev/metrics
- **Etherscan Sepolia:** https://sepolia.etherscan.io/

---

## 🆘 Troubleshooting

### CORS Errors

L'API è configurata con `CORS_ALLOWED_ORIGINS=*`. In produzione, configura domini specifici.

### WebSocket non si connette

1. Verifica di essere autenticato
2. Controlla che il token sia valido
3. Verifica la console per errori

### Token scaduto

I token JWT scadono dopo 24 ore. Implementa un sistema di refresh o chiedi all'utente di rifare login.

---

## 📚 Risorse

- [Svelte Documentation](https://svelte.dev/docs)
- [SvelteKit Documentation](https://kit.svelte.dev/docs)
- [Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)
- [WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)

---

**Hai domande?** Apri una issue su GitHub o contatta il team di sviluppo.
