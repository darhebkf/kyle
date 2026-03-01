#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Format Detection ==="
setup_temp

cat > Kylefile << 'EOF'
# kyle: toml
name = "test"
[tasks.hello]
run = "echo hello-toml"
EOF
$KYLE hello 2>&1 | grep -q "hello-toml" && pass "format detection: toml header" || fail "format detection: toml header"
rm Kylefile

cat > Kylefile.yaml << 'EOF'
name: test
tasks:
  hello:
    run: echo hello-yaml
EOF
$KYLE hello 2>&1 | grep -q "hello-yaml" && pass "format detection: .yaml extension" || fail "format detection: .yaml extension"
rm Kylefile.yaml

cat > Kylefile.toml << 'EOF'
name = "test"
[tasks.hello]
run = "echo hello-toml-ext"
EOF
$KYLE hello 2>&1 | grep -q "hello-toml-ext" && pass "format detection: .toml extension" || fail "format detection: .toml extension"
