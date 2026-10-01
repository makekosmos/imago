//! Engine broadcast events over the Engine WebSocket (`ws_port` in
//! engine.lock.json). The runtime pushes events the HTTP client cannot
//! receive here — dictation hotkey triggers (`dictation_toggle_trigger`,
//! `dictation_ptt_trigger`), state/config/pending changes and local-model
//! download progress (`runtime/src/ws_server`, `runtime/src/dictation`).
//!
//! Handshake (mirrors `ws_server/handshake.rs` + the Electron ark client):
//! the first client frame is a text JSON `hello` carrying `apiVersion`,
//! the lock-file `auth_token` and this process PID (PID-binding check).
//! The server answers `{"kind":"hello_ok"}` — or `hello_error` with a code —
//! then streams `{"event": ...}` text frames to every connected client.
use serde_json::{json, Value};
use std::net::TcpStream;
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

use crate::engine::{Engine, CLIENT_CLASS};
use crate::engine_error::{EngineError, ErrorKind};

/// Blocking event stream on a dedicated worker thread (the socket read
/// blocks, so never poll it from the UI thread). `next_event` returns
/// `None` once the connection drops — the caller reconnects via
/// `Engine::subscribe` under its own backoff policy.
pub struct EventStream {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl Engine {
    /// Connect `ws://127.0.0.1:<ws_port>` and complete the `hello`
    /// handshake. `accept_async` on the server side accepts any request
    /// path — the lock-file port + token are the actual gate.
    pub fn subscribe(&self) -> Result<EventStream, EngineError> {
        let lock = self.lock()?;
        if lock.ws_port == 0 {
            return Err(EngineError::local(
                ErrorKind::NotCompatible,
                "engine.lock.json has no ws_port",
            ));
        }
        let (mut socket, _response) =
            tungstenite::connect(format!("ws://127.0.0.1:{}/", lock.ws_port)).map_err(|e| {
                EngineError::local(ErrorKind::Transport, format!("ws connect failed: {e}"))
            })?;
        let hello = json!({
            "kind": "hello",
            // Same contract version the HTTP client sends in
            // `X-Kosmos-Api-Version`; the server accepts any 1.x.y.
            "apiVersion": "1.0.0",
            "token": lock.auth_token,
            "pid": std::process::id(),
            "clientId": format!("{}-{}", CLIENT_CLASS, std::process::id()),
            "clientClass": CLIENT_CLASS,
            "clientVersion": env!("CARGO_PKG_VERSION"),
        });
        socket
            .send(Message::text(hello.to_string()))
            .map_err(|e| EngineError::local(ErrorKind::Transport, format!("hello send: {e}")))?;
        loop {
            match socket.read() {
                Ok(Message::Text(text)) => {
                    let value: Value = serde_json::from_str(&text).map_err(|e| {
                        EngineError::local(ErrorKind::Malformed, format!("hello not json: {e}"))
                    })?;
                    match value.get("kind").and_then(Value::as_str) {
                        Some("hello_ok") => return Ok(EventStream { socket }),
                        Some("hello_error") => {
                            let detail = value
                                .get("message")
                                .and_then(Value::as_str)
                                .unwrap_or("handshake rejected");
                            return Err(EngineError::engine(detail));
                        }
                        _ => {
                            return Err(EngineError::local(
                                ErrorKind::Malformed,
                                "unexpected handshake frame",
                            ))
                        }
                    }
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    // tungstenite already queued the Pong — flush it.
                    if socket.flush().is_err() {
                        return Err(EngineError::local(ErrorKind::Transport, "socket dropped"));
                    }
                }
                Ok(Message::Close(_)) => {
                    // `read` queued the close reply on the socket — flush so
                    // the peer sees a real close handshake (RFC 6455 §5.5.1)
                    // instead of a bare TCP FIN.
                    let _ = socket.flush();
                    return Err(EngineError::local(ErrorKind::Transport, "socket closed"));
                }
                Err(error) => {
                    return Err(EngineError::local(
                        ErrorKind::Transport,
                        format!("socket error: {error}"),
                    ));
                }
                Ok(_) => {}
            }
        }
    }
}

impl EventStream {
    /// Next broadcast `{"event": ...}` payload. `None` means the socket
    /// closed or errored — reconnect with `Engine::subscribe`.
    pub fn next_event(&mut self) -> Option<Value> {
        loop {
            match self.socket.read() {
                Ok(Message::Text(text)) => match serde_json::from_str(&text) {
                    Ok(value) => return Some(value),
                    // Malformed single frame — keep the stream alive.
                    Err(_) => continue,
                },
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    if self.socket.flush().is_err() {
                        return None;
                    }
                }
                Ok(Message::Close(_)) => {
                    // Same close-reply flush as the handshake path — the
                    // caller dropping the stream must not skip it.
                    let _ = self.socket.flush();
                    return None;
                }
                Err(_) => return None,
                Ok(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::Engine;

    /// Without engine.lock.json the subscription fails with the same
    /// "Engine not running" error the HTTP surface reports — no socket
    /// attempt, no panic.
    #[test]
    fn subscribe_without_lock_reports_engine_down() {
        let engine = Engine {
            data_dir: Some(std::env::temp_dir().join("mundus-gpui-kit-no-such-dir")),
        };
        let error = match engine.subscribe() {
            Err(error) => error,
            Ok(_) => panic!("subscribe must fail without a lock file"),
        };
        assert_eq!(
            error.kind,
            crate::engine_error::ErrorKind::NotRunning,
            "unexpected error: {error}"
        );
        assert!(
            error.message().contains("не запущен"),
            "unexpected message: {}",
            error.message()
        );
    }
}
