use std::sync::OnceLock;

fn rdap_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .user_agent("tldscan/0.1")
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("failed to create rdap http client")
    })
}

pub async fn check_domain(domain: &str) -> Option<bool> {
    let url = format!("https://rdap.org/domain/{}", domain);

    match rdap_client().get(&url).send().await {
        Ok(resp) => match resp.status().as_u16() {
            404 => Some(true),
            200 => Some(false),
            410 => Some(false),
            _ => None,
        },
        Err(_) => None,
    }
}
