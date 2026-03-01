#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Init Command ==="
setup_temp

info "Testing init --toml..."
echo "n" | $KYLE init testproject --toml > /dev/null
grep -q "# kyle: toml" Kylefile && pass "init: has toml header" || fail "init: has toml header"
grep -qE 'version = "[0-9]+\.[0-9]+\.[0-9]+"' Kylefile && pass "init: has version" || fail "init: has version"
grep -q 'name = "testproject"' Kylefile && pass "init: has project name" || fail "init: has project name"
rm Kylefile

info "Testing init --yaml..."
echo "n" | $KYLE init yamlproject --yaml > /dev/null
grep -q "# kyle: yaml" Kylefile && pass "init yaml: has yaml header" || fail "init yaml: has yaml header"
grep -qE 'version: "[0-9]+\.[0-9]+\.[0-9]+"' Kylefile && pass "init yaml: has version" || fail "init yaml: has version"
grep -q "name: yamlproject" Kylefile && pass "init yaml: has project name" || fail "init yaml: has project name"
