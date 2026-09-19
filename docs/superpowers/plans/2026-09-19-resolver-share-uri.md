# VelvLens Resolver and Share URI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make VelvLens preserve original sources, extract every real proxy endpoint from JSON/Xray profiles, and expose an honest protocol-specific `Available`/`Limited`/`Unavailable` share URI contract to copy, export, and QR flows.

**Architecture:** Keep the existing Fetch → Detect → Decode → Extract → Parse → Normalize → Deduplicate pipeline. Replace the overloaded `ProxyConfig.raw_uri` field with `OriginalRepresentation` and `ShareUriResult`; ordinary URI candidates keep their exact source URI, while JSON candidates share one reference-counted source document and carry profile/outbound/endpoint identity. Xray parsing remains the only JSON resolver and delegates URI generation to `src/share_uri/` protocol modules.

**Tech Stack:** Rust 2021, serde/serde_json, url, base64, existing eframe/egui UI, existing resolver limits and tests.

**Spec:** `C:\Users\Kirill\.codex\attachments\a855c9d4-7993-415e-8724-dd866e14853f\pasted-text-1.txt`

## Global Constraints

- Work directly on `main`; do not create or use a side branch.
- Preserve the existing Fetch → Detect → Decode → Extract → Parse → Normalize → Deduplicate pipeline and all configured recursion, byte, candidate, URL, timeout, and cancellation limits.
- Keep Compact mode, Wide mode, Russian UI, dark theme, LTE filter, search, multi-select, Inspector, Export, and QR.
- Never treat JSON as an ordinary share URI and never silently emit a lossy URI as exact.
- Keep all JSON parsing and URI conversion local; do not send configuration data to external services or log secrets/full URIs/profiles.
- Use TDD for every production behavior change: write a failing test, observe the expected failure, implement the smallest fix, then run the relevant suite.

### Task 1: Split source, endpoint, and share URI in the model

**Files:**
- Modify: `Cargo.toml` (enable serde support for `Arc`)
- Modify: `src/model/proxy_config.rs`
- Modify: `src/model/mod.rs`
- Modify: `src/protocols/common.rs`
- Modify: `src/protocols/mod.rs`
- Modify: `src/dedup.rs`
- Test: `tests/model_and_redaction.rs`, `tests/format_fixtures.rs`

**Interfaces:**
- Add `OriginalRepresentation::ShareUri(String)` and `OriginalRepresentation::JsonProfile { document: Arc<str>, profile_index: usize, outbound_index: usize, endpoint_index: usize }`.
- Add `ConversionLimitation` and `ShareUriResult::{Available, Limited, Unavailable}` with display-safe reason text and helpers `uri()`, `is_available()`, `is_limited()`, and `is_unavailable()`.
- Replace `ProxyConfig.raw_uri` with `original: OriginalRepresentation` and `share_uri: ShareUriResult`.
- Add `ProxyConfig::original_text()`, `original_is_json()`, and `original_identity_key()`; expand `semantic_key()` to include all functional endpoint fields.
- Make `protocols::parse_uri()` set `original = ShareUri(exact_input)` and `share_uri = Available(exact_input)` after protocol parsing.

- [x] Write tests proving ordinary URI input keeps byte-for-byte source text, JSON source identity is reference-counted and indexed, and `ShareUriResult` states expose only copyable URIs.
- [x] Run the model/format tests and confirm the old `raw_uri` assertions fail for the intended missing field/contract.
- [x] Implement the new model and update parser construction/dedup keys without changing resolver behavior yet.
- [x] Run `cargo test --all-targets --all-features` and confirm all existing parser/model tests pass after their assertions are updated to the new contract.

### Task 2: Add protocol-specific share URI conversion

**Files:**
- Create: `src/share_uri/mod.rs`
- Create: `src/share_uri/common.rs`
- Create: `src/share_uri/vless.rs`
- Create: `src/share_uri/vmess.rs`
- Create: `src/share_uri/trojan.rs`
- Create: `src/share_uri/shadowsocks.rs`
- Create: `src/share_uri/hysteria2.rs`
- Create: `src/share_uri/tuic.rs`
- Modify: `src/lib.rs`
- Test: `tests/protocol_parsers.rs`, new `tests/share_uri.rs`

**Interfaces:**
- Implement `share_uri::from_json_config(&ProxyConfig, Vec<ConversionLimitation>) -> ShareUriResult`.
- Implement protocol modules that validate required credentials and serialize only documented fields; no generic `protocol://host:port` fallback.
- Use percent-encoded userinfo/query/fragment values and bracket IPv6 hosts.
- Hysteria2 supports password/auth, host, single port, documented `port`/port-hop representation already accepted by the resolver, SNI, ALPN, TLS insecurity, obfuscation, bandwidth, and protocol options when present; unsupported vendor QUIC fields become typed limitations.
- VLESS supports UUID, encryption, flow, security, transport, SNI, fingerprint, Reality public key/short ID, path, host header, gRPC service/mode, and preserved supported query values.
- VMess uses a base64 JSON share payload only when required fields can be represented; unsupported outbound fields become `Limited` instead of being dropped.
- Trojan, Shadowsocks, and TUIC each have their own serializer and required-credential validation.

- [x] Add failing round-trip tests for VLESS Reality, Hysteria2 with SNI/ALPN/port range, and one JSON-derived endpoint per remaining supported protocol.
- [x] Add failing tests for missing credentials, unsupported QUIC fields, IPv6, encoded auth, and `Limited`/`Unavailable` results.
- [x] Run the new tests and verify each failure is a missing serializer/validation behavior, not a fixture error.
- [x] Implement the protocol modules and common encoding helpers.
- [x] Parse generated available URIs through the existing protocol parsers and compare functional endpoint fields; assert limitations for fields that cannot round-trip.

### Task 3: Replace first-outbound Xray parsing with endpoint extraction

**Files:**
- Modify: `src/resolver/xray.rs`
- Modify: `src/resolver/pipeline.rs` only where result accounting/messages need the new count
- Modify: `src/resolver/extract.rs` if structural JSON strings need to be excluded
- Create/modify: `tests/fixtures/ready_xray_multi.json`, `tests/fixtures/ready_xray_hysteria2_limits.json`
- Test: `tests/xray_resolver.rs`

**Interfaces:**
- `parse_configurations(text, source, depth)` returns one `ProxyConfig` per supported endpoint across every profile/outbound/server/vnext/user, sharing one `Arc<str>` source document.
- Ignore `freedom`, `blackhole`, `dns`, local-only inbounds, routing rules, blocklists, and arbitrary URL strings.
- Stable JSON identity includes profile, outbound, and endpoint indexes; UI ids remain unique even when two endpoints share host/port.
- Extract Xray fields for endpoint, credentials, transport, TLS/Reality, Hysteria2, and supported unknown/extra fields; pass unsupported structural fields to the share converter as typed limitations.
- Do not recursively queue nested HTTP URLs from `ContentKind::JsonConfiguration`; preserve existing JSON configuration boundary and limits.

- [x] Add failing fixtures/tests for multiple proxy outbounds, multiple `vnext` users, multiple `servers`, service outbounds, DNS/routing/inbounds, unknown protocol, and Hysteria2 port ranges/vendor QUIC settings.
- [x] Run those tests against the current parser and observe the expected first-outbound/first-server/raw-JSON failures.
- [x] Implement indexed endpoint traversal and shared original-document ownership.
- [x] Attach `ShareUriResult` from the protocol-specific converter for every JSON endpoint.
- [x] Run all resolver tests and verify candidate/config counts do not include service or routing values.

### Task 4: Move copy, export, and QR to the new contract

**Files:**
- Modify: `src/export/raw.rs`
- Modify: `src/export/base64.rs`
- Modify: `src/export/json.rs`
- Modify: `src/export/mod.rs`
- Modify: `src/ui/details.rs`
- Modify: `src/app.rs`
- Modify: `src/ui/main_view.rs` only for export request/selection wiring if required
- Test: `tests/export_and_diagnostics.rs`, new `tests/share_export.rs`

**Interfaces:**
- Add `export::ShareExportSummary { available, limited, unavailable }` and `share_uri_lines()` that emits only exact/available standard URIs, one per line.
- Add separate original-source export for JSON profiles; never concatenate a URI with a multi-line JSON document in the share-URI export.
- Single-config UI copies the original URI unchanged for `Available` ordinary inputs, copies generated URI for JSON `Available`, offers explicit “Скопировать ссылку с ограничениями” for `Limited`, and never copies a JSON document as a link for `Unavailable`.
- Show concrete limitation/reason text and offer “Скопировать исходный JSON” for unavailable JSON profiles.
- QR encodes only a `ShareUriResult` URI; it is disabled for `Unavailable` and never receives original JSON.
- Bulk export previews counts for available/limited/unavailable before copying; the primary bulk action copies only available URIs, with explicit separate actions for limited links and original JSON.
- Sanitized JSON export redacts original JSON and generated share URI credentials while sensitive export preserves the exact original source payload.

- [x] Add failing tests proving JSON `raw_lines` never contains full JSON, available share export is line-oriented, limited/unavailable entries are counted, and QR receives only a share URI.
- [x] Run the export/UI tests and observe current JSON-as-URI failures.
- [x] Implement summary/export helpers and wire single, mass, selected, base64, JSON, and QR actions.
- [x] Run export and UI smoke tests; inspect the generated strings for secrets and multi-line mixing.

### Task 5: Pipeline provenance, fixtures, and live acceptance evidence

**Files:**
- Modify: `src/resolver/detect.rs`, `src/resolver/extract.rs`, or `src/resolver/pipeline.rs` only when tests prove a JSON provenance bug
- Add: fixtures covering the 16 scenarios from the specification under `tests/fixtures/`
- Add: `tests/resolver_share_acceptance.rs`
- Modify: `README.md` only if the final behavior/limitations need user-facing documentation

**Interfaces:**
- Keep redirects, nested subscriptions, Base64/Base64URL, HTML, bounds, cancellation, and HAPP classification unchanged.
- Assert JSON routing/metadata URLs are not fetched or emitted as nested sources.
- Assert ordinary URI, Base64 mixed protocols, JSON single/multi-endpoint, unsupported protocol, duplicates, and mixed export behavior.

- [x] Write failing acceptance fixtures/tests for all required scenarios, including JSON with ordinary HTTP URLs in routing/metadata.
- [x] Run targeted tests and classify each failure as resolver, model, conversion, or export.
- [x] Implement only provenance/limit fixes proven by those tests.
- [x] Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`, and `cargo build --release`.
- [x] Run the working Windows UI where available and verify clipboard output for ordinary URI and JSON-derived endpoints; if native UI is unavailable, report that gate as unverified rather than claiming acceptance.

### Task 6: Delivery on main and evidence report

**Files:**
- Modify: `README.md` only for final download/behavior notes if needed
- Commit: all implementation and test files on `main`

- [x] Review `git diff`, `git status`, and the complete requirement checklist.
- [ ] Commit the resolver/share/export change directly on `main` with a focused message.
- [ ] Push `main` and tag a new release only after all local gates pass.
- [ ] Confirm GitHub Actions Linux checks, Windows release build, artifact upload, and tagged release asset.
- [ ] Report root cause, architecture, protocol support, Hysteria2 limitations, export behavior, exact verification commands/results, commit HEAD, changed files, and any unverified native UI gate.
