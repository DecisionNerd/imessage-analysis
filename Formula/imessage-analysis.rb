# Reference copy — the canonical formula lives in github.com/DecisionNerd/homebrew-tap.
# This file is updated automatically by the release workflow.

class ImessageAnalysis < Formula
  desc "Extract, query, and analyse your Mac iMessage history"
  homepage "https://github.com/DecisionNerd/imessage-analysis"
  url "https://github.com/DecisionNerd/imessage-analysis/archive/refs/tags/v0.1.0.tar.gz"
  sha256 ""
  license "GPL-3.0-only"

  depends_on "rust" => :build
  depends_on :macos

  def install
    system "cargo", "build", "--release", "--locked",
           "--bin", "imessage-analysis",
           "--bin", "imessage-mcp"
    bin.install "target/release/imessage-analysis"
    bin.install "target/release/imessage-mcp"

    # Sign with Contacts entitlement so macOS shows the permission dialog on first sync.
    # Without this the binary has no bundle identifier and TCC silently denies it.
    entitlements = buildpath/"entitlements.plist"
    entitlements.write <<~XML
      <?xml version="1.0" encoding="UTF-8"?>
      <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
      <plist version="1.0">
      <dict>
          <key>com.apple.security.contacts.read-write</key>
          <true/>
      </dict>
      </plist>
    XML
    system "codesign", "--force", "--sign", "-",
           "--entitlements", entitlements, bin/"imessage-analysis"

    # Shell completions
    generate_completions_from_executable(bin/"imessage-analysis", "completions")
  end

  test do
    system "#{bin}/imessage-analysis", "--version"
  end
end
