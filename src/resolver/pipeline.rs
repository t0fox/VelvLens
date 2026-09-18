use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    sync::{atomic::AtomicUsize, Arc},
    time::{Duration, Instant},
};

use reqwest::{redirect::Policy, Client};
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    dedup::{deduplicate, DuplicateReport},
    error::{Result, SubLensError},
    model::ProxyConfig,
    protocols::parse_uri,
};

use super::{
    decode::decode_candidates,
    detect::{detect_content, ContentKind},
    extract::extract_items,
    fetch::{fetch_source, RedirectHop},
    stage::{PipelineStage, StageKind},
};

#[derive(Debug, Clone)]
pub struct ResolverConfig {
    pub max_depth: u8,
    pub max_total_bytes: usize,
    pub max_bytes_per_source: usize,
    pub max_discovered_urls: usize,
    pub max_redirects: usize,
    pub max_configs: usize,
    pub request_timeout: Duration,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            max_depth: 8,
            max_total_bytes: 10 * 1024 * 1024,
            max_bytes_per_source: 2 * 1024 * 1024,
            max_discovered_urls: 128,
            max_redirects: 16,
            max_configs: 2048,
            request_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisReport {
    pub source: String,
    pub stages: Vec<PipelineStage>,
    pub redirect_chain: Vec<RedirectHop>,
    pub configs: Vec<ProxyConfig>,
    pub duplicates: DuplicateReport,
    pub protocol_counts: BTreeMap<String, usize>,
    pub discovered_urls: usize,
    pub downloaded_bytes: usize,
}

pub struct Resolver {
    client: Client,
    pub config: ResolverConfig,
}

impl Resolver {
    pub fn new(_client: Client, config: ResolverConfig) -> Self {
        let client = Client::builder()
            .redirect(Policy::none())
            .user_agent("SubLens/0.1")
            .build()
            .expect("reqwest client configuration is valid");
        Self { client, config }
    }

    pub async fn analyze(&self, source: &str, cancel: CancellationToken) -> Result<AnalysisReport> {
        let start = Url::parse(source)
            .map_err(|error| SubLensError::InvalidInput(format!("invalid source URL: {error}")))?;
        if !matches!(start.scheme(), "http" | "https") {
            return Err(SubLensError::InvalidInput(
                "analysis source must use HTTP or HTTPS".to_owned(),
            ));
        }

        let mut report = AnalysisReport {
            source: source.to_owned(),
            ..AnalysisReport::default()
        };
        let total_bytes = Arc::new(AtomicUsize::new(0));
        let mut queue = VecDeque::from([WorkItem::Remote {
            url: start,
            depth: 0,
        }]);
        let mut visited_urls = HashSet::new();
        let mut fingerprints = HashSet::new();

        while let Some(item) = queue.pop_front() {
            if cancel.is_cancelled() {
                return Err(SubLensError::Cancelled);
            }
            match item {
                WorkItem::Remote { url, depth } => {
                    if depth > self.config.max_depth || !visited_urls.insert(canonical_url(&url)) {
                        continue;
                    }
                    let started = Instant::now();
                    let fetched = fetch_source(
                        &self.client,
                        url.clone(),
                        &self.config,
                        &cancel,
                        &total_bytes,
                    )
                    .await?;
                    report.redirect_chain.extend(fetched.redirects.clone());
                    report.stages.push(PipelineStage::success(
                        StageKind::Http,
                        format!("HTTP {} · {} bytes", 200, fetched.body.len()),
                        fetched.body.len(),
                        started.elapsed(),
                    ));
                    for hop in fetched.redirects {
                        report.stages.push(PipelineStage::success(
                            StageKind::Redirect,
                            format!("HTTP {} → {}", hop.status, hop.to),
                            1,
                            Duration::ZERO,
                        ));
                    }
                    queue.push_back(WorkItem::Inline {
                        text: String::from_utf8_lossy(&fetched.body).into_owned(),
                        content_type: fetched.content_type,
                        source: fetched.final_url,
                        depth,
                    });
                }
                WorkItem::Inline {
                    text,
                    content_type,
                    source,
                    depth,
                } => {
                    if depth > self.config.max_depth
                        || text.len() > self.config.max_bytes_per_source
                    {
                        report.stages.push(PipelineStage::failure(
                            StageKind::Decode,
                            Some(source.as_str()),
                            format!(
                                "payload exceeded {} bytes or recursion depth",
                                self.config.max_bytes_per_source
                            ),
                        ));
                        continue;
                    }
                    let fingerprint = hex_digest(text.as_bytes());
                    if !fingerprints.insert(fingerprint) {
                        continue;
                    }
                    process_inline(
                        &text,
                        content_type.as_deref(),
                        &source,
                        depth,
                        &mut queue,
                        &mut report,
                        &self.config,
                        &cancel,
                    )?;
                }
            }
        }

        report.duplicates = deduplicate(&mut report.configs);
        for config in &report.configs {
            *report
                .protocol_counts
                .entry(config.protocol.as_str().to_owned())
                .or_default() += 1;
        }
        report.discovered_urls = visited_urls.len();
        report.downloaded_bytes = total_bytes.load(std::sync::atomic::Ordering::Relaxed);
        report.stages.push(PipelineStage::success(
            StageKind::Deduplication,
            format!("{} configurations normalized", report.configs.len()),
            report.configs.len(),
            Duration::ZERO,
        ));
        Ok(report)
    }
}

enum WorkItem {
    Remote {
        url: Url,
        depth: u8,
    },
    Inline {
        text: String,
        content_type: Option<String>,
        source: Url,
        depth: u8,
    },
}

#[allow(clippy::too_many_arguments)]
fn process_inline(
    text: &str,
    content_type: Option<&str>,
    source: &Url,
    depth: u8,
    queue: &mut VecDeque<WorkItem>,
    report: &mut AnalysisReport,
    config: &ResolverConfig,
    cancel: &CancellationToken,
) -> Result<()> {
    let kind = detect_content(text.as_bytes(), content_type);
    report.stages.push(PipelineStage::success(
        StageKind::Detection,
        format!("{:?}", kind),
        1,
        Duration::ZERO,
    ));

    let extraction = extract_items(text, kind);
    report.stages.push(PipelineStage::success(
        StageKind::Extract,
        format!(
            "{} proxy URIs · {} nested URLs",
            extraction.proxy_uris.len(),
            extraction.nested_urls.len()
        ),
        extraction.proxy_uris.len() + extraction.nested_urls.len(),
        Duration::ZERO,
    ));
    for uri in extraction.proxy_uris {
        if cancel.is_cancelled() {
            return Err(SubLensError::Cancelled);
        }
        if report.configs.len() >= config.max_configs {
            return Err(SubLensError::LimitExceeded(format!(
                "more than {} configurations",
                config.max_configs
            )));
        }
        match parse_uri(&uri, Some(source), depth) {
            Ok(config) => report.configs.push(config),
            Err(error) => report.stages.push(PipelineStage::failure(
                StageKind::Parse,
                Some(&uri),
                error.to_string(),
            )),
        }
    }
    for nested in extraction.nested_urls {
        if depth >= config.max_depth || queue.len() >= config.max_discovered_urls {
            report.stages.push(PipelineStage::failure(
                StageKind::Extract,
                Some(source.as_str()),
                format!("nested source limit {} reached", config.max_discovered_urls),
            ));
            break;
        }
        if let Ok(url) = Url::parse(&nested) {
            if matches!(url.scheme(), "http" | "https") {
                queue.push_back(WorkItem::Remote {
                    url,
                    depth: depth + 1,
                });
            }
        }
    }

    let mut decode_inputs = extraction.payloads;
    if kind == ContentKind::Base64Candidate || (!text.contains("://") && kind != ContentKind::Html)
    {
        decode_inputs.push(text.to_owned());
    }
    for encoded in decode_inputs {
        for candidate in decode_candidates(&encoded) {
            report.stages.push(PipelineStage::success(
                StageKind::Decode,
                format!(
                    "{:?} · {} lines",
                    candidate.codec,
                    candidate.text.lines().count()
                ),
                candidate.text.lines().count(),
                Duration::ZERO,
            ));
            if depth < config.max_depth {
                queue.push_back(WorkItem::Inline {
                    text: candidate.text,
                    content_type: None,
                    source: source.clone(),
                    depth: depth + 1,
                });
            }
        }
    }
    Ok(())
}

fn canonical_url(url: &Url) -> String {
    let mut normalized = url.clone();
    normalized.set_fragment(None);
    normalized
        .to_string()
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn hex_digest(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    format!("{digest:x}")
}
