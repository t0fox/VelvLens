use std::sync::atomic::{AtomicUsize, Ordering};

use futures_util::StreamExt;
use reqwest::{
    header::{HeaderValue, LOCATION, USER_AGENT},
    Client,
};
use tokio::{select, time::timeout};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    error::{Result, SubLensError},
    security::redact_uri,
};

use super::metadata::SubscriptionMetadata;
use super::pipeline::ResolverConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchProfile {
    Default,
    HappCompatible,
}

impl FetchProfile {
    fn user_agent(self) -> &'static str {
        match self {
            Self::Default => "SubLens/0.1",
            Self::HappCompatible => "Happ/1.0",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectHop {
    pub status: u16,
    pub from: String,
    pub to: String,
}

#[derive(Debug)]
pub struct FetchedSource {
    pub final_url: Url,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
    pub redirects: Vec<RedirectHop>,
    pub metadata: SubscriptionMetadata,
    pub happ_compatibility_hint: bool,
}

pub async fn fetch_source(
    client: &Client,
    start: Url,
    config: &ResolverConfig,
    cancel: &CancellationToken,
    total_bytes: &AtomicUsize,
    profile: FetchProfile,
) -> Result<FetchedSource> {
    let mut current = start;
    let mut redirects = Vec::new();

    for _ in 0..=config.max_redirects {
        if cancel.is_cancelled() {
            return Err(SubLensError::Cancelled);
        }
        let mut request = client.get(current.clone());
        if let Ok(user_agent) = HeaderValue::from_str(profile.user_agent()) {
            request = request.header(USER_AGENT, user_agent);
        }
        let response = timeout(config.request_timeout, request.send())
            .await
            .map_err(|_| SubLensError::Fetch {
                origin: redact_uri(current.as_str()),
                message: format!("HTTP request timed out after {:?}", config.request_timeout),
            })?
            .map_err(|error| SubLensError::Fetch {
                origin: redact_uri(current.as_str()),
                message: error.to_string(),
            })?;

        if response.status().is_redirection() {
            let status = response.status();
            let Some(location) = response
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok())
            else {
                return Err(SubLensError::Fetch {
                    origin: redact_uri(current.as_str()),
                    message: format!("HTTP {} redirect has no Location header", status),
                });
            };
            let next = current
                .join(location)
                .map_err(|error| SubLensError::Fetch {
                    origin: redact_uri(current.as_str()),
                    message: format!("invalid redirect target: {error}"),
                })?;
            redirects.push(RedirectHop {
                status: status.as_u16(),
                from: redact_uri(current.as_str()),
                to: redact_uri(next.as_str()),
            });
            current = next;
            continue;
        }

        if !response.status().is_success() {
            return Err(SubLensError::Fetch {
                origin: redact_uri(current.as_str()),
                message: format!("HTTP status {}", response.status()),
            });
        }

        let happ_compatibility_hint = has_happ_compatibility_headers(response.headers());
        let metadata = SubscriptionMetadata::from_headers(response.headers());
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let response_status = response.status().as_u16();
        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(next) = select! {
            _ = cancel.cancelled() => return Err(SubLensError::Cancelled),
            next = stream.next() => next,
        } {
            let chunk = next.map_err(|error| SubLensError::Fetch {
                origin: redact_uri(current.as_str()),
                message: error.to_string(),
            })?;
            let new_source_bytes = body.len().saturating_add(chunk.len());
            if new_source_bytes > config.max_bytes_per_source {
                return Err(SubLensError::LimitExceeded(format!(
                    "source exceeded {} bytes",
                    config.max_bytes_per_source
                )));
            }
            let previous_total = total_bytes.fetch_add(chunk.len(), Ordering::Relaxed);
            if previous_total.saturating_add(chunk.len()) > config.max_total_bytes {
                return Err(SubLensError::LimitExceeded(format!(
                    "analysis exceeded {} downloaded bytes",
                    config.max_total_bytes
                )));
            }
            body.extend_from_slice(&chunk);
        }
        return Ok(FetchedSource {
            final_url: current,
            status: response_status,
            content_type,
            body,
            redirects,
            metadata,
            happ_compatibility_hint,
        });
    }

    Err(SubLensError::LimitExceeded(format!(
        "redirect chain exceeded {} hops",
        config.max_redirects
    )))
}

fn has_happ_compatibility_headers(headers: &reqwest::header::HeaderMap) -> bool {
    [
        "x-app-key-number",
        "providerid",
        "subscription-always-hwid-enable",
        "per-app-proxy-enable",
    ]
    .iter()
    .any(|name| headers.contains_key(*name))
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderValue};

    use super::has_happ_compatibility_headers;

    #[test]
    fn recognizes_happ_profile_headers_without_retaining_device_values() {
        let mut headers = HeaderMap::new();
        headers.insert("x-app-key-number", HeaderValue::from_static("present"));
        assert!(has_happ_compatibility_headers(&headers));

        let ordinary = HeaderMap::new();
        assert!(!has_happ_compatibility_headers(&ordinary));
    }
}
