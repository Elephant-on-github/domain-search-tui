class DomainSearchTui < Formula
  desc "Terminal app that checks domain availability across 20 TLDs and compares registrar prices"
  homepage "https://github.com/Elephant-on-github/domain-search-tui"
  license "MIT"

  stable do
    on_macos do
      url "https://github.com/Elephant-on-github/domain-search-tui/releases/download/v1.0.0/domain_search_aggregator_tui-x86_64-apple-darwin.tar.gz"
      sha256 "TODO"
    end

    on_linux do
      url "https://github.com/Elephant-on-github/domain-search-tui/releases/download/v1.0.0/domain_search_aggregator_tui-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "TODO"
    end
  end

  def install
    bin.install "domain_search_aggregator_tui"
  end
end
