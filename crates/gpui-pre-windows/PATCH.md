# Local patch vs crates.io gpui-pre-windows 0.3.8

Same fix that used to live in vendor/gpui-0.2.2-patched; the backend moved
from the monolithic gpui crate to `gpui-pre-windows`. Rebased from the 0.3.6
vendor onto the 0.3.8 base.

`WM_NCHITTEST` resolves `window_control_area` hitboxes at the cursor position
carried by the message (`lparam` -> `ScreenToClient` -> logical px). The
backend stores that position while invoking the upstream-compatible no-arg
`PlatformWindow::on_hit_test_window_control` callback, and exposes it through
`PlatformWindow::hit_test_window_control_position`. Upstream instead asks
`Window` to match `mouse_hit_test`, which is only refreshed on dispatched
mouse events — one event stale — so caption/drag areas flapped between client
and non-client hit codes and hover stopped tracking the cursor.

Files changed:
- src/events.rs — `handle_hit_test_msg` computes the client point first,
  converts to logical pixels, and publishes it for the duration of the
  callback. Upstream ordering kept: `is_movable`/`is_resizable`/
  `is_minimizable` gates on control areas and the top-edge `HTTOP*` resize
  zone checked before `drag_area`.
- src/window.rs — adds `Callbacks::hit_test_window_control_position`,
  implements `PlatformWindow::hit_test_window_control_position`, keeps
  the upstream `on_hit_test_window_control` callback signature, and adds
  the `size_move_loop`/`size_move_resized` state cells used by events.rs.
- src/events.rs — `handle_mouse_leave_msg` now also dispatches
  `PlatformInput::MouseExited` (with the live cursor position) through the
  input callback. Upstream only toggles `hovered_status_change`, so GPUI
  never delivered `hovered=false` when the cursor left the client area and
  the last hovered element stayed highlighted forever.
- src/events.rs — `draw_window` skips painting inside a modal
  `WM_ENTERSIZEMOVE` loop until a `WM_SIZE` arrives. A pure position change
  leaves the scene identical (DWM translates the composited surface), so
  every timer/vsync-driven repaint just stalled the modal message pump and
  made titlebar dragging judder. `WM_ENTERSIZEMOVE`/`WM_EXITSIZEMOVE` are
  now tracked via `WindowsWindowState::size_move_loop`/`size_move_resized`
  (`WM_ENTERMENULOOP` keeps the timer but does not suppress painting), and
  `handle_size_msg` marks real resizes so live-resize still paints.

Mirrored in `crates/gpui-pre` (default trait method in platform.rs and the
`Window` closure in window.rs). The trait keeps the upstream callback
signature and only adds a provided method, so the macOS/Linux/web backend
crates continue to compile without local mirrors.

### set_background_appearance: reset the Mica system backdrop on leave

`set_background_appearance` set `DWMWA_SYSTEMBACKDROP_TYPE` for
Mica/MicaAlt but never cleared it, so switching back to Opaque/Blurred
left Mica rendering behind the window. The non-Mica arms now call
`dwm_set_window_composition_attribute(hwnd, DWMSBT_NONE)` before
restoring the legacy accent policy.

Files changed:
- src/window.rs — `Opaque`, `Transparent`, and `Blurred` arms of
  `set_background_appearance` reset the system backdrop type first.

### set_background_appearance: actually show the system backdrop

Upstream set `DWMWA_SYSTEMBACKDROP_TYPE` but never extended the glass
frame, so DWM had no frame region to draw the material into and Mica
rendered nothing. `dwm_set_window_composition_attribute` now calls
`DwmExtendFrameIntoClientArea` with `-1` margins while a backdrop is
active (and restores `0` when cleared), and returns success so the
`Blurred` arm can prefer the documented `DWMSBT_TRANSIENTWINDOW` acrylic
backdrop, falling back to the undocumented accent policy only on
pre-22621 builds where it silently did nothing anyway.

Files changed:
- src/window.rs — backdrop arms use `DWMSBT_*` constants, the helper
  extends/restores the glass frame and returns `bool`.

### set_window_appearance: app-level light/dark override

The trait already declares `set_window_appearance` (default no-op, used by
the macOS backend); the Windows backend ignored it, so window appearance
always followed the OS. Apps that offer their own light/dark theme
(Zed-style) need the window — and especially the DWM-drawn Mica/MicaAlt
material tint — to follow the app theme instead.

Files changed:
- src/platform.rs — `WindowsPlatform` gains a shared
  `appearance_override: Arc<Cell<Option<WindowAppearance>>>`, passes it to
  every new window through `WindowCreationInfo`, reports it from
  `window_appearance`, and implements `set_window_appearance`: stores the
  override, applies `configure_dwm_dark_mode` (which also tints the Mica
  backdrop) to every tracked hwnd, and fires `appearance_changed` so
  windows repaint.
- src/window.rs — `WindowsWindowState` stores the shared
  `appearance_override`; window creation resolves the effective
  appearance through it.
- src/events.rs — `ImmersiveColorSet` handling applies the override on
  top of the fresh system appearance, so OS theme flips do not clobber
  the app-pinned mode.

### Frame pacing and present scheduling (agenda-gpui vendor)

The apps' FPS meters and background throttling. The UI thread encodes
scenes; the VSyncProvider thread issues `Present` after the compositor
tick and wakes for input/animation activity. `AGENDA_FRAME_LOG`,
`AGENDA_UNTHROTTLED` and `AGENDA_VSYNC_SOURCE=dwm` are the A/B switches.

Files changed:
- src/directx_renderer.rs — `PresentSlot` registry: the UI thread bumps
  `submitted` after encoding, the vsync thread presents it
  (`present_submitted_frames`). `can_draw` gates re-encoding while a
  present is pending. `next_frame_wait`/`take_due_frame_windows` compute
  each window's `FramePacing` cadence (display clock or 60 Hz active,
  1 Hz idle/background).
- src/vsync.rs — `VSyncProvider` resolves
  `DCompositionWaitForCompositorClock` dynamically (Windows 11+, falls
  back to `DwmFlush`; `AGENDA_VSYNC_SOURCE=dwm` forces the old source).
  `FramePacing`, `register_frame_thread`/`wake_frame_thread`, `qpc_now`.
- src/platform.rs — vsync thread: MMCSS `AvSetMmThreadCharacteristicsW`
  + `THREAD_PRIORITY_TIME_CRITICAL`; posts `WM_GPUI_VSYNC` (with a QPC
  timestamp) to due windows instead of `RedrawWindow`/`RDW_INVALIDATE`,
  and presents submitted frames after each compositor tick.
  `Platform::run` sets MMCSS + `timeBeginPeriod(1)` on the UI thread.
- src/events.rs — `WM_GPUI_VSYNC` handler drains queued duplicates and
  calls `draw_window`; input messages call `note_frame_activity`.
- src/window.rs — `can_draw`/`set_frame_pacing`/`note_frame_activity`
  forward to the renderer; `PresentSlot` registered per window.
- src/directx_devices.rs — `ID3D11Multithread::SetMultithreadProtected`:
  `Present` runs on the vsync thread while the UI thread issues
  immediate-context calls; without runtime-level serialization the
  NVIDIA driver crashes (nvwgf2umx 0xc0000005).
- Cargo.toml — `Win32_Media_Multimedia` feature for `timeBeginPeriod`.

## Adapted for the 0.3.8 base

- 0.3.8 added headless/windowed switching: the vsync thread is created by
  `begin_vsync_thread` (called from `attach_gpu`) and stopped via
  `vsync_stop`. The pacing loop keeps the `stop` flag check and the
  `all_windows` weak-upgrade exit on top of its own `strong_count == 0`
  exit.
- 0.3.8 added `PlatformFrameSignal`/`FrameRequestSource`: windows are
  tracked as `TrackedWindow { handle, frame_signal }` and
  `wait_for_vsync` returns a `FrameRequestSource`. The pacing loop
  records the captured signal into each *due* window's `frame_signal`
  (keeping upstream's `IsWindowVisible`/`IsIconic` gate) before posting
  `WM_GPUI_VSYNC`, so `RequestFrameOptions.signal_at/signal_source` stay
  populated. The compositor-clock adaptation keeps the new return type:
  `LocalSchedule` after a failed or (DwmFlush-only) too-short wait,
  `NativeCallback` otherwise; a short *successful* compositor-clock wait
  counts as a real tick.
- 0.3.8 added the `draw_coordinator` re-entrant-draw guard and
  `force_render_pending` in `draw_window`; the size-move paint skip is
  applied on top of it.
- `windows` crate bump: `ID3D11Multithread::SetMultithreadProtected` now
  returns `BOOL`, result explicitly discarded.
- Upstream's own `timeBeginPeriod` in `dispatcher.rs`
  (`increase_timer_resolution`) is orthogonal — scoped to executor work —
  and kept; the platform-level call covers the message pump.

## Dropped (upstream has it)

Nothing: every 0.3.6 patch item is still missing upstream in 0.3.8.
`handle_mouse_leave_msg` still only fires `hovered_status_change`,
`handle_hit_test_msg` still hit-tests the stale `mouse_hit_test` snapshot
through `Window`, `set_background_appearance` still never clears
`DWMWA_SYSTEMBACKDROP_TYPE` nor extends the glass frame, and there is no
`set_window_appearance` implementation.
