use std::sync::Arc;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config;
use crate::pricing::PricingDb;
use crate::search::{self, DomainResult, SearchStats};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    Domain,
    Price,
    Status,
}

impl SortBy {
    pub fn label(&self) -> &'static str {
        match self {
            SortBy::Domain => "domain name",
            SortBy::Price => "best price",
            SortBy::Status => "availability",
        }
    }
}

pub struct App {
    pub input: String,
    pub results: Vec<DomainResult>,
    pub stats: SearchStats,
    pub pricing_db: Option<Arc<PricingDb>>,
    pub loading: bool,
    pub has_searched: bool,
    pub error: Option<String>,
    pub selected_row: usize,
    pub selected_column: Option<usize>,
    pub sort_by: SortBy,
    pub scroll_offset: usize,
    pub notification: Option<(String, u8)>,
}

impl App {
    pub fn new(pricing_db: Option<Arc<PricingDb>>) -> Self {
        Self {
            input: String::new(),
            results: Vec::new(),
            stats: SearchStats::default(),
            pricing_db,
            loading: false,
            has_searched: false,
            error: None,
            selected_row: 0,
            selected_column: None,
            sort_by: SortBy::Status,
            scroll_offset: 0,
            notification: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.loading {
            return Ok(());
        }

        match key.code {
            // (+) opens cheapest registrar for selected row
            KeyCode::Char('+') | KeyCode::Char('=') if key.modifiers == KeyModifiers::SHIFT => {
                self.open_cheapest();
            }
            KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                self.selected_column = None;
                self.input.push(c);
            }
            KeyCode::Char(c) if key.modifiers == KeyModifiers::SHIFT => {
                self.selected_column = None;
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                if self.selected_column.is_some() && !self.results.is_empty() {
                    self.open_selected_column();
                } else if !self.input.is_empty() {
                    self.start_search();
                }
            }
            KeyCode::Left => {
                if !self.results.is_empty() {
                    self.move_column(-1);
                    let msg = self.column_notification();
                    self.notification = Some((msg, 12));
                }
            }
            KeyCode::Right => {
                if !self.results.is_empty() {
                    self.move_column(1);
                    let msg = self.column_notification();
                    self.notification = Some((msg, 12));
                }
            }
            KeyCode::Up => {
                if !self.results.is_empty() {
                    self.selected_row = self.selected_row.saturating_sub(1);
                    self.clamp_scroll();
                }
            }
            KeyCode::Down => {
                if !self.results.is_empty() {
                    self.selected_row = (self.selected_row + 1).min(self.results.len() - 1);
                    self.clamp_scroll();
                }
            }
            KeyCode::Tab => {
                if !self.results.is_empty() {
                    self.sort_by = match self.sort_by {
                        SortBy::Status => SortBy::Price,
                        SortBy::Price => SortBy::Domain,
                        SortBy::Domain => SortBy::Status,
                    };
                    self.sort_results();
                    let msg = format!(" Sorted by {} ", self.sort_by.label());
                    self.notification = Some((msg, 12));
                }
            }
            KeyCode::Esc => {
                self.selected_column = None;
                self.input.clear();
                self.results.clear();
                self.has_searched = false;
                self.selected_row = 0;
                self.notification = None;
            }
            _ => {}
        }
        Ok(())
    }

    fn move_column(&mut self, dir: isize) {
        if self.results.is_empty() {
            return;
        }
        let n = config::REGISTRARS.len();
        match self.selected_column {
            None => {
                self.selected_column = if dir > 0 { Some(0) } else { Some(n - 1) };
            }
            Some(i) => {
                let next = i as isize + dir;
                if next < 0 || next >= n as isize {
                    self.selected_column = None;
                } else {
                    self.selected_column = Some(next as usize);
                }
            }
        }
    }

    fn open_cheapest(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let (slug_owned, domain) = {
            let entry = &self.results[self.selected_row];
            let slug = entry.best.as_ref().map(|b| b.slug.clone()).or_else(|| {
                entry
                    .prices
                    .iter()
                    .filter_map(|p| Some((p.register?, p.registrar_slug.clone())))
                    .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(_, s)| s)
            });
            (slug, entry.domain.clone())
        };
        match slug_owned {
            Some(s) => self.open_registrar_url(&s, &domain),
            None => {
                self.notification = Some((" No price data to open a registrar ".to_string(), 12));
            }
        }
    }

    fn open_selected_column(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let col = match self.selected_column {
            Some(c) if c < config::REGISTRARS.len() => c,
            _ => return,
        };
        let slug = config::REGISTRARS[col].slug;
        let domain = self.results[self.selected_row].domain.clone();
        self.open_registrar_url(slug, &domain);
    }

    fn open_registrar_url(&mut self, slug: &str, domain: &str) {
        match config::registrar_search_url(slug, domain) {
            Some(url) => {
                let msg = format!(" Opening {} for {} ", slug, domain);
                self.notification = Some((msg, 12));
                let _ = std::thread::spawn(move || {
                    if let Err(e) = open::that(&url) {
                        eprintln!("warning: failed to open browser: {e}");
                    }
                });
            }
            None => {
                self.notification = Some((format!(" No URL for registrar {} ", slug), 12));
            }
        }
    }

    pub fn tick_notification(&mut self) {
        if let Some((_, ref mut frames)) = self.notification {
            *frames = frames.saturating_sub(1);
            if *frames == 0 {
                self.notification = None;
            }
        }
    }

    fn start_search(&mut self) {
        self.loading = true;
        self.has_searched = true;
        self.error = None;
    }

    pub async fn run_search(&mut self) {
        let query = self.input.trim().to_lowercase();
        if query.is_empty() {
            self.loading = false;
            return;
        }

        let pricing = match self.pricing_db.clone() {
            Some(db) => db,
            None => {
                self.error = Some("No pricing data available".to_string());
                self.loading = false;
                return;
            }
        };

        let tlds = config::TLDS;
        let (results, stats) = search::run_search(&query, tlds, &pricing).await;

        self.results = results;
        self.stats = stats;
        self.selected_row = 0;
        self.selected_column = None;
        self.scroll_offset = 0;
        self.sort_results();
        self.loading = false;
    }

    fn sort_results(&mut self) {
        match self.sort_by {
            SortBy::Domain => {
                self.results.sort_by(|a, b| a.domain.cmp(&b.domain));
            }
            SortBy::Price => {
                self.results.sort_by(|a, b| {
                    let pa = search::best_price(a);
                    let pb = search::best_price(b);
                    pa.partial_cmp(&pb)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.available.cmp(&a.available))
                });
            }
            SortBy::Status => {
                self.results.sort_by(|a, b| {
                    b.available
                        .cmp(&a.available)
                        .then_with(|| a.domain.cmp(&b.domain))
                });
            }
        }
    }

    fn clamp_scroll(&mut self) {
        if self.selected_row < self.scroll_offset {
            self.scroll_offset = self.selected_row;
        }
    }

    fn column_notification(&self) -> String {
        match self.selected_column {
            Some(i) if i < config::REGISTRARS.len() => {
                format!(" Column: {} ", config::REGISTRARS[i].label)
            }
            _ => " Column: none ".to_string(),
        }
    }

    pub fn status_text(&self) -> String {
        if self.loading {
            return "Searching...".to_string();
        }
        if self.results.is_empty() {
            if self.has_searched {
                return "No results".to_string();
            }
            return "Type a domain name and press Enter".to_string();
        }
        let pricing_age = self
            .pricing_db
            .as_ref()
            .map(|db| crate::pricing::age_string(db))
            .unwrap_or_default();
        let col_info = match self.selected_column {
            Some(i) if i < config::REGISTRARS.len() => {
                format!(" · [{}] selected", config::REGISTRARS[i].label)
            }
            _ => "".to_string(),
        };
        format!(
            "{}/{} available · {} taken · {} · prices {}{}",
            self.stats.available,
            self.stats.total,
            self.stats.taken,
            self.sort_by.label(),
            pricing_age,
            col_info,
        )
    }
}
