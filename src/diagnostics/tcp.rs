use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use tokio::{net::TcpStream, select, time::timeout};
use tokio_util::sync::CancellationToken;

use crate::error::Result;

use super::CheckStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpResult {
    pub status: CheckStatus,
    pub latency_ms: Option<u128>,
}

pub async fn check_addresses(
    addresses: &[SocketAddr],
    request_timeout: Duration,
    cancel: &CancellationToken,
) -> Result<TcpResult> {
    let mut last_error = None;
    for address in addresses {
        if cancel.is_cancelled() {
            return Err(crate::error::SubLensError::Cancelled);
        }
        let started = Instant::now();
        let result = select! {
            _ = cancel.cancelled() => return Err(crate::error::SubLensError::Cancelled),
            result = timeout(request_timeout, TcpStream::connect(address)) => result,
        };
        match result {
            Ok(Ok(_stream)) => {
                return Ok(TcpResult {
                    status: CheckStatus::Passed,
                    latency_ms: Some(started.elapsed().as_millis()),
                })
            }
            Ok(Err(error)) => last_error = Some(error.to_string()),
            Err(_) => {
                last_error = Some(format!(
                    "TCP connection timed out after {:?}",
                    request_timeout
                ))
            }
        }
    }
    Ok(TcpResult {
        status: CheckStatus::Failed(
            last_error.unwrap_or_else(|| "no DNS address returned".to_owned()),
        ),
        latency_ms: None,
    })
}
