//! Typed Engine error classes shared by the GPUI apps (manager-gpui,
//! memoria-gpui, agenda-gpui). `kind` selects the user-facing Russian text;
//! `detail` keeps the raw `error` code the Engine sent on the wire (or a
//! local failure tag) — it belongs in the app log, never on screen.

/// Why an Engine call failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError {
    pub kind: ErrorKind,
    pub detail: String,
}

impl EngineError {
    /// A `{ok:false, error}` reply (or an HTTP error body) from Engine.
    pub fn engine(code: &str) -> Self {
        Self {
            kind: ErrorKind::from_engine_code(code),
            detail: code.to_owned(),
        }
    }

    /// A failure produced on this side of the wire — lock file unreadable,
    /// socket refused, malformed reply.
    pub fn local(kind: ErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    /// User-facing Russian text for the error class. Written for a
    /// non-programmer: it never names the wire code.
    pub fn message(&self) -> String {
        match self.kind {
            ErrorKind::NotRunning => {
                "Engine не запущен. Запустите Mundus и обновите список.".into()
            }
            ErrorKind::NotCompatible => {
                "Состояние Engine несовместимо. Обновите Mundus.".into()
            }
            ErrorKind::Transport => {
                "Нет подтверждения от Engine. Обновите список и проверьте, что изменение сохранилось."
                    .into()
            }
            ErrorKind::InvalidRequest => {
                "Engine отклонил операцию: данные не прошли проверку. Обновите Mundus и сообщите о проблеме."
                    .into()
            }
            ErrorKind::Forbidden => {
                "Engine запретил операцию для этого приложения. Обновите Mundus.".into()
            }
            ErrorKind::Conflict => {
                "Данные изменились в другом приложении. Список обновлён — повторите изменение."
                    .into()
            }
            ErrorKind::NotFound => "Запись не найдена в Engine. Обновите список.".into(),
            ErrorKind::Timeout => "Engine не ответил вовремя. Повторите попытку.".into(),
            ErrorKind::Unavailable => "Engine временно недоступен. Повторите попытку.".into(),
        }
    }
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The display form is for logs: kind + the raw engine code/detail.
        write!(f, "{:?}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for EngineError {}

/// Engine rejection classes — the `error` codes on `/v1/rpc` replies
/// (`invalid-request`, `forbidden`, `conflict`, `not-found`, `timeout`,
/// `unavailable`; see cortex `public_app_error`) — plus the local failures
/// that can precede a reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// engine.lock.json is missing/unreadable — Engine is not running.
    NotRunning,
    /// The lock file exists but its format/api version is incompatible.
    NotCompatible,
    /// The request left the app but no confirmed reply came back.
    Transport,
    InvalidRequest,
    Forbidden,
    Conflict,
    NotFound,
    Timeout,
    Unavailable,
}

impl ErrorKind {
    /// The wire carries the raw code (`canonical_ingress:invalid_request:*`,
    /// `object_conflict:*`, plain class names); classify by its stable tokens.
    pub fn from_engine_code(code: &str) -> Self {
        let code = code.strip_prefix("package worker: ").unwrap_or(code);
        if code.contains("conflict") {
            Self::Conflict
        } else if code.contains("invalid-request") || code.contains("invalid_request") {
            Self::InvalidRequest
        } else if code.contains("forbidden") {
            Self::Forbidden
        } else if code.contains("not-found")
            || code.contains("not_found")
            || code.contains("not found")
        {
            Self::NotFound
        } else if code.contains("timeout") || code.contains("timed out") {
            Self::Timeout
        } else {
            Self::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every public Engine class maps to its kind — regression net for the
    /// token classification in `from_engine_code`.
    #[test]
    fn classifies_every_public_engine_error_class() {
        let cases = [
            ("invalid-request", ErrorKind::InvalidRequest),
            (
                "canonical_ingress:invalid_request:canonical_field:/recurrence/dayOfMonth",
                ErrorKind::InvalidRequest,
            ),
            ("forbidden", ErrorKind::Forbidden),
            ("package worker: forbidden", ErrorKind::Forbidden),
            ("conflict", ErrorKind::Conflict),
            ("object_conflict:stale_write", ErrorKind::Conflict),
            ("not-found", ErrorKind::NotFound),
            ("timeout", ErrorKind::Timeout),
            ("unavailable", ErrorKind::Unavailable),
        ];
        for (code, kind) in cases {
            assert_eq!(ErrorKind::from_engine_code(code), kind, "code: {code}");
            assert_eq!(EngineError::engine(code).kind, kind, "code: {code}");
        }
    }

    /// Unknown codes must degrade to Unavailable, not panic or leak a class
    /// that has no user-facing meaning.
    #[test]
    fn unknown_codes_become_unavailable() {
        assert_eq!(
            ErrorKind::from_engine_code("C:\\private\\path\\leak"),
            ErrorKind::Unavailable
        );
        assert_eq!(
            ErrorKind::from_engine_code("something else entirely"),
            ErrorKind::Unavailable
        );
    }

    /// Every kind produces Russian user text — and none of it leaks the raw
    /// wire code.
    #[test]
    fn every_kind_has_user_text_without_wire_detail() {
        for kind in [
            ErrorKind::NotRunning,
            ErrorKind::NotCompatible,
            ErrorKind::Transport,
            ErrorKind::InvalidRequest,
            ErrorKind::Forbidden,
            ErrorKind::Conflict,
            ErrorKind::NotFound,
            ErrorKind::Timeout,
            ErrorKind::Unavailable,
        ] {
            let error = EngineError {
                kind,
                detail: "canonical_ingress:invalid_request:canonical_field:/x".into(),
            };
            let message = error.message();
            assert!(!message.is_empty(), "{kind:?} has no user text");
            assert!(
                !message.contains("canonical_ingress"),
                "{kind:?} leaks the wire code"
            );
        }
    }

    /// Display is the log form: it keeps the raw detail for support.
    #[test]
    fn display_carries_kind_and_raw_detail() {
        let error = EngineError::engine("object_conflict:x");
        let rendered = error.to_string();
        assert!(rendered.contains("Conflict"));
        assert!(rendered.contains("object_conflict:x"));
    }
}
