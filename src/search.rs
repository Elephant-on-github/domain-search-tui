use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::StreamExt;

use crate::pricing::{self, BestEntry, PriceEntry, PricingDb};
use crate::rdap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DomainResult {
    pub domain: String,
    pub tld: String,
    pub available: Option<bool>,
    pub prices: Vec<PriceEntry>,
    pub price_map: HashMap<String, PriceEntry>,
    pub best: Option<BestEntry>,
}

#[derive(Debug, Default)]
pub struct SearchStats {
    pub total: usize,
    pub available: usize,
    pub taken: usize,
    pub unknown: usize,
}

pub async fn run_search(
    query: &str,
    tlds: &[&str],
    pricing: &Arc<PricingDb>,
) -> (Vec<DomainResult>, SearchStats) {
    let pricing = Arc::clone(pricing);

    let futs: Vec<_> = tlds
        .iter()
        .map(|tld| {
            let query = query.to_string();
            let tld_str = tld.to_string();
            let pricing = Arc::clone(&pricing);
            async move {
                let domain = format!("{}.{}", query, tld_str);
                let available = rdap::check_domain(&domain).await;
                let prices = pricing::get_prices(&pricing, &tld_str);
                let price_map: HashMap<String, PriceEntry> = prices
                    .iter()
                    .map(|p| (p.registrar_slug.clone(), p.clone()))
                    .collect();
                let best = pricing::get_best_preferred(&pricing, &tld_str);
                DomainResult {
                    domain,
                    tld: tld_str,
                    available,
                    prices: prices.to_vec(),
                    price_map,
                    best,
                }
            }
        })
        .collect();

    let mut results: Vec<DomainResult> = Vec::with_capacity(futs.len());
    let mut stream: FuturesUnordered<_> = futs.into_iter().map(tokio::spawn).collect();

    let deadline = tokio::time::sleep(Duration::from_secs(6));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            next = stream.next() => match next {
                Some(Ok(r)) => results.push(r),
                Some(Err(_)) => {} // spawn error
                None => break,     // all tasks done
            },
            _ = &mut deadline => break,
        }
    }

    // Abort any remaining tasks
    for task in &stream {
        task.abort();
    }

    let mut stats = SearchStats::default();
    for r in &results {
        stats.total += 1;
        match r.available {
            Some(true) => stats.available += 1,
            Some(false) => stats.taken += 1,
            None => stats.unknown += 1,
        }
    }

    (results, stats)
}

pub fn best_price(entry: &DomainResult) -> Option<f64> {
    // Preference-weighted best price (used for sorting)
    if let Some(ref best) = entry.best {
        if let Some(p) = best.price {
            return Some(crate::config::adjusted_price(p, &best.slug));
        }
    }
    // Fallback: cheapest among tracked registrars (no preference data)
    entry
        .prices
        .iter()
        .filter_map(|p| p.register)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

pub fn status_symbol(available: Option<bool>) -> &'static str {
    match available {
        Some(true) => "✓ FREE",
        Some(false) => "✗ TAKEN",
        None => "? UNKNOWN",
    }
}
