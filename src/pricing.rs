use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config;

fn cache_dir() -> Result<PathBuf> {
    let dir = directories::ProjectDirs::from("", "", config::APP_NAME)
        .ok_or_else(|| anyhow::anyhow!("cannot find project directories"))?;
    let cache = dir.cache_dir().to_path_buf();
    std::fs::create_dir_all(&cache)?;
    Ok(cache)
}

fn cache_path() -> Result<PathBuf> {
    Ok(cache_dir()?.join("prices.json"))
}

// ── Price matrix (per-registrar) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceMatrixData {
    #[serde(default)]
    pub rows: Vec<PriceMatrixRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceMatrixRow {
    pub tld: String,
    #[serde(default)]
    pub cells: Vec<RegistrarCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrarCell {
    #[serde(default)]
    pub registrar_slug: String,
    #[serde(default)]
    pub registrar_name: String,
    #[serde(default)]
    pub register_usd: Option<f64>,
    #[serde(default)]
    pub renew_usd: Option<f64>,
    #[serde(default)]
    pub transfer_usd: Option<f64>,
    #[serde(default)]
    pub captured_at: Option<String>,
}

// ── Price index (best price from any registrar) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceIndexData {
    #[serde(default)]
    pub rows: Vec<PriceIndexRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceIndexRow {
    pub tld: String,
    #[serde(default)]
    pub register: Option<PriceIndexEntry>,
    #[serde(default)]
    pub renew: Option<PriceIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceIndexEntry {
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub usd: Option<f64>,
}

// ── Final data ──

#[derive(Debug, Clone)]
pub struct PricingDb {
    pub by_tld: HashMap<String, Vec<PriceEntry>>,
    pub best_by_tld: HashMap<String, BestEntry>,
    pub fetched_at: SystemTime,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PriceEntry {
    pub registrar_slug: String,
    pub registrar_name: String,
    pub register: Option<f64>,
    pub renew: Option<f64>,
    #[allow(dead_code)]
    pub transfer: Option<f64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BestEntry {
    pub slug: String,
    pub name: String,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheFile {
    fetched_at_epoch: u64,
    matrix: PriceMatrixData,
    index: Option<PriceIndexData>,
}

pub async fn fetch_pricing() -> Result<PricingDb> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(45))
        .user_agent("tldscan/0.1")
        .build()?;

    let (matrix_resp, index_resp) = tokio::join!(
        client.get(config::PRICING_URL).send(),
        client.get(config::PRICE_INDEX_URL).send(),
    );

    let matrix: PriceMatrixData = matrix_resp?.json().await?;
    let index: Option<PriceIndexData> = match index_resp {
        Ok(resp) => resp.json().await.ok(),
        Err(_) => None,
    };

    if let Err(e) = save_cache(&matrix, &index).await {
        eprintln!("warning: failed to cache pricing: {e}");
    }

    Ok(build_db(matrix, index, SystemTime::now()))
}

fn build_db(
    matrix: PriceMatrixData,
    index: Option<PriceIndexData>,
    fetched_at: SystemTime,
) -> PricingDb {
    let mut by_tld = HashMap::new();
    for row in matrix.rows {
        let entries: Vec<PriceEntry> = row
            .cells
            .into_iter()
            .filter(|c| c.register_usd.is_some() || c.renew_usd.is_some())
            .map(|c| PriceEntry {
                registrar_slug: c.registrar_slug,
                registrar_name: c.registrar_name,
                register: c.register_usd,
                renew: c.renew_usd,
                transfer: c.transfer_usd,
            })
            .collect();
        by_tld.insert(row.tld.to_lowercase(), entries);
    }

    let mut best_by_tld = HashMap::new();
    if let Some(index) = index {
        for row in index.rows {
            let entry = BestEntry {
                slug: row
                    .register
                    .as_ref()
                    .map(|r| r.slug.clone())
                    .unwrap_or_default(),
                name: row
                    .register
                    .as_ref()
                    .map(|r| r.name.clone())
                    .unwrap_or_default(),
                price: row.register.as_ref().and_then(|r| r.usd),
            };
            best_by_tld.insert(row.tld.to_lowercase(), entry);
        }
    }

    PricingDb {
        by_tld,
        best_by_tld,
        fetched_at,
    }
}

async fn save_cache(matrix: &PriceMatrixData, index: &Option<PriceIndexData>) -> Result<()> {
    let path = cache_path()?;
    let cache = CacheFile {
        fetched_at_epoch: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        matrix: matrix.clone(),
        index: index.clone(),
    };
    let json = serde_json::to_string(&cache)?;
    std::fs::write(&path, &json)?;
    Ok(())
}

pub fn load_cached_pricing() -> Option<PricingDb> {
    let path = cache_path().ok()?;
    let raw = std::fs::read_to_string(path).ok()?;
    let cache: CacheFile = serde_json::from_str(&raw).ok()?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let age_hours = (now - cache.fetched_at_epoch) / 3600;
    if age_hours > config::CACHE_TTL_HOURS {
        return None;
    }

    let fetched_at = SystemTime::UNIX_EPOCH + Duration::from_secs(cache.fetched_at_epoch);
    Some(build_db(cache.matrix, cache.index, fetched_at))
}

pub fn get_prices<'a>(db: &'a PricingDb, tld: &str) -> &'a [PriceEntry] {
    db.by_tld
        .get(&tld.to_lowercase())
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

pub fn get_best<'a>(db: &'a PricingDb, tld: &str) -> Option<&'a BestEntry> {
    db.best_by_tld.get(&tld.to_lowercase())
}

/// Returns the best entry after applying the preferred-registrar weighting.
/// If a preferred tracked registrar is within ~5% of the index champion,
/// we show that familiar registrar instead.  Returns the **raw** price
/// (not adjusted) so the UI column stays truthful.
pub fn get_best_preferred(db: &PricingDb, tld: &str) -> Option<BestEntry> {
    let index_entry = get_best(db, tld)?;
    let raw_champ_price = index_entry.price?;

    // Gather candidates: the index champ + every preferred tracked registrar
    let mut candidates: Vec<(String, String, f64)> = Vec::new();
    candidates.push((
        index_entry.slug.clone(),
        index_entry.name.clone(),
        raw_champ_price,
    ));

    for pe in get_prices(db, tld) {
        if let Some(reg_price) = pe.register {
            if config::is_preferred(&pe.registrar_slug) {
                candidates.push((
                    pe.registrar_slug.clone(),
                    pe.registrar_name.clone(),
                    reg_price,
                ));
            }
        }
    }

    // Pick the candidate with the lowest ADJUSTED price
    candidates
        .into_iter()
        .min_by(|(s1, _, p1), (s2, _, p2)| {
            let a1 = config::adjusted_price(*p1, s1);
            let a2 = config::adjusted_price(*p2, s2);
            a1.partial_cmp(&a2).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(slug, name, price)| BestEntry {
            slug,
            name,
            price: Some(price),
        })
}

pub fn format_price(price: Option<f64>) -> String {
    match price {
        Some(p) => format!("${:.2}", p),
        None => "—".to_string(),
    }
}

pub fn age_string(db: &PricingDb) -> String {
    let now = SystemTime::now();
    let age = now.duration_since(db.fetched_at).unwrap_or_default();
    let hours = age.as_secs() / 3600;
    let mins = (age.as_secs() % 3600) / 60;
    if hours > 0 {
        format!("{h}h{m:02}m ago", h = hours, m = mins)
    } else {
        format!("{m}min ago", m = mins)
    }
}
