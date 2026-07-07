class Rivulet < Formula
  desc "A terminal RSS reader with 3-panel layout, categories, rich preview, and OPML support"
  homepage "https://github.com/elpeix/rivulet"
  license "GPL-3.0-only"
  version "1.5.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "408e8bf407377978ee8a498a048e362f7bfa49acec77078394cd1772165426b0"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "35be92f02fb752aebd11f7707daba33a4510f8137fc4f6bacd6b64697f711e32"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-aarch64-linux-gnu.tar.gz"
      sha256 "3c21cd06786b61d1d6ecdf207dc6568dcae0ee945a668958654c7e12aad60ed2"
    else
      url "https://github.com/elpeix/rivulet/releases/download/v#{version}/rivulet-v#{version}-x86_64-linux-gnu.tar.gz"
      sha256 "c37481dc2cf82c588c31ce0f90af1f65971bf049f04b54229a725f131e68f445"
    end
  end

  def install
    bin.install "rivulet"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rivulet --version")
  end
end
