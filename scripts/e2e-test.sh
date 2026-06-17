#!/usr/bin/env bash
set -euo pipefail

echo "=== NbS Bond Protocol — E2E Smoke Test ==="

# 1. Check toolchain
echo "[1/6] Checking toolchain..."
cargo --version && soroban --version && node --version && npm --version

# 2. Build all contracts
echo "[2/6] Building contracts..."
cd contracts
cargo build --release
cd ..

# 3. Run contract unit + integration tests
echo "[3/6] Running contract tests..."
cd contracts
cargo test -- --nocapture
cd ..

# 4. Build API
echo "[4/6] Building API..."
cd api
npm ci --silent
npm run build
cd ..

# 5. Build Frontend
echo "[5/6] Building Frontend..."
cd frontend
npm ci --silent
npm run build
cd ..

# 6. Summary
echo "[6/6] === ALL CHECKS PASSED ==="
echo "  Contracts: $(find contracts -name '*.wasm' | wc -l) wasm files"
echo "  API: $(find api/dist -name '*.js' | wc -l) compiled modules"
echo "  Frontend: $(find frontend/dist -name '*.js' | wc -l) compiled bundles"
echo "  Tests: $(grep -r '#\[test\]' contracts --include='*.rs' | wc -l) Rust tests"
