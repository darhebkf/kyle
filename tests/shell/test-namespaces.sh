#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Namespace Support ==="
setup_temp

mkdir -p backend frontend apps/mobile

cat > Kylefile << 'EOF'
# kyle: toml
name = "root"
includes = ["./backend", "./frontend"]

[tasks.build]
run = "echo root-build"

[tasks.deploy]
deps = ["backend:build", "frontend:build"]
run = "echo deploying"
EOF

cat > backend/Kylefile << 'EOF'
# kyle: toml
name = "backend"

[tasks.build]
run = "echo backend-build"

[tasks.test]
run = "echo backend-test"
EOF

cat > frontend/Kylefile.yaml << 'EOF'
name: frontend
tasks:
  build:
    run: echo frontend-build
  test:
    run: echo frontend-test
EOF

cat > apps/mobile/Makefile << 'EOF'
# Build mobile app
build:
	echo "mobile-build"
EOF

$KYLE backend:build 2>&1 | grep -q "backend-build" && pass "namespace: kyle backend:build" || fail "namespace: kyle backend:build"
! $KYLE backend:build 2>&1 | grep -q "warning" && pass "namespace: no warning with explicit namespace" || fail "namespace: no warning with explicit namespace"
$KYLE apps/mobile:build 2>&1 | grep -q "mobile-build" && pass "namespace: nested apps/mobile:build" || fail "namespace: nested apps/mobile:build"

$KYLE deploy 2>&1 | grep -q "backend-build" && pass "namespace: cross-dep runs backend:build" || fail "namespace: cross-dep runs backend:build"
$KYLE deploy 2>&1 | grep -q "frontend-build" && pass "namespace: cross-dep runs frontend:build" || fail "namespace: cross-dep runs frontend:build"
$KYLE deploy 2>&1 | grep -q "deploying" && pass "namespace: cross-dep runs main task" || fail "namespace: cross-dep runs main task"

$KYLE 2>&1 | grep -q "Discovered namespaces" && pass "namespace: discovery section shown" || fail "namespace: discovery section shown"
$KYLE 2>&1 | grep -q "backend:" && pass "namespace: discovers backend" || fail "namespace: discovers backend"
$KYLE 2>&1 | grep -q "frontend:" && pass "namespace: discovers frontend" || fail "namespace: discovers frontend"
$KYLE 2>&1 | grep -q "apps/mobile:" && pass "namespace: discovers apps/mobile" || fail "namespace: discovers apps/mobile"

$KYLE 2>&1 | grep -q "Namespaces (from includes)" && pass "namespace: includes section shown" || fail "namespace: includes section shown"
$KYLE nonexistent:build 2>&1 | grep -q "not found" && pass "namespace: error for nonexistent namespace" || fail "namespace: error for nonexistent namespace"
$KYLE backend:nonexistent 2>&1 | grep -q "task not found" && pass "namespace: error for nonexistent task" || fail "namespace: error for nonexistent task"

rm -rf backend frontend apps Kylefile

echo ""
echo "=== Namespace Discovery (Multi-Format) ==="
setup_temp

mkdir -p node-app go-svc rust-lib

cat > Kylefile << 'EOF'
# kyle: toml
name = "root"
[tasks.hello]
run = "echo root"
EOF

cat > node-app/package.json << 'EOF'
{"name": "node-app", "scripts": {"build": "echo node-build", "test": "echo node-test"}}
EOF

cat > go-svc/go.mod << 'EOF'
module example.com/go-svc
go 1.21
EOF

cat > rust-lib/Cargo.toml << 'EOF'
[package]
name = "rust-lib"
version = "0.1.0"
EOF

$KYLE 2>&1 | grep -q "node-app:" && pass "namespace: discovers node-app (package.json)" || fail "namespace: discovers node-app"
$KYLE 2>&1 | grep -q "go-svc:" && pass "namespace: discovers go-svc (go.mod)" || fail "namespace: discovers go-svc"
$KYLE 2>&1 | grep -q "rust-lib:" && pass "namespace: discovers rust-lib (Cargo.toml)" || fail "namespace: discovers rust-lib"
$KYLE node-app:build 2>&1 | grep -q "node-build" && pass "namespace: runs node-app:build" || fail "namespace: runs node-app:build"

rm -rf node-app go-svc rust-lib Kylefile

echo ""
echo "=== Namespace Separators (: and .) ==="

mkdir -p backend

cat > Kylefile << 'EOF'
# kyle: toml
name = "root"

[tasks."test:rust"]
run = "echo local-test-rust"

[tasks."build.debug"]
run = "echo local-build-debug"

[tasks.build]
run = "echo root-build"
EOF

cat > backend/Kylefile << 'EOF'
# kyle: toml
name = "backend"

[tasks.build]
run = "echo backend-build"

[tasks.rust]
run = "echo backend-rust"

[tasks.debug]
run = "echo backend-debug"
EOF

$KYLE "test:rust" 2>&1 | grep -q "local-test-rust" && pass "separator: local task 'test:rust' runs (not namespace)" || fail "separator: local task with colon"
$KYLE "build.debug" 2>&1 | grep -q "local-build-debug" && pass "separator: local task 'build.debug' runs (not namespace)" || fail "separator: local task with dot"
$KYLE backend:build 2>&1 | grep -q "backend-build" && pass "separator: namespace colon backend:build" || fail "separator: namespace colon"
$KYLE backend.build 2>&1 | grep -q "backend-build" && pass "separator: namespace dot backend.build" || fail "separator: namespace dot"
$KYLE backend.rust 2>&1 | grep -q "backend-rust" && pass "separator: namespace dot backend.rust" || fail "separator: namespace dot task"
$KYLE nonexistent.build 2>&1 | grep -q "not found" && pass "separator: error for nonexistent.build" || fail "separator: nonexistent namespace dot"

rm -rf backend Kylefile
