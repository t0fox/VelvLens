use std::sync::atomic::{AtomicUsize, Ordering};

use futures_util::StreamExt;
use reqwest::{header::LOCATION, Client, StatusCode};
use tokio::{select, time::timeout};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    error::{Result, SubLensError},
    security::redact_uri,
};

use super::pipeline::ResolverConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectHop {
    pub status: u16,
    pub from: String,
    pub to: String,
}

#[derive(Debug)]
pub struct FetchedSource {
    pub final_url: Url,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
    pub redirects: Vec<RedirectHop>,
}

pub async fn fetch_source(
    client: &Client,
    start: Url,
    config: &ResolverConfig,
    cancel: &CancellationToken,
    total_bytes: &AtomicUsize,
) -> Result<FetchedSource> {
    let mut current = start;
    let mut redirects = Vec::new();

    for _ in 0..=config.max_redirects {
        if cancel.is_cancelled() {
            return Err(SubLensError::Cancelled);
        }
        let response = timeout(config.request_timeout, client.get(current.clone()).send())
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

        if response.status() != StatusCode::OK {
            return Err(SubLensError::Fetch {
                origin: redact_uri(current.as_str()),
                message: format!("HTTP status {}", response.status()),
            });
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
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
            content_type,
            body,
            redirects,
        });
    }

    Err(SubLensError::LimitExceeded(format!(
        "redirect chain exceeded {} hops",
        config.max_redirects
    )))
}
