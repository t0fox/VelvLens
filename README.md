# SubLens

SubLens is a local Windows visual inspector for proxy subscription and share URLs. It resolves redirects and nested sources, decodes text/Base64/Base64URL/JSON/HTML payloads, parses proxy URIs into a normalized model, and shows the pipeline and configurations in a dark egui desktop interface.

## Workflow

1. Open SubLens.
2. Paste a subscription or share URL.
3. Select **Analyze**.
4. Review the HTTP → Decode → Extract → Parse stages.
5. Select a card to open the Inspector; use the visible **Copy URI**, **Copy safe**, or **QR** actions.
6. Select protocol categories, combine Security/Transport/Status/Duplicates filters, use **Select visible**, or check individual cards.
7. Use **Test** for local DNS/TCP connectivity evidence, or **Export** for raw URI, v2rayN, Base64, and sanitized JSON output.
8. Use **Copy selected** or **Copy for v2rayN** and paste the result into v2rayN's **Import bulk URL from clipboard** action.

## Supported content

- HTTP(S) sources with manual redirect-chain capture.
- Raw line-oriented URI lists.
- Standard Base64 and URL-safe Base64 without padding.
- Nested JSON strings/arrays/objects.
- HTML DOM text, links, attributes, scripts, and embedded JSON.
- Nested subscription URLs with loop/depth/byte/count limits.

Supported protocols are VLESS, VMess, Trojan, Shadowsocks, Hysteria/Hysteria2/Hy2, and TUIC. Unknown URI schemes are preserved as inspectable **Unknown** configurations instead of being silently discarded.

The catalog supports multiple protocol filters at once. The name/host search accepts exclusion terms such as `-LTE` or `-Torrent`, so unwanted categories can be left out before using **Select visible**.

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
