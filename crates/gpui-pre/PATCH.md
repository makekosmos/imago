# Local patch vs crates.io gpui-pre 0.3.6

Single patched gpui revision shared by the three GPUI apps (KOS-328):
memoria-gpui, agenda-gpui and cortex/manager-gpui all `[patch.crates-io]`
to this directory via one imago rev, replacing the three divergent vendored
copies (agenda vendor/gpui-pre-0.3.5-patched, manager
vendor/gpui-pre-0.3.6-patched, memoria unpinned crates.io).

Carries two patch sets, ported onto the 0.3.6 base:

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
- Cargo.toml + build.rs — `windows-manifest` removed from default features
  and the embed made a no-op: the apps embed their own manifest (a superset
  of gpui.manifest.xml) and a second RT_MANIFEST id=1 fails to link.
- src/platform/test/window.rs — TestWindow gains `present_pending` /
  `draw_count` for the frame-instrumented draw path.

## From cortex manager-gpui vendor/gpui-pre-0.3.6-patched (KOS-142)

- src/platform/test/window.rs — `impl PlatformWindow for TestWindow` gains an
  `a11y_init` override that fires `callbacks.activation()` once, plus the
  `A11yCallbacks` import. Test windows create no platform adapter, so
  `PlatformWindow::a11y_init` was never invoked and
  `Window::debug_a11y_tree_json` always returned `None` in `#[gpui::test]`
  runs. The test platform only compiles under
  `cfg(any(test, feature = "test-support"))`, so production windows are
  untouched.
