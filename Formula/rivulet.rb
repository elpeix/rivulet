class Rivulet < Formula
  desc "A terminal RSS reader with 3-panel layout, categories, rich preview, and OPML support"
  homepage "https://github.com/elpeix/rivulet"
  license "GPL-3.0-only"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "rivulet"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rivulet --version")
  end
end
