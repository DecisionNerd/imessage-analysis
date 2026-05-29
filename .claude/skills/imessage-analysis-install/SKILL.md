---
name: imessage-analysis-install
description: Check whether imessage-analysis is installed and the MCP server is registered, and fix anything that's missing.
---

# iMessage Analysis Install

Verify the installation is complete and working. Install or register anything that's missing.

## When to Use

- After running `npx skills add DecisionNerd/imessage-analysis` for the first time
- "Set up iMessage analysis"
- "Install imessage-analysis"
- "Is imessage-analysis set up?"
- "Register the MCP server"
- On first use if the MCP tools aren't responding

## Steps

### 0 — Install skills (if not already done)

If the user hasn't installed the analysis skills yet, do it now:

```bash
npx skills add DecisionNerd/imessage-analysis
```

This installs all skills including `period-in-review`, `needs-reply`, `contact-deep-dive`, and `compare-contacts`.

### 1 — Check the binary

```bash
which imessage-analysis && imessage-analysis --version
```

**If missing**, install via Homebrew (recommended — handles signing automatically):

```bash
brew tap DecisionNerd/tap && brew install imessage-analysis
```

**If Homebrew isn't available**, install from source and sign manually:

```bash
cargo install --git https://github.com/DecisionNerd/imessage-analysis --bins
```

Then sign it (required for Contacts access to work):

```bash
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
codesign --force --sign - --entitlements /tmp/imessage-entitlements.plist $(which imessage-analysis)
```

### 2 — Check the Contacts entitlement

Even if the binary is already installed, verify it has the required entitlement:

```bash
codesign -d --entitlements - $(which imessage-analysis) 2>&1 | grep contacts
```

If the output is empty, the binary isn't signed correctly — re-run the signing step from above.

### 3 — Check the MCP server binary

```bash
which imessage-mcp
```

If missing, re-run the install step — `imessage-mcp` is installed alongside `imessage-analysis`.

### 4 — Check MCP registration

```bash
claude mcp list   # Claude Code
codex mcp list    # Codex
```

Look for `imessage-analysis` in the output. If it's not listed, register it for whichever CLI the user is running:

**Claude Code:**
```bash
claude mcp add imessage-analysis $(which imessage-mcp)
```

**Codex:**
```bash
codex mcp add imessage-analysis -- $(which imessage-mcp)
```

### 5 — Check the dataset

```bash
imessage-analysis status
```

If the output shows `No dataset found` or `0 messages`, the first sync hasn't been run yet.

**The first sync must be run from Apple Terminal.app.** macOS requires a direct window-server connection to show the Contacts permission dialog — tmux, cmux, iTerm2, and other multiplexers suppress it.

Tell the user clearly:

> Open **Apple Terminal.app** (not your current terminal), run:
>
> ```
> imessage-analysis sync
> ```
>
> Grant Contacts access when the dialog appears. Come back here when it's done.

After that first sync, `imessage-analysis sync` works from any terminal.

### 6 — Report status

Summarise what was found and what (if anything) was fixed:

- Binary: installed at `<path>` ✓ / installed now ✓ / not found ✗
- Contacts entitlement: present ✓ / signed now ✓ / missing ✗
- MCP server: registered ✓ / registered now ✓
- Dataset: `<N>` messages, last synced `<time>` ✓ / needs first sync from Apple Terminal

If everything is good, offer to run `/period-in-review` or `/contact-deep-dive`.
