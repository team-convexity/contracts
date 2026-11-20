#!/bin/bash
# Upgrade existing Soroban Contract

set -e  # exit on error

# Color output
RED='\033[0;31m'
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}   Soroban Contract Upgrade Script${NC}"
echo -e "${BLUE}================================================${NC}\n"

# configuration
NETWORK="${NETWORK:-testnet}" 
DEPLOYER_ACCOUNT="${DEPLOYER_ACCOUNT:-}"
EXISTING_CONTRACT_ID="${EXISTING_CONTRACT_ID:-}"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RAW_WASM="$PROJECT_ROOT/target/wasm32-unknown-unknown/release/project_contract.wasm"

# validation
if [ -z "$DEPLOYER_ACCOUNT" ]; then
    echo -e "${RED} Error: DEPLOYER_ACCOUNT is not set${NC}"
    echo "   Set it with: export DEPLOYER_ACCOUNT='your-account-id'"
    echo "   Or pass it: DEPLOYER_ACCOUNT='...' ./upgrade-contract.sh"
    exit 1
fi

if [ -z "$EXISTING_CONTRACT_ID" ]; then
    echo -e "${RED} Error: EXISTING_CONTRACT_ID is not set${NC}"
    echo "   Set it with: export EXISTING_CONTRACT_ID='your-contract-id'"
    echo "   Or pass it: EXISTING_CONTRACT_ID='...' ./upgrade-contract.sh"
    exit 1
fi

echo -e "${YELLOW}Building the release WASM${NC}"
cd "$PROJECT_ROOT"
cargo build --locked --target wasm32-unknown-unknown --release
WASM_DIR="$(mktemp -d)"
WASM_FILE="$WASM_DIR/project_contract.optimized.wasm"
trap 'rm -f -- "$WASM_FILE"; rmdir -- "$WASM_DIR"' EXIT
soroban contract optimize --wasm "$RAW_WASM" --wasm-out "$WASM_FILE"
test -s "$WASM_FILE"

echo -e "${GREEN}✓${NC} Configuration:"
echo "   Network: $NETWORK"
echo "   Deployer: $DEPLOYER_ACCOUNT"
echo "   Contract: $EXISTING_CONTRACT_ID"
echo "   WASM: $(basename "$WASM_FILE")"
shasum -a 256 "$WASM_FILE"
echo ""

# Confirm
read -p "$(echo -e ${YELLOW}'Continue with upgrade? (yes/no): '${NC})" -r
echo
if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
    echo -e "${RED} Upgrade cancelled${NC}"
    exit 1
fi

echo -e "${BLUE}📦 Step 1: Uploading new WASM...${NC}"
WASM_HASH=$(soroban contract install \
  --wasm "$WASM_FILE" \
  --source "$DEPLOYER_ACCOUNT" \
  --network "$NETWORK" 2>&1 | grep -o '[a-f0-9]\{64\}' | head -1)

if [ -z "$WASM_HASH" ]; then
    echo -e "${RED} Failed to upload WASM${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} WASM uploaded successfully"
echo "   Hash: $WASM_HASH"
echo ""

echo -e "${BLUE}🔄 Step 2: Upgrading contract...${NC}"
soroban contract upgrade \
  --contract-id "$EXISTING_CONTRACT_ID" \
  --wasm-hash "$WASM_HASH" \
  --source "$DEPLOYER_ACCOUNT" \
  --network "$NETWORK"

echo -e "${GREEN}✓${NC} Contract upgraded successfully!"
echo ""

echo -e "${BLUE}🧪 Step 3: Verifying upgrade...${NC}"
# Test a read operation to verify data is intact
echo "   Testing get_roles function..."
soroban contract invoke \
  --id "$EXISTING_CONTRACT_ID" \
  --source "$DEPLOYER_ACCOUNT" \
  --network "$NETWORK" \
  -- get_roles \
  --project_id "default_project" || true

echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN} Upgrade Complete!${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
echo "Next steps:"
echo "1. Test all critical functions on $NETWORK"
echo "2. Monitor for any issues"
echo ""
echo "Rollback command (if needed):"
echo "   soroban contract upgrade \\"
echo "     --contract-id $EXISTING_CONTRACT_ID \\"
echo "     --wasm-hash <OLD_WASM_HASH> \\"
echo "     --source $DEPLOYER_ACCOUNT \\"
echo "     --network $NETWORK"
