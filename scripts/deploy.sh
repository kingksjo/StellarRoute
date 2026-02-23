#!/bin/bash
# StellarRoute — Deploy Contract to Stellar Network
# Usage: ./scripts/deploy.sh --network testnet

set -euo pipefail
source "$(dirname "$0")/lib/common.sh"

parse_network_flag "$@"
ensure_soroban_cli
ensure_log_dir
configure_network

# ── Step 1: Build ─────────────────────────────────────────────────────

build_wasm
optimize_wasm

# ── Step 2: Deploy ────────────────────────────────────────────────────

log_info "Deploying contract to ${NETWORK}..."
CONTRACT_ID=$(soroban_cmd contract deploy \
    --wasm "${WASM_FILE}" \
    --source "${IDENTITY}" \
    --network "${NETWORK}")

log_ok "Contract deployed: ${CONTRACT_ID}"
log_tx "${CONTRACT_ID}" "deploy"

# ── Step 3: Initialize ───────────────────────────────────────────────

ADMIN_ADDRESS=$(soroban_cmd keys address "${IDENTITY}")
FEE_RATE=30
FEE_TO="${ADMIN_ADDRESS}"

log_info "Initializing contract (admin=${ADMIN_ADDRESS}, fee_rate=${FEE_RATE})..."
invoke_contract "${CONTRACT_ID}" "initialize" \
    --admin "${ADMIN_ADDRESS}" \
    --fee_rate "${FEE_RATE}" \
    --fee_to "${FEE_TO}"

log_ok "Contract initialized"

# ── Step 4: Save Deployment Artifact ──────────────────────────────────

save_deployment "${CONTRACT_ID}"

# ── Step 5: Verify Deployment ─────────────────────────────────────────

log_info "Verifying deployment via get_admin()..."
DEPLOYED_ADMIN=$(invoke_contract "${CONTRACT_ID}" "get_admin")

if [[ "${DEPLOYED_ADMIN}" == *"${ADMIN_ADDRESS}"* ]]; then
    log_ok "Deployment verified: admin matches"
else
    log_error "Deployment verification FAILED: expected ${ADMIN_ADDRESS}, got ${DEPLOYED_ADMIN}"
    exit 1
fi

echo ""
log_ok "===== DEPLOYMENT COMPLETE ====="
log_ok "Network:     ${NETWORK}"
log_ok "Contract ID: ${CONTRACT_ID}"
log_ok "Admin:       ${ADMIN_ADDRESS}"
log_ok "Fee Rate:    ${FEE_RATE} bps"
log_ok "Artifact:    $(deployment_file)"
