//! Engine transport: discovery via engine.lock.json, POST /v1/rpc and the
//! GET /v1/status surfaces the Vue Manager reaches through Electron IPC.
//! Engine is the only owner of state — this client never opens databases.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{path::PathBuf, time::Duration};

pub(crate) const CLIENT_CLASS: &str = "manager-gpui";

#[derive(Deserialize)]
pub(crate) struct Lock {
    format_version: u32,
    api_version: Version,
    pub http_port: u16,
    /// Broadcast/event socket the runtime exposes next to the HTTP API
    /// (`ws_server`). Absent in pre-WS locks — serde default keeps them
    /// readable; `subscribe` rejects `0`.
    #[serde(default)]
    pub ws_port: u16,
    pub auth_token: String,
}

#[derive(Deserialize)]
struct Version {
    major: u32,
}

#[derive(Clone, Default)]
pub struct Engine {
    pub data_dir: Option<PathBuf>,
}

impl Engine {
    /// POST /v1/rpc. Params merge with `operation`/`_req_id` like the
    /// Electron engine-client does.
    pub fn rpc(&self, operation: &str, mut params: Value) -> Result<Value, String> {
        let lock = self.lock()?;
        if !params.is_object() {
            return Err("Некорректный запрос Engine".into());
        }
        params["operation"] = json!(operation);
        params["_req_id"] = json!(uuid::Uuid::new_v4().to_string());
        // `X-Kosmos-*` is the Engine wire protocol — running Engines accept
        // only these header names; the brand rename does not retag the RPC.
        let response = or_status_body(
            agent()
                .post(&format!("http://127.0.0.1:{}/v1/rpc", lock.http_port))
                .set("Authorization", &format!("Bearer {}", lock.auth_token))
                .set("X-Kosmos-Api-Version", "1.0.0")
                .set("X-Kosmos-Client-Class", CLIENT_CLASS)
                .set("X-Kosmos-Client-Version", env!("CARGO_PKG_VERSION"))
                .set("X-Kosmos-Client-Pid", &std::process::id().to_string())
                .send_json(params),
        )
        .ok_or_else(|| {
            "Нет подтверждения от Engine. Обновите список перед повтором.".to_string()
        })?;
        decode(response)
    }

    /// GET /v1/health or /v1/info — plain status surfaces the Manager header
    /// uses for Engine reachability.
    pub fn status(&self, path: &str) -> Result<Value, String> {
        let lock = self.lock()?;
        let response = or_status_body(
            agent()
                .get(&format!("http://127.0.0.1:{}/v1/{path}", lock.http_port))
                .set("Authorization", &format!("Bearer {}", lock.auth_token))
                .set("X-Kosmos-Api-Version", "1.0.0")
                .set("X-Kosmos-Client-Class", CLIENT_CLASS)
                .set("X-Kosmos-Client-Version", env!("CARGO_PKG_VERSION"))
                .set("X-Kosmos-Client-Pid", &std::process::id().to_string())
                .call(),
        )
        .ok_or_else(|| "Нет подтверждения от Engine. Обновите список.".to_string())?;
        // Status endpoints answer `{"ok":true,...}` without a `data` envelope.
        let value: Value = response
            .into_json()
            .map_err(|_| "Некорректный ответ Engine")?;
        if value["ok"] != true {
            return Err("Engine отклонил запрос состояния".into());
        }
        Ok(value)
    }

    /// Engine discovery read on every call: Engine may have restarted.
    pub(crate) fn lock(&self) -> Result<Lock, String> {
        let directory = self.data_dir.clone().map(Ok).unwrap_or_else(data_dir)?;
        let bytes = std::fs::read(directory.join("engine.lock.json"))
            .map_err(|_| "Engine не запущен. Запустите Mundus и обновите список.".to_string())?;
        let lock: Lock =
            serde_json::from_slice(&bytes).map_err(|_| "Некорректный файл состояния Engine")?;
        if lock.format_version != 1
            || lock.api_version.major != 1
            || lock.http_port == 0
            || lock.auth_token.len() != 64
            || !lock.auth_token.bytes().all(|v| v.is_ascii_hexdigit())
        {
            return Err("Несовместимое состояние Engine. Обновите Mundus.".into());
        }
        Ok(lock)
    }
}

/// A non-2xx status still carries Engine's `{"ok":false,"error":...}` body —
/// keep the response so `decode` surfaces the real rejection instead of
/// reporting the Engine as unreachable. `None` is a transport-level failure.
fn or_status_body(result: Result<ureq::Response, ureq::Error>) -> Option<ureq::Response> {
    match result {
        Ok(response) | Err(ureq::Error::Status(_, response)) => Some(response),
        Err(_) => None,
    }
}

fn decode(response: ureq::Response) -> Result<Value, String> {
    let value: Value = response
        .into_json()
        .map_err(|_| "Некорректный ответ Engine")?;
    if value["ok"] != true {
        let detail = value
            .get("error")
            .map(|e| match e {
                Value::String(s) => s.clone(),
                other => other
                    .get("message")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| other.to_string()),
            })
            .unwrap_or_else(|| "неизвестная ошибка".into());
        return Err(format!("Engine отклонил операцию: {detail}"));
    }
    Ok(value.get("data").cloned().unwrap_or(Value::Null))
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(15))
        .redirects(0)
        .build()
}

/// Per-OS config base the Engine data dir hangs off (`%APPDATA%`,
/// `~/Library/Application Support`, `$XDG_CONFIG_HOME`/`~/.config`).
fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base =
        std::env::var_os("HOME").map(|v| PathBuf::from(v).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|v| PathBuf::from(v).join(".config")));
    base
}

fn env_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Mundus data dir candidates in discovery order. The legacy `KOSMOS_*` env
/// and `Kosmos` dirs stay reachable while installs move over — an app can
/// still meet a pre-Mundus Engine or a deferred data move.
fn data_dir_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(dir) = env_dir("MUNDUS_DATA_DIR") {
        candidates.push(dir);
    }
    // MIGRATION(KOS-267): remove after 2026-11-01
    if let Some(dir) = env_dir("KOSMOS_DATA_DIR") {
        candidates.push(dir);
    }
    if let Some(base) = config_dir() {
        candidates.push(base.join("Mundus"));
        // MIGRATION(KOS-267): remove after 2026-11-01
        candidates.push(base.join("Kosmos"));
    }
    candidates
}

/// First candidate holding `marker` wins; without one the first existing
/// dir, then the first candidate — the answer stays deterministic when the
/// Engine has not written its lock yet (fresh install, Engine not started).
fn pick_dir(candidates: &[PathBuf], marker: &str) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|d| d.join(marker).is_file())
        .or_else(|| candidates.iter().find(|d| d.is_dir()))
        .or_else(|| candidates.first())
        .cloned()
}

/// Engine data dir: the first candidate that already holds
/// `engine.lock.json` wins — a running Engine decides the location, env vars
/// only point discovery. Callers needing live discovery re-invoke this per
/// use (the Engine may start after the app, or move dirs on upgrade).
pub fn data_dir() -> Result<PathBuf, String> {
    pick_dir(&data_dir_candidates(), "engine.lock.json")
        .ok_or("Не найдена папка данных Mundus".into())
}

/// userData dir owning the browser.json persistence flag. The legacy
/// Electron Manager build wrote it under "Kosmos Manager"; the flag moves to
/// "Mundus Manager" and the legacy dir stays as the fallback while it still
/// exists on disk.
pub fn host_user_data() -> Option<PathBuf> {
    let base = config_dir()?;
    let candidates = [
        base.join("Mundus Manager"),
        // MIGRATION(KOS-267): remove after 2026-11-01
        base.join("Kosmos Manager"),
    ];
    pick_dir(&candidates, "browser.json")
}

/// Packaged Agenda GPUI lives next to this exe as
/// `resources/components/agenda/Agenda.exe` (KOS-137).
/// `MUNDUS_AGENDA_EXECUTABLE` overrides for dev/local runs; the Kosmos names
/// are legacy fallbacks while pre-Mundus installs still exist.
pub fn agenda_executable() -> Option<PathBuf> {
    // MIGRATION(KOS-267): remove KOSMOS_AGENDA_EXECUTABLE after 2026-11-01
    for var in ["MUNDUS_AGENDA_EXECUTABLE", "KOSMOS_AGENDA_EXECUTABLE"] {
        // A set-but-missing override must not mask the later legs — same
        // "first candidate that resolves wins" rule as data_dir discovery.
        if let Some(path) = env_dir(var).filter(|p| p.is_file()) {
            return Some(path);
        }
    }
    let exe = std::env::current_exe().ok()?;
    let components = exe.parent()?.parent()?;
    // MIGRATION(KOS-267): remove "Kosmos Agenda.exe" after 2026-11-01
    ["Agenda.exe", "Kosmos Agenda.exe"]
        .iter()
        .map(|name| components.join("agenda").join(name))
        .find(|candidate| candidate.is_file())
}

/// Launch the sibling Agenda component; the child inherits this process env
/// (the shell sets MUNDUS_DATA_DIR at spawn). `data_dir` re-pins the same
/// Engine lock when Manager itself was started directly.
pub fn open_agenda(data_dir: Option<&std::path::Path>) -> Result<(), String> {
    let exe = agenda_executable().ok_or("Agenda не входит в эту сборку Mundus.")?;
    let mut command = std::process::Command::new(exe);
    if let Some(dir) = data_dir {
        command.env("MUNDUS_DATA_DIR", dir);
        // Pre-Mundus children still read KOSMOS_DATA_DIR.
        // MIGRATION(KOS-267): remove after 2026-11-01
        command.env("KOSMOS_DATA_DIR", dir);
    }
    command
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Не удалось запустить Agenda: {e}"))
}

/// Open a file/folder with the OS handler (Electron shell.openPath parity).
pub fn open_path(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let cmd = "explorer";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let cmd = "xdg-open";
    std::process::Command::new(cmd)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Не удалось открыть путь: {e}"))
}

/// Open an https/http URL in the system browser (shell.openExternal parity).
/// URL schemes are case-insensitive (RFC 3986 §3.1) — `HTTPS://…` is valid.
pub fn open_url(url: &str) -> Result<(), String> {
    let http = url
        .get(..8)
        .is_some_and(|p| p.eq_ignore_ascii_case("https://"))
        || url
            .get(..7)
            .is_some_and(|p| p.eq_ignore_ascii_case("http://"));
    if !http {
        return Err("Недопустимый URL".into());
    }
    open_path(std::path::Path::new(url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::Path;

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
        assert_eq!(error, "Engine отклонил операцию: bad token");
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
        assert_eq!(error, "Engine отклонил запрос состояния");
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
        assert_eq!(data_dir().unwrap_err(), "Не найдена папка данных Mundus");
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
}
