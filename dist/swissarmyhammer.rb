class Swissarmyhammer < Formula
  desc "MCP (Model Context Protocol) server for managing prompts as markdown files"
  homepage "https://github.com/wballard/swissarmyhammer"
  license "MIT"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/wballard/swissarmyhammer/releases/download/v#{version}/swissarmyhammer-x86_64-apple-darwin"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_X86_64"
    else
      url "https://github.com/wballard/swissarmyhammer/releases/download/v#{version}/swissarmyhammer-aarch64-apple-darwin"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_AARCH64"
    end
  end

  on_linux do
    url "https://github.com/wballard/swissarmyhammer/releases/download/v#{version}/swissarmyhammer-x86_64-unknown-linux-gnu"
    sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_LINUX"
  end

  def install
    bin.install Dir["*"].first => "swissarmyhammer"
  end

  test do
    system "#{bin}/swissarmyhammer", "--version"
    system "#{bin}/swissarmyhammer", "doctor"
  end

  def caveats
    <<~EOS
      To get started with swissarmyhammer:

      1. Run the doctor command to check your setup:
         swissarmyhammer doctor

      2. Add to your Claude Code MCP configuration:
         {
           "mcpServers": {
             "swissarmyhammer": {
               "command": "swissarmyhammer",
               "args": ["serve"]
             }
           }
         }

      3. Create prompts in ~/.swissarmyhammer/prompts/

      For more information, visit: https://github.com/wballard/swissarmyhammer
    EOS
  end
end