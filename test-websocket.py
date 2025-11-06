#!/usr/bin/env python3

"""
WebSocket Test Script for ERC-8004 Indexer (Python)

Usage:
    pip install websockets requests
    python test-websocket.py [username] [password] [api-url]

Example:
    python test-websocket.py admin admin123 https://api-8004-dev.fly.dev
"""

import asyncio
import json
import sys
import signal
from datetime import datetime
import requests
import websockets


class WebSocketTester:
    def __init__(self, api_url, username, password):
        self.api_url = api_url
        self.username = username
        self.password = password
        self.ws_url = api_url.replace('http://', 'ws://').replace('https://', 'wss://')
        self.event_count = 0
        self.connection_time = None
        self.running = True

    def login(self):
        """Login and get JWT token"""
        print(f"🔐 Logging in as '{self.username}'...")

        try:
            response = requests.post(
                f"{self.api_url}/login",
                json={"username": self.username, "password": self.password},
                timeout=10
            )
            response.raise_for_status()
            token = response.json()['token']
            print("✅ Login successful!\n")
            return token
        except requests.exceptions.RequestException as e:
            print(f"❌ Login error: {e}")
            sys.exit(1)

    async def connect_websocket(self, token):
        """Connect to WebSocket and handle events"""
        url = f"{self.ws_url}/ws?token={token}"
        print(f"🔌 Connecting to WebSocket...")
        print(f"   URL: {url}\n")

        try:
            async with websockets.connect(url) as websocket:
                self.connection_time = datetime.now()
                print("✅ WebSocket connected!")
                print("📊 Waiting for events...\n")
                print("Press Ctrl+C to disconnect\n")
                print("─" * 80)

                while self.running:
                    try:
                        message = await asyncio.wait_for(websocket.recv(), timeout=1.0)
                        self.handle_message(message)
                    except asyncio.TimeoutError:
                        continue
                    except websockets.exceptions.ConnectionClosed as e:
                        print(f"\n🔌 Connection closed: {e}")
                        break

        except websockets.exceptions.WebSocketException as e:
            print(f"❌ WebSocket error: {e}")
        except Exception as e:
            print(f"❌ Unexpected error: {e}")
        finally:
            self.print_summary()

    def handle_message(self, message):
        """Handle incoming WebSocket message"""
        self.event_count += 1
        timestamp = datetime.now().isoformat()

        try:
            event = json.loads(message)

            print(f"\n📦 Event #{self.event_count} received at {timestamp}")
            print("─" * 80)
            print(f"Contract:     {event.get('contract_address', 'N/A')}")
            print(f"Event Type:   {event.get('event_type', 'N/A')}")
            print(f"Block:        {event.get('block_number', 'N/A')}")
            print(f"TX Hash:      {event.get('transaction_hash', 'N/A')}")
            print(f"Log Index:    {event.get('log_index', 'N/A')}")

            if event.get('block_timestamp'):
                block_time = datetime.fromtimestamp(event['block_timestamp'])
                print(f"Timestamp:    {block_time.isoformat()}")

            if event.get('data'):
                print("\nEvent Data:")
                print(json.dumps(event['data'], indent=2))

            print("─" * 80)

        except json.JSONDecodeError as e:
            print(f"⚠️  Failed to parse event: {e}")
            print(f"Raw data: {message}")

    def print_summary(self):
        """Print connection summary"""
        if self.connection_time:
            duration = (datetime.now() - self.connection_time).total_seconds()
            print(f"\n📊 Connection Summary:")
            print(f"   Duration: {duration:.2f}s")
            print(f"   Events received: {self.event_count}")
            if self.event_count > 0:
                print(f"   Avg rate: {self.event_count / duration:.2f} events/sec")

    def handle_shutdown(self, signum, frame):
        """Handle graceful shutdown"""
        print("\n\n⏹️  Shutting down...")
        self.running = False


async def main():
    # Parse command line arguments
    username = sys.argv[1] if len(sys.argv) > 1 else 'admin'
    password = sys.argv[2] if len(sys.argv) > 2 else 'admin123'
    api_url = sys.argv[3] if len(sys.argv) > 3 else 'https://api-8004-dev.fly.dev'

    print("🔐 ERC-8004 WebSocket Test")
    print("========================\n")

    tester = WebSocketTester(api_url, username, password)

    # Setup signal handler for graceful shutdown
    signal.signal(signal.SIGINT, tester.handle_shutdown)

    # Login and connect
    token = tester.login()
    await tester.connect_websocket(token)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n👋 Goodbye!")
    except Exception as e:
        print(f"❌ Fatal error: {e}")
        sys.exit(1)
