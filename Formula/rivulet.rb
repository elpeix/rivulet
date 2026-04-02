class Rivulet < Formula
  desc "A terminal RSS reader with 3-panel layout, categories, rich preview, and OPML support"
  homepage "https://github.com/elpeix/rivulet"
  license "GPL-3.0-only"
  version "1.3.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "3d5fc85bad552937901c7114dd63d289a9acd81387b5a97644913937c62e78ce"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "b9f2ef6a83378e20e1c4009be1d55cdece1ed5498ea56ff529b395fc9088fa48"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-linux-gnu.tar.gz"
      sha256 "edfa5a4d00792b62b880a01ecaeaba5fdbb86eb48dd7684d8901d20e08e25404"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-linux-gnu.tar.gz"
      sha256 "6825e2f9deff0873df755dce8cb434f5b9334dbe5d6891d41ba42c1054c71eee"
    end
  end

  def install
    bin.install "rivulet"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rivulet --version")
  end
end
