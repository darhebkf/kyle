#!/bin/bash
# Shared test helpers for kyle shell tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KYLE="$SCRIPT_DIR/target/release/kyle"
TEMP_DIR="/tmp/kyle-test-$$"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}✓${NC} $1"; }
fail() { echo -e "${RED}✗${NC} $1"; exit 1; }
info() { echo -e "${YELLOW}→${NC} $1"; }

setup_temp() {
    mkdir -p "$TEMP_DIR"
    cd "$TEMP_DIR"
}

cleanup_temp() {
    rm -rf "$TEMP_DIR"
}

trap cleanup_temp EXIT
