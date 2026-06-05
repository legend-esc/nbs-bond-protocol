#!/usr/bin/env bash
set -euo pipefail

# ── Source environment ──────────────────────────────────────────
ENV_FILE="${ENV_FILE:-.env.mainnet}"
if [ -f "$ENV_FILE" ]; then
  echo "Sourcing ${ENV_FILE}"
  set -a
  source "$ENV_FILE"
  set +a
fi

# ── Logging ─────────────────────────────────────────────────────
LOG_FILE="deploy-mainnet-$(date +%Y%m%d-%H%M%S).log"
exec > >(tee -a "$LOG_FILE") 2>&1

# ── Configuration ───────────────────────────────────────────────
NETWORK=mainnet
ADMIN_ADDRESS="${STELLAR_PUBLIC_KEY:?STELLAR_PUBLIC_KEY not set}"
CONTRACTS=(
  "shared"
  "project-registry"
  "bond-issuer"
  "coupon-engine"
  "oracle-consumer"
  "dex-router"
  "credit-retirement"
)

declare -A PKG_MAP=(
  ["shared"]="nbbs-shared"
  ["project-registry"]="nbbs-project-registry"
  ["bond-issuer"]="nbbs-bonds"
  ["coupon-engine"]="nbbs-coupon-engine"
  ["oracle-consumer"]="nbbs-oracle-consumer"
  ["dex-router"]="nbbs-dex-router"
  ["credit-retirement"]="nbbs-credit-retirement"
)

declare -A ENV_MAP=(
  ["project-registry"]="CONTRACT_PROJECT_REGISTRY"
  ["bond-issuer"]="CONTRACT_BOND_ISSUER"
  ["coupon-engine"]="CONTRACT_COUPON_ENGINE"
  ["oracle-consumer"]="CONTRACT_ORACLE_CONSUMER"
  ["dex-router"]="CONTRACT_DEX_ROUTER"
  ["credit-retirement"]="CONTRACT_CREDIT_RETIREMENT"
)

echo ""
echo "⚠️  ⚠️  ⚠️  MAINNET DEPLOYMENT ⚠️  ⚠️  ⚠️"
echo ""
echo "Network:  ${NETWORK}"
echo "Admin:    ${ADMIN_ADDRESS}"
echo "Log file: ${LOG_FILE}"
echo ""
echo "⚠️  This will deploy REAL contracts to MAINNET."
echo "⚠️  Ensure contracts have been audited before proceeding."
echo ""

for contract in "${CONTRACTS[@]}"; do
  pkg="${PKG_MAP[$contract]}"
  wasm_name="${contract//-/_}"
  wasm="target/wasm32-unknown-unknown/release/nbbs_${wasm_name}.wasm"

  echo ""
  echo "══════════════════════════════════════════════════════════════"
  echo "  NEXT: ${contract}"
  echo "══════════════════════════════════════════════════════════════"
  read -p "Continue? (y/N) " -r confirm
  if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "  ✗ Skipped ${contract}"
    continue
  fi

  echo "── ${contract} ──"

  if [ "$contract" = "shared" ]; then
    echo "  ↪ Building shared library..."
    (cd contracts && soroban contract build --package "$pkg")
    echo "  ✓ Done (library only, no deployment)"
    continue
  fi

  echo "  Building..."
  (cd contracts && soroban contract build --package "$pkg")

  echo "  Deploying..."
  address=$(soroban contract deploy \
    --wasm "contracts/${wasm}" \
    --network "$NETWORK")

  echo "  Address: ${address}"

  echo "  Initializing..."
  soroban contract invoke \
    --id "$address" \
    --fn __constructor \
    --arg "$ADMIN_ADDRESS" \
    --network "$NETWORK"

  env_key="${ENV_MAP[$contract]}"
  if grep -q "^#\?${env_key}=" "$ENV_FILE" 2>/dev/null; then
    sed -i "s/^#\?${env_key}=.*/${env_key}=${address}/" "$ENV_FILE"
  else
    echo "${env_key}=${address}" >> "$ENV_FILE"
  fi

  echo "  ✓ ${contract} → ${env_key}=${address}"
done

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "  All contracts deployed to ${NETWORK}"
echo "══════════════════════════════════════════════════════════════"
for contract in "${CONTRACTS[@]}"; do
  env_key="${ENV_MAP[$contract]}"
  if [ -n "${env_key:-}" ]; then
    echo "  ${env_key}=$(grep "^${env_key}=" "$ENV_FILE" | cut -d= -f2)"
  fi
done
echo "══════════════════════════════════════════════════════════════"
echo ""
echo "⚠️  Verify contract addresses on Stellar Expert before using in production"
echo "  Log saved to ${LOG_FILE}"
