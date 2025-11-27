#!/bin/sh
set -e

REPO="behradeslamifar/kyle"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"

# Colors (will be disabled if not a tty)
if [ -t 1 ]; then
    BOLD='\033[1m'
    DIM='\033[2m'
    GREEN='\033[32m'
    YELLOW='\033[33m'
    CYAN='\033[36m'
    RESET='\033[0m'
else
    BOLD='' DIM='' GREEN='' YELLOW='' CYAN='' RESET=''
fi

print_banner() {
    printf "\n"
    printf "${BOLD}${CYAN}  Kyle Installer${RESET}\n"
    printf "${DIM}  ───────────────${RESET}\n\n"
}

print_step() {
    printf "  ${CYAN}→${RESET} %s\n" "$1"
}

print_success() {
    printf "  ${GREEN}✓${RESET} %s\n" "$1"
}

print_warn() {
    printf "  ${YELLOW}!${RESET} %s\n" "$1"
}

prompt() {
    printf "  %s [%s]: " "$1" "$2"
    read -r input
    echo "${input:-$2}"
}

main() {
    print_banner

    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$ARCH" in
        x86_64) ARCH="amd64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        *)
            print_warn "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    case "$OS" in
        linux|darwin) ;;
        *)
            print_warn "Unsupported OS: $OS"
            print_warn "For Windows, use install.ps1"
            exit 1
            ;;
    esac

    printf "  Detected: ${BOLD}%s/%s${RESET}\n\n" "$OS" "$ARCH"

    INSTALL_DIR=$(prompt "Install location" "$DEFAULT_INSTALL_DIR")

    printf "\n"
    print_step "Fetching latest release..."

    LATEST=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | grep '"tag_name"' | cut -d'"' -f4)

    if [ -z "$LATEST" ]; then
        print_warn "No releases found, building from source..."
        install_from_source "$INSTALL_DIR"
    else
        install_from_release "$INSTALL_DIR" "$LATEST" "$OS" "$ARCH"
    fi

    printf "\n"
    print_success "Installed kyle to $INSTALL_DIR/kyle"

    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        printf "\n"
        print_warn "Add to your PATH:"
        printf "\n"
        printf "      ${DIM}export PATH=\"%s:\$PATH\"${RESET}\n" "$INSTALL_DIR"
        printf "\n"
        printf "  Add this to your ${BOLD}~/.bashrc${RESET} or ${BOLD}~/.zshrc${RESET}\n"
    fi

    printf "\n"
    printf "  Run ${BOLD}kyle help${RESET} to get started.\n\n"
}

install_from_release() {
    INSTALL_DIR="$1"
    VERSION="$2"
    OS="$3"
    ARCH="$4"

    URL="https://github.com/$REPO/releases/download/$VERSION/kyle-${OS}-${ARCH}"

    print_step "Downloading kyle $VERSION..."

    mkdir -p "$INSTALL_DIR"

    if command -v curl > /dev/null; then
        curl -sL "$URL" -o "$INSTALL_DIR/kyle"
    elif command -v wget > /dev/null; then
        wget -q "$URL" -O "$INSTALL_DIR/kyle"
    else
        print_warn "curl or wget required"
        exit 1
    fi

    chmod +x "$INSTALL_DIR/kyle"
}

install_from_source() {
    INSTALL_DIR="$1"

    if ! command -v go > /dev/null; then
        print_warn "Go is required to build from source"
        printf "  Install Go from ${BOLD}https://go.dev/doc/install${RESET}\n"
        exit 1
    fi

    TMPDIR=$(mktemp -d)
    cd "$TMPDIR"

    print_step "Cloning repository..."
    git clone --depth 1 --quiet "https://github.com/$REPO.git" kyle
    cd kyle

    print_step "Building..."
    go build -o kyle ./cmd/do

    mkdir -p "$INSTALL_DIR"
    mv kyle "$INSTALL_DIR/kyle"

    cd /
    rm -rf "$TMPDIR"
}

main
