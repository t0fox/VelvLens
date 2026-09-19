# SubLens

SubLens is a local Windows visual inspector for proxy subscription and share URLs. It resolves redirects and nested sources, decodes text/Base64/Base64URL/JSON/HTML payloads, parses proxy URIs into a normalized model, and shows the pipeline and configurations in a dark egui desktop interface.

## Workflow

1. Open SubLens.
2. Paste a subscription or share URL.
3. Select **Analyze**.
4. Review the HTTP → Decode → Extract → Parse stages.
5. Select a card to open the Inspector; use the visible **Copy configuration**, **Copy**, or **QR** actions. Ready-made JSON profiles copy their original profile JSON and do not offer QR because they are not share URIs.
6. Select protocol categories, combine Security/Transport/Status/Duplicates filters, use **Select visible**, or check individual cards.
7. Use **Test** for local DNS/TCP connectivity evidence, or **Export** for raw payload and JSON output; Base64 is available when the result is a URI subscription.
8. Use **Copy selected** or **Copy all configs** when you need an explicit raw URI list.

## Supported content

- HTTP(S) sources with manual redirect-chain capture.
- Raw line-oriented URI lists.
- Standard Base64 and URL-safe Base64 without padding.
- Nested JSON strings/arrays/objects.
- Ready-made Xray JSON profile arrays, normalized for inspection while retaining each profile's original JSON payload.
- HTML DOM text, links, attributes, scripts, and embedded JSON.
- Nested subscription URLs with loop/depth/byte/count limits.

Supported protocols are VLESS, VMess, Trojan, Shadowsocks, Socks5, Hysteria/Hysteria2/Hy2, and TUIC. Unknown URI schemes are preserved as inspectable **Unknown** configurations instead of being silently discarded.

The catalog supports multiple protocol filters at once. The name/host search accepts exclusion terms such as `-LTE` or `-Torrent`, so unwanted categories can be left out before using **Select visible**.

## Privacy and safety

- No analytics, telemetry, cloud backend, or external QR service.
- HTTP, DNS, TCP, decoding, parsing, and QR generation run locally.
- Pipeline previews, logs, and errors redact credentials and tokens. The selected configuration view and explicit copy/export actions preserve the original URI and fields.
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

- [Download the latest Windows executable](../../releases/latest/download/sublens.exe)
- [Open GitHub Actions builds](../../actions/workflows/ci.yml)

Every push and pull request keeps a downloadable Actions artifact. A `v*` tag also publishes `sublens.exe` to a GitHub Release, which is the stable download linked above.

## Live-safe acceptance

The optional manual command uses the normal bounded resolver and prints only sanitized stages, counts, protocol totals, and byte totals:

```text
cargo run --bin live_acceptance -- --url <your-subscription-url>
```

The live URL is never used by automated tests, and subscription contents/credentials are not committed.

## Known limitations

- Diagnostics currently stop at DNS and TCP connectivity; protocol-level Xray-core testing is not included.
- Windows DPAPI history is implemented only when history is explicitly enabled.
- Export keeps original raw URIs and unknown query keys, but does not rewrite every provider-specific URI into a new canonical format.
