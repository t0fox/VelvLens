# SubLens

SubLens is a local Windows visual inspector for proxy subscription and share URLs. It resolves redirects and nested sources, decodes text/Base64/Base64URL/JSON/HTML payloads, parses proxy URIs into a normalized model, and shows the pipeline and configurations in a dark egui desktop interface.

## Prototype workflow

1. Open SubLens.
2. Paste a subscription or share URL.
3. Select **Analyze**.
4. Review the HTTP → Decode → Extract → Parse stages.
5. Select a card to open the Inspector, copy a URI, or generate a local QR code.
6. Use **Copy all for v2rayN** and paste it into v2rayN's **Import bulk URL from clipboard** action.

## Supported content

- HTTP(S) sources with manual redirect-chain capture.
- Raw line-oriented URI lists.
- Standard Base64 and URL-safe Base64 without padding.
- Nested JSON strings/arrays/objects.
- HTML DOM text, links, attributes, scripts, and embedded JSON.
- Nested subscription URLs with loop/depth/byte/count limits.

Supported protocols are VLESS, VMess, Trojan, Shadowsocks, Hysteria2/Hy2, and TUIC. Unknown content is reported in the pipeline and does not crash the application.

## Privacy and safety

- No analytics, telemetry, cloud backend, or external QR service.
- HTTP, DNS, TCP, decoding, parsing, and QR generation run locally.
- UUIDs, passwords, tokens, Reality keys, raw URI previews, logs, and errors are redacted by default.
- Raw response bodies are not written to disk automatically.
- URL history is disabled by default. When enabled on Windows, it is protected with DPAPI and can be cleared.
- DNS/TCP checks are connectivity-only; TCP success is not a claim that a VLESS/Reality protocol session works.

## Build

Install Rust stable and the Visual C++ Build Tools workload, then run from a Developer Command Prompt:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

The Windows executable is `target/release/sublens.exe`.

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
