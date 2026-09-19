# SubLens

SubLens is a local Windows visual inspector for proxy subscription and share URLs. It resolves redirects and nested sources, decodes text/Base64/Base64URL/JSON/HTML payloads, parses proxy URIs into a normalized model, and shows the pipeline and configurations in a dark egui desktop interface.

## Workflow

1. Open SubLens.
2. Paste a subscription or share URL.
3. Select **Analyze**.
4. Review the HTTP → Decode → Extract → Parse stages.
5. Select a card to open the Inspector; on compact windows this is a separate details screen with **Back** and a pinned copy action. JSON profiles are resolved into individual endpoints and expose a generated share URI when the target protocol can represent the endpoint.
6. Select protocol categories, combine Security/Transport/Status/Duplicates filters, use the three-state LTE filter, or enter selection mode for bulk copy.
7. Use **Test** for local DNS/TCP connectivity evidence, or **Export** for separate available share URIs, explicitly limited URIs, original JSON documents, model JSON, and Base64 of available share URIs.
8. Use **Copy selected** or **Copy all** to review counts for Available/Limited/Unavailable before copying. The primary bulk export contains only standard share URIs; original JSON is never mixed into that list.

## Supported content

- HTTP(S) sources with manual redirect-chain capture.
- Raw line-oriented URI lists.
- Standard Base64 and URL-safe Base64 without padding.
- Nested JSON strings/arrays/objects.
- Xray-compatible JSON profiles, expanded into separate proxy endpoints from `outbounds`, `vnext`, and `servers` while retaining one shared original document per source.
- HTML DOM text, links, attributes, scripts, and embedded JSON.
- Nested subscription URLs with loop/depth/byte/count limits.

Supported protocols are VLESS, VMess, Trojan, Shadowsocks, Socks5, Hysteria/Hysteria2/Hy2, and TUIC. Unknown URI schemes are preserved as inspectable **Unknown** configurations instead of being silently discarded.

The catalog supports multiple protocol filters at once. The name/host search accepts exclusion terms such as `-LTE` or `-Torrent`, so unwanted categories can be left out before using selection mode. LTE detection is token-boundary aware and only examines the display name, so names such as `COMPLETE` and `DELETE` are not falsely excluded.

The layout is compact-first below 1000 logical pixels: the catalog and Inspector become separate scrollable screens. The mouse wheel scrolls the entire catalog column, including its filters and search controls. Wide windows keep the catalog and Inspector side by side. Repaints follow the active Windows monitor refresh rate, with a bounded 60 Hz fallback when the rate cannot be read.

## Privacy and safety

- No analytics, telemetry, cloud backend, or external QR service.
- HTTP, DNS, TCP, decoding, parsing, and QR generation run locally.
- Pipeline previews, logs, and errors redact credentials and tokens. The selected configuration view and explicit copy/export actions preserve the original URI, source JSON, and usable credentials.
- Raw response bodies are not written to disk automatically.
- URL history is disabled by default. When enabled on Windows, it is protected with DPAPI and can be cleared.
- DNS/TCP checks are connectivity-only; TCP success is not a claim that a VLESS/Reality protocol session works.

## Build

Install Rust stable and the Visual C++ Build Tools workload, then run from a Developer Command Prompt:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release --bins
```

The Windows executable is `target/release/sublens.exe`.

## Download

- [Скачать последний Windows exe из Release](../../releases/latest/download/sublens.exe)
- [Открыть последний запуск GitHub Actions и скачать artifact](../../actions/workflows/ci.yml)

Каждый push и pull request проходит сборку и сохраняет downloadable Actions
artifact `sublens-windows-x86_64`. Push тега `v*` дополнительно публикует
`sublens.exe` в GitHub Release; стабильная ссылка выше всегда указывает на
последний опубликованный exe.

## Live-safe acceptance

The optional manual command uses the normal bounded resolver and prints only sanitized stages, counts, protocol totals, and byte totals:

```text
cargo run --bin live_acceptance -- --url <your-subscription-url>
```

The live URL is never used by automated tests, and subscription contents/credentials are not committed.

## Known limitations

- Diagnostics currently stop at DNS and TCP connectivity; protocol-level Xray-core testing is not included.
- Windows DPAPI history is implemented only when history is explicitly enabled.
- Ordinary input URIs are copied byte-for-byte from their original representation. JSON endpoints have protocol-specific serializers for VLESS, VMess, Trojan, Shadowsocks, Hysteria2, and TUIC. Unsupported or ambiguous fields are reported as Limited/Unavailable instead of being silently dropped.
- Hysteria2 follows the official URI scheme for credentials, SNI, display name, and multi-port authority syntax. Vendor-specific TLS fingerprints and QUIC extensions remain explicitly Limited; the original JSON export is the lossless fallback.
