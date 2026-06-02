# Reference copy only — kept for git history and local testing.
# The canonical formula that users install from lives in github.com/DecisionNerd/homebrew-tap
# and is updated automatically by the release workflow via Formula/imessage-analysis.rb.tmpl.

class ImessageAnalysis < Formula
  desc "Query and analyse your iMessage history — AI agent, CLI, or Python"
  homepage "https://github.com/DecisionNerd/imessage-analysis"
  url "https://github.com/DecisionNerd/imessage-analysis/releases/download/v0.1.3/imessage-analysis-0.1.3-macos-arm64.tar.gz"
  sha256 ""
  license "GPL-3.0-only"
  version "0.1.3"

  depends_on :macos

  def install
    bin.install "imessage-analysis"
    bin.install "imessage-mcp"

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

    generate_completions_from_executable(bin/"imessage-analysis", "completions")
  end

  test do
    system "#{bin}/imessage-analysis", "--version"
  end
end
