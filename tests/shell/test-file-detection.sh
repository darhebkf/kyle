#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Makefile Detection ==="
setup_temp

cat > Makefile << 'EOF'
# Build the project
build:
	echo "building from makefile"

# Run tests
test: build
	echo "testing from makefile"

.PHONY: build test
EOF
$KYLE 2>&1 | grep -q "build" && pass "makefile: lists targets" || fail "makefile: lists targets"
! $KYLE 2>&1 | grep -q "no Kylefile" && pass "makefile: no spurious kylefile warning" || fail "makefile: no spurious kylefile warning"
$KYLE build 2>&1 | grep -q "building from makefile" && pass "makefile: runs target" || fail "makefile: runs target"
$KYLE 2>&1 | grep -q "Build the project" && pass "makefile: extracts description from comment" || fail "makefile: extracts description from comment"
rm Makefile

echo ""
echo "=== Justfile Detection ==="
cat > justfile << 'EOF'
# Build the project
build:
    echo "building from justfile"

# Run tests
test: build
    echo "testing from justfile"
EOF
$KYLE 2>&1 | grep -q "build" && pass "justfile: lists recipes" || fail "justfile: lists recipes"
! $KYLE 2>&1 | grep -q "no Kylefile" && pass "justfile: no spurious kylefile warning" || fail "justfile: no spurious kylefile warning"
$KYLE build 2>&1 | grep -q "building from justfile" && pass "justfile: runs recipe" || fail "justfile: runs recipe"
$KYLE 2>&1 | grep -q "Build the project" && pass "justfile: extracts description from comment" || fail "justfile: extracts description from comment"
rm justfile

echo ""
echo "=== Kylefile Priority Over Makefile ==="
cat > Kylefile << 'EOF'
# kyle: toml
name = "test"
[tasks.build]
run = "echo building from kylefile"
EOF
cat > Makefile << 'EOF'
build:
	echo "building from makefile"
EOF
$KYLE build 2>&1 | grep -q "building from kylefile" && pass "priority: kylefile over makefile" || fail "priority: kylefile over makefile"
! $KYLE build 2>&1 | grep -q "warning" && pass "priority: no warning with kylefile" || fail "priority: no warning with kylefile"
rm Kylefile Makefile

echo ""
echo "=== package.json Detection ==="
cat > package.json << 'EOF'
{"name": "test-app", "scripts": {"build": "echo building-from-packagejson", "test": "echo testing-from-packagejson"}}
EOF
$KYLE 2>&1 | grep -q "build" && pass "package.json: lists scripts" || fail "package.json: lists scripts"
! $KYLE 2>&1 | grep -q "no Kylefile" && pass "package.json: no spurious kylefile warning" || fail "package.json: no spurious kylefile warning"
$KYLE build 2>&1 | grep -q "building-from-packagejson" && pass "package.json: runs script" || fail "package.json: runs script"
rm package.json

echo ""
echo "=== deno.json Detection ==="
cat > deno.json << 'EOF'
{"tasks": {"start": "echo starting-from-denojson", "test": "echo testing-from-denojson"}}
EOF
$KYLE 2>&1 | grep -q "start" && pass "deno.json: lists tasks" || fail "deno.json: lists tasks"
$KYLE start 2>&1 | grep -q "starting-from-denojson" && pass "deno.json: runs task" || fail "deno.json: runs task"
rm deno.json

echo ""
echo "=== Taskfile.yml Detection ==="
cat > Taskfile.yml << 'EOF'
version: '3'
tasks:
  build:
    desc: Build it
    cmds:
      - echo building-from-taskfile
  test:
    cmds:
      - echo testing-from-taskfile
    deps:
      - build
EOF
$KYLE 2>&1 | grep -q "build" && pass "taskfile: lists tasks" || fail "taskfile: lists tasks"
$KYLE build 2>&1 | grep -q "building-from-taskfile" && pass "taskfile: runs task" || fail "taskfile: runs task"
$KYLE 2>&1 | grep -q "Build it" && pass "taskfile: extracts description" || fail "taskfile: extracts description"
rm Taskfile.yml

echo ""
echo "=== Cargo.toml Detection ==="
cat > Cargo.toml << 'EOF'
[package]
name = "test-crate"
version = "0.1.0"
EOF
$KYLE 2>&1 | grep -q "build" && pass "cargo.toml: lists standard tasks" || fail "cargo.toml: lists standard tasks"
$KYLE 2>&1 | grep -q "test" && pass "cargo.toml: has test task" || fail "cargo.toml: has test task"
! $KYLE 2>&1 | grep -q "no Kylefile" && pass "cargo.toml: no spurious kylefile warning" || fail "cargo.toml: no spurious kylefile warning"
rm Cargo.toml

echo ""
echo "=== go.mod Detection ==="
cat > go.mod << 'EOF'
module example.com/test
go 1.21
EOF
$KYLE 2>&1 | grep -q "build" && pass "go.mod: lists standard tasks" || fail "go.mod: lists standard tasks"
$KYLE 2>&1 | grep -q "vet" && pass "go.mod: has vet task" || fail "go.mod: has vet task"
! $KYLE 2>&1 | grep -q "no Kylefile" && pass "go.mod: no spurious kylefile warning" || fail "go.mod: no spurious kylefile warning"
rm go.mod

echo ""
echo "=== Local Bin PATH Injection ==="
cat > package.json << 'EOF'
{"name": "test-app", "scripts": {"greet": "mybin"}}
EOF
mkdir -p node_modules/.bin
cat > node_modules/.bin/mybin << 'SCRIPT'
#!/bin/sh
echo "hello-from-node-modules-bin"
SCRIPT
chmod +x node_modules/.bin/mybin
$KYLE greet 2>&1 | grep -q "hello-from-node-modules-bin" && pass "path: resolves node_modules/.bin" || fail "path: resolves node_modules/.bin"
rm -rf package.json node_modules

cat > package.json << 'EOF'
{"name": "test-app", "scripts": {"greet": "mybin"}}
EOF
mkdir -p vendor/bin
cat > vendor/bin/mybin << 'SCRIPT'
#!/bin/sh
echo "hello-from-vendor-bin"
SCRIPT
chmod +x vendor/bin/mybin
$KYLE greet 2>&1 | grep -q "hello-from-vendor-bin" && pass "path: resolves vendor/bin" || fail "path: resolves vendor/bin"
rm -rf package.json vendor

echo ""
echo "=== Namespace Discovery (parent → child) ==="
mkdir -p frontend
echo '{"name":"frontend","scripts":{"build":"echo building-frontend","lint":"echo linting-frontend"}}' > frontend/package.json

$KYLE frontend:build 2>&1 | grep -q "building-frontend" && pass "ns: explicit namespace syntax works" || fail "ns: explicit namespace syntax works"
$KYLE build 2>&1 | grep -q "building-frontend" && pass "ns: auto-discovers unique child task" || fail "ns: auto-discovers unique child task"

mkdir -p backend
echo '{"name":"backend","scripts":{"build":"echo building-backend","deploy":"echo deploying-backend"}}' > backend/package.json

$KYLE deploy 2>&1 | grep -q "deploying-backend" && pass "ns: runs task unique to one child" || fail "ns: runs task unique to one child"
$KYLE build 2>&1 | grep -q "multiple namespaces" && pass "ns: conflict error for ambiguous task" || fail "ns: conflict error for ambiguous task"
$KYLE nonexistent 2>&1 | grep -q "not found" && pass "ns: task not found shows error" || fail "ns: task not found shows error"
$KYLE nonexistent 2>&1 | grep -q "frontend" && pass "ns: task not found lists namespaces" || fail "ns: task not found lists namespaces"

rm -rf frontend backend

echo ""
echo "=== Cycle Detection ==="
cat > Kylefile << 'EOF'
# kyle: toml
name = "test"
[tasks.a]
run = "echo a"
deps = ["b"]
[tasks.b]
run = "echo b"
deps = ["a"]
EOF
$KYLE a 2>&1 | grep -q "circular dependency" && pass "cycle: detects a → b → a" || fail "cycle: detects circular dependency"
rm Kylefile
