#!/bin/bash
# Test chains.yaml configuration validity

set -e

echo "==================================="
echo "Testing chains.yaml Configuration"
echo "==================================="
echo ""

# Check if chains.yaml exists
if [ ! -f "chains.yaml" ]; then
    echo "❌ Error: chains.yaml not found"
    exit 1
fi

echo "✅ chains.yaml exists"
echo ""

# Use Ruby to parse YAML (built-in on macOS)
echo "📋 Validating YAML syntax and displaying configuration..."
echo ""

ruby << 'EOF'
require 'yaml'

begin
  config = YAML.load_file('chains.yaml')

  chains = config['chains'] || []
  enabled_chains = chains.select { |c| c['enabled'] }
  disabled_chains = chains.reject { |c| c['enabled'] }

  puts "📊 Total chains configured: #{chains.length}"
  puts "✅ Enabled chains: #{enabled_chains.length}"
  puts "⏸️  Disabled chains: #{disabled_chains.length}"
  puts ""

  puts "🌐 Enabled Chains:"
  puts "-" * 80
  enabled_chains.each do |chain|
    puts "  • #{chain['name'].ljust(25)} (Chain ID: #{chain['chain_id'].to_s.rjust(10)})"

    # Handle both old (rpc_url) and new (rpc_providers) formats
    if chain['rpc_providers'] && chain['rpc_providers'].length > 0
      primary_provider = chain['rpc_providers'][0]
      rpc = primary_provider['url'][0..60]
      rpc += "..." if primary_provider['url'].length > 60
      puts "    Primary RPC: #{rpc}"
      puts "    Total providers: #{chain['rpc_providers'].length}"
    elsif chain['rpc_url']
      rpc = chain['rpc_url'][0..60]
      rpc += "..." if chain['rpc_url'].length > 60
      puts "    RPC: #{rpc}"
    end

    puts "    Poll interval: #{chain['poll_interval_ms']}ms"
    puts "    Identity Registry: #{chain['contracts']['identity_registry']}"
    puts ""
  end

  if disabled_chains.length > 0
    puts "⏸️  Disabled Chains:"
    puts "-" * 80
    disabled_chains.each do |chain|
      puts "  • #{chain['name'].ljust(25)} (Chain ID: #{chain['chain_id'].to_s.rjust(10)})"
    end
    puts ""
  end

  puts "⚙️  Global Settings:"
  puts "-" * 80
  global_config = config['global'] || {}
  puts "  • Max indexer retries: #{global_config['max_indexer_retries']}"
  puts "  • Retry base delay: #{global_config['retry_base_delay_ms']}ms"
  puts "  • Retry max delay: #{global_config['retry_max_delay_ms']}ms"
  puts "  • Adaptive polling: #{global_config['adaptive_polling_enabled']}"
  puts "  • Max parallel blocks: #{global_config['max_parallel_blocks']}"
  puts ""

  puts "✅ Configuration appears valid!"

rescue => e
  puts "❌ Error parsing YAML: #{e.message}"
  exit 1
end
EOF

echo ""
echo "==================================="
echo "Configuration test completed!"
echo "==================================="
