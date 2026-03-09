class Rivulet < Formula
  desc "A terminal RSS reader with 3-panel layout, categories, rich preview, and OPML support"
  homepage "https://github.com/elpeix/rivulet"
  license "GPL-3.0-only"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "4acfd9cf0f9fe0debeb013ad5509f74dfd611cae578454a8e3b4a9e1f598acce"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "061ccf7863e93e85010fa3c5f0b3785e9ef72048b42427e39b9e12e7a3fe8035"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "6ed0a15ea5e6c95c7254bbd613199d1d57ed53707130b6fa98be5fd54c0071e6"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0e746ae3f275084fece4d03b66c6596192e69ca44ea69bb40333d1a2c8790a95"
    end
  end

  def install
    bin.install "rivulet"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rivulet --version")
  end
end
