use std::time::{Duration, Instant};

use tokio::{net::lookup_host, select, time::timeout};
use tokio_util::sync::CancellationToken;

use crate::{error::Result, model::ProxyConfig};

use super::{tcp, CheckStatus, DiagnosticResult};

pub async fn check(
    config: &ProxyConfig,
    request_timeout: Duration,
    cancel: &CancellationToken,
) -> Result<DiagnosticResult> {
    let started = Instant::now();
    let lookup = select! {
        _ = cancel.cancelled() => return Err(crate::error::SubLensError::Cancelled),
        result = timeout(request_timeout, lookup_host((config.host.as_str(), config.port))) => result,
    };
    let addresses = match lookup {
        Ok(Ok(addresses)) => addresses.collect::<Vec<_>>(),
        Ok(Err(error)) => {
            return Ok(DiagnosticResult {
                host: config.host.clone(),
                port: config.port,
                dns: CheckStatus::Failed(error.to_string()),
                tcp: CheckStatus::Skipped,
                latency_ms: None,
                connectivity_only: true,
            })
        }
        Err(_) => {
            return Ok(DiagnosticResult {
                host: config.host.clone(),
                port: config.port,
                dns: CheckStatus::Failed(format!(
                    "DNS lookup timed out after {:?}",
                    request_timeout
                )),
                tcp: CheckStatus::Skipped,
                latency_ms: None,
                connectivity_only: true,
            })
        }
    };
    let tcp_result = tcp::check_addresses(&addresses, request_timeout, cancel).await?;
    Ok(DiagnosticResult {
        host: config.host.clone(),
        port: config.port,
        dns: CheckStatus::Passed,
        tcp: tcp_result.status,
        latency_ms: tcp_result
            .latency_ms
            .or_else(|| Some(started.elapsed().as_millis())),
        connectivity_only: true,
    })
}
