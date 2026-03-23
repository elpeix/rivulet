class Rivulet < Formula
  desc "A terminal RSS reader with 3-panel layout, categories, rich preview, and OPML support"
  homepage "https://github.com/elpeix/rivulet"
  license "GPL-3.0-only"
  version "1.3.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "534b8d89eb3f16d90b57b7dd3278b65e63ef1e33b7d5c1a2b39a08f4e81e4f08"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "2172fcbeb43ee9f950fd8c76de479b38d9f4f297417e71643e0889055ec70dd2"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-linux-gnu.tar.gz"
      sha256 "71f7a1b01f4533df5f4de608d4b78f7dcf44b54b6e87f29b3deccbc1f2b450c3"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-linux-gnu.tar.gz"
      sha256 "d0f37de5e84e12013e14d53dd65acf3525af8c5526a5a3d8562fdf68d7af7fe6"
    end
  end

  def install
    bin.install "rivulet"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rivulet --version")
  end
end
