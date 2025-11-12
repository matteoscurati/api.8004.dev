# Documentation

Comprehensive documentation for the ERC-8004 Multi-Chain Event Indexer.

## Getting Started

Start here if you're new to the project:

- **[Quick Start Guide](QUICK_START.md)** - Get up and running in 5 minutes
- **[Local Testing](LOCAL_TESTING.md)** - Setup local development environment
- **[API Examples](API_EXAMPLES.md)** - Common API usage patterns

## Core Documentation

### API & Authentication

- **[API Examples](API_EXAMPLES.md)** - Request/response examples for all endpoints
- **[API Authentication](API_AUTHENTICATION.md)** - JWT authentication and security
- **[Chain Status Monitoring](CHAIN_STATUS_MONITORING.md)** - Real-time monitoring and metrics

### Deployment & Operations

- **[Deployment Guide](DEPLOYMENT.md)** - Production deployment to Fly.io
- **[Production Deploy](PRODUCTION_DEPLOY.md)** - Production-specific configurations
- **[CI/CD Setup](CI_CD_SETUP.md)** - Automated deployment pipeline
- **[Security](SECURITY.md)** - Security best practices and guidelines

### Development

- **[Local Testing](LOCAL_TESTING.md)** - Development environment setup
- **[Quick Test](QUICK_TEST.md)** - Fast testing workflows
- **[Multichain Implementation](MULTICHAIN_IMPLEMENTATION.md)** - Multi-chain architecture

### Advanced Topics

- **[RPC Endpoints](RPC_ENDPOINTS.md)** - RPC provider configuration and failover
- **[Provider Optimization](PROVIDER_OPTIMIZATION.md)** - RPC performance tuning
- **[Svelte Integration](SVELTE_INTEGRATION.md)** - Frontend integration examples

## Reference Documentation

### Status & Coverage

- **[Real Events Coverage](REAL_EVENTS_COVERAGE.md)** - Event detection status per chain
- **[Pre-Deploy Status](PRE_DEPLOY_STATUS.md)** - Deployment readiness checklist
- **[Cleanup Summary](CLEANUP_SUMMARY.md)** - Codebase cleanup and refactoring notes
- **[Next Steps](NEXT_STEPS.md)** - Roadmap and future enhancements

## Quick Links by Task

### I want to...

#### ...get started quickly
1. Read [Quick Start Guide](QUICK_START.md)
2. Follow [Local Testing](LOCAL_TESTING.md)
3. Try [API Examples](API_EXAMPLES.md)

#### ...deploy to production
1. Review [Security](SECURITY.md)
2. Follow [Deployment Guide](DEPLOYMENT.md)
3. Setup [CI/CD](CI_CD_SETUP.md)
4. Check [Production Deploy](PRODUCTION_DEPLOY.md)

#### ...monitor the indexer
1. Use [Chain Status Monitoring](CHAIN_STATUS_MONITORING.md)
2. Review [Real Events Coverage](REAL_EVENTS_COVERAGE.md)

#### ...integrate with a frontend
1. See [API Examples](API_EXAMPLES.md)
2. Check [API Authentication](API_AUTHENTICATION.md)
3. Review [Svelte Integration](SVELTE_INTEGRATION.md)

#### ...optimize performance
1. Read [Provider Optimization](PROVIDER_OPTIMIZATION.md)
2. Configure [RPC Endpoints](RPC_ENDPOINTS.md)
3. Review [Multichain Implementation](MULTICHAIN_IMPLEMENTATION.md)

#### ...add a new chain
1. Read [Multichain Implementation](MULTICHAIN_IMPLEMENTATION.md)
2. Configure in `chains.yaml`
3. Test with [Local Testing](LOCAL_TESTING.md)

## Architecture Overview

```
ERC-8004 Multi-Chain Indexer
│
├── Indexer Core
│   ├── Event Detection (per chain)
│   ├── RPC Provider Management
│   └── Database Persistence
│
├── API Server
│   ├── REST Endpoints
│   ├── WebSocket Streaming
│   └── JWT Authentication
│
└── Monitoring
    ├── Chain Status
    ├── Sync Progress
    └── Event Statistics
```

## Contributing

When adding new documentation:

1. Place in `docs/` directory
2. Use clear, descriptive filenames
3. Add to this README index
4. Cross-reference related docs
5. Include code examples
6. Keep content up to date

## Document Templates

### API Documentation Template
```markdown
# [Feature Name]

## Overview
Brief description of the feature.

## Usage
How to use the feature with examples.

## Parameters
List of parameters and their types.

## Response
Expected response format.

## Examples
Real-world usage examples.

## See Also
Links to related documentation.
```

### Guide Template
```markdown
# [Guide Title]

## Prerequisites
What you need before starting.

## Step-by-Step
1. Step one
2. Step two
3. ...

## Troubleshooting
Common issues and solutions.

## See Also
Related guides and references.
```

## Feedback

Found an issue in the documentation? Please:
1. Open an issue on GitHub
2. Include the document name
3. Describe the problem or suggestion
4. Provide corrections if applicable

## Version

Documentation version: 1.0
Last updated: January 2025
Project version: See [Cargo.toml](../Cargo.toml)
