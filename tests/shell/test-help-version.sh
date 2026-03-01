#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Help & Version ==="
$KYLE help | head -1 | grep -q "kyle - task runner" && pass "kyle help" || fail "kyle help"
$KYLE --help | head -1 | grep -q "kyle - task runner" && pass "kyle --help" || fail "kyle --help"
$KYLE -h | head -1 | grep -q "kyle - task runner" && pass "kyle -h" || fail "kyle -h"
$KYLE version | grep -qE "kyle v[0-9]+\.[0-9]+\.[0-9]+" && pass "kyle version" || fail "kyle version"
$KYLE --version | grep -qE "kyle v[0-9]+\.[0-9]+\.[0-9]+" && pass "kyle --version" || fail "kyle --version"
$KYLE -v | grep -qE "kyle v[0-9]+\.[0-9]+\.[0-9]+" && pass "kyle -v" || fail "kyle -v"
