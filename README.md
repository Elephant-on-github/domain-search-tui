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

grab a binary from [Releases](https://github.com/Elephant-on-github/domain-search-tui/releases), or build it yourself:

```bash
cargo install --git https://github.com/Elephant-on-github/domain-search-tui
```

requirements: Rust 1.75+, a terminal that likes ratatui (most do), and an internet connection for the first run (prices get cached after that).

## data sources

| source | what for |
|---|---|
| [rdap.org](https://rdap.org) | real-time availability (free, standardized) |
| [tldwise.com](https://tldwise.com) | registrar-pricing matrix + home-price-index (free, hourly) |

## why though

because clicking through 20 registrar tabs to compare prices on a domain name is a waste of a perfectly good afternoon.

## license

MIT
