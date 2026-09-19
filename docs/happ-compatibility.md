# HAPP compatibility boundary

SubLens follows the public HAPP formats without copying private app behavior or
spoofing device identity.

| Input | Content type | Encoding | Headers/directives | Output |
| --- | --- | --- | --- | --- |
| VLESS, VMess, Trojan, Shadowsocks, Socks5, Hysteria2/Hy2 | text/plain | URI text | none required | normalized fields + exact `raw_uri` |
| Plain text URI list | text/plain | UTF-8 | none required | one `ProxyConfig` per parsed URI |
| Subscription payload | text/plain or provider-defined | standard Base64, Base64URL, missing padding | optional HAPP metadata | recursively decoded candidates; one `Happ/1.0` retry when HAPP profile headers are present |
| JSON arrays / nested JSON strings | application/json | UTF-8, embedded Base64 accepted | optional HAPP metadata | extracted URI and nested URL candidates |
| Ready-made Xray JSON objects | application/json | UTF-8 | optional HAPP metadata | `JsonConfiguration` profiles parsed into inspectable rows; original profile JSON remains the raw copy payload |
| HTML links, scripts, data attributes | text/html | UTF-8 | none required | URI and nested HTTP(S) candidates; JavaScript is not executed |
| Hysteria2 multi-port `port=1234,5000-6000,7044` | text/plain | URI query | none required | full range displayed; first port used for endpoint diagnostics |
| `happ://crypt4/` and `happ://crypt5/` | application-specific | encrypted | app-embedded key | `EncryptedHappSubscription` classification; the key is not available to SubLens, so no decryption attempt |
| `profile-title`, `profile-update-interval`, `subscription-userinfo`, `support-url`, and documented HAPP management headers | any subscription response | header value or `#key: value` body line | captured into core metadata; device/HWID headers are deliberately excluded | UI remains compact; metadata is not printed by live summary |
| HAPP app-management directives such as `change-user-agent`, `profile-web-page-url`, `announce`, and sorting flags | any subscription response | header value or `#key: value` body line | preserved in `directives`; only the documented metadata fields above are projected into typed fields | never executed implicitly by SubLens |
| HAPP device headers or HWID | provider-defined | not fabricated | deliberately absent | only documented `User-Agent: Happ/1.0` retry is used |

## Public implementation observations

The public [happ-cli](https://github.com/GenkaOk/happ-cli) fork of
[proxray](https://github.com/aimuzov/proxray) documents the same compatibility
boundary: HAPP-compatible fetches can return Base64 share-link lists or JSON
ready-made Xray configurations, with INCY/HAPP subscription metadata. Its
server list can parse and display Hysteria2 while marking it unsupported for
connection. SubLens adopts only the observable parsing/fetch behavior; it does
not embed Xray, start TUN/system proxy modes, or fabricate HWID/device headers.
Remote app-management directives are treated as metadata rather than commands:
SubLens records unknown keys for inspection but does not change its User-Agent,
sorting, routing, or desktop settings because a subscription body requested it.

References: [HAPP protocol links](https://github.com/HappDev/happ_su/blob/main/faq/adding-configuration-subscription.md), [HAPP examples](https://github.com/HappDev/happ_su/blob/main/dev-docs/examples-of-links-and-parameters.md), [HAPP app management](https://github.com/HappDev/happ_su/blob/main/dev-docs/app-management.md), [HAPP encrypted links](https://www.happ.su/main/zh/dev-docs/crypto-link), [happ-cli](https://github.com/GenkaOk/happ-cli), and [proxray](https://github.com/aimuzov/proxray).
