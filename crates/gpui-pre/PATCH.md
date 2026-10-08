# Local patch vs crates.io gpui-pre 0.3.8

Single patched gpui revision shared by the three GPUI apps (KOS-328):
memoria-gpui, agenda-gpui and cortex/manager-gpui all `[patch.crates-io]`
to this directory via one imago rev, replacing the three divergent vendored
copies (agenda vendor/gpui-pre-0.3.5-patched, manager
vendor/gpui-pre-0.3.6-patched, memoria unpinned crates.io).

Carries two patch sets, rebased from the 0.3.6 vendor onto the 0.3.8 base
(gpui-kit / gpui-component / gpui-base 0.7.1 pin `gpui-pre =0.3.8`).

## From agenda-gpui vendor/gpui-pre-0.3.5-patched

- src/platform.rs — adds `PlatformWindow::hit_test_window_control_position`
  with a `None` default. The existing `on_hit_test_window_control` callback
  signature remains upstream-compatible, so non-Windows backend crates do not
  need local patches.
- src/window.rs — the closure hit-tests `rendered_frame` at the backend's
  in-flight hit-test position, falling back to `window.mouse_position`,
  instead of using the stale `window.mouse_hit_test` snapshot.
- src/window.rs — `bounds_changed` only marks the window dirty when
  `scale_factor`, `viewport_size` or `display_id` actually changed; a pure
  position change no longer triggers a full scene re-render inside the modal
  move loop on Windows.
- src/window.rs — `dispatch_event` records `mouse_position` from
  `MouseExited`, so the per-frame `mouse_hit_test` recomputation does not
  resurrect hover on the element under the stale position.
- src/elements/div.rs — `update_hover` notifies the element's view after
  invoking the `on_hover` listener, so `on_hover` callbacks that only mutate
  state still produce a redraw.
- src/window.rs — frame instrumentation used by the apps' FPS meters:
  `drain_present_deltas`, `take_frame_busy_ns`, `log_frame_diagnostic`,
  `note_present` (Windows), `AGENDA_UNTHROTTLED` / `AGENDA_FRAME_LOG` env
  switches.
- src/window.rs + src/platform.rs — frame pacing for the apps' meters and
  background throttling: `Window::set_continuous_present`,
  `Window::set_frame_pacing`, `PlatformWindow::{can_draw, set_frame_pacing,
  note_frame_activity}` (default no-ops). `Window::present` defers the draw
  when `can_draw` is false instead of overwriting a pending scene, and
  `request_animation_frame` / `dispatch_event` call `note_frame_activity`.
- src/window.rs — `PaintIndex` snapshots `window_control_hitboxes` and
  `reuse_paint` replays them, so cached views keep their native caption
  controls and titlebar drag regions across repaints
  (`test_cached_window_controls_survive_repaint`).
- Cargo.toml + build.rs — `windows-manifest` removed from default features
  and the embed made a no-op: the apps embed their own manifest (a superset
  of gpui.manifest.xml) and a second RT_MANIFEST id=1 fails to link.
- src/platform/test/window.rs — TestWindow gains `present_pending` /
  `draw_count` for the frame-instrumented draw path
  (`test_pending_present_defers_draw_but_not_callbacks`).

## From cortex manager-gpui vendor/gpui-pre-0.3.6-patched (KOS-142)

- src/platform/test/window.rs — `impl PlatformWindow for TestWindow` gains an
  `a11y_init` override that fires `callbacks.activation()` once, plus the
  `A11yCallbacks` import. Test windows create no platform adapter, so
  `PlatformWindow::a11y_init` was never invoked and
  `Window::debug_a11y_tree_json` always returned `None` in `#[gpui::test]`
  runs. The test platform only compiles under
  `cfg(any(test, feature = "test-support"))`, so production windows are
  untouched.

## Adapted for the 0.3.8 base

- 0.3.8 reworked the `on_request_frame` closure: `RequestFrameOptions` gained
  `signal_at` / `signal_source` (`PlatformFrameSignal`), the inactive/thermal
  cap became a `min_frame_interval: Option<(Duration, FrameRateLimit)>`
  pair, and the draw branch is `should_draw`. The `agenda_unthrottled`,
  `continuous_present`, busy-time and frame-log hooks were re-applied onto
  that structure; behavior is unchanged.
- 0.3.8 gates `debug_selector` recording on
  `all(debug_assertions, test-support)` (was `any(test, test-support)`).
  Workspaces that disable dep assertions in dev need
  `[profile.dev.package.gpui-pre] debug-assertions = true` (plus the same for
  `gpui-pre-macros`) for `cx.debug_bounds` to populate; the imago root
  `Cargo.toml` carries that override.
- 0.3.8's own new machinery is kept as-is: `PlatformFrameSignal` /
  `FrameRequestSource` plumbing and the `draw_in_progress` re-entrant-draw
  guard.

## Dropped (upstream has it)

Nothing: every 0.3.6 patch item is still missing upstream in 0.3.8. Checked
per item — `hit_test_window_control_position` absent from `platform.rs`,
`bounds_changed` still refreshes unconditionally, `dispatch_event` still
drops the `MouseExited` position, `update_hover` still does not notify,
`windows-manifest` is still a default feature, and `TestWindow` still has no
`a11y_init`.
