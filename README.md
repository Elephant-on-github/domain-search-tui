# domain-search-tui

[![Built with OpenCode](https://img.shields.io/badge/Built%20with-OpenCode-000?style=flat-square&labelColor=000&color=555)](https://opencode.ai)

a lil terminal app that checks domain availability across 20 TLDs and shows you who's got the best price — all in one table. no api keys needed.

```
> coffeeshop
┌──────────────────────────────────────────────────────────────┐
│ Domain          Status  Best            Cloudflare  Porkbun  │
├──────────────────────────────────────────────────────────────┤
▸ coffeeshop.com  ✓ FREE  $10.30 Porkbun  $10.50     $10.30   │
  coffeeshop.io   ✗ TAKEN $35.82 OVHcloud $38.00     $37.50   │
  coffeeshop.dev  ✓ FREE  $12.41 Cloudflare $12.41   $13.00   │
  ...
```

## features

- **parallel rdap checks** — 20 TLDs checked at once, results in ~4s
- **pricing from tldwise.com** — free hourly-updated feed covers 6 major registrars + a 50-registrar index
- **preferred registrar weighting** — well-known US/EU registrars get a ~5% boost when picking the "Best" column, so you're not sent to some random shop to save 30¢
- **open in browser** — hit `+` to open the cheapest registrar's site with your domain pre-filled, or arrow through columns and press Enter for a specific one
- **cache to disk** — pricing data cached ~24h, background refresh on startup
- **sort any way** — Tab cycles through domain/price/availability sort

## keys

| key | what it does |
|---|---|
| type | start typing to search |
| Enter | run the search |
| `+` (shift+=) | open cheapest registrar in browser |
| ← → | select a registrar column |
| Tab | cycle sort mode |
| ↑ ↓ | navigate results |
| Esc | clear everything |
| q | quit |
| Ctrl+C | also quit |

## install

### option 1: binary (easiest)

grab the right file for your OS from [Releases](https://github.com/Elephant-on-github/domain-search-tui/releases), unzip it, and chuck it somewhere in your `PATH`.

| platform | file | what to do |
|---|---|---|
| **Windows** | `domain_search_aggregator_tui-x86_64-pc-windows-msvc.zip` | unzip, run `domain_search_aggregator_tui.exe` |
| **macOS** | `domain_search_aggregator_tui-x86_64-apple-darwin.tar.gz` | `tar xzf file.tar.gz && mv domain_search_aggregator_tui /usr/local/bin/` |
| **Linux** | `domain_search_aggregator_tui-x86_64-unknown-linux-gnu.tar.gz` | same as macOS |

### option 2: cargo install

requires a Rust toolchain (1.75+). takes a minute to compile.

```bash
cargo install --git https://github.com/Elephant-on-github/domain-search-tui
```

### option 3: build from source

```bash
git clone https://github.com/Elephant-on-github/domain-search-tui
cd domain-search-tui
cargo build --release
# binary at ./target/release/domain_search_aggregator_tui
```

### requirements

- a terminal that supports ratatui (most do — Windows Terminal, iTerm2, gnome-terminal, etc.)
- internet on first run (pricing data gets cached for ~24h after that)

## data sources

| source | what for |
|---|---|
| [rdap.org](https://rdap.org) | real-time availability (free, standardized) |
| [tldwise.com](https://tldwise.com) | registrar-pricing matrix + home-price-index (free, hourly) |

## why though

because clicking through 20 registrar tabs to compare prices on a domain name is a waste of a perfectly good afternoon.

## license

MIT
