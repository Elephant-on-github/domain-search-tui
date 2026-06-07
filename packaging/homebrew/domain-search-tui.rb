class DomainSearchTui < Formula
  desc "Terminal app that checks domain availability across 20 TLDs and compares registrar prices"
  homepage "https://github.com/Elephant-on-github/domain-search-tui"
  license "MIT"

  stable do
    on_macos do
      url "https://github.com/Elephant-on-github/domain-search-tui/releases/download/v1.0.0/domain-search-tui-x86_64-apple-darwin.tar.gz"
      sha256 "42573a545682b3c37bca1326bba862adab566d02ada35b0ae9dfc6fc538d98e3"
    end

    on_linux do
      url "https://github.com/Elephant-on-github/domain-search-tui/releases/download/v1.0.0/domain-search-tui-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "ec41205d903cde6ba5b843d9b7b5669ba39710125fc4b073f05bdc30a2039783"
    end
  end

  def install
    bin.install "domain-search-tui"
  end
end
