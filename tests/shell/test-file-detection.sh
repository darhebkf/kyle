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
$KYLE 2>&1 | grep -q "warning" && pass "makefile: shows no-kylefile warning" || fail "makefile: shows no-kylefile warning"
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
$KYLE 2>&1 | grep -q "warning" && pass "justfile: shows no-kylefile warning" || fail "justfile: shows no-kylefile warning"
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
