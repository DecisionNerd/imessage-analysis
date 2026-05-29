BINARY := `which imessage-analysis 2>/dev/null || echo ~/.local/bin/imessage-analysis`
MCP_BINARY := `which imessage-mcp 2>/dev/null || echo ~/.local/bin/imessage-mcp`

# Build, sign, and install both binaries
install: build sign copy
    @echo "✓ Installed imessage-analysis and imessage-mcp"

# Build release binaries
build:
    cargo build --release --bin imessage-analysis --bin imessage-mcp

# Sign the binary with Contacts entitlement
sign:
    #!/usr/bin/env bash
    set -euo pipefail
    PLIST=$(mktemp /tmp/imessage-entitlements.XXXXXX.plist)
    cat > "$PLIST" << 'EOF'
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
        <key>com.apple.security.contacts.read-write</key>
        <true/>
    </dict>
    </plist>
    EOF
    codesign --force --sign - --entitlements "$PLIST" target/release/imessage-analysis
    rm "$PLIST"

# Copy binaries to install location
copy:
    #!/usr/bin/env bash
    set -euo pipefail
    DEST=$(dirname {{BINARY}})
    mkdir -p "$DEST"
    cp target/release/imessage-analysis "$DEST/imessage-analysis"
    cp target/release/imessage-mcp "$DEST/imessage-mcp"
    echo "  → $DEST/imessage-analysis"
    echo "  → $DEST/imessage-mcp"

# Register the MCP server with Claude Code
register:
    claude mcp add imessage-analysis {{MCP_BINARY}}
    @echo "✓ Registered imessage-analysis MCP server"

# Full setup: build, sign, install, register
setup: install register
    @echo ""
    @echo "✓ Ready. Open Apple Terminal.app and run:"
    @echo "    imessage-analysis sync"
    @echo ""
    @echo "  Grant Contacts access when prompted, then come back here."

# Run tests
test:
    cargo test --all

# Lint
lint:
    cargo clippy -- -D warnings
    cargo fmt --check
