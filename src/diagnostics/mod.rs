pub mod dns;
pub mod tcp;

use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::{error::Result, model::ProxyConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Passed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticResult {
    pub host: String,
    pub port: u16,
    pub dns: CheckStatus,
    pub tcp: CheckStatus,
    pub latency_ms: Option<u128>,
    pub connectivity_only: bool,
}

pub async fn check(
    config: &ProxyConfig,
    timeout: Duration,
    cancel: &CancellationToken,
) -> Result<DiagnosticResult> {
    dns::check(config, timeout, cancel).await
}
