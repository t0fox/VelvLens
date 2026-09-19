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
    fetch::{fetch_source, FetchProfile, RedirectHop},
    metadata::{parse_body_directives, SubscriptionMetadata},
    stage::{PipelineStage, StageKind},
};

#[derive(Debug, Clone)]
pub struct ResolverConfig {
    pub max_depth: u8,
    pub max_total_bytes: usize,
    pub max_bytes_per_source: usize,
    pub max_discovered_urls: usize,
    pub max_candidates: usize,
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
            max_candidates: 4096,
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
    pub metadata: SubscriptionMetadata,
    pub source_classification: Option<SourceClassification>,
    pub candidate_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceClassification {
    EncryptedHappSubscription,
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
        self.analyze_with_stage_sink(source, cancel, |_| {}).await
    }

    pub(crate) async fn analyze_with_stage_sink<F>(
        &self,
        source: &str,
        cancel: CancellationToken,
        mut sink: F,
    ) -> Result<AnalysisReport>
    where
        F: FnMut(PipelineStage) + Send,
    {
        if let Some(kind) = encrypted_happ_scheme(source) {
            let mut report = AnalysisReport {
                source: source.to_owned(),
                source_classification: Some(SourceClassification::EncryptedHappSubscription),
                ..AnalysisReport::default()
            };
            record_stage(
                &mut report,
                &mut sink,
                PipelineStage::success(
                    StageKind::Detection,
                    format!("Encrypted HAPP subscription recognized · {kind}"),
                    1,
                    Duration::ZERO,
                ),
            );
            record_stage(
                &mut report,
                &mut sink,
                PipelineStage::failure(
                    StageKind::Decode,
                    None,
                    "HAPP decryption key is embedded in the HAPP app; decryption was not attempted"
                        .to_owned(),
                ),
            );
            return Ok(report);
        }
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
        let start_key = canonical_url(&start);
        let mut discovered_urls = HashSet::from([start_key]);
        let mut queue = VecDeque::from([WorkItem::Remote {
            url: start,
            depth: 0,
            profile: FetchProfile::Default,
        }]);
        let mut visited_urls = HashSet::new();
        let mut fingerprints = HashSet::new();
        let mut happ_retried = HashSet::new();

        while let Some(item) = queue.pop_front() {
            if cancel.is_cancelled() {
                return Err(SubLensError::Cancelled);
            }
            match item {
                WorkItem::Remote {
                    url,
                    depth,
                    profile,
                } => {
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
                        profile,
                    )
                    .await?;
                    let fetched = if depth == 0
                        && profile == FetchProfile::Default
                        && looks_like_happ_retry(&fetched)
                        && happ_retried.insert(canonical_url(&url))
                    {
                        record_stage(
                            &mut report,
                            &mut sink,
                            PipelineStage::success(
                                StageKind::Http,
                                "HTTP retry · User-Agent Happ/1.0".to_owned(),
                                1,
                                Duration::ZERO,
                            ),
                        );
                        fetch_source(
                            &self.client,
                            url.clone(),
                            &self.config,
                            &cancel,
                            &total_bytes,
                            FetchProfile::HappCompatible,
                        )
                        .await?
                    } else {
                        fetched
                    };
                    report.metadata.merge(fetched.metadata.clone());
                    report.redirect_chain.extend(fetched.redirects.clone());
                    record_stage(
                        &mut report,
                        &mut sink,
                        PipelineStage::success(
                            StageKind::Http,
                            format!("HTTP {} · {} bytes", fetched.status, fetched.body.len()),
                            fetched.body.len(),
                            started.elapsed(),
                        ),
                    );
                    for hop in fetched.redirects {
                        record_stage(
                            &mut report,
                            &mut sink,
                            PipelineStage::success(
                                StageKind::Redirect,
                                format!("HTTP {} → {}", hop.status, hop.to),
                                1,
                                Duration::ZERO,
                            ),
                        );
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
                        record_stage(
                            &mut report,
                            &mut sink,
                            PipelineStage::failure(
                                StageKind::Decode,
                                Some(source.as_str()),
                                format!(
                                    "payload exceeded {} bytes or recursion depth",
                                    self.config.max_bytes_per_source
                                ),
                            ),
                        );
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
                        &mut discovered_urls,
                        &mut sink,
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
        report.discovered_urls = discovered_urls.len();
        report.downloaded_bytes = total_bytes.load(std::sync::atomic::Ordering::Relaxed);
        let normalized_count = report.configs.len();
        record_stage(
            &mut report,
            &mut sink,
            PipelineStage::success(
                StageKind::Deduplication,
                format!("{} configurations normalized", normalized_count),
                normalized_count,
                Duration::ZERO,
            ),
        );
        Ok(report)
    }
}

enum WorkItem {
    Remote {
        url: Url,
        depth: u8,
        profile: FetchProfile,
    },
    Inline {
        text: String,
        content_type: Option<String>,
        source: Url,
        depth: u8,
    },
}

fn record_stage(
    report: &mut AnalysisReport,
    sink: &mut dyn FnMut(PipelineStage),
    stage: PipelineStage,
) {
    sink(stage.clone());
    report.stages.push(stage);
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
    discovered_urls: &mut HashSet<String>,
    sink: &mut dyn FnMut(PipelineStage),
) -> Result<()> {
    let kind = detect_content(text.as_bytes(), content_type);
    report.metadata.merge(parse_body_directives(text));
    record_stage(
        report,
        sink,
        PipelineStage::success(
            StageKind::Detection,
            format!("{:?}", kind),
            1,
            Duration::ZERO,
        ),
    );

    let extraction_limit = config.max_candidates.saturating_add(1);
    let extraction = super::extract::extract_items_with_limit(text, kind, extraction_limit);
    let json_configs = if kind == ContentKind::JsonConfiguration {
        super::xray::parse_configurations(text, source, depth)
    } else {
        Vec::new()
    };
    record_stage(
        report,
        sink,
        PipelineStage::success(
            StageKind::Extract,
            format!(
                "{} proxy URIs · {} JSON configurations · {} nested URLs",
                extraction.proxy_uris.len(),
                json_configs.len(),
                extraction.nested_urls.len()
            ),
            extraction.proxy_uris.len() + json_configs.len() + extraction.nested_urls.len(),
            Duration::ZERO,
        ),
    );
    let mut parsed_count = 0;
    for config_value in json_configs {
        if cancel.is_cancelled() {
            return Err(SubLensError::Cancelled);
        }
        if report.candidate_count >= config.max_candidates {
            record_stage(
                report,
                sink,
                PipelineStage::failure(
                    StageKind::Extract,
                    Some(source.as_str()),
                    format!("candidate limit {} reached", config.max_candidates),
                ),
            );
            break;
        }
        report.candidate_count += 1;
        if report.configs.len() >= config.max_configs {
            return Err(SubLensError::LimitExceeded(format!(
                "more than {} configurations",
                config.max_configs
            )));
        }
        report.configs.push(config_value);
        parsed_count += 1;
    }
    for uri in extraction.proxy_uris {
        if cancel.is_cancelled() {
            return Err(SubLensError::Cancelled);
        }
        if report.candidate_count >= config.max_candidates {
            record_stage(
                report,
                sink,
                PipelineStage::failure(
                    StageKind::Extract,
                    Some(source.as_str()),
                    format!("candidate limit {} reached", config.max_candidates),
                ),
            );
            break;
        }
        report.candidate_count += 1;
        if report.configs.len() >= config.max_configs {
            return Err(SubLensError::LimitExceeded(format!(
                "more than {} configurations",
                config.max_configs
            )));
        }
        if let Some(kind) = encrypted_happ_scheme(&uri) {
            report.source_classification = Some(SourceClassification::EncryptedHappSubscription);
            record_stage(
                report,
                sink,
                PipelineStage::success(
                    StageKind::Detection,
                    format!("Encrypted HAPP subscription recognized · {kind}"),
                    1,
                    Duration::ZERO,
                ),
            );
            record_stage(
                report,
                sink,
                PipelineStage::failure(
                    StageKind::Decode,
                    None,
                    "HAPP decryption key is embedded in the HAPP app; decryption was not attempted"
                        .to_owned(),
                ),
            );
            continue;
        }
        match parse_uri(&uri, Some(source), depth) {
            Ok(config) => {
                report.configs.push(config);
                parsed_count += 1;
            }
            Err(error) => record_stage(
                report,
                sink,
                PipelineStage::failure(StageKind::Parse, Some(&uri), error.to_string()),
            ),
        }
    }
    if parsed_count > 0 {
        record_stage(
            report,
            sink,
            PipelineStage::success(
                StageKind::Parse,
                format!("{parsed_count} configurations parsed"),
                parsed_count,
                Duration::ZERO,
            ),
        );
    }
    for nested in extraction.nested_urls {
        if kind == ContentKind::JsonConfiguration {
            continue;
        }
        if report.candidate_count >= config.max_candidates {
            record_stage(
                report,
                sink,
                PipelineStage::failure(
                    StageKind::Extract,
                    Some(source.as_str()),
                    format!("candidate limit {} reached", config.max_candidates),
                ),
            );
            break;
        }
        report.candidate_count += 1;
        if depth >= config.max_depth {
            record_stage(
                report,
                sink,
                PipelineStage::failure(
                    StageKind::Extract,
                    Some(source.as_str()),
                    format!("nested source limit {} reached", config.max_discovered_urls),
                ),
            );
            break;
        }
        if let Ok(url) = Url::parse(&nested) {
            if matches!(url.scheme(), "http" | "https") {
                let canonical = canonical_url(&url);
                if discovered_urls.contains(&canonical)
                    || discovered_urls.len() < config.max_discovered_urls
                {
                    if discovered_urls.insert(canonical) {
                        queue.push_back(WorkItem::Remote {
                            url,
                            depth: depth + 1,
                            profile: FetchProfile::Default,
                        });
                    }
                } else {
                    record_stage(
                        report,
                        sink,
                        PipelineStage::failure(
                            StageKind::Extract,
                            Some(source.as_str()),
                            format!("nested source limit {} reached", config.max_discovered_urls),
                        ),
                    );
                    break;
                }
            }
        }
    }

    let mut decode_inputs = extraction.payloads;
    if kind == ContentKind::Base64Candidate || (!text.contains("://") && kind != ContentKind::Html)
    {
        decode_inputs.push(text.to_owned());
    }
    for encoded in decode_inputs {
        if report.candidate_count >= config.max_candidates {
            record_stage(
                report,
                sink,
                PipelineStage::failure(
                    StageKind::Decode,
                    Some(source.as_str()),
                    format!("candidate limit {} reached", config.max_candidates),
                ),
            );
            break;
        }
        for candidate in decode_candidates(&encoded) {
            if report.candidate_count >= config.max_candidates {
                record_stage(
                    report,
                    sink,
                    PipelineStage::failure(
                        StageKind::Decode,
                        Some(source.as_str()),
                        format!("candidate limit {} reached", config.max_candidates),
                    ),
                );
                break;
            }
            report.candidate_count += 1;
            record_stage(
                report,
                sink,
                PipelineStage::success(
                    StageKind::Decode,
                    format!(
                        "{:?} · {} lines",
                        candidate.codec,
                        candidate.text.lines().count()
                    ),
                    candidate.text.lines().count(),
                    Duration::ZERO,
                ),
            );
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

fn looks_like_happ_retry(fetched: &super::fetch::FetchedSource) -> bool {
    if fetched.happ_compatibility_hint {
        return true;
    }
    if fetched.body.is_empty() {
        return true;
    }
    let text = String::from_utf8_lossy(&fetched.body);
    if matches!(
        detect_content(fetched.body.as_slice(), fetched.content_type.as_deref()),
        ContentKind::Html
    ) {
        let extracted = extract_items(&text, ContentKind::Html);
        return extracted.proxy_uris.is_empty() && extracted.nested_urls.is_empty();
    }
    matches!(
        detect_content(fetched.body.as_slice(), fetched.content_type.as_deref()),
        ContentKind::Unknown
    )
}

fn encrypted_happ_scheme(source: &str) -> Option<&'static str> {
    let lower = source.trim().to_ascii_lowercase();
    if lower.starts_with("happ://crypt4") || lower.starts_with("happ://crypto4") {
        Some("crypt4")
    } else if lower.starts_with("happ://crypt5") || lower.starts_with("happ://crypto5") {
        Some("crypt5")
    } else if lower.starts_with("happ://crypt") || lower.starts_with("happ://crypto") {
        Some("encrypted")
    } else {
        None
    }
}

fn canonical_url(url: &Url) -> String {
    let mut normalized = url.clone();
    normalized.set_fragment(None);
    let mut value = normalized.to_string();
    if normalized.path() == "/" && normalized.query().is_none() {
        value.pop();
    }
    value
}

fn hex_digest(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    format!("{digest:x}")
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::{canonical_url, process_inline};
    use crate::resolver::{stage::StageKind, AnalysisReport, ResolverConfig};
    use tokio_util::sync::CancellationToken;
    use url::Url;

    #[test]
    fn canonical_url_preserves_case_sensitive_paths_and_queries() {
        let upper = Url::parse("https://example.test/Source/ABC?Token=One#fragment").unwrap();
        let lower = Url::parse("https://example.test/source/abc?token=one").unwrap();
        assert_ne!(canonical_url(&upper), canonical_url(&lower));

        let root = Url::parse("https://example.test/").unwrap();
        assert_eq!(canonical_url(&root), "https://example.test");
    }

    #[test]
    fn nested_encrypted_happ_link_is_classified_without_unknown_config() {
        let source = Url::parse("https://example.test/source").unwrap();
        let mut queue = VecDeque::new();
        let mut report = AnalysisReport::default();
        process_inline(
            "happ://crypt5/public-example",
            Some("text/plain"),
            &source,
            0,
            &mut queue,
            &mut report,
            &ResolverConfig::default(),
            &CancellationToken::new(),
            &mut std::collections::HashSet::new(),
            &mut |_| {},
        )
        .unwrap();
        assert!(report.configs.is_empty());
        assert_eq!(
            report.source_classification,
            Some(super::SourceClassification::EncryptedHappSubscription)
        );
        assert!(report.stages.iter().any(|stage| {
            stage.kind == StageKind::Decode
                && stage
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("not attempted"))
        }));
    }

    #[test]
    fn valid_base64_with_happ_profile_headers_still_requests_compatibility_retry() {
        let final_url = Url::parse("https://example.test/profile").unwrap();
        let fetched = super::super::fetch::FetchedSource {
            final_url,
            status: 200,
            content_type: Some("text/plain".to_owned()),
            body: b"dmxlc3M6Ly8=".to_vec(),
            redirects: Vec::new(),
            metadata: Default::default(),
            happ_compatibility_hint: true,
        };
        assert!(super::looks_like_happ_retry(&fetched));
    }
}
