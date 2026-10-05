# Shared settings and About UI

`imago-gpui` owns the reusable visual implementation extracted from
`cortex/manager-gpui`. Applications keep routes, RPC calls, loaded values,
persistence, support IO, capability checks and native window policies.
There is no dependency on `ManagerApp` or `Agenda` in these components.

## Components

- `settings::UiStyle`: resolved foreground/muted/accent/border/surface colors
  and independent text scale. Defaults use Imago's active palette. Supply
  application overrides explicitly; it does not mutate global theme state.
- `settings::{page_stack, page_sections, section_group}` and `UiStyle` methods:
  cards, rows, key/value fields, section labels, settings rows, inputs and H1.
  Cortex's physical spacing and soft ink plaques are shared; typography can
  scale without changing the layout grid.
- `appearance::{miniature, system_preview, ModeTile}`: palette previews and
  the system/light/dark selector tiles. `ModeTile` accepts a persistent
  `FocusHandle`, selection, disabled state, optional icon, style and callback.
  Pointer, Enter and Space use the same callback; disabled tiles do not fire.
- `toggle::{Toggle, ToggleColors, toggle}`: Cortex's switch geometry,
  180 ms reversible thumb motion, optional glass recipe and native macOS
  selection feedback. `gpui_base::Switch` supplies keyboard/focus/a11y and
  disabled behavior. The switch geometry remains physical, not font-scaled.
- `about::{AboutHero, Diagnostics, DiagnosticCheck, DiagnosticState}`:
  product icon/name/version/copyright and the expandable diagnostics plaque.
  They do not fetch data. Pass real checks from the application; loading is
  `Pending`, not `Ok`. Product assets and attribution are caller-supplied.

Example (inside an application's render method):

```rust,ignore
let style = UiStyle { accent: resolved_accent, font_scale: font_size / 13., ..Default::default() };
let switch = imago_gpui::toggle::toggle("vsync", self.vsync, cx, |app, next, _| {
    app.vsync = next;
}).accessibility_label("Вертикальная синхронизация")
  .glass(glass)
  .colors(ToggleColors { accent: style.accent, foreground: style.foreground, surface: style.surface });

style.card().child(style.row("VSync", "Частота обновления экрана").child(switch))
```

Use `imago_gpui::assets::Assets` as an asset fallback (including sun/moon/
monitor and the underlying GPUI asset bundle). Do not copy shared SVGs into
consumer applications.

## Application integration

Agenda and Cortex now contain small adapters that pass their resolved accent,
font scale, material, selection and persistence callbacks into these components.
Cortex retains Engine-owned settings, wallpaper/accent controls, typography,
updates and support actions. Agenda retains its Engine-owned per-app appearance policy.
Sharing a renderer does **not** make unrelated applications write Cortex's
configuration or diagnostics.

The old `mundus-gpui-kit::fields` exports remain compatible. Newly extracted
visuals live in `imago-gpui`; Engine/client helpers remain in `mundus-gpui-kit`.

## Local development before publishing a revision

Consumers still pin both Imago crates to the same Git revision. Unpublished
changes can be tested with a local Cargo config patch for **both** crates:

```toml
[patch."https://github.com/makekosmos/imago.git"]
imago-gpui = { path = "/absolute/path/to/imago/crates/imago-gpui" }
mundus-gpui-kit = { path = "/absolute/path/to/imago/crates/mundus-gpui-kit" }
```

Save this as `.cargo/imago-local.toml` in the application (do not commit
machine-specific paths), then use `cargo --config .cargo/imago-local.toml
check` / `test` / `run`. Agenda's `./dev-watch.sh --local-imago` creates this
config and watches both its sources and the adjacent Imago checkout.

A local-override lockfile is development-only. Before shipping, publish the
reviewed Imago commit, update **both** consumer Git pins together, remove the
local override and regenerate the lockfile against that Git revision. Do not
ship consumer changes with the previous revision that lacks these modules.

## Verification

The Imago workspace patches its own vendored GPUI snapshots for tests so its
TestWindow exposes the same accessibility tree as the consumers. Those
workspace patches do not propagate into consumer manifests.

`cargo test --workspace` covers shared switch interaction/disabled/haptics,
preview tile keyboard activation, product/a11y semantics and text scaling.
Consumer tests retain the real application adapters and Engine policy checks.
