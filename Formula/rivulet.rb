class Rivulet < Formula
  desc "A terminal RSS reader with 3-panel layout, categories, rich preview, and OPML support"
  homepage "https://github.com/elpeix/rivulet"
  license "GPL-3.0-only"
  version "1.2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "7de030fc5ad2386b1fe6b1907a35c7eceb23ae7eae84421a0c7664d41a09b074"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "3156f830a0e8b3c31293217c657b06677a2734416dd1b258066b254901d40b08"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-linux-gnu.tar.gz"
      sha256 "5572f6af45eafe243a128c5082931c6dab853efaae243dc7c6cf0b6790d82713"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-linux-gnu.tar.gz"
      sha256 "1ea645db5b399bb0aaa9b27782585b59b1da170e1f724776159b95c648f5f882"
    end
  end

  def install
    bin.install "rivulet"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rivulet --version")
  end
end
