#!/bin/bash

# Deploy script for Fly.io
# Usage: ./deploy-flyio.sh [init|deploy|logs|status]

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

APP_NAME="api-8004-dev"
DB_NAME="api-8004-dev-db"
REGION="ams" # Amsterdam - change if needed

# Check if flyctl is installed
command -v flyctl >/dev/null 2>&1 || {
    echo -e "${RED}Error: flyctl is not installed.${NC}"
    echo "Install it with: curl -L https://fly.io/install.sh | sh"
    exit 1
}

# Function to display usage
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  init       - Initialize new Fly.io app and database"
    echo "  deploy     - Deploy the application"
    echo "  secrets    - Set secrets (interactive)"
    echo "  logs       - View application logs"
    echo "  status     - Check application status"
    echo "  db-console - Connect to PostgreSQL console"
    echo "  ssh        - SSH into the app machine"
    echo "  scale      - Scale the application"
    echo "  destroy    - Destroy the application (WARNING: irreversible)"
    echo ""
    exit 1
}

# Initialize Fly.io app and database
init_app() {
    echo -e "${GREEN}🚀 Initializing Fly.io application...${NC}"

    # Check if already logged in
    if ! flyctl auth whoami >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Not logged in. Please login first:${NC}"
        flyctl auth login
    fi

    # Create app if it doesn't exist
    if ! flyctl apps list | grep -q "$APP_NAME"; then
        echo -e "${GREEN}📦 Creating app: $APP_NAME${NC}"
        flyctl apps create "$APP_NAME" --org personal
    else
        echo -e "${YELLOW}⚠️  App $APP_NAME already exists${NC}"
    fi

    # Create PostgreSQL database if it doesn't exist
    if ! flyctl postgres list | grep -q "$DB_NAME"; then
        echo -e "${GREEN}🐘 Creating PostgreSQL database: $DB_NAME${NC}"
        flyctl postgres create --name "$DB_NAME" --region "$REGION" --initial-cluster-size 1 --vm-size shared-cpu-1x --volume-size 1
    else
        echo -e "${YELLOW}⚠️  Database $DB_NAME already exists${NC}"
    fi

    # Attach database to app
    echo -e "${GREEN}🔗 Attaching database to app...${NC}"
    flyctl postgres attach "$DB_NAME" --app "$APP_NAME" || echo -e "${YELLOW}Database already attached${NC}"

    echo -e "${GREEN}✅ Initialization complete!${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Set secrets: ./deploy-flyio.sh secrets"
    echo "2. Deploy: ./deploy-flyio.sh deploy"
}

# Set secrets
set_secrets() {
    echo -e "${GREEN}🔐 Setting application secrets...${NC}"
    echo ""

    # Check if .env file exists
    if [ -f .env ]; then
        echo -e "${YELLOW}Found .env file. Do you want to use values from it? (y/n)${NC}"
        read -r use_env
        if [ "$use_env" = "y" ]; then
            source .env
        fi
    fi

    # RPC URL
    echo -e "${YELLOW}Enter RPC URL (e.g., https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY):${NC}"
    read -r RPC_URL
    flyctl secrets set RPC_URL="$RPC_URL" --app "$APP_NAME"

    # Contract addresses
    echo -e "${YELLOW}Enter IDENTITY_REGISTRY_ADDRESS:${NC}"
    read -r IDENTITY_REGISTRY_ADDRESS
    flyctl secrets set IDENTITY_REGISTRY_ADDRESS="$IDENTITY_REGISTRY_ADDRESS" --app "$APP_NAME"

    echo -e "${YELLOW}Enter REPUTATION_REGISTRY_ADDRESS:${NC}"
    read -r REPUTATION_REGISTRY_ADDRESS
    flyctl secrets set REPUTATION_REGISTRY_ADDRESS="$REPUTATION_REGISTRY_ADDRESS" --app "$APP_NAME"

    echo -e "${YELLOW}Enter VALIDATION_REGISTRY_ADDRESS:${NC}"
    read -r VALIDATION_REGISTRY_ADDRESS
    flyctl secrets set VALIDATION_REGISTRY_ADDRESS="$VALIDATION_REGISTRY_ADDRESS" --app "$APP_NAME"

    # Starting block
    echo -e "${YELLOW}Enter STARTING_BLOCK (default: latest):${NC}"
    read -r STARTING_BLOCK
    STARTING_BLOCK=${STARTING_BLOCK:-latest}
    flyctl secrets set STARTING_BLOCK="$STARTING_BLOCK" --app "$APP_NAME"

    # JWT Secret
    echo -e "${YELLOW}Enter JWT_SECRET (min 32 chars) or press Enter to generate:${NC}"
    read -r JWT_SECRET
    if [ -z "$JWT_SECRET" ]; then
        JWT_SECRET=$(openssl rand -base64 48)
        echo -e "${GREEN}Generated JWT_SECRET: $JWT_SECRET${NC}"
    fi
    flyctl secrets set JWT_SECRET="$JWT_SECRET" --app "$APP_NAME"

    # Auth credentials
    echo -e "${YELLOW}Enter AUTH_USERNAME (default: admin):${NC}"
    read -r AUTH_USERNAME
    AUTH_USERNAME=${AUTH_USERNAME:-admin}
    flyctl secrets set AUTH_USERNAME="$AUTH_USERNAME" --app "$APP_NAME"

    echo -e "${YELLOW}Enter AUTH_PASSWORD:${NC}"
    read -rs AUTH_PASSWORD
    echo ""

    # Generate bcrypt hash
    echo -e "${GREEN}Generating bcrypt hash...${NC}"
    # Note: This requires htpasswd or a Rust binary to generate bcrypt hash
    # For now, we'll set the plain password and warn the user
    flyctl secrets set AUTH_PASSWORD="$AUTH_PASSWORD" --app "$APP_NAME"
    echo -e "${YELLOW}⚠️  Using plain password. For production, generate a bcrypt hash and set AUTH_PASSWORD_HASH instead.${NC}"

    echo -e "${GREEN}✅ Secrets set successfully!${NC}"
}

# Deploy application
deploy_app() {
    echo -e "${GREEN}🚀 Deploying application to Fly.io...${NC}"

    # Run tests first
    echo -e "${YELLOW}Running tests...${NC}"
    cargo test --lib || {
        echo -e "${RED}❌ Tests failed! Aborting deployment.${NC}"
        exit 1
    }

    echo -e "${GREEN}✅ Tests passed!${NC}"

    # Deploy
    flyctl deploy --app "$APP_NAME"

    echo -e "${GREEN}✅ Deployment complete!${NC}"
    echo ""
    echo "Your app is available at: https://$APP_NAME.fly.dev"
    echo ""
    echo "View logs with: ./deploy-flyio.sh logs"
}

# View logs
view_logs() {
    echo -e "${GREEN}📋 Viewing logs...${NC}"
    flyctl logs --app "$APP_NAME"
}

# Check status
check_status() {
    echo -e "${GREEN}📊 Application status:${NC}"
    flyctl status --app "$APP_NAME"
    echo ""
    echo -e "${GREEN}🐘 Database status:${NC}"
    flyctl postgres db list --app "$DB_NAME"
}

# Database console
db_console() {
    echo -e "${GREEN}🐘 Connecting to PostgreSQL console...${NC}"
    flyctl postgres connect --app "$DB_NAME"
}

# SSH into machine
ssh_machine() {
    echo -e "${GREEN}🔌 SSH into machine...${NC}"
    flyctl ssh console --app "$APP_NAME"
}

# Scale application
scale_app() {
    echo -e "${GREEN}📈 Current scaling:${NC}"
    flyctl scale show --app "$APP_NAME"
    echo ""
    echo -e "${YELLOW}Enter new VM size (shared-cpu-1x, shared-cpu-2x, etc.) or press Enter to skip:${NC}"
    read -r vm_size
    if [ -n "$vm_size" ]; then
        flyctl scale vm "$vm_size" --app "$APP_NAME"
    fi

    echo -e "${YELLOW}Enter number of instances or press Enter to skip:${NC}"
    read -r count
    if [ -n "$count" ]; then
        flyctl scale count "$count" --app "$APP_NAME"
    fi
}

# Destroy application
destroy_app() {
    echo -e "${RED}⚠️  WARNING: This will permanently delete the application and all data!${NC}"
    echo -e "${YELLOW}Type 'yes' to confirm:${NC}"
    read -r confirm

    if [ "$confirm" = "yes" ]; then
        echo -e "${RED}Destroying app...${NC}"
        flyctl apps destroy "$APP_NAME" --yes
        echo -e "${RED}Destroying database...${NC}"
        flyctl apps destroy "$DB_NAME" --yes
        echo -e "${GREEN}✅ Application destroyed${NC}"
    else
        echo -e "${GREEN}Cancelled${NC}"
    fi
}

# Main script
case "${1:-}" in
    init)
        init_app
        ;;
    deploy)
        deploy_app
        ;;
    secrets)
        set_secrets
        ;;
    logs)
        view_logs
        ;;
    status)
        check_status
        ;;
    db-console)
        db_console
        ;;
    ssh)
        ssh_machine
        ;;
    scale)
        scale_app
        ;;
    destroy)
        destroy_app
        ;;
    *)
        usage
        ;;
esac
