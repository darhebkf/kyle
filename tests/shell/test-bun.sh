#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Bun project detection ==="
setup_temp

cat > package.json << 'EOF'
{
  "name": "my-bun-app",
  "private": true,
  "workspaces": ["apps/*"],
  "scripts": {
    "dev": "echo running bun dev",
    "build": "echo bun build"
  }
}
EOF
# Presence of bun.lockb is not required for parsing, but emulate a real bun repo.
: > bun.lockb

$KYLE 2>&1 | grep -q "dev" && pass "bun: lists dev script" || fail "bun: lists dev script"
$KYLE 2>&1 | grep -q "build" && pass "bun: lists build script" || fail "bun: lists build script"
$KYLE dev 2>&1 | grep -q "running bun dev" && pass "bun: runs dev script" || fail "bun: runs dev script"
$KYLE build 2>&1 | grep -q "bun build" && pass "bun: runs build script" || fail "bun: runs build script"

echo ""
echo "=== Bun workspace namespace discovery ==="
mkdir -p apps/frontend
cat > apps/frontend/package.json << 'EOF'
{
  "name": "frontend",
  "scripts": {
    "start": "echo starting frontend"
  }
}
EOF
$KYLE apps/frontend:start 2>&1 | grep -q "starting frontend" && pass "bun: runs workspace namespaced task" || fail "bun: runs workspace namespaced task"
$KYLE 2>&1 | grep -q "apps/frontend" && pass "bun: lists workspace namespace" || fail "bun: lists workspace namespace"

cleanup_temp
