use std::{collections::HashMap, sync::Arc};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
};
use tokio_util::sync::CancellationToken;

use sublens::{
    dedup::deduplicate,
    model::Protocol,
    resolver::{Resolver, ResolverConfig},
};

struct TestServer {
    address: String,
    shutdown: Option<oneshot::Sender<()>>,
}

impl TestServer {
    async fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let (shutdown, mut stop) = oneshot::channel();
        let encoded = STANDARD.encode("trojan://synthetic-password@nested.example:443");
        let wrapped = STANDARD.encode(
            "vless://12345678-1234-1234-1234-123456789abc@wrapped.example:443\n\
hysteria2://synthetic-password@hy.example:443?sni=hy.example",
        );
        let wrapped = wrapped
            .as_bytes()
            .chunks(19)
            .map(std::str::from_utf8)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .join("\n");
        let many_body = format!(
            "{{\"items\":[\"http://{address}/a\",\"http://{address}/b\",\"http://{address}/c\"]}}"
        );
        let body = format!(
            "{{\"items\":[\"vless://12345678-1234-1234-1234-123456789abc@direct.example:443\",\"{}\"]}}",
            encoded
        );
        let json_profile = format!(
            "{{\"remarks\":\"structured\",\"outbounds\":[{{\"protocol\":\"vless\",\"settings\":{{\"vnext\":[{{\"address\":\"structured.example\",\"port\":443,\"users\":[{{\"id\":\"00000000-0000-4000-8000-000000000001\"}}]}}]}}}}],\"routing\":{{\"rules\":[{{\"domain\":[\"http://{address}/must-not-fetch\"]}}]}}}}"
        );
        let happ_body = "hysteria2://synthetic-password@happ.example:443?port=1234,5000-6000";
        let responses = Arc::new(HashMap::from([
            (
                "/redirect",
                "HTTP/1.1 302 Found\r\nLocation: /nested\r\nContent-Length: 0\r\n\r\n".to_owned(),
            ),
            (
                "/nested",
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                ),
            ),
            (
                "/json-profile",
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    json_profile.len(),
                    json_profile
                ),
            ),
            (
                "/loop",
                "HTTP/1.1 302 Found\r\nLocation: /loop\r\nContent-Length: 0\r\n\r\n".to_owned(),
            ),
            (
                "/wrapped",
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    wrapped.len(),
                    wrapped
                ),
            ),
            (
                "/many",
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    many_body.len(),
                    many_body
                ),
            ),
            (
                "/a",
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 0\r\n\r\n"
                    .to_owned(),
            ),
            (
                "/partial",
                "HTTP/1.1 206 Partial Content\r\nContent-Type: text/plain\r\nContent-Length: 64\r\n\r\nvless://12345678-1234-1234-1234-123456789abc@partial.example:443"
                    .to_owned(),
            ),
        ]));
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut stop => break,
                    connection = listener.accept() => {
                        let Ok((mut stream, _)) = connection else { break };
                        let responses = Arc::clone(&responses);
                        tokio::spawn(async move {
                            let mut request = [0_u8; 2048];
                            let length = stream.read(&mut request).await.unwrap_or_default();
                            let line = String::from_utf8_lossy(&request[..length]);
                            let path = line.split_whitespace().nth(1).unwrap_or("/");
                            let response = if path == "/happ" {
                                if line.to_ascii_lowercase().contains("user-agent: happ/1.0") {
                                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", happ_body.len(), happ_body)
                                } else {
                                    let landing = "<html>sign in required</html>";
                                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}", landing.len(), landing)
                                }
                            } else {
                                responses.get(path).cloned().unwrap_or_else(|| "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_owned())
                            };
                            let _ = stream.write_all(response.as_bytes()).await;
                        });
                    }
                }
            }
        });
        Self {
            address: format!("http://{address}"),
            shutdown: Some(shutdown),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }
}

#[tokio::test]
async fn retries_suspicious_source_with_happ_user_agent() {
    let server = TestServer::start().await;
    let report = Resolver::new(reqwest::Client::new(), ResolverConfig::default())
        .analyze(&server.url("/happ"), CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(report.configs.len(), 1);
    assert_eq!(report.configs[0].protocol, Protocol::Hysteria2);
    assert!(report
        .stages
        .iter()
        .any(|stage| stage.preview.contains("Happ/1.0")));
}

#[tokio::test]
async fn candidate_limit_stops_expansion_without_infinite_recursion() {
    let server = TestServer::start().await;
    let config = ResolverConfig {
        max_candidates: 1,
        ..ResolverConfig::default()
    };
    let report = Resolver::new(reqwest::Client::new(), config)
        .analyze(&server.url("/nested"), CancellationToken::new())
        .await
        .unwrap();
    assert!(report.candidate_count <= 1);
    assert!(report
        .stages
        .iter()
        .filter_map(|stage| stage.error.as_deref())
        .any(|error| error.contains("candidate limit")));
}

#[tokio::test]
async fn discovered_url_limit_is_global_across_the_source_graph() {
    let server = TestServer::start().await;
    let config = ResolverConfig {
        max_discovered_urls: 2,
        ..ResolverConfig::default()
    };
    let report = Resolver::new(reqwest::Client::new(), config)
        .analyze(&server.url("/many"), CancellationToken::new())
        .await
        .unwrap();

    assert_eq!(report.discovered_urls, 2);
    assert!(report
        .stages
        .iter()
        .filter_map(|stage| stage.error.as_deref())
        .any(|error| error.contains("nested source limit")));
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

#[tokio::test]
async fn follows_redirects_and_nested_payloads_without_looping() {
    let server = TestServer::start().await;
    let report = Resolver::new(reqwest::Client::new(), ResolverConfig::default())
        .analyze(&server.url("/redirect"), CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(report.configs.len(), 2);
    assert!(report.redirect_chain.iter().any(|hop| hop.status == 302));
    assert!(report.protocol_counts.contains_key("VLESS"));
    assert!(report.protocol_counts.contains_key("Trojan"));
    let parsed = report
        .stages
        .iter()
        .filter(|stage| {
            stage.kind == sublens::resolver::stage::StageKind::Parse
                && stage.status == sublens::resolver::stage::StageStatus::Success
        })
        .map(|stage| stage.found)
        .sum::<usize>();
    assert_eq!(parsed, 2);
}

#[tokio::test]
async fn structured_json_does_not_turn_routing_urls_into_sources() {
    let server = TestServer::start().await;
    let report = Resolver::new(reqwest::Client::new(), ResolverConfig::default())
        .analyze(&server.url("/json-profile"), CancellationToken::new())
        .await
        .unwrap();

    assert_eq!(report.configs.len(), 1);
    assert_eq!(report.configs[0].protocol, Protocol::Vless);
    assert_eq!(report.discovered_urls, 1);
    assert!(!report
        .stages
        .iter()
        .any(|stage| stage.preview.contains("must-not-fetch")));
}

#[tokio::test]
async fn redirect_loop_is_bounded_by_redirect_limit() {
    let server = TestServer::start().await;
    let config = ResolverConfig {
        max_redirects: 2,
        ..ResolverConfig::default()
    };
    let result = Resolver::new(reqwest::Client::new(), config)
        .analyze(&server.url("/loop"), CancellationToken::new())
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("redirect"));
}

#[tokio::test]
async fn decodes_line_wrapped_base64_from_http_source() {
    let server = TestServer::start().await;
    let report = Resolver::new(reqwest::Client::new(), ResolverConfig::default())
        .analyze(&server.url("/wrapped"), CancellationToken::new())
        .await
        .unwrap();

    assert_eq!(report.configs.len(), 2);
    assert_eq!(report.configs[0].protocol, Protocol::Vless);
    assert_eq!(report.configs[1].protocol, Protocol::Hysteria2);
}

#[tokio::test]
async fn accepts_successful_non_200_status_and_reports_it() {
    let server = TestServer::start().await;
    let report = Resolver::new(reqwest::Client::new(), ResolverConfig::default())
        .analyze(&server.url("/partial"), CancellationToken::new())
        .await
        .unwrap();

    assert_eq!(report.configs.len(), 1);
    assert!(report.stages.iter().any(|stage| stage.kind
        == sublens::resolver::stage::StageKind::Http
        && stage.preview.starts_with("HTTP 206 ·")));
}

#[test]
fn groups_exact_and_semantic_duplicates_without_dropping_originals() {
    let mut configs = vec![
        sublens::protocols::parse_uri(
            "vless://12345678-1234-1234-1234-123456789abc@example.com:443/a",
            None,
            0,
        )
        .unwrap(),
        sublens::protocols::parse_uri(
            "vless://12345678-1234-1234-1234-123456789abc@example.com:443/a",
            None,
            0,
        )
        .unwrap(),
        sublens::protocols::parse_uri(
            "vless://12345678-1234-1234-1234-123456789abc@example.com:443/b",
            None,
            0,
        )
        .unwrap(),
    ];
    let report = deduplicate(&mut configs);
    assert_eq!(configs.len(), 3);
    assert_eq!(report.exact_groups.len(), 1);
    assert_eq!(report.semantic_groups.len(), 1);
    assert_eq!(configs[0].protocol, Protocol::Vless);
}
