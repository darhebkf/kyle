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
rm Cargo.toml

echo ""
echo "=== go.mod Detection ==="
cat > go.mod << 'EOF'
module example.com/test
go 1.21
EOF
$KYLE 2>&1 | grep -q "build" && pass "go.mod: lists standard tasks" || fail "go.mod: lists standard tasks"
$KYLE 2>&1 | grep -q "vet" && pass "go.mod: has vet task" || fail "go.mod: has vet task"
rm go.mod

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
