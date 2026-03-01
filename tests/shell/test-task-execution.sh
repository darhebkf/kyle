#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Task Listing (using our Kylefile) ==="
cd "$SCRIPT_DIR"
$KYLE | grep -q "Available tasks" && pass "kyle (list tasks)" || fail "kyle (list tasks)"
$KYLE | grep -q "build" && pass "  - has build task" || fail "  - has build task"
$KYLE | grep -q "test" && pass "  - has test task" || fail "  - has test task"

echo ""
echo "=== Task Execution ==="
$KYLE check > /dev/null 2>&1 && pass "kyle check" || fail "kyle check"

echo ""
echo "=== Error Handling ==="
$KYLE nonexistent 2>&1 | grep -q "task not found" && pass "error: task not found" || fail "error: task not found"

echo ""
echo "=== Argument Passthrough ==="
setup_temp
cat > Kylefile << 'EOF'
# kyle: toml
name = "test"
[tasks.echo]
run = "echo args:"
EOF

$KYLE echo --flag1 --flag2 value 2>&1 | grep -q "args: --flag1 --flag2 value" && pass "arg passthrough: kyle echo --flag1 --flag2 value" || fail "arg passthrough without --"
$KYLE echo -- --release -v 2>&1 | grep -q "args: --release -v" && pass "arg passthrough: kyle echo -- --release -v" || fail "arg passthrough with --"
