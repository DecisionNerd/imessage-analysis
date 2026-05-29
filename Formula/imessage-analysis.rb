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

    # Shell completions
    generate_completions_from_executable(bin/"imessage-analysis", "completions")
  end

  test do
    system "#{bin}/imessage-analysis", "--version"
  end
end
