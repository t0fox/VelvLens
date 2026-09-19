# SubLens visual system

SubLens is a desktop inspection tool. The interface is a focused workspace, not a marketing page or a dashboard mosaic.

## Direction

- Dark navy application canvas with calm elevation between surfaces.
- Violet is the single action accent and is reserved for the primary analysis action, selected filters, and selected configuration.
- Green communicates completed pipeline stages and healthy diagnostics. Amber and red are reserved for warnings and failures.
- The first viewport is compact-first: identity, source input, analysis action, one-line pipeline status, then the current page.
- Below the layout budget, the catalog and inspector are separate screens with an explicit Back action. On wide windows they become a dense catalog plus inspector split.
- The compact list is the primary navigation surface; the whole profile row is clickable and selection mode is a separate, explicit state.

## Tokens

| Token | Value | Use |
| --- | --- | --- |
| Canvas | `#0B111C` | Window and working area |
| Surface | `#121A27` | Panels and cards |
| Raised surface | `#182233` | Hover and selected controls |
| Border | `#27364B` | Quiet separation |
| Text | `#F4F7FB` | Primary content |
| Muted text | `#8D9AAF` | Labels and secondary metadata |
| Accent | `#7C3AED` | Primary action and selected state |
| Accent hover | `#8B5CF6` | Hover and pressed feedback |
| Success | `#35D07F` | Passed stages and healthy status |
| Warning | `#E7AD3C` | Slow or attention state |
| Error | `#F16B6B` | Failed operations |

## Interaction rules

- Controls keep a visible hover state and a darker pressed state.
- Primary buttons use a short press response and explicit status feedback.
- No decorative motion is added to a repeated workflow. State transitions stay immediate and readable.
- Long source URLs and hostnames stay in a single clipped line; the full value remains available through the existing copy actions.
- Keyboard focus must remain visible through egui's native focus treatment.
- Compact actions are kept in a More menu; the pinned bottom action is reserved for the current task (copy or bulk copy).
- LTE filtering is a real three-state filter (All, Exclude LTE, Only LTE) and matches token boundaries in display names only.

## Layout rhythm

- Base spacing: 4px, 8px, 12px, 16px, 24px.
- Outer panel padding: 16px.
- Catalog row gap: 8px.
- Small radius: 6px for controls and badges.
- Surface radius: 12px for inspector and configuration cards.

## Responsive budget

- Compact layout: below `1000` logical px. The list and details are separate pages, each with its own vertical scroll state.
- Wide layout: `1000` logical px and above. The catalog starts at 286px and the inspector keeps a readable working width.
- The threshold reflects the minimum catalog, readable detail surface, gutters, and splitter; it is a layout budget rather than a device class.
