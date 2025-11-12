#!/bin/bash
# Find blocks with real events for each chain using block explorer APIs
# This script queries Etherscan-like APIs to find actual events from deployed contracts

set -e

echo "=========================================="
echo "Finding Blocks with Events via Block Explorers"
echo "=========================================="
echo ""

# Function to get block explorer API endpoint for a chain
get_explorer_api() {
    local chain_id=$1
    case $chain_id in
        11155111)
            echo "https://api-sepolia.etherscan.io/api"
            ;;
        84532)
            echo "https://api-sepolia.basescan.org/api"
            ;;
        59141)
            echo "https://api-sepolia.lineascan.build/api"
            ;;
        80002)
            echo "https://api-amoy.polygonscan.com/api"
            ;;
        296)
            echo "https://api.hedera.com/api"  # Note: Hedera may not have standard Etherscan API
            ;;
        *)
            echo ""
            ;;
    esac
}

# Function to get block explorer name
get_explorer_name() {
    local chain_id=$1
    case $chain_id in
        11155111) echo "Etherscan (Sepolia)" ;;
        84532) echo "Basescan (Sepolia)" ;;
        59141) echo "Lineascan (Sepolia)" ;;
        80002) echo "Polygonscan (Amoy)" ;;
        296) echo "Hedera Explorer" ;;
        *) echo "Unknown" ;;
    esac
}

# Parse chains.yaml and find blocks with events
ruby << 'RUBY_SCRIPT'
require 'yaml'
require 'net/http'
require 'json'
require 'uri'

config = YAML.load_file('chains.yaml')
chains = config['chains'] || []

# Only process enabled chains
enabled_chains = chains.select { |c| c['enabled'] }

results = {}

# Block explorer API endpoints
EXPLORER_APIS = {
  11155111 => 'https://api-sepolia.etherscan.io/api',
  84532 => 'https://api-sepolia.basescan.org/api',
  59141 => 'https://api-sepolia.lineascan.build/api',
  80002 => 'https://api-amoy.polygonscan.com/api',
  296 => nil  # Hedera doesn't have standard Etherscan-like API
}

EXPLORER_NAMES = {
  11155111 => 'Etherscan (Sepolia)',
  84532 => 'Basescan (Sepolia)',
  59141 => 'Lineascan (Sepolia)',
  80002 => 'Polygonscan (Amoy)',
  296 => 'Hedera Explorer'
}

enabled_chains.each do |chain|
  puts "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  puts "Chain: #{chain['name']} (ID: #{chain['chain_id']})"
  puts "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  chain_id = chain['chain_id']
  api_url = EXPLORER_APIS[chain_id]
  explorer_name = EXPLORER_NAMES[chain_id]

  if api_url.nil?
    puts "  ⚠ No block explorer API available for this chain, skipping..."
    puts ""
    next
  end

  puts "  Using: #{explorer_name}"
  puts "  API: #{api_url}"
  puts ""

  # Get contract addresses
  contracts = {
    'identity_registry' => chain['contracts']['identity_registry'],
    'reputation_registry' => chain['contracts']['reputation_registry'],
    'validation_registry' => chain['contracts']['validation_registry']
  }

  chain_events = {}
  total_events = 0

  contracts.each do |contract_type, contract_address|
    puts "  📝 Checking #{contract_type}: #{contract_address}"

    begin
      # Query block explorer API for logs
      # Using getLogs endpoint (works without API key for most explorers)
      uri = URI(api_url)
      params = {
        module: 'logs',
        action: 'getLogs',
        address: contract_address,
        fromBlock: '0',
        toBlock: 'latest',
        page: '1',
        offset: '100'  # Get last 100 events
      }
      uri.query = URI.encode_www_form(params)

      http = Net::HTTP.new(uri.host, uri.port)
      http.use_ssl = (uri.scheme == 'https')
      http.read_timeout = 30

      request = Net::HTTP::Get.new(uri)
      response = http.request(request)

      if response.code == '200'
        data = JSON.parse(response.body)

        if data['status'] == '1' && data['result'].is_a?(Array) && !data['result'].empty?
          logs = data['result']
          event_count = logs.length

          puts "    ✓ Found #{event_count} events"

          # Group events by block number
          events_by_block = logs.group_by { |log| log['blockNumber'].to_i(16) }

          # Get a sample block with events (latest one)
          sample_block_number = events_by_block.keys.max
          sample_block_hex = "0x#{sample_block_number.to_s(16)}"
          sample_events = events_by_block[sample_block_number]

          puts "    📍 Sample block: #{sample_block_number} (#{sample_events.length} events)"

          # Analyze event topics to identify event types
          event_types = sample_events.map do |e|
            begin
              e['topics'][0]
            rescue
              nil
            end
          end.compact.uniq
          puts "    🏷️  Unique event signatures: #{event_types.length}"

          latest_logs_data = logs.first(5).map do |log|
            topic0 = nil
            begin
              topic0 = log['topics'][0] if log['topics']
            rescue
            end
            {
              block_number: log['blockNumber'].to_i(16),
              tx_hash: log['transactionHash'],
              topic0: topic0
            }
          end

          chain_events[contract_type] = {
            address: contract_address,
            total_events: event_count,
            sample_block: sample_block_number,
            sample_block_hex: sample_block_hex,
            events_in_sample: sample_events.length,
            unique_topics: event_types.length,
            latest_logs: latest_logs_data
          }

          total_events += event_count
        elsif data['status'] == '0'
          puts "    ⚠ No events found (#{data['message']})"
        else
          puts "    ⚠ Unexpected response format"
        end
      else
        puts "    ✗ HTTP error: #{response.code}"
      end

    rescue => e
      puts "    ✗ Error: #{e.message}"
    end

    # Rate limiting - be nice to free APIs
    sleep 0.5
  end

  if total_events > 0
    results[chain['name']] = {
      chain_id: chain_id,
      explorer: explorer_name,
      total_events: total_events,
      contracts: chain_events
    }
    puts ""
    puts "  ✅ Total events found: #{total_events}"
  else
    puts ""
    puts "  ⚠ No events found for any contract on this chain"
  end

  puts ""
end

# Write results to JSON file
if results.empty?
  puts "=========================================="
  puts "❌ No events found on any chain"
  puts "=========================================="
  puts ""
  puts "This could mean:"
  puts "  - Contracts haven't been deployed yet"
  puts "  - Contracts exist but no transactions have been made"
  puts "  - Block explorer APIs need API keys (add to script)"
  puts ""
  File.write('test-blocks.json', JSON.pretty_generate({ error: 'No events found' }))
else
  File.write('test-blocks.json', JSON.pretty_generate(results))
  puts "=========================================="
  puts "✅ Results saved to test-blocks.json"
  puts "=========================================="
  puts ""
  puts JSON.pretty_generate(results)
end
RUBY_SCRIPT

echo ""
echo "Done! Check test-blocks.json for detailed results."
echo ""
