use serde::Serialize;

/// Frontend-serializable error type
#[derive(Debug, Serialize, Clone)]
pub struct FrontendError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl std::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

// -- IMAP Errors --
#[derive(Debug)]
pub enum ImapError {
    Connection(String),
    Auth(String),
    Fetch(String),
    Folder(String),
    Timeout,
}

impl From<ImapError> for FrontendError {
    fn from(e: ImapError) -> Self {
        match e {
            ImapError::Connection(msg) => FrontendError {
                code: "IMAP_CONNECTION_FAILED".into(),
                message: msg,
                retryable: true,
            },
            ImapError::Auth(msg) => FrontendError {
                code: "IMAP_AUTH_FAILED".into(),
                message: msg,
                retryable: false,
            },
            ImapError::Fetch(msg) => FrontendError {
                code: "IMAP_FETCH_FAILED".into(),
                message: msg,
                retryable: true,
            },
            ImapError::Folder(msg) => FrontendError {
                code: "IMAP_FOLDER_ERROR".into(),
                message: msg,
                retryable: false,
            },
            ImapError::Timeout => FrontendError {
                code: "IMAP_TIMEOUT".into(),
                message: "IMAP operation timed out".into(),
                retryable: true,
            },
        }
    }
}

// -- AI Errors --
#[derive(Debug)]
pub enum AiError {
    ApiRequest(String),
    InvalidResponse(String),
    BudgetExceeded,
    ProviderUnavailable(String),
}

impl From<AiError> for FrontendError {
    fn from(e: AiError) -> Self {
        match e {
            AiError::ApiRequest(msg) => FrontendError {
                code: "AI_REQUEST_FAILED".into(),
                message: msg,
                retryable: true,
            },
            AiError::InvalidResponse(msg) => FrontendError {
                code: "AI_INVALID_RESPONSE".into(),
                message: msg,
                retryable: true,
            },
            AiError::BudgetExceeded => FrontendError {
                code: "AI_BUDGET_EXCEEDED".into(),
                message: "Monthly AI budget limit reached".into(),
                retryable: false,
            },
            AiError::ProviderUnavailable(msg) => FrontendError {
                code: "AI_PROVIDER_UNAVAILABLE".into(),
                message: msg,
                retryable: true,
            },
        }
    }
}

// -- Auth Errors --
#[derive(Debug)]
pub enum AuthError {
    TokenExpired,
    RefreshFailed(String),
    KeychainAccess(String),
    InvalidCredentials,
}

impl From<AuthError> for FrontendError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::TokenExpired => FrontendError {
                code: "AUTH_TOKEN_EXPIRED".into(),
                message: "Authentication token expired".into(),
                retryable: true,
            },
            AuthError::RefreshFailed(msg) => FrontendError {
                code: "AUTH_REFRESH_FAILED".into(),
                message: msg,
                retryable: false,
            },
            AuthError::KeychainAccess(msg) => FrontendError {
                code: "AUTH_KEYCHAIN_ERROR".into(),
                message: msg,
                retryable: false,
            },
            AuthError::InvalidCredentials => FrontendError {
                code: "AUTH_INVALID_CREDENTIALS".into(),
                message: "Invalid credentials".into(),
                retryable: false,
            },
        }
    }
}

// -- Calendar Errors --
#[derive(Debug)]
pub enum CalendarError {
    ProviderUnavailable(String),
    EventCreationFailed(String),
    ConflictCheck(String),
}

impl From<CalendarError> for FrontendError {
    fn from(e: CalendarError) -> Self {
        match e {
            CalendarError::ProviderUnavailable(msg) => FrontendError {
                code: "CALENDAR_PROVIDER_UNAVAILABLE".into(),
                message: msg,
                retryable: true,
            },
            CalendarError::EventCreationFailed(msg) => FrontendError {
                code: "CALENDAR_EVENT_FAILED".into(),
                message: msg,
                retryable: true,
            },
            CalendarError::ConflictCheck(msg) => FrontendError {
                code: "CALENDAR_CONFLICT_CHECK_FAILED".into(),
                message: msg,
                retryable: true,
            },
        }
    }
}

// Convenience: allow using FrontendError as Tauri command error
impl From<FrontendError> for String {
    fn from(e: FrontendError) -> Self {
        serde_json::to_string(&e).unwrap_or_else(|_| format!("{{\"code\":\"{}\",\"message\":\"{}\"}}", e.code, e.message))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imap_connection_error_is_retryable() {
        let err: FrontendError = ImapError::Connection("timeout".into()).into();
        assert_eq!(err.code, "IMAP_CONNECTION_FAILED");
        assert!(err.retryable);
    }

    #[test]
    fn imap_auth_error_is_not_retryable() {
        let err: FrontendError = ImapError::Auth("bad password".into()).into();
        assert_eq!(err.code, "IMAP_AUTH_FAILED");
        assert!(!err.retryable);
    }

    #[test]
    fn ai_budget_exceeded_is_not_retryable() {
        let err: FrontendError = AiError::BudgetExceeded.into();
        assert_eq!(err.code, "AI_BUDGET_EXCEEDED");
        assert!(!err.retryable);
    }

    #[test]
    fn ai_request_error_is_retryable() {
        let err: FrontendError = AiError::ApiRequest("network error".into()).into();
        assert_eq!(err.code, "AI_REQUEST_FAILED");
        assert!(err.retryable);
    }

    #[test]
    fn auth_token_expired_is_retryable() {
        let err: FrontendError = AuthError::TokenExpired.into();
        assert_eq!(err.code, "AUTH_TOKEN_EXPIRED");
        assert!(err.retryable);
    }

    #[test]
    fn frontend_error_serializes_to_json() {
        let err = FrontendError { code: "TEST".into(), message: "test msg".into(), retryable: true };
        let json: String = err.into();
        assert!(json.contains("\"code\":\"TEST\""));
        assert!(json.contains("\"retryable\":true"));
    }

    #[test]
    fn calendar_error_conversion() {
        let err: FrontendError = CalendarError::EventCreationFailed("API error".into()).into();
        assert_eq!(err.code, "CALENDAR_EVENT_FAILED");
        assert!(err.retryable);
    }
}
