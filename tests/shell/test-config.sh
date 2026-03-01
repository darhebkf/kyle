#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Config Commands ==="
$KYLE config list | grep -q "default_format" && pass "kyle config list" || fail "kyle config list"
$KYLE config get default_format | grep -qE "toml|yaml" && pass "kyle config get" || fail "kyle config get"
$KYLE config path | grep -q "config.toml" && pass "kyle config path" || fail "kyle config path"
