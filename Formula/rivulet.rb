class Rivulet < Formula
  desc "A terminal RSS reader with 3-panel layout, categories, rich preview, and OPML support"
  homepage "https://github.com/elpeix/rivulet"
  license "GPL-3.0-only"
  version "1.5.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "2a94816673454f07b75c54456b2e05c8e4146f93004a969b64b1a2bcc9d188e4"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "5b9bf5aadad9469849e4871c3247cce492885a6f158f6153c6907d70f16916f7"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-linux-gnu.tar.gz"
      sha256 "a92602a809da6b55c078c46eea90aeece8ccfbf69d1bd35a117720e6b4b6572f"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-linux-gnu.tar.gz"
      sha256 "8d8259edbf43f2fda0d3a4a2729b19c5cdf9a489b96d70a5b0e7ba4c2be8ddc9"
    end
  end

  def install
    bin.install "rivulet"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rivulet --version")
  end
end
