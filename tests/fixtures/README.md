# Sanitized fixtures

These files contain synthetic hosts, identifiers, and credentials only. They
exercise the resolver without contacting a provider or storing a live
subscription response.

- `plain_vless.txt`, `raw_mixed.txt`: raw URI lists.
- `base64_subscription.txt`, `base64url_no_padding.txt`: encoded payloads.
- `happ_formats.txt`: VMess, Socks5, Hysteria2, and metadata directives.
- `nested.json`, `ready_xray.json`, `embedded.html`: recursive JSON/HTML cases.
- `duplicate_uris.txt`, `semantic_duplicates.txt`: deduplication behavior.
- `loop.json`, `bad_base64.txt`, `unknown_scheme.txt`: limits and graceful
  failure/unknown-format behavior.
- `visual_sanitized.txt`: the local UI screenshot fixture.

The live acceptance command is deliberately separate and never writes its
response into this directory.
