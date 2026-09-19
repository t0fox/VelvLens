# SubLens visual system

SubLens is a compact Windows desktop inspector for proxy subscriptions. It is
a focused tool for resolving, filtering, selecting, inspecting, and exporting
configurations — not a dashboard and not a marketing surface.

## Product direction

- Calm dark workspace with one violet action accent.
- Dense catalog rows that remain readable at `520×550`.
- The whole profile row is the navigation target; explicit selection mode is
  used for bulk actions.
- Compact windows show one page at a time (catalog or details); wide windows
  show catalog and details side by side.
- All visible copy is Russian. Technical protocol names and configuration
  values remain unchanged: `VLESS`, `VMess`, `Trojan`, `Shadowsocks`,
  `Hysteria2`, `TUIC`, `REALITY`, `TLS`, `TCP`, `XHTTP`, `QUIC`, `JSON`,
  `Base64`, `SNI`, `UUID`, `Host`, and `Fingerprint`.

## Color tokens

| Token | Value | Use |
| --- | --- | --- |
| Background | `#111721` | Window and working area |
| Surface | `#1B2330` | Panels and cards |
| Surface hover | `#252E40` | Hovered and raised controls |
| Surface selected | `#29213F` | Selected profile/filter |
| Border | `#333D50` | Quiet separation |
| Border selected | `#8B5CF6` | Selected card and focus |
| Text primary | `#F3F4F8` | Titles and values |
| Text secondary | `#AAB5C8` | Metadata and secondary controls |
| Text muted | `#8793A7` | Labels and hints |
| Accent | `#7C3AED` | Primary actions |
| Accent hover | `#9061F9` | Hover and focus feedback |
| Success | `#4ADE80` | Completed stages and healthy status |
| Warning | `#F5B942` | Attention and running state |
| Error | `#F87171` | Failed operations |

## Geometry and typography

All geometry is centralized in `src/ui/theme.rs`:

| Token | Value |
| --- | --- |
| Window/panel/card radius | `12px` |
| Input/button radius | `10px` |
| Badge radius | `6px` |
| Rhythm | `2 / 4 / 6 / 8 / 12 / 16 / 24px` |
| Dense card height | `68px` |

On Windows, the existing `Segoe UI` system font is placed first for the
proportional family so Cyrillic and status glyphs render without squares; egui
Ubuntu remains the fallback when the system font is unavailable. Titles use
17–18px, card names 13–14px, body 12–13px, metadata 11–12px, and technical
labels 10–11px.

## Interaction rules

- Normal, hover, pressed, selected, disabled, and focus-visible states are
  defined centrally through egui visuals.
- Search is a raised rounded field with a search icon and the placeholder
  `Поиск конфигурации...`. LTE filtering is exposed as `Все`, `Без LTE`, and
  `Только LTE`; the old `-LTE` hint is not shown in the interface.
- Normal catalog mode has one `Выбрать` action. Selection mode exposes
  `Все видимые`, the selected count, `Действия`, `Готово`, and a bottom bulk
  copy action.
- Long hosts, keys, and URIs are clipped in dense layouts with the full value
  available on hover or through copy. Secrets are not masked on the details
  page and are never logged.
- The bottom action panel is reserved for the current task and reports
  `Скопировано` after a successful copy; the status uses text rather than a
  font-dependent symbol so Windows never renders a square glyph.
- Repeated workflows use immediate state changes; no decorative animation is
  added. Focus and hover states remain visible and keyboard reachable.

## Responsive budget

- Default window: `800×600`.
- Minimum window: `520×450`.
- Compact mode: below `1000px`; target review sizes are `520×550`,
  `640×480`, and `800×600`.
- Wide mode: `1000px` and above; the catalog remains between `286px` and
  `420px`, with a readable details surface beside it.
- Scrollable lists use egui's row virtualization and keep the catalog wheel
  responsive even when the pointer is over search/filter chrome.

## Decisions

- Visual redesign changes presentation and interaction density only. Resolver,
  parser, protocol support, deduplication, export, and local-only processing
  remain the existing owners.
- No framework, network service, compatibility UI, or second theme system was
  introduced.
- The design is reviewed with the offscreen visual fixture when native GUI
  automation is unavailable; such screenshots are explicitly labeled as
  fixture evidence.
