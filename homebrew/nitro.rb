class Nitro < Formula
  desc "Fast, modern package manager leveraging Homebrew formulae"
  homepage "https://github.com/bengold/Nitro"
  url "https://github.com/bengold/Nitro/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/bengold/Nitro.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def post_install
    # Create data directories
    (var/"nitro").mkpath
    (var/"nitro/taps").mkpath
    (var/"nitro/cache").mkpath
  end

  test do
    assert_match "Nitro", shell_output("#{bin}/nitro --version")
    assert_match "Package manager", shell_output("#{bin}/nitro --help")
  end

  def caveats
    <<~EOS
      To get started with Nitro:
        nitro homebrew import    # Import existing Homebrew taps
        nitro search <package>   # Search for packages
        nitro install <package>  # Install packages

      Nitro data is stored in:
        #{var}/nitro
    EOS
  end
end