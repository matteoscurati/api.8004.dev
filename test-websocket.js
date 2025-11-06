#!/usr/bin/env node

/**
 * WebSocket Test Script for ERC-8004 Indexer
 *
 * Usage:
 *   npm install ws node-fetch
 *   node test-websocket.js [username] [password] [api-url]
 *
 * Example:
 *   node test-websocket.js admin admin123 https://api-8004-dev.fly.dev
 */

const WebSocket = require('ws');
const fetch = require('node-fetch');

// Parse command line arguments
const username = process.argv[2] || 'admin';
const password = process.argv[3] || 'admin123';
const apiUrl = process.argv[4] || 'https://api-8004-dev.fly.dev';

// Convert HTTP(S) URL to WS(S) URL
const wsUrl = apiUrl.replace(/^http/, 'ws');

console.log('🔐 ERC-8004 WebSocket Test');
console.log('========================\n');

async function login() {
    console.log(`📡 Logging in as '${username}'...`);

    try {
        const response = await fetch(`${apiUrl}/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ username, password })
        });

        if (!response.ok) {
            const error = await response.text();
            throw new Error(`Login failed: ${response.status} - ${error}`);
        }

        const data = await response.json();
        console.log('✅ Login successful!\n');
        return data.token;
    } catch (error) {
        console.error('❌ Login error:', error.message);
        process.exit(1);
    }
}

function connectWebSocket(token) {
    const url = `${wsUrl}/ws?token=${token}`;
    console.log(`🔌 Connecting to WebSocket...`);
    console.log(`   URL: ${url}\n`);

    const ws = new WebSocket(url);
    let eventCount = 0;
    let connectionTime = null;

    ws.on('open', () => {
        connectionTime = Date.now();
        console.log('✅ WebSocket connected!');
        console.log('📊 Waiting for events...\n');
        console.log('Press Ctrl+C to disconnect\n');
        console.log('─'.repeat(80));
    });

    ws.on('message', (data) => {
        eventCount++;
        const timestamp = new Date().toISOString();

        try {
            const event = JSON.parse(data.toString());

            console.log(`\n📦 Event #${eventCount} received at ${timestamp}`);
            console.log('─'.repeat(80));
            console.log(`Contract:     ${event.contract_address}`);
            console.log(`Event Type:   ${event.event_type}`);
            console.log(`Block:        ${event.block_number}`);
            console.log(`TX Hash:      ${event.transaction_hash}`);
            console.log(`Log Index:    ${event.log_index}`);
            console.log(`Timestamp:    ${new Date(event.block_timestamp * 1000).toISOString()}`);

            if (event.data) {
                console.log('\nEvent Data:');
                console.log(JSON.stringify(event.data, null, 2));
            }

            console.log('─'.repeat(80));
        } catch (error) {
            console.error('⚠️  Failed to parse event:', error.message);
            console.log('Raw data:', data.toString());
        }
    });

    ws.on('error', (error) => {
        console.error('\n❌ WebSocket error:', error.message);
    });

    ws.on('close', (code, reason) => {
        const duration = connectionTime ? ((Date.now() - connectionTime) / 1000).toFixed(2) : 'N/A';
        console.log(`\n🔌 WebSocket closed`);
        console.log(`   Code: ${code}`);
        console.log(`   Reason: ${reason || 'No reason provided'}`);
        console.log(`   Duration: ${duration}s`);
        console.log(`   Events received: ${eventCount}`);
    });

    ws.on('ping', () => {
        console.log('🏓 Ping received from server');
    });

    ws.on('pong', () => {
        console.log('🏓 Pong received from server');
    });

    // Handle graceful shutdown
    process.on('SIGINT', () => {
        console.log('\n\n⏹️  Shutting down...');
        ws.close(1000, 'Client disconnecting');
        setTimeout(() => process.exit(0), 1000);
    });
}

// Main execution
(async () => {
    try {
        const token = await login();
        connectWebSocket(token);
    } catch (error) {
        console.error('❌ Fatal error:', error.message);
        process.exit(1);
    }
})();
