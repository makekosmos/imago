//! Engine transport: discovery via engine.lock.json, POST /v1/rpc and the
//! GET /v1/status surfaces the Vue Manager reaches through Electron IPC.
//! Engine is the only owner of state — this client never opens databases.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{path::PathBuf, time::Duration};

use crate::engine_error::{EngineError, ErrorKind};

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
    pub fn rpc(&self, operation: &str, mut params: Value) -> Result<Value, EngineError> {
        let lock = self.lock()?;
        if !params.is_object() {
            return Err(EngineError::local(
                ErrorKind::InvalidRequest,
                "local: params not an object",
            ));
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
        .ok_or_else(|| EngineError::local(ErrorKind::Transport, "no response from Engine"))?;
        decode(response)
    }

    /// GET /v1/health or /v1/info — plain status surfaces the Manager header
    /// uses for Engine reachability.
    pub fn status(&self, path: &str) -> Result<Value, EngineError> {
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
        .ok_or_else(|| EngineError::local(ErrorKind::Transport, "no response from Engine"))?;
        // Status endpoints answer `{"ok":true,...}` without a `data` envelope.
        let value: Value = response.into_json().map_err(|e| {
            EngineError::local(ErrorKind::Malformed, format!("response not json: {e}"))
        })?;
        if value["ok"] != true {
            return Err(reply_error(&value));
        }
        Ok(value)
    }

    /// Engine discovery read on every call: Engine may have restarted.
    pub(crate) fn lock(&self) -> Result<Lock, EngineError> {
        let directory = self.data_dir.clone().map(Ok).unwrap_or_else(data_dir)?;
        let bytes = std::fs::read(directory.join("engine.lock.json")).map_err(|e| {
            EngineError::local(
                ErrorKind::NotRunning,
                format!("engine.lock.json unreadable: {e}"),
            )
        })?;
        let lock: Lock = serde_json::from_slice(&bytes).map_err(|e| {
            EngineError::local(
                ErrorKind::NotCompatible,
                format!("engine.lock.json malformed: {e}"),
            )
        })?;
        if lock.format_version != 1
            || lock.api_version.major != 1
            || lock.http_port == 0
            || lock.auth_token.len() != 64
            || !lock.auth_token.bytes().all(|v| v.is_ascii_hexdigit())
        {
            return Err(EngineError::local(
                ErrorKind::NotCompatible,
                "engine.lock.json: incompatible format",
            ));
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

fn decode(response: ureq::Response) -> Result<Value, EngineError> {
    let value: Value = response
        .into_json()
        .map_err(|e| EngineError::local(ErrorKind::Malformed, format!("response not json: {e}")))?;
    if value["ok"] != true {
        return Err(reply_error(&value));
    }
    Ok(value.get("data").cloned().unwrap_or(Value::Null))
}

/// The rejection a `{ok:false}` body carries. `error` present → classify the
/// wire code (`message()` renders the Russian text, `detail` stays in the
/// app log). Absent/null → the reply shape itself is off-spec: Malformed,
/// not a guessed class like "unavailable" that lies about reachability.
fn reply_error(value: &Value) -> EngineError {
    match wire_error(value) {
        Some(code) => EngineError::engine(&code),
        None => EngineError::local(ErrorKind::Malformed, "{ok:false} reply with no error code"),
    }
}

/// The `error` field of a `{ok:false}` reply: a plain string code, or an
/// object carrying `message` (both shapes appear on the wire).
fn wire_error(value: &Value) -> Option<String> {
    let error = value.get("error")?;
    if error.is_null() {
        return None;
    }
    Some(match error {
        Value::String(s) => s.clone(),
        other => other
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| other.to_string()),
    })
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
    // Empty env vars are unset, like in `env_dir` — a "" base would turn
    // into cwd-relative candidates (`Mundus`, `.config/Mundus`), pointing
    // discovery at whatever directory the app was launched from.
    #[cfg(target_os = "windows")]
    let base = env_dir("APPDATA");
    #[cfg(target_os = "macos")]
    let base = env_dir("HOME").map(|v| v.join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = env_dir("XDG_CONFIG_HOME").or_else(|| env_dir("HOME").map(|v| v.join(".config")));
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
pub fn data_dir() -> Result<PathBuf, EngineError> {
    pick_dir(&data_dir_candidates(), "engine.lock.json")
        .ok_or_else(|| EngineError::local(ErrorKind::NotRunning, "no data dir candidate resolves"))
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
    let rest = if url
        .get(..8)
        .is_some_and(|p| p.eq_ignore_ascii_case("https://"))
    {
        url.get(8..)
    } else if url
        .get(..7)
        .is_some_and(|p| p.eq_ignore_ascii_case("http://"))
    {
        url.get(7..)
    } else {
        None
    };
    // A bare scheme ("https://") has no authority to open — reject it like
    // the non-http schemes instead of handing the OS handler garbage.
    if rest.is_none_or(|r| r.trim().is_empty()) {
        return Err("Недопустимый URL".into());
    }
    open_path(std::path::Path::new(url))
}
