#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Reserved Keywords ==="
setup_temp

cat > Kylefile << 'EOF'
# kyle: toml
name = "test"

[tasks.upgrade]
run = "echo should-not-run"

[tasks.build]
run = "echo build-ok"
EOF

# Warning shown for reserved task
$KYLE 2>&1 | grep -q "shadows a built-in command" && pass "reserved: warning for 'upgrade' task" || fail "reserved: warning for 'upgrade' task"

# Built-in command still works (upgrade checks for updates, doesn't run the task)
! $KYLE upgrade 2>&1 | grep -q "should-not-run" && pass "reserved: built-in upgrade takes priority" || fail "reserved: built-in upgrade takes priority"

# Non-reserved tasks still work
$KYLE build 2>&1 | grep -q "build-ok" && pass "reserved: non-reserved task runs fine" || fail "reserved: non-reserved task runs fine"

# --summary excludes reserved tasks
! $KYLE --summary 2>/dev/null | grep -q "upgrade" && pass "reserved: --summary excludes reserved tasks" || fail "reserved: --summary excludes reserved tasks"
$KYLE --summary 2>/dev/null | grep -q "build" && pass "reserved: --summary includes normal tasks" || fail "reserved: --summary includes normal tasks"

rm Kylefile
