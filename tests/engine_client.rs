//! Integration tests for `engine` — the module stays under the source-size
//! budget; these tests only touch its public surface.
use mundus_gpui_kit::engine::*;
use mundus_gpui_kit::engine_error::ErrorKind;
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

fn engine_with_lock(dir: &std::path::Path, http_port: u16) -> Engine {
    std::fs::create_dir_all(dir).unwrap();
    let lock = json!({
        "format_version": 1,
        "api_version": {"major": 1},
        "http_port": http_port,
        "auth_token": "a".repeat(64),
    });
    std::fs::write(dir.join("engine.lock.json"), lock.to_string()).unwrap();
    Engine {
        data_dir: Some(dir.to_path_buf()),
    }
}

/// Drains headers and the `Content-Length` body. Answering after a single
/// `read` could close the socket with request bytes still unread, which
/// resets the connection (Windows) before the client sees the response.
fn read_request(stream: &mut std::net::TcpStream) {
    let mut request = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let Ok(n) = stream.read(&mut buf) else { return };
        if n == 0 {
            return;
        }
        request.extend_from_slice(&buf[..n]);
        let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..end]).to_ascii_lowercase();
        let body_len = headers
            .lines()
            .find_map(|l| l.strip_prefix("content-length:"))
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(0);
        if request.len() >= end + 4 + body_len {
            return;
        }
    }
}

/// One-shot HTTP stub: accepts a single request, answers `status` +
/// `body`. Returns the bound port for the lock file.
fn serve_once(status: &str, body: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let status = status.to_string();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        let response = format!(
            "{status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
    });
    port
}

/// Engine rejecting an operation with a non-2xx status still answers
/// `{"ok":false,"error":...}` — the client must surface that rejection,
/// not report the Engine as unreachable.
#[test]
fn rpc_decodes_engine_error_on_http_error_status() {
    let dir = std::env::temp_dir().join(format!("kgk-rpc-{}", std::process::id()));
    let port = serve_once(
        "HTTP/1.1 401 Unauthorized",
        r#"{"ok":false,"error":"bad token"}"#,
    );
    let engine = engine_with_lock(&dir, port);
    let error = engine.rpc("demo.op", json!({})).unwrap_err();
    let _ = std::fs::remove_dir_all(&dir);
    // The raw code stays in `detail` for the app log; `message()` is the
    // user-facing Russian text and must not contain it.
    assert_eq!(error.detail, "bad token");
    assert!(!error.message().contains("bad token"));
}

#[test]
fn status_decodes_engine_error_on_http_error_status() {
    let dir = std::env::temp_dir().join(format!("kgk-status-{}", std::process::id()));
    let port = serve_once(
        "HTTP/1.1 503 Service Unavailable",
        r#"{"ok":false,"error":"booting"}"#,
    );
    let engine = engine_with_lock(&dir, port);
    let error = engine.status("health").unwrap_err();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(error.kind, ErrorKind::Unavailable);
    assert_eq!(error.detail, "booting");
}

// --- data_dir / host_user_data candidate ordering ------------------------

/// Env is process-global — discovery tests that set vars must hold this
/// lock or they race each other.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Restores env vars on drop so neighbour tests see a clean process.
struct EnvRestore {
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl EnvRestore {
    fn capture(vars: &[&'static str]) -> Self {
        Self {
            saved: vars.iter().map(|v| (*v, std::env::var_os(v))).collect(),
        }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        for (key, value) in self.saved.drain(..) {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

/// All vars `data_dir_candidates`/`host_user_data` read — set every leg
/// to a throwaway dir regardless of host OS so tests stay hermetic.
fn env_restore() -> EnvRestore {
    EnvRestore::capture(&[
        "MUNDUS_DATA_DIR",
        "KOSMOS_DATA_DIR",
        "MUNDUS_AGENDA_EXECUTABLE",
        "KOSMOS_AGENDA_EXECUTABLE",
        "APPDATA",
        "HOME",
        "XDG_CONFIG_HOME",
    ])
}

fn set_env(key: &str, value: Option<&std::path::Path>) {
    match value {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
}

/// Point the data-dir env vars and every per-OS config base at throwaway
/// dirs — never let a test stat real user dirs.
fn redirect_env(mundus: Option<&Path>, kosmos: Option<&Path>, config: Option<&Path>) {
    set_env("MUNDUS_DATA_DIR", mundus);
    set_env("KOSMOS_DATA_DIR", kosmos);
    set_env("APPDATA", config);
    set_env("HOME", config);
    set_env("XDG_CONFIG_HOME", config);
}

/// `<config>/<name>` — on macOS the config base lives under
/// `~/Library/Application Support`, elsewhere it is the base itself.
fn config_child(base: &Path, name: &str) -> PathBuf {
    #[cfg(target_os = "macos")]
    let base = base.join("Library/Application Support");
    #[cfg(not(target_os = "macos"))]
    let base = base.to_path_buf();
    base.join(name)
}

fn tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mgk-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn write_lock(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("engine.lock.json"), "{}").unwrap();
}

/// Every candidate holds a lock → the MUNDUS_DATA_DIR leg wins.
#[test]
fn data_dir_prefers_mundus_env_lock() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let (m, k, c) = (tmp_dir("a-m"), tmp_dir("a-k"), tmp_dir("a-c"));
    write_lock(&m);
    write_lock(&k);
    write_lock(&config_child(&c, "Mundus"));
    write_lock(&config_child(&c, "Kosmos"));
    redirect_env(Some(&m), Some(&k), Some(&c));
    assert_eq!(data_dir().unwrap(), m);
    for d in [m, k, c] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// MUNDUS_DATA_DIR without a lock loses to the legacy env leg that has
/// one — the lock is what proves a running Engine, not the var order.
#[test]
fn data_dir_lockless_env_loses_to_legacy_env_lock() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let (m, k, c) = (tmp_dir("b-m"), tmp_dir("b-k"), tmp_dir("b-c"));
    write_lock(&k);
    write_lock(&config_child(&c, "Mundus"));
    redirect_env(Some(&m), Some(&k), Some(&c));
    assert_eq!(data_dir().unwrap(), k);
    for d in [m, k, c] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// No env vars: the Mundus config dir beats the legacy Kosmos config dir.
#[test]
fn data_dir_prefers_config_mundus_over_legacy() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let c = tmp_dir("c-c");
    write_lock(&config_child(&c, "Mundus"));
    write_lock(&config_child(&c, "Kosmos"));
    redirect_env(None, None, Some(&c));
    assert_eq!(data_dir().unwrap(), config_child(&c, "Mundus"));
    let _ = std::fs::remove_dir_all(c);
}

/// Deferred move: only the legacy Kosmos dir has a lock → it is used.
#[test]
fn data_dir_falls_back_to_legacy_config_lock() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let c = tmp_dir("d-c");
    write_lock(&config_child(&c, "Kosmos"));
    redirect_env(None, None, Some(&c));
    assert_eq!(data_dir().unwrap(), config_child(&c, "Kosmos"));
    let _ = std::fs::remove_dir_all(c);
}

/// No lock anywhere (Engine not started): the first existing dir wins so
/// sibling paths (logs/crashes) land where the data actually is.
#[test]
fn data_dir_without_lock_picks_first_existing_dir() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let c = tmp_dir("e-c");
    std::fs::create_dir_all(config_child(&c, "Kosmos")).unwrap();
    redirect_env(None, None, Some(&c));
    assert_eq!(data_dir().unwrap(), config_child(&c, "Kosmos"));
    let _ = std::fs::remove_dir_all(c);
}

/// Fresh machine: no env vars, nothing on disk → the Mundus default.
#[test]
fn data_dir_defaults_to_mundus_dir() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let c = tmp_dir("f-c");
    redirect_env(None, None, Some(&c));
    assert_eq!(data_dir().unwrap(), config_child(&c, "Mundus"));
}

/// An explicit MUNDUS_DATA_DIR with no lock still beats the config dirs
/// — the env var is a deliberate override, not a discovery hint.
#[test]
fn data_dir_env_override_without_lock() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let (m, c) = (tmp_dir("g-m"), tmp_dir("g-c"));
    redirect_env(Some(&m), None, Some(&c));
    assert_eq!(data_dir().unwrap(), m);
}

/// Nothing resolvable at all → the only remaining failure mode.
#[test]
fn data_dir_errors_when_no_candidate_resolves() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    redirect_env(None, None, None);
    let error = data_dir().unwrap_err();
    assert_eq!(error.kind, ErrorKind::NotRunning);
    assert_eq!(
        error.message(),
        "Engine не запущен. Запустите Mundus и обновите список."
    );
}

/// browser.json shared with the legacy Electron Manager: the Mundus dir
/// wins when both carry the flag; the legacy dir is read while it is the
/// only one that has it; a fresh install defaults to "Mundus Manager".
#[test]
fn host_user_data_prefers_mundus_with_legacy_fallback() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let c = tmp_dir("h-c");
    let mundus = config_child(&c, "Mundus Manager");
    let kosmos = config_child(&c, "Kosmos Manager");
    redirect_env(None, None, Some(&c));

    assert_eq!(host_user_data().unwrap(), mundus);

    std::fs::create_dir_all(&kosmos).unwrap();
    std::fs::write(kosmos.join("browser.json"), "{}").unwrap();
    assert_eq!(host_user_data().unwrap(), kosmos);

    std::fs::create_dir_all(&mundus).unwrap();
    std::fs::write(mundus.join("browser.json"), "{}").unwrap();
    assert_eq!(host_user_data().unwrap(), mundus);

    let _ = std::fs::remove_dir_all(c);
}

/// MUNDUS_AGENDA_EXECUTABLE overrides; the legacy var still works when
/// the new one is unset or points at a missing file.
#[test]
fn agenda_executable_env_override_order() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let dir = tmp_dir("i-bin");
    std::fs::create_dir_all(&dir).unwrap();
    let (new, old, gone) = (
        dir.join("mundus.exe"),
        dir.join("kosmos.exe"),
        dir.join("gone.exe"),
    );
    std::fs::write(&new, "x").unwrap();
    std::fs::write(&old, "x").unwrap();

    set_env("MUNDUS_AGENDA_EXECUTABLE", Some(&new));
    set_env("KOSMOS_AGENDA_EXECUTABLE", Some(&old));
    assert_eq!(agenda_executable().unwrap(), new);

    // Set-but-missing must not shadow the legacy var or the bundled path.
    set_env("MUNDUS_AGENDA_EXECUTABLE", Some(&gone));
    assert_eq!(agenda_executable().unwrap(), old);

    set_env("MUNDUS_AGENDA_EXECUTABLE", None);
    assert_eq!(agenda_executable().unwrap(), old);

    let _ = std::fs::remove_dir_all(dir);
}

/// An empty MUNDUS_DATA_DIR is not an override — discovery continues to
/// the legacy env leg.
#[test]
fn data_dir_ignores_empty_env_value() {
    let _hold = ENV_LOCK.lock().unwrap();
    let _env = env_restore();
    let (k, c) = (tmp_dir("j-k"), tmp_dir("j-c"));
    write_lock(&k);
    redirect_env(None, Some(&k), Some(&c));
    std::env::set_var("MUNDUS_DATA_DIR", "");
    assert_eq!(data_dir().unwrap(), k);
    for d in [k, c] {
        let _ = std::fs::remove_dir_all(d);
    }
}
