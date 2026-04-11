use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    SessionNotFound,
    SessionClosed,
    ApprovalNotFound,
    ApprovalExpired,
    InvalidCommand,
    Unauthorized,
    RateLimited,
    InternalError,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::SessionNotFound => "session_not_found",
            ErrorCode::SessionClosed => "session_closed",
            ErrorCode::ApprovalNotFound => "approval_not_found",
            ErrorCode::ApprovalExpired => "approval_expired",
            ErrorCode::InvalidCommand => "invalid_command",
            ErrorCode::Unauthorized => "unauthorized",
            ErrorCode::RateLimited => "rate_limited",
            ErrorCode::InternalError => "internal_error",
        }
    }
}
