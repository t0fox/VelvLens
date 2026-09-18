# SubLens First Working Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and verify a Windows-first Rust/egui SubLens prototype that resolves subscription content recursively, parses the required proxy URI families into a normalized model, and lets a user inspect, filter, copy, export, and QR-display configurations without blocking the UI.

**Architecture:** A platform-independent core owns models, redaction, bounded recursive resolution, protocol parsers, deduplication, diagnostics, and exports. An eframe/egui shell owns presentation only; a Tokio background job manager sends progress and completion events over channels and supports cancellation. The first prototype prioritizes a complete vertical path from URL to visible cards while preserving the final architecture's security and extension points.

**Tech Stack:** Rust 2021, `eframe`, `egui`, `tokio`, `reqwest` with rustls, `serde`, `serde_json`, `base64`, `url`, `regex`, `scraper`, `arboard`, `qrcode`, `tracing`, `tracing-subscriber`, `thiserror`, `directories`, `tokio-util`, and `sha2`.

**Spec:** `docs/superpowers/specs/2026-09-18-sublens-design.md`

## Global Constraints

- No Electron, external backend, analytics, telemetry, or external QR service.
- The egui update thread must never perform HTTP, DNS, TCP, file, or expensive decode work.
- Resolver defaults are bounded: depth 8, 10 MiB total bytes, 2 MiB per response, 128 discovered URLs, 16 redirects per source, and a 10-second request timeout.
- Raw subscription bodies and raw credentials are memory-only unless the user explicitly exports/copies them.
- Logs, previews, errors, redirect displays, and live-test output must use centralized redaction.
- Unknown query parameters are preserved for export; unknown content is reported instead of crashing the analysis.
- Tests use synthetic fixtures and deterministic local servers; no test depends on `vlv.one`.
- Every task runs its focused test plus `cargo fmt --check`; tasks that touch multiple crates also run clippy and the full test suite before commit.

---

### Task 1: Scaffold the crate and security-safe normalized model

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/error.rs`
- Create: `src/model/mod.rs`
- Create: `src/model/proxy_config.rs`
- Create: `src/security/mod.rs`
- Create: `src/security/redact.rs`
- Create: `tests/model_and_redaction.rs`

**Interfaces:**
- `model::ProxyConfig`, `model::Protocol`, `model::Security`, `model::Transport`, `model::ConfigMetadata`.
- `security::redact_secret(value: &str) -> String` and `security::redact_uri(uri: &str) -> String`.
- `error::SubLensError` with `InvalidInput`, `Fetch`, `Decode`, `Extract`, `Parse`, `LimitExceeded`, `Cancelled`, `Diagnostics`, and `Export` variants.

- [ ] **Step 1: Write failing model and redaction tests**

```rust
#[test]
fn redaction_hides_credentials_and_keeps_shape() {
    assert_eq!(redact_secret("1234567890abcdef"), "12••••••ef");
    assert_eq!(
        redact_uri("vless://12345678-1234-1234-1234-123456789abc@example.com:443?pbk=private-key"),
        "vless://••••••••@example.com:443?pbk=••••••••"
    );
}

#[test]
fn semantic_key_excludes_display_name_but_tracks_endpoint() {
    let left = fixture_config("name-a", "example.com", 443);
    let right = fixture_config("name-b", "example.com", 443);
    assert_eq!(left.semantic_key(), right.semantic_key());
}
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run: `cargo test --test model_and_redaction`

Expected: compilation failure because the crate, model types, and redaction functions do not exist.

- [ ] **Step 3: Add the package manifest and module skeleton**

Use a single library target plus a binary target. `reqwest` must use `rustls-tls`, `tokio` must include `rt-multi-thread`, `macros`, `net`, `time`, and `sync`, and `eframe` must use its default native features. Define a `test-fixtures` feature only if it is needed by later integration helpers; do not add a second application framework.

- [ ] **Step 4: Implement the model and redaction functions**

`ProxyConfig` stores optional sensitive values in memory, `unknown_params` as `BTreeMap<String, Vec<String>>`, `raw_uri` as the original string, and `metadata` containing source URL, depth, exact duplicate count, and semantic key. Implement `semantic_key()` from protocol, host, port, security, transport, SNI, fingerprint, Reality identifiers, and protocol-relevant credentials, excluding `name` and `raw_uri`.

`redact_secret` returns the first two and last two Unicode-safe characters with `••••••` in between for values longer than eight characters, and `••••••` for shorter values. `redact_uri` parses URI userinfo and sensitive query keys (`uuid`, `password`, `pass`, `token`, `pbk`, `sid`, `private-key`, `secret`, `auth`) and replaces their values without logging the original.

- [ ] **Step 5: Run focused tests and formatting**

Run: `cargo fmt --check; cargo test --test model_and_redaction`

Expected: formatting passes and all model/redaction tests pass.

- [ ] **Step 6: Commit**

```text
git add Cargo.toml .gitignore src tests/model_and_redaction.rs
git commit -m "feat: add SubLens core model and redaction"
```

### Task 2: Implement content detection, Base64 decoding, and extraction

**Files:**
- Create: `src/resolver/mod.rs`
- Create: `src/resolver/detect.rs`
- Create: `src/resolver/decode.rs`
- Create: `src/resolver/extract.rs`
- Create: `src/resolver/stage.rs`
- Create: `tests/resolver_content.rs`
- Create: `tests/fixtures/raw_mixed.txt`
- Create: `tests/fixtures/nested.json`
- Create: `tests/fixtures/embedded.html`

**Interfaces:**
- `resolver::detect::detect_content(bytes: &[u8], content_type: Option<&str>) -> ContentKind`.
- `resolver::decode::decode_candidates(text: &str) -> Vec<DecodedCandidate>`.
- `resolver::extract::extract_items(text: &str, kind: ContentKind) -> ExtractionResult`.
- `resolver::stage::{PipelineStage, StageKind, StageStatus}`.

- [ ] **Step 1: Write failing decoder/detector/extractor tests**

```rust
#[test]
fn decodes_standard_and_unpadded_url_safe_base64() {
    let standard = encode_standard("vless://uuid@example.com:443");
    let url_safe = encode_url_safe_without_padding("vless://uuid@example.com:443");
    assert!(decode_candidates(&standard).iter().any(|c| c.text.contains("vless://")));
    assert!(decode_candidates(&url_safe).iter().any(|c| c.text.contains("vless://")));
}

#[test]
fn walks_nested_json_and_html_for_proxy_and_subscription_urls() {
    let json = include_str!("fixtures/nested.json");
    let result = extract_items(json, ContentKind::Json);
    assert!(result.proxy_uris.iter().any(|u| u.starts_with("trojan://")));
    assert!(result.nested_urls.iter().any(|u| u.starts_with("https://")));

    let html = include_str!("fixtures/embedded.html");
    let result = extract_items(html, ContentKind::Html);
    assert!(result.proxy_uris.iter().any(|u| u.starts_with("vless://")));
}
```

- [ ] **Step 2: Run focused tests and verify failure**

Run: `cargo test --test resolver_content`

Expected: compilation failure because detector, decoder, extractor, and fixtures are not implemented.

- [ ] **Step 3: Implement detection and bounded candidate decoding**

Recognize protocol prefixes, JSON by trimmed first byte and parseability, HTML by content type or DOM markers, line-oriented text by supported schemes, and Base64 candidates by alphabet/padding plus decoded usefulness. Normalize URL-safe `-/_`, add missing padding, decode standard and URL-safe variants, and discard decoded text that contains no supported URI, HTTP(S) source, JSON, or HTML marker. Return the decoder used and decoded line count for Inspector previews.

- [ ] **Step 4: Implement recursive JSON and HTML extraction helpers**

Walk `serde_json::Value` recursively and inspect every string. Use `scraper::Html` to inspect text and all attribute values, then inspect script contents and raw HTML for JSON/base64 candidates. Use a compiled regex to locate supported proxy schemes and HTTP(S) URLs, trim surrounding punctuation, and deduplicate candidates while retaining discovery order.

- [ ] **Step 5: Implement pipeline stage types and sanitized previews**

`PipelineStage::success(kind, preview, found)` and `PipelineStage::failure(kind, source, error)` must redact URI-like values before storing previews or errors. Include duration and counts but never raw body text.

- [ ] **Step 6: Run tests and formatting**

Run: `cargo fmt --check; cargo test --test resolver_content`

Expected: all decoder, detector, JSON, HTML, and stage-redaction tests pass.

- [ ] **Step 7: Commit**

```text
git add src/resolver tests/resolver_content.rs tests/fixtures
git commit -m "feat: detect decode and extract subscription content"
```

### Task 3: Add independent protocol parsers and parser tests

**Files:**
- Create: `src/protocols/mod.rs`
- Create: `src/protocols/common.rs`
- Create: `src/protocols/vless.rs`
- Create: `src/protocols/vmess.rs`
- Create: `src/protocols/trojan.rs`
- Create: `src/protocols/shadowsocks.rs`
- Create: `src/protocols/hysteria2.rs`
- Create: `src/protocols/tuic.rs`
- Create: `tests/protocol_parsers.rs`

**Interfaces:**
- `protocols::parse_uri(uri: &str, source: Option<&Url>, depth: u8) -> Result<ProxyConfig, SubLensError>`.
- `protocols::is_supported_scheme(uri: &str) -> bool`.
- One parser module per scheme, all preserving unknown query keys.

- [ ] **Step 1: Write failing parser tests**

```rust
#[test]
fn parses_vless_reality_xhttp_and_preserves_unknown_query() {
    let config = parse_uri(
        "vless://12345678-1234-1234-1234-123456789abc@example.com:443/name?security=reality&type=xhttp&sni=example.org&fp=chrome&pbk=public&sid=abcd&x-custom=value",
        None,
        0,
    ).unwrap();
    assert_eq!(config.protocol, Protocol::Vless);
    assert_eq!(config.security, Security::Reality);
    assert_eq!(config.transport, Transport::XHttp);
    assert_eq!(config.unknown_params["x-custom"], vec!["value"]);
}

#[test]
fn parses_all_required_schemes_without_crashing_on_extra_parameters() {
    for uri in fixture_protocol_uris() {
        let config = parse_uri(&uri, None, 0).unwrap();
        assert_ne!(config.protocol, Protocol::Unknown);
        assert!(!config.host.is_empty());
    }
}

#[test]
fn decodes_vmess_json_payload() {
    let config = parse_uri(&vmess_fixture_uri(), None, 0).unwrap();
    assert_eq!(config.protocol, Protocol::Vmess);
    assert_eq!(config.port, 443);
}
```

- [ ] **Step 2: Run focused parser tests and verify failure**

Run: `cargo test --test protocol_parsers`

Expected: compilation failure because parser modules and scheme dispatch do not exist.

- [ ] **Step 3: Implement common URL/query normalization**

Parse with `url::Url`, decode percent-encoded userinfo and query values, parse ports with a clear `SubLensError::Parse` message, normalize security and transport aliases, and collect every unrecognized query key/value in `unknown_params`.

- [ ] **Step 4: Implement VLESS, Trojan, and Shadowsocks parsers**

VLESS must map UUID, host, port, name, security, flow, type, SNI, fp, pbk, sid, path, host header, serviceName, and mode. Trojan must map password and TLS/SNI/transport. Shadowsocks must support `method:password@host:port`, percent-encoded credentials, and base64 userinfo forms.

- [ ] **Step 5: Implement VMess, Hysteria2, and TUIC parsers**

VMess must decode standard or URL-safe Base64 JSON and map `ps`, `add`, `port`, `id`, `scy`, `tls`, `sni`, `net`, `path`, `host`, and `serviceName`. Hysteria2/Hy2 and TUIC must map endpoint, auth credentials, port, TLS/SNI, and common transport fields while preserving unknown keys.

- [ ] **Step 6: Run parser tests and clippy**

Run: `cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --test protocol_parsers`

Expected: all parser tests pass with no warnings.

- [ ] **Step 7: Commit**

```text
git add src/protocols tests/protocol_parsers.rs
git commit -m "feat: parse supported proxy URI protocols"
```

### Task 4: Build the bounded recursive resolver and deduplication report

**Files:**
- Create: `src/resolver/fetch.rs`
- Create: `src/resolver/pipeline.rs`
- Modify: `src/resolver/mod.rs`
- Create: `src/dedup.rs`
- Create: `tests/resolver_pipeline.rs`

**Interfaces:**
- `resolver::ResolverConfig` with exact bounded defaults from Global Constraints.
- `resolver::Resolver::new(reqwest::Client, ResolverConfig) -> Self`.
- `Resolver::analyze(&self, source: &str, cancel: CancellationToken) -> Result<AnalysisReport, SubLensError>`.
- `dedup::deduplicate(configs: Vec<ProxyConfig>) -> DuplicateReport`.

- [ ] **Step 1: Write failing recursive integration tests**

Create a Tokio test server using `tokio::net::TcpListener` that serves `/redirect` with `302 Location: /nested`, `/nested` with JSON containing a Base64URL payload, and `/loop` redirecting to itself. Assert that `analyze()` records the redirect chain, returns normalized configurations, stops the loop, and reports a limit stage rather than hanging.

```rust
#[tokio::test]
async fn follows_redirects_and_nested_payloads_without_looping() {
    let server = TestServer::start().await;
    let report = Resolver::new(reqwest::Client::new(), ResolverConfig::default())
        .analyze(&server.url("/redirect"), CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(report.configs.len(), 2);
    assert!(report.redirect_chain.iter().any(|hop| hop.status == 302));
    assert!(report.stages.iter().any(|stage| stage.kind == StageKind::Decode));
}
```

- [ ] **Step 2: Run the focused integration test and verify failure**

Run: `cargo test --test resolver_pipeline -- --nocapture`

Expected: compilation failure because fetch, traversal, report, and deduplication are not implemented.

- [ ] **Step 3: Implement manual bounded HTTP fetching**

Use `reqwest::redirect::Policy::none()`. Read response streams in chunks, reject a body once per-source or total byte limits are exceeded, resolve relative redirects, record sanitized hops, and return content type plus final URL. Apply the configured timeout to each request and check cancellation before request, during stream read, and before following a redirect.

- [ ] **Step 4: Implement traversal and report assembly**

Use a queue of `(source_url, depth, origin)` nodes. For every body, detect content, record stage, extract direct URI/nested URL candidates, decode useful candidates, enqueue nested sources, parse proxy URIs, and continue until the queue is empty or a limit is reached. Track visited canonical URLs, body SHA-256 fingerprints, discovered URL count, and redirect count. Parse failures become per-item parse stages and do not discard valid siblings.

- [ ] **Step 5: Implement exact and semantic duplicate grouping**

Group by raw URI for exact duplicates and by `ProxyConfig::semantic_key()` for semantic duplicates. Preserve every config with group ids and counts. Return protocol totals for the UI and live report.

- [ ] **Step 6: Run full core verification**

Run: `cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test`

Expected: resolver fixtures, parser tests, redaction tests, redirect handling, loop protection, and duplicate grouping pass.

- [ ] **Step 7: Commit**

```text
git add src/resolver src/dedup.rs tests/resolver_pipeline.rs
git commit -m "feat: add bounded recursive subscription resolver"
```

### Task 5: Add clipboard-safe exports, QR generation, and connectivity diagnostics

**Files:**
- Create: `src/export/mod.rs`
- Create: `src/export/raw.rs`
- Create: `src/export/base64.rs`
- Create: `src/export/json.rs`
- Create: `src/export/v2rayn.rs`
- Create: `src/diagnostics/mod.rs`
- Create: `src/diagnostics/dns.rs`
- Create: `src/diagnostics/tcp.rs`
- Create: `src/qr.rs`
- Create: `tests/export_and_diagnostics.rs`

**Interfaces:**
- `export::raw_lines(configs: &[ProxyConfig]) -> String`.
- `export::v2rayn_bulk(configs: &[ProxyConfig]) -> String`.
- `export::base64_subscription(configs: &[ProxyConfig]) -> String`.
- `export::json_dump(configs: &[ProxyConfig], include_sensitive: bool) -> Result<String, SubLensError>`.
- `diagnostics::check(config: &ProxyConfig, timeout: Duration, cancel: CancellationToken) -> DiagnosticResult`.
- `qr::encode(uri: &str) -> QrMatrix` with no network dependency.

- [ ] **Step 1: Write failing export, QR, and diagnostic tests**

```rust
#[test]
fn v2rayn_export_is_line_oriented_and_preserves_unknown_params() {
    let text = v2rayn_bulk(&[fixture_config_with_unknown_param()]);
    assert!(text.ends_with('\n'));
    assert!(text.contains("x-custom=value"));
}

#[test]
fn sanitized_json_hides_credentials() {
    let json = json_dump(&[fixture_sensitive_config()], false).unwrap();
    assert!(json.contains("••••••"));
    assert!(!json.contains("real-password"));
}

#[test]
fn qr_matrix_is_local_and_nonempty() {
    assert!(!encode("vless://example").modules().is_empty());
}
```

- [ ] **Step 2: Run focused tests and verify failure**

Run: `cargo test --test export_and_diagnostics`

Expected: compilation failure because export, QR, and diagnostic interfaces do not exist.

- [ ] **Step 3: Implement raw, v2rayN, Base64, and JSON exports**

Raw and v2rayN output is one original URI per line with a trailing newline. Base64 export encodes that same line-oriented content with standard Base64. JSON export serializes normalized fields, masks secrets by default, and includes `unknown_params`, metadata, and protocol names.

- [ ] **Step 4: Implement local QR matrix generation**

Wrap `qrcode::QrCode` and expose module dimensions and cell lookup. The UI will convert the matrix to an egui `ColorImage`; no external service or network call is allowed.

- [ ] **Step 5: Implement DNS and TCP checks**

Use `tokio::net::lookup_host` for DNS and `tokio::time::timeout` around `TcpStream::connect` for TCP. Return elapsed milliseconds, explicit status enums, and redacted errors. Mark results `ConnectivityOnly` in the model.

- [ ] **Step 6: Run tests and clippy**

Run: `cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test`

Expected: all core tests pass with no warnings.

- [ ] **Step 7: Commit**

```text
git add src/export src/diagnostics src/qr.rs tests/export_and_diagnostics.rs
git commit -m "feat: add exports QR and connectivity checks"
```

### Task 6: Add the responsive egui shell and cancellable background jobs

**Files:**
- Create: `src/app.rs`
- Create: `src/ui/mod.rs`
- Create: `src/ui/main_view.rs`
- Create: `src/ui/config_card.rs`
- Create: `src/ui/details.rs`
- Create: `src/ui/inspector.rs`
- Create: `src/ui/settings.rs`
- Create: `src/jobs.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`
- Create: `tests/ui_smoke.rs`

**Interfaces:**
- `jobs::JobManager::start_analysis(url: String) -> JobHandle`.
- `jobs::JobManager::cancel(handle: &JobHandle)`.
- `jobs::JobEvent::{Started, Stage, Completed, Failed, Cancelled}`.
- `app::SubLensApp` owns input, filters, selection, report, job state, settings, and egui textures.

- [ ] **Step 1: Write a job-manager smoke test**

Use a local deterministic server and assert that starting an analysis emits `Started`, at least one `Stage`, and `Completed`; canceling a long-running test emits `Cancelled` and leaves the UI state usable.

- [ ] **Step 2: Run the smoke test and verify failure**

Run: `cargo test --test ui_smoke`

Expected: compilation failure because job manager and application state do not exist.

- [ ] **Step 3: Implement `JobManager` on a dedicated Tokio runtime**

Create one runtime owned by the manager. `start_analysis` returns a cancellation token and receiver. Spawn resolver work, forward stage updates, and map errors to redacted user-facing messages. Never send raw bodies or credentials through job events.

- [ ] **Step 4: Implement the dark egui shell**

Configure a dark `Visuals` theme, 8/12/16 px spacing, rounded cards, compact status badges, clear typography hierarchy, hover states, and separators. Build the URL bar, Paste/Analyze/Cancel controls, pipeline strip, protocol/security/transport/search/status filters, and card list.

- [ ] **Step 5: Implement cards, Inspector, filters, and copy actions**

Cards read only `ProxyConfig` fields, mask credentials, show protocol/endpoint/security/transport/SNI/fingerprint/status/latency, and expose Copy, Details, QR, and Test buttons. Inspector shows the structured fields, redirect chain and stage list, masked raw URI, sanitized copy, and local QR. Use `arboard` only inside explicit button handlers.

- [ ] **Step 6: Wire the native entry point and window behavior**

`main.rs` initializes tracing with a redacting layer, creates `NativeOptions` with a 1440×900 default viewport and minimum 1100×700 size, and launches `SubLensApp`. No synchronous network or filesystem call occurs in `update()`.

- [ ] **Step 7: Run UI/core verification**

Run: `cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; cargo build --release`

Expected: job smoke test, all core tests, clippy, and a release build pass.

- [ ] **Step 8: Commit**

```text
git add src/app.rs src/ui src/jobs.rs src/main.rs src/lib.rs tests/ui_smoke.rs
git commit -m "feat: add responsive SubLens desktop prototype"
```

### Task 7: Add protected settings/history and finish prototype UX

**Files:**
- Create: `src/settings.rs`
- Create: `src/history.rs`
- Create: `src/platform/mod.rs`
- Create: `src/platform/windows_dpapi.rs`
- Modify: `src/app.rs`
- Modify: `src/ui/settings.rs`
- Create: `tests/settings_and_history.rs`

**Interfaces:**
- `settings::AppSettings` with history disabled by default, resolver limits, timeout, and sensitive-display session flag.
- `history::HistoryStore::{load, append, clear}`.
- `platform::protect` and `platform::unprotect` behind Windows cfg, with a safe disabled fallback on non-Windows.

- [ ] **Step 1: Write failing settings/history tests**

Assert default history is disabled, clear removes all entries, duplicate URLs are not appended twice, and non-Windows/test fallback never writes plaintext secrets.

- [ ] **Step 2: Implement settings and DPAPI-backed history**

On Windows use `CryptProtectData`/`CryptUnprotectData` through `windows-sys`, store only the encrypted history blob below the `directories` config directory, and fail closed if protection fails. On non-Windows or when DPAPI is unavailable, keep history disabled and in memory.

- [ ] **Step 3: Add settings UI and session-only sensitive reveal**

Expose history toggle, clear button, limits, timeout, and a session reset for sensitive visibility. Persist only non-secret settings in a small JSON file; never persist raw URI or revealed values in settings.

- [ ] **Step 4: Run verification and commit**

Run: `cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; cargo build --release`

```text
git add src/settings.rs src/history.rs src/platform src/app.rs src/ui/settings.rs tests/settings_and_history.rs
git commit -m "feat: add protected history and settings"
```

### Task 8: Add README, CI, live-safe command, and evidence gates

**Files:**
- Create: `README.md`
- Create: `LICENSE`
- Create: `.github/workflows/ci.yml`
- Create: `src/bin/live_acceptance.rs`
- Create: `tests/fixtures/README.md`
- Modify: `src/lib.rs`

**Interfaces:**
- `live_acceptance` prints only status, content type, stage names, total count, and protocol counts.
- CI runs `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, Linux core build/test, and Windows release build.

- [ ] **Step 1: Write a redaction test for live output**

Pass a synthetic report containing a UUID and password to the live summary formatter and assert neither value appears in its output.

- [ ] **Step 2: Implement the optional live command**

Fetch only when explicitly invoked, use the normal resolver, and print sanitized lines such as `URL fetched: PASS`, `Decode pipeline: PASS`, `Configs extracted: N`, and protocol totals. Exit nonzero on fetch failure but never print response content.

- [ ] **Step 3: Add README and fixture documentation**

Document supported protocols/formats, privacy guarantees, build/run commands, v2rayN import, QR/export behavior, diagnostics limitation, resolver limits, history behavior, known limitations, and include a screenshot only when a real screenshot is captured during acceptance.

- [ ] **Step 4: Add CI workflow**

Use stable Rust on Ubuntu for core checks and `windows-latest` for formatting, clippy, tests, and `cargo build --release`. Keep live acceptance manual and never run it in CI.

- [ ] **Step 5: Run final prototype verification**

Run locally:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
cargo run --bin live_acceptance -- --url <your-subscription-url>
```

Record the exact results without secrets. If the live source is unavailable, record the sanitized HTTP/network error and keep the prototype architecture unchanged.

- [ ] **Step 6: Commit**

```text
git add README.md LICENSE .github src/bin tests/fixtures/README.md src/lib.rs
git commit -m "docs: add SubLens build CI and live acceptance"
```

## Plan self-review

- Spec coverage: model/security are Task 1; detection/decode/extraction are Task 2; all required parsers are Task 3; redirects, recursion, limits, stages, and deduplication are Task 4; exports/QR/diagnostics are Task 5; responsiveness, cards, Inspector, filters, and cancellation are Task 6; protected history and settings are Task 7; README, CI, and live evidence are Task 8.
- Completeness scan: no unfinished markers or unbounded “add appropriate handling” steps are present.
- Type consistency: `ProxyConfig`, `PipelineStage`, `ResolverConfig`, `AnalysisReport`, `JobEvent`, and export/diagnostic signatures are introduced before their consumers.
- Scope check: Tasks 1–6 produce the first working prototype; Tasks 7–8 close the explicit security, documentation, CI, and acceptance requirements without changing the core architecture.
