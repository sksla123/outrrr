use anyhow::{bail, Result};
use shoutrrr::transport::ReqwestClient;

pub struct Notifier {
    urls: Vec<String>,
    client: ReqwestClient,
}

impl Notifier {
    pub fn new(urls: &[String]) -> Result<Self> {
        let prepared_urls = urls.iter().map(|url| prepare(url)).collect();

        Ok(Self {
            urls: prepared_urls,
            client: ReqwestClient::new(),
        })
    }

    pub async fn send(&self, message: &str) -> Result<()> {
        let mut failures = Vec::new();
        
        for (index, url) in self.urls.iter().enumerate() {
            if let Err(e) = shoutrrr::send(&self.client, url, message).await {
                failures.push(format!("destination[{}]: {:?}", index, e));
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            bail!("notification destination(s) failed: {}", failures.join("; "));
        }
    }
}

fn prepare(url: &str) -> String {
    if !url.get(..10).is_some_and(|prefix| prefix.eq_ignore_ascii_case("discord://")) {
        return url.to_string();
    }
    if has_query_key(url, "splitlines") {
        return url.to_string();
    }
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{}splitLines=No", separator)
}

fn has_query_key(url: &str, key: &str) -> bool {
    let Some((_head, query)) = url.split_once('?') else {
        return false;
    };
    query.split('&').any(|part| {
        part.split_once('=')
            .map(|(name, _value)| name)
            .unwrap_or(part)
            .eq_ignore_ascii_case(key)
    })
}
