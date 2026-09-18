# SubLens — visual proxy subscription inspector

## Status

Approved architectural direction; implementation is intentionally not started until this specification is reviewed.

## Goal

SubLens is a local Windows desktop utility for inspecting proxy subscription and share URLs. It accepts an HTTP(S) source, follows safe recursive resolution rules, detects and decodes embedded content, extracts proxy URIs, parses them into a normalized model, displays a visual pipeline and configuration inspector, runs DNS/TCP connectivity checks, and exports results for v2rayN and other consumers.

The product is a real utility, not a Base64 decoder or console proof-of-concept. It has no cloud backend, telemetry, analytics, or Electron runtime.

## Product boundaries

### In scope

- Windows-first desktop application built with Rust, `eframe`, and `egui`.
- Platform-independent resolver, parser, export, redaction, and diagnostics core.
- Recursive source resolution for redirects, text, Base64, Base64URL, JSON, HTML, nested URLs, and raw proxy URIs.
- VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, and graceful unknown-protocol handling.
- Normalized configuration model shared by UI, export, deduplication, and diagnostics.
- Visual pipeline inspector, configuration cards, detail panel, filters, search, deduplication views, QR generation, clipboard actions, and cancellation.
- Unit tests, fixture-based integration tests, deterministic mock HTTP scenarios, CI, and an optional live acceptance command.

### Explicitly out of scope for the first release

- A local proxy engine or traffic forwarding.
- Full protocol health testing or claims of end-to-end VLESS/Reality availability.
- Automatic credential rotation or synchronization.
- External geolocation/QR/analytics services.
- Xray-core integration; the diagnostics and normalized model must leave a clean extension point for it later.

## Architecture

The application uses a clean-core architecture:

```text
egui application
    │  commands/events over channels
    ▼
background job manager + Tokio runtime
    │
    ▼
SubLens Core
    ├── model
    ├── resolver
    ├── protocols
    ├── diagnostics
    ├── export
    └── security
```

The core does not depend on `egui`. The UI consumes an `AnalysisReport` and does not parse raw URI strings or infer protocol fields itself. The job manager owns cancellation, progress messages, and runtime lifetime; no HTTP, DNS, or TCP operation executes on the egui update thread.

## Core contracts

### Normalized configuration

`ProxyConfig` is the canonical representation for all downstream features:

- stable `id`;
- `Protocol` enum: VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, Unknown;
- optional display name;
- host and port;
- UUID, username, password, and other sensitive credentials as optional fields;
- `Security` enum: Reality, TLS, None, Unknown;
- `Transport` enum: TCP, XHTTP, WebSocket, gRPC, HTTP2, QUIC, Unknown;
- SNI, fingerprint, Reality public key, Reality short ID, flow, and protocol-specific normalized fields;
- original `raw_uri` retained in memory;
- `unknown_params` preserving unsupported query keys and values;
- source and metadata such as depth, source URL, labels, exact duplicate count, and semantic duplicate key.

Sensitive fields are never used directly for display. The UI asks the security module for redacted projections.

### Analysis report

`AnalysisReport` contains:

- ordered `PipelineStage` values;
- sanitized redirect chain;
- normalized configurations;
- duplicate groups and exact-duplicate counts;
- optional DNS/TCP diagnostics;
- protocol counters and total counts;
- resolver limits and summary metadata.

Each pipeline stage includes operation type, status, sanitized source, bounded preview, counters, duration, and a typed/sanitized error when applicable. Unknown content is represented as a successful detection stage with an unsupported content type rather than an application-wide failure.

## Resolver design

Resolution is a bounded graph traversal rather than a single decode operation:

```text
source URL
  → manual HTTP fetch + redirect capture
  → content detection
  → text/HTML/JSON URI and nested-source extraction
  → Base64/Base64URL candidate decoding
  → repeat for discovered sources
  → protocol parsing
  → exact + semantic deduplication
  → AnalysisReport
```

HTTP redirects are followed manually with a `reqwest` client configured for timeouts and no automatic redirect following. Each response records status, sanitized source/destination, content type, and bounded body size. Relative `Location` values resolve against the current URL.

The traversal uses a visited canonical URL set, content fingerprints, and explicit limits:

- maximum recursion depth: 8 by default;
- maximum downloaded bytes per source and total;
- maximum discovered source URLs;
- maximum redirect hops;
- maximum number of extracted proxy URIs;
- request and total operation timeout;
- cancellation checks at fetch, read, decode, extraction, and traversal boundaries.

Content detection is heuristic but explainable. It recognizes raw protocol schemes, line-oriented URI lists, JSON, HTML, standard Base64, URL-safe Base64, and unknown text/binary content. Base64 candidates are accepted only when decoding yields meaningful supported structures or proxy URI lines, limiting false-positive expansion.

Extraction walks all JSON strings, arrays, and nested objects. HTML extraction covers DOM text, attributes, inline scripts, JSON blobs, data attributes, and embedded payload text. HTTP(S) URLs found inside content become nested sources subject to the same limits and visited set.

## Protocol parsers

Each protocol has an independent module and a shared parser interface. Parsers tolerate unknown query parameters, preserve them in `unknown_params`, redact errors, and return typed parse failures without aborting sibling configurations.

- VLESS: UUID, address, port, query parameters, security, flow, type, SNI, fingerprint, Reality public key/short ID, path, host, service name, and mode.
- VMess: Base64 payload decode plus JSON field normalization.
- Trojan: password, address, port, TLS/SNI and transport options.
- Shadowsocks: common SIP002 and legacy URI forms, including method/userinfo variants.
- Hysteria2/Hy2: password/auth, server, port, TLS/SNI and transport fields at a basic normalized level.
- TUIC: UUID/password, server, port, congestion/TLS/SNI fields at a basic normalized level.

Unknown schemes are retained as unknown candidates with raw data and an Inspector explanation. They are never silently discarded.

## Deduplication

Deduplication runs after parsing and does not destroy originals.

- Exact duplicates use the complete raw URI string.
- Semantic duplicates use a stable normalized key made from protocol, endpoint, security, transport, SNI, fingerprint, Reality identifiers, and protocol-relevant credentials where necessary.
- The report retains every original, marks its group, and exposes exact/semantic counts.
- UI options hide exact duplicates or group semantic duplicates without deleting data.

## Diagnostics

DNS resolution and TCP connection checks are separate asynchronous operations with independent timeouts and cancellation. Results include DNS status, TCP status, elapsed latency, and sanitized errors. The UI labels these checks as connectivity-only and explicitly does not represent TCP success as protocol health or VLESS/Reality success.

The diagnostic trait accepts a future protocol-engine adapter so a later Xray-core integration can add deeper checks without changing cards or the normalized model.

## UI design

The main window is a compact dark utility interface optimized for 1920×1080:

1. Header/input bar with subscription URL, Paste, Analyze, Cancel, Settings, and optional recent history.
2. Pipeline strip showing HTTP → Decode → Extract → Parse with stage badges, progress, previews, counters, redirect chain, and errors.
3. Filter/search row for All, protocol, Security, Transport, Working/Failed/Unknown, and duplicate handling.
4. Scrollable configuration card grid/list with protocol badge, name, endpoint, transport, security, SNI/fingerprint, status, latency, Copy, Details, QR, and Test actions.
5. Inspector panel/modal for structured details, sanitized raw URI, raw URI reveal/copy, v2rayN export, and local QR display.

UI state is derived from core results. Sensitive fields are masked by default and reveal is session-only. Long operations show progress and a working state while keeping controls responsive.

## Privacy and security

- No telemetry, analytics, cloud backend, or external QR service.
- `redact_secret` is the centralized path for logs, previews, errors, redirect display, and diagnostics.
- UUIDs, passwords, tokens, private keys, and sensitive query values are hidden by default.
- Raw subscription responses are held in memory and are never written automatically.
- Clipboard writes happen only after an explicit user action.
- History is disabled by default. When enabled, recent subscription URLs are stored locally using Windows DPAPI, with clear-history and disable-history controls. If protected storage is unavailable, history remains disabled rather than silently writing plaintext.
- Logs contain operation metadata and counts, never credentials or raw subscription bodies.

## Export and QR

Exports consume normalized configurations but preserve each parser's original raw URI and unknown parameters.

- selected raw URI;
- all raw URIs, one per line;
- v2rayN bulk URL clipboard output, one importable URI per line;
- `.txt` raw URI list;
- Base64 subscription;
- JSON normalized dump with sensitive fields masked unless explicitly requested;
- local QR generated from the raw URI using the Rust QR library and rendered in egui.

## Testing strategy

Unit tests cover redaction, URL canonicalization, Base64/Base64URL padding, detector decisions, JSON/HTML/text extraction, each protocol parser, semantic key generation, and deduplication.

Fixture integration tests cover raw VLESS, Base64 VLESS, Base64URL, VMess, Trojan, Shadowsocks, mixed protocols, malformed content, nested JSON/HTML, nested subscriptions, redirect chains, recursion loops, limits, and duplicates. Fixtures contain synthetic credentials only.

A deterministic local mock HTTP server tests redirects, nested URLs, content types, timeout behavior, and loop protection. No test depends on a personal subscription URL.

The optional live command fetches a user-supplied subscription URL and writes only status, content type, stage summaries, totals, and protocol counts. It must never write the subscription, raw URIs, UUIDs, or credentials to logs or fixtures.

## Delivery and acceptance

The project must pass, on the relevant platform:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

GitHub Actions run formatting, clippy, tests, Windows release build, and preferably Linux core tests. Final reporting separates focused test results, Windows build evidence, GUI smoke evidence, live acceptance, and unverified external behavior. The application is not called complete if a mandatory Definition of Done item lacks implementation or evidence.

## Implementation sequence

1. Scaffold Cargo package, workspace conventions, error types, model, redaction, and fixtures.
2. Implement detector/decoders/extractors and bounded resolver with deterministic tests.
3. Implement protocol parsers, normalized metadata, and deduplication.
4. Implement exports, QR support, diagnostics, and clipboard integration.
5. Implement the background job manager and complete egui application shell.
6. Add settings/history protection, inspector, filters, cards, and responsive states.
7. Add CI, README, screenshots/evidence, release build, and optional live acceptance.

Each major stage runs formatting, clippy, and tests before moving forward.
