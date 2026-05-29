#!/usr/bin/env bash
# install.sh — install imessage-analysis and register the MCP server
# Usage: curl -fsSL https://raw.githubusercontent.com/DecisionNerd/imessage-analysis/main/scripts/install.sh | bash
set -euo pipefail

REPO="DecisionNerd/imessage-analysis"
BIN_DIR="${IMESSAGE_BIN_DIR:-$HOME/.local/bin}"
ENTITLEMENTS_PLIST="$(mktemp /tmp/imessage-entitlements.XXXXXX.plist)"

# ── colours ────────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
  BOLD="\033[1m"; GREEN="\033[32m"; YELLOW="\033[33m"; RED="\033[31m"; RESET="\033[0m"
else
  BOLD=""; GREEN=""; YELLOW=""; RED=""; RESET=""
fi

ok()   { echo -e "${GREEN}✓${RESET} $*"; }
warn() { echo -e "${YELLOW}⚠${RESET} $*"; }
err()  { echo -e "${RED}✗${RESET} $*" >&2; exit 1; }
step() { echo -e "\n${BOLD}$*${RESET}"; }

cleanup() { rm -f "$ENTITLEMENTS_PLIST"; }
trap cleanup EXIT

# ── 1. check platform ──────────────────────────────────────────────────────────
step "Checking platform…"
[ "$(uname)" = "Darwin" ] || err "imessage-analysis requires macOS."
ok "macOS detected"

# ── 2. install binary ──────────────────────────────────────────────────────────
step "Installing binary…"

if command -v brew &>/dev/null; then
  if brew list imessage-analysis &>/dev/null 2>&1; then
    ok "imessage-analysis already installed via Homebrew"
  else
    echo "  Installing via Homebrew…"
    brew tap DecisionNerd/tap &>/dev/null
    brew install imessage-analysis
    ok "Installed via Homebrew"
  fi
  IMESSAGE_BIN="$(brew --prefix)/bin/imessage-analysis"
  IMESSAGE_MCP_BIN="$(brew --prefix)/bin/imessage-mcp"
else
  if command -v imessage-analysis &>/dev/null; then
    ok "imessage-analysis already installed at $(which imessage-analysis)"
    IMESSAGE_BIN="$(which imessage-analysis)"
    IMESSAGE_MCP_BIN="$(which imessage-mcp 2>/dev/null || echo "$BIN_DIR/imessage-mcp")"
  else
    echo "  Homebrew not found — building from source (requires Rust)…"
    command -v cargo &>/dev/null || err "Rust/cargo not found. Install from https://rustup.rs or install Homebrew first."
    mkdir -p "$BIN_DIR"
    TMP_DIR="$(mktemp -d)"
    git clone --depth 1 "https://github.com/$REPO.git" "$TMP_DIR" &>/dev/null
    pushd "$TMP_DIR" &>/dev/null
    cargo build --release --locked --bin imessage-analysis --bin imessage-mcp 2>&1 | tail -3
    cp target/release/imessage-analysis "$BIN_DIR/"
    cp target/release/imessage-mcp "$BIN_DIR/"
    popd &>/dev/null
    rm -rf "$TMP_DIR"
    IMESSAGE_BIN="$BIN_DIR/imessage-analysis"
    IMESSAGE_MCP_BIN="$BIN_DIR/imessage-mcp"
    ok "Built and installed to $BIN_DIR"
  fi
fi

# ── 3. sign with Contacts entitlement ──────────────────────────────────────────
step "Signing binary with Contacts entitlement…"

# Check if already signed correctly
if codesign -d --entitlements - "$IMESSAGE_BIN" 2>&1 | grep -q "contacts"; then
  ok "Already signed"
else
  cat > "$ENTITLEMENTS_PLIST" << 'XML'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.contacts.read-write</key>
    <true/>
</dict>
</plist>
XML
  codesign --force --sign - --entitlements "$ENTITLEMENTS_PLIST" "$IMESSAGE_BIN"
  ok "Signed"
fi

# ── 4. register MCP server ─────────────────────────────────────────────────────
step "Registering MCP server…"

REGISTERED=0

# Claude Code
if command -v claude &>/dev/null; then
  if claude mcp list 2>/dev/null | grep -q "imessage-analysis"; then
    ok "Claude Code: already registered"
  else
    claude mcp add imessage-analysis "$IMESSAGE_MCP_BIN"
    ok "Claude Code: registered"
  fi
  REGISTERED=1
fi

# Codex
if command -v codex &>/dev/null; then
  if codex mcp list 2>/dev/null | grep -q "imessage-analysis"; then
    ok "Codex: already registered"
  else
    codex mcp add imessage-analysis -- "$IMESSAGE_MCP_BIN"
    ok "Codex: registered"
  fi
  REGISTERED=1
fi

# Claude Desktop
CLAUDE_DESKTOP_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
if [ -d "$HOME/Library/Application Support/Claude" ]; then
  if [ -f "$CLAUDE_DESKTOP_CONFIG" ] && grep -q "imessage" "$CLAUDE_DESKTOP_CONFIG" 2>/dev/null; then
    ok "Claude Desktop: already registered"
  else
    if [ ! -f "$CLAUDE_DESKTOP_CONFIG" ]; then
      echo '{"mcpServers":{}}' > "$CLAUDE_DESKTOP_CONFIG"
    fi
    # Use python3 to safely merge JSON
    python3 - "$CLAUDE_DESKTOP_CONFIG" "$IMESSAGE_MCP_BIN" << 'PYEOF'
import json, sys
path, mcp_bin = sys.argv[1], sys.argv[2]
with open(path) as f:
    cfg = json.load(f)
cfg.setdefault("mcpServers", {})["imessage-analysis"] = {"command": mcp_bin}
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
PYEOF
    ok "Claude Desktop: registered (restart the app to apply)"
  fi
  REGISTERED=1
fi

# Cursor — global config
CURSOR_CONFIG="$HOME/.cursor/mcp.json"
if [ -d "$HOME/.cursor" ]; then
  if [ -f "$CURSOR_CONFIG" ] && grep -q "imessage" "$CURSOR_CONFIG" 2>/dev/null; then
    ok "Cursor: already registered"
  else
    if [ ! -f "$CURSOR_CONFIG" ]; then
      echo '{"mcpServers":{}}' > "$CURSOR_CONFIG"
    fi
    python3 - "$CURSOR_CONFIG" "$IMESSAGE_MCP_BIN" << 'PYEOF'
import json, sys
path, mcp_bin = sys.argv[1], sys.argv[2]
with open(path) as f:
    cfg = json.load(f)
cfg.setdefault("mcpServers", {})["imessage-analysis"] = {"command": mcp_bin}
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
PYEOF
    ok "Cursor: registered (restart Cursor to apply)"
  fi
  REGISTERED=1
fi

if [ "$REGISTERED" -eq 0 ]; then
  warn "No supported AI client detected. Add this to your client's MCP config manually:"
  echo ""
  echo '  { "mcpServers": { "imessage-analysis": { "command": "'"$IMESSAGE_MCP_BIN"'" } } }'
  echo ""
fi

# ── 5. first sync ──────────────────────────────────────────────────────────────
step "Checking dataset…"

if imessage-analysis status 2>/dev/null | grep -q "Messages:"; then
  ok "Dataset already exists"
  imessage-analysis status
else
  echo ""
  warn "No dataset yet. You need to run the first sync from ${BOLD}Apple Terminal.app${RESET}."
  echo ""
  echo "  macOS requires a direct window-server connection to show the Contacts"
  echo "  permission dialog. Other terminals (iTerm2, tmux, cmux, etc.) suppress it."
  echo ""
  echo -e "  ${BOLD}Open Apple Terminal.app and run:${RESET}"
  echo ""
  echo "    imessage-analysis sync"
  echo ""
  echo "  Grant Contacts access when prompted. After that, sync works from any terminal."
fi

# ── done ───────────────────────────────────────────────────────────────────────
echo ""
ok "Done. Ask your AI agent: \"Who do I text the most?\""
