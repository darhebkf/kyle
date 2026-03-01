#!/bin/sh
# Kyle installer for Unix systems (Linux, macOS)
# Usage: curl -fsSL https://kylefile.dev/install.sh | sh

set -e

REPO="darhebkf/kyle"
INSTALL_DIR="${KYLE_INSTALL_DIR:-$HOME/.local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { printf "${GREEN}info${NC}: %s\n" "$1"; }
warn() { printf "${YELLOW}warn${NC}: %s\n" "$1"; }
error() { printf "${RED}error${NC}: %s\n" "$1" >&2; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "darwin" ;;
        *)       error "Unsupported OS: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Map to target triple
get_target() {
    local os="$1"
    local arch="$2"

    case "$os-$arch" in
        linux-x86_64)   echo "x86_64-unknown-linux-musl" ;;
        linux-aarch64)  echo "aarch64-unknown-linux-musl" ;;
        darwin-x86_64)  echo "x86_64-apple-darwin" ;;
        darwin-aarch64) echo "aarch64-apple-darwin" ;;
        *)              error "Unsupported platform: $os-$arch" ;;
    esac
}

# Get latest release version
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
        grep '"tag_name":' |
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
install() {
    local os=$(detect_os)
    local arch=$(detect_arch)
    local target=$(get_target "$os" "$arch")

    info "Detected platform: $os-$arch ($target)"

    # Get version
    local version="${KYLE_VERSION:-$(get_latest_version)}"
    if [ -z "$version" ]; then
        error "Could not determine latest version"
    fi
    info "Installing kyle $version"

    # Download URL
    local url="https://github.com/$REPO/releases/download/$version/kyle-$target.tar.gz"

    # Create temp directory
    local tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    # Download
    info "Downloading from $url"
    curl -fsSL "$url" -o "$tmp_dir/kyle.tar.gz" || error "Download failed"

    # Extract
    tar -xzf "$tmp_dir/kyle.tar.gz" -C "$tmp_dir" || error "Extraction failed"

    # Install
    mkdir -p "$INSTALL_DIR"
    mv "$tmp_dir/kyle" "$INSTALL_DIR/kyle" || error "Installation failed"
    chmod +x "$INSTALL_DIR/kyle"

    info "Installed kyle to $INSTALL_DIR/kyle"

    # Detect shell profile
    local profile=""
    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        profile="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        profile="$HOME/.bashrc"
    elif [ -f "$HOME/.profile" ]; then
        profile="$HOME/.profile"
    fi

    local path_added=false

    # Check if in PATH
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        printf "Add to PATH? [Y/n] "
        read -r answer
        if [ "$answer" != "n" ] && [ "$answer" != "N" ]; then
            if [ -n "$profile" ]; then
                echo "" >> "$profile"
                echo "# Kyle" >> "$profile"
                echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$profile"
                info "Added to $profile"
                path_added=true
            else
                warn "Could not detect shell profile"
                echo ""
                echo "Add this manually to your shell profile:"
                echo ""
                echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
            fi
        fi
        echo ""
    fi

    # Auto-upgrade prompt
    echo ""
    printf "Enable automatic updates? [y/N] "
    read -r answer
    if [ "$answer" = "y" ] || [ "$answer" = "Y" ]; then
        "$INSTALL_DIR/kyle" config set auto_upgrade true 2>/dev/null && info "Auto-upgrade enabled"
    fi

    # Shell completions prompt
    echo ""
    printf "Install shell completions? [Y/n] "
    read -r answer
    if [ "$answer" != "n" ] && [ "$answer" != "N" ]; then
        if [ -n "$profile" ]; then
            local shell_type=""
            case "$profile" in
                *.zshrc) shell_type="zsh" ;;
                *)       shell_type="bash" ;;
            esac
            echo "" >> "$profile"
            echo "# Kyle completions" >> "$profile"
            echo 'eval "$('$INSTALL_DIR'/kyle completions '$shell_type')"' >> "$profile"
            info "Shell completions added to $profile"
        else
            warn "Could not detect shell profile — run 'kyle completions bash >> ~/.bashrc' manually"
        fi
    fi

    # MCP setup prompt
    echo ""
    printf "Set up MCP for AI tools? [y/N] "
    read -r answer
    if [ "$answer" = "y" ] || [ "$answer" = "Y" ]; then
        echo ""
        echo "  1) Claude Code"
        echo "  2) Cursor"
        echo "  3) Windsurf"
        echo "  4) Skip"
        echo ""
        printf "Select AI client [1-4]: "
        read -r client

        local mcp_config='{"mcpServers":{"kyle":{"command":"'$INSTALL_DIR'/kyle","args":["mcp"]}}}'

        case "$client" in
            1)
                local cc_dir="$HOME/.claude"
                mkdir -p "$cc_dir"
                local cc_file="$cc_dir/claude_desktop_config.json"
                if [ -f "$cc_file" ]; then
                    warn "$cc_file already exists — add kyle MCP manually:"
                    echo ""
                    echo "  $INSTALL_DIR/kyle mcp --config"
                else
                    echo "$mcp_config" > "$cc_file"
                    info "MCP config written to $cc_file"
                fi
                ;;
            2)
                local cursor_dir="$HOME/.cursor"
                mkdir -p "$cursor_dir"
                local cursor_file="$cursor_dir/mcp.json"
                if [ -f "$cursor_file" ]; then
                    warn "$cursor_file already exists — add kyle MCP manually:"
                    echo ""
                    echo "  $INSTALL_DIR/kyle mcp --config"
                else
                    echo "$mcp_config" > "$cursor_file"
                    info "MCP config written to $cursor_file"
                fi
                ;;
            3)
                local ws_dir="$HOME/.codeium/windsurf"
                mkdir -p "$ws_dir"
                local ws_file="$ws_dir/mcp_config.json"
                if [ -f "$ws_file" ]; then
                    warn "$ws_file already exists — add kyle MCP manually:"
                    echo ""
                    echo "  $INSTALL_DIR/kyle mcp --config"
                else
                    echo "$mcp_config" > "$ws_file"
                    info "MCP config written to $ws_file"
                fi
                ;;
            *)
                info "Skipped MCP setup. Run 'kyle mcp --config' anytime to get the config."
                ;;
        esac
    fi

    echo ""
    printf "${GREEN}✓${NC} kyle $version installed successfully!\n"
    echo ""

    # Verify installation
    if command -v "$INSTALL_DIR/kyle" >/dev/null 2>&1; then
        info "Verified: $("$INSTALL_DIR/kyle" --version 2>/dev/null)"
    fi

    echo ""
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo "Run 'kyle --help' to get started."
    elif [ "$path_added" = true ]; then
        echo "To start using kyle, either:"
        echo ""
        echo "  1. Open a new terminal, or"
        echo "  2. Run: source $profile"
    fi
}

install
