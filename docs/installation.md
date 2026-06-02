# Installation

## Requirements

- macOS (the iMessage database only exists on Mac)
- For Homebrew / binary installs: no other dependencies
- For source builds: Rust 1.70+
- For the Python package: Python 3.11+

## Homebrew (recommended)

```sh
brew tap DecisionNerd/tap
brew install imessage-analysis
```

Downloads and installs pre-built binaries — no Rust compilation required. This installs two binaries:

| Binary | Purpose |
|---|---|
| `imessage-analysis` | CLI for sync and analysis |
| `imessage-mcp` | MCP server for AI agents |

To upgrade later:

```sh
brew upgrade imessage-analysis
```

The package is distributed via a third-party tap (`decisionnerd/tap`), so it shows as `decisionnerd/tap/imessage-analysis` in `brew upgrade` output rather than just `imessage-analysis`. This is normal — the install and upgrade experience is identical to any other Homebrew package. If it is eventually accepted into homebrew-core, the prefix will go away.

## From source

```sh
git clone https://github.com/DecisionNerd/imessage-analysis
cd imessage-analysis
cargo build --release --locked
```

Binaries are written to `target/release/`. After building, **sign and install** with the `just` recipe — this handles copying and signing the Contacts entitlement in one step:

```sh
just install
```

If you don't have `just`, do it manually:

```sh
cp target/release/imessage-analysis target/release/imessage-mcp ~/.local/bin/

cat > /tmp/imessage-entitlements.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.contacts.read-write</key>
    <true/>
</dict>
</plist>
EOF
codesign --force --sign - --entitlements /tmp/imessage-entitlements.plist ~/.local/bin/imessage-analysis
```

> **Important:** The signing step is required every time you rebuild. Without it, macOS will not show the Contacts permission dialog and all contact names will appear as phone numbers. The Homebrew formula handles this automatically.

To use `just`:

```sh
brew install just
```

## Python package

```sh
pip install imessage-analysis
```

The Python package is query-only. Build or update the dataset using the CLI (`imessage-analysis sync`) first, then use the Python package to query it. Query functions work on any platform where the Parquet dataset is present.

## Permissions

macOS requires **Full Disk Access** for any process that reads `~/Library/Messages/chat.db`.

1. Open **System Settings → Privacy & Security → Full Disk Access**
2. Click **+** and add your terminal application (Terminal, iTerm2, etc.)
3. Re-launch the terminal

Without this, `imessage-analysis sync` will fail with a permissions error.
