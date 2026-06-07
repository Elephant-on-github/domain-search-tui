pub const TLDS: &[&str] = &[
    "com", "io", "dev", "app", "net", "org", "co", "me", "ai", "xyz",
    "info", "shop", "store", "cloud", "tech", "online", "site", "pro", "gg", "to",
];

pub const REGISTRARS: &[RegistrarInfo] = &[
    RegistrarInfo { slug: "cloudflare", label: "Cloudflare" },
    RegistrarInfo { slug: "porkbun", label: "Porkbun" },
    RegistrarInfo { slug: "namecheap", label: "Namecheap" },
    RegistrarInfo { slug: "godaddy", label: "GoDaddy" },
    RegistrarInfo { slug: "dynadot", label: "Dynadot" },
    RegistrarInfo { slug: "spaceship", label: "Spaceship" },
];

pub struct RegistrarInfo {
    pub slug: &'static str,
    pub label: &'static str,
}

/// Search URL templates per registrar.  `{domain}` is replaced with the full domain.
/// Registrars using JS-based SPAs link to their search page without `{domain}`.
/// If no template exists for a registrar, the user sees a notification.
pub const REGISTRAR_URLS: &[(&str, &str)] = &[
    // ── Tracked registrars ──
    ("cloudflare", "https://domains.cloudflare.com/?domain={domain}"),
    ("porkbun", "https://porkbun.com/checkout/search?q={domain}"),
    ("namecheap", "https://www.namecheap.com/domains/registration/results/?domain={domain}"),
    ("godaddy", "https://www.godaddy.com/domainsearch/find?domainToCheck={domain}"),
    ("dynadot", "https://www.dynadot.com/domain/search.html?domain={domain}"),
    ("spaceship", "https://www.spaceship.com/domain-search/"),
    // ── Frequent index-cheapest registrars ──
    ("regery", "https://regery.com/en/domains/search?q={domain}"),
    ("mchost", "https://mchost.com/services/domains/check/?domain={domain}"),
    ("sav", "https://www.sav.com/?domain={domain}"),
    ("ovhcloud", "https://www.ovhcloud.com/en/domains/domain-name-checker/"),
    ("ionos", "https://www.ionos.com/domains/domain-check?domainName={domain}"),
    ("one", "https://www.one.com/en/domain/?q={domain}"),
    ("infomaniak", "https://www.infomaniak.com/en/domains/find?q={domain}"),
    ("gname", "https://www.gname.com/domain_check?domain={domain}"),
    ("domgate", "https://www.domgate.com/domaincheck?domain={domain}"),
    ("101domain", "https://www.101domain.com/domain_search.htm?domain={domain}"),
    ("inwx", "https://www.inwx.com/en/domain/search"),
    ("ovh", "https://www.ovh.com/us/domains/domain_search/?search={domain}"),
    ("atakdomain", "https://www.atakdomain.com/en/domain/search?q={domain}"),
    ("onlydomains", "https://www.onlydomains.com/domain-search?q={domain}"),
    ("gandi", "https://www.gandi.net/en/domain/search?q={domain}"),
    ("namesilo", "https://www.namesilo.com/domain/search-domains?q={domain}"),
    ("openprovider", "https://www.openprovider.com/domain-check/?domain={domain}"),
    ("loopia", "https://www.loopia.com/domain/?q={domain}"),
    ("hexonet", "https://www.hexonet.net/domain-check?domain={domain}"),
    ("planethoster", "https://www.planethoster.com/en/Domain-Search?q={domain}"),
    ("netim", "https://www.netim.com/en/domain-name-search?q={domain}"),
    ("above", "https://www.above.com/domain-search/?q={domain}"),
    ("squarespace", "https://www.squarespace.com/domain-search?q={domain}"),
    ("namecom", "https://www.name.com/domain/search/{domain}"),
    ("bigrock", "https://www.bigrock.in/domain-search?q={domain}"),
    ("blacknight", "https://www.blacknight.com/domain-names/?q={domain}"),
    ("dreamhost", "https://www.dreamhost.com/domains/?q={domain}"),
    ("hostinger", "https://www.hostinger.com/domain-name-search?q={domain}"),
    ("wix", "https://www.wix.com/domain/names?q={domain}"),
    ("wordpress", "https://wordpress.com/domains/?q={domain}"),
    ("dinahosting", "https://www.dinahosting.com/en/domains/search?q={domain}"),
    ("network-solutions", "https://www.networksolutions.com/domain-name-search?q={domain}"),
    ("internetbs", "https://internetbs.net/en/domain-search?q={domain}"),
    ("uk2", "https://www.uk2.net/domain-check/?q={domain}"),
    ("123-reg", "https://www.123-reg.co.uk/domain-search?q={domain}"),
    ("encirca", "https://www.encirca.com/domain-search/?q={domain}"),
    ("instra", "https://www.instra.com/en/domain-names/domain-name-search?domain={domain}"),
    ("nicenic", "https://nic.names/domain/{domain}"),
    ("web.z", "https://www.z.com/jp/domain/search/?q={domain}"),
];

pub fn registrar_search_url(slug: &str, domain: &str) -> Option<String> {
    let template = REGISTRAR_URLS.iter().find(|(s, _)| *s == slug)?.1;
    Some(template.replace("{domain}", domain))
}

/// Preferred popular registrars (well-known US/EU) — their prices get a ~5% bonus
/// when sorting and when choosing the "Best" entry.
pub const PREFERRED_REGISTRARS: &[&str] = &[
    "cloudflare",  "porkbun",   "namecheap", "godaddy",
    "dynadot",     "spaceship", "sav",       "namecom",
    "squarespace", "ionos",     "ovhcloud",  "ovh",
    "gandi",       "infomaniak","namesilo",  "hostinger",
    "one",         "loopia",    "netim",     "dinahosting",
    "dreamhost",   "wordpress", "wix",       "blacknight",
    "gname",
];

/// Price adjustment multiplier for preferred registrars (0.95 = 5% advantage).
pub const PREFERENCE_MULTIPLIER: f64 = 0.95;

pub fn is_preferred(slug: &str) -> bool {
    PREFERRED_REGISTRARS.contains(&slug)
}

pub fn adjusted_price(price: f64, slug: &str) -> f64 {
    if is_preferred(slug) {
        price * PREFERENCE_MULTIPLIER
    } else {
        price
    }
}

pub const CACHE_TTL_HOURS: u64 = 24;
pub const PRICING_URL: &str = "https://tldwise.com/data/registrar-price-matrix.json";
pub const PRICE_INDEX_URL: &str = "https://tldwise.com/data/home-price-index.json";

pub const APP_NAME: &str = "domain-search";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registrar_search_url_prefills_domain() {
        let url = registrar_search_url("porkbun", "example.com")
            .expect("porkbun should have a URL template");
        assert_eq!(url, "https://porkbun.com/checkout/search?q=example.com",
            "porkbun URL should contain the domain");

        let url = registrar_search_url("cloudflare", "example.com")
            .expect("cloudflare should have a URL template");
        assert_eq!(url, "https://domains.cloudflare.com/?domain=example.com",
            "cloudflare URL should contain the domain");

        let url = registrar_search_url("namecom", "example.com")
            .expect("namecom should have a URL template");
        assert_eq!(url, "https://www.name.com/domain/search/example.com",
            "name.com URL should contain the domain in the path");

        let url = registrar_search_url("godaddy", "example.com")
            .expect("godaddy should have a URL template");
        assert_eq!(url, "https://www.godaddy.com/domainsearch/find?domainToCheck=example.com",
            "godaddy URL should contain the domain");
    }

    #[test]
    fn test_registrar_search_url_missing_slug() {
        assert!(registrar_search_url("nonexistent-slug", "x.com").is_none(),
            "unknown slug should return None");
    }

    #[test]
    fn test_all_tracked_registrars_have_urls() {
        for reg in REGISTRARS {
            let url = registrar_search_url(reg.slug, "test.com");
            assert!(url.is_some(),
                "tracked registrar {} is missing a URL template", reg.slug);
        }
    }

    #[test]
    fn test_preferred_slugs_table() {
        // Every preferred slug should also have a URL template
        for slug in PREFERRED_REGISTRARS {
            if registrar_search_url(slug, "x.com").is_none() {
                eprintln!("WARN: preferred registrar '{}' has no URL template", slug);
            }
        }
    }

    #[test]
    fn test_adjusted_price() {
        let raw = 10.00;
        let preferred = adjusted_price(raw, "porkbun");
        assert!((preferred - 9.50).abs() < 0.001, "preferred 10.00 should become 9.50");

        let unpreferred = adjusted_price(raw, "regery");
        assert!((unpreferred - 10.00).abs() < 0.001, "non-preferred 10.00 should stay 10.00");
    }

    #[test]
    fn test_is_preferred() {
        assert!(is_preferred("porkbun"), "porkbun should be preferred");
        assert!(is_preferred("cloudflare"), "cloudflare should be preferred");
        assert!(!is_preferred("regery"), "regery should not be preferred");
        assert!(!is_preferred("mchost"), "mchost should not be preferred");
    }
}
