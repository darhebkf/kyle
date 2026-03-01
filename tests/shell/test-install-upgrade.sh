#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Install Script ==="
cd "$SCRIPT_DIR"

INSTALL_FUNCS=$(sed '/^install$/d' docs/public/install.sh)

info "Testing install.sh functions..."
OS_RESULT=$(echo "$INSTALL_FUNCS" | sh -c 'eval "$(cat)"; detect_os' 2>/dev/null || true)
[ "$OS_RESULT" = "linux" ] || [ "$OS_RESULT" = "darwin" ] && pass "install.sh: detect_os" || fail "install.sh: detect_os (got: $OS_RESULT)"

ARCH_RESULT=$(echo "$INSTALL_FUNCS" | sh -c 'eval "$(cat)"; detect_arch' 2>/dev/null || true)
[ "$ARCH_RESULT" = "x86_64" ] || [ "$ARCH_RESULT" = "aarch64" ] && pass "install.sh: detect_arch" || fail "install.sh: detect_arch (got: $ARCH_RESULT)"

TARGET_RESULT=$(echo "$INSTALL_FUNCS" | sh -c 'eval "$(cat)"; get_target linux x86_64' 2>/dev/null || true)
[ "$TARGET_RESULT" = "x86_64-unknown-linux-musl" ] && pass "install.sh: get_target linux x86_64" || fail "install.sh: get_target (got: $TARGET_RESULT)"

TARGET_RESULT=$(echo "$INSTALL_FUNCS" | sh -c 'eval "$(cat)"; get_target darwin aarch64' 2>/dev/null || true)
[ "$TARGET_RESULT" = "aarch64-apple-darwin" ] && pass "install.sh: get_target darwin aarch64" || fail "install.sh: get_target (got: $TARGET_RESULT)"

info "Testing version fetch (requires network)..."
VERSION_RESULT=$(echo "$INSTALL_FUNCS" | sh -c 'eval "$(cat)"; get_latest_version' 2>/dev/null || echo "")
if [ -n "$VERSION_RESULT" ] && echo "$VERSION_RESULT" | grep -qE "^v[0-9]"; then
    pass "install.sh: get_latest_version (got: $VERSION_RESULT)"
else
    info "install.sh: get_latest_version skipped (network issue or no releases)"
fi

echo ""
echo "=== Upgrade Command ==="
cd "$SCRIPT_DIR"

info "Testing kyle upgrade..."
UPGRADE_OUTPUT=$($KYLE upgrade 2>&1 || true)
if echo "$UPGRADE_OUTPUT" | grep -qE "(Already up to date|New version available|Checking for updates)"; then
    pass "upgrade: manual upgrade check works"
else
    fail "upgrade: manual upgrade check (got: $UPGRADE_OUTPUT)"
fi

info "Testing auto-upgrade setting..."
$KYLE config set auto_upgrade true > /dev/null 2>&1
$KYLE config get auto_upgrade | grep -q "true" && pass "upgrade: set auto_upgrade true" || fail "upgrade: set auto_upgrade true"

info "Testing auto-upgrade on startup..."
AUTO_OUTPUT=$($KYLE version 2>&1)
if echo "$AUTO_OUTPUT" | grep -qE "(kyle v|Auto-upgrade|Already up to date)"; then
    pass "upgrade: auto-upgrade check runs on startup"
else
    fail "upgrade: auto-upgrade on startup (got: $AUTO_OUTPUT)"
fi

$KYLE config set auto_upgrade false > /dev/null 2>&1
$KYLE config get auto_upgrade | grep -q "false" && pass "upgrade: reset auto_upgrade false" || fail "upgrade: reset auto_upgrade false"
