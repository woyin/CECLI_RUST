pub mod approval;
pub mod command;
pub mod error;
pub mod event;
pub mod session;

pub use approval::{ApprovalDecision, ApprovalRequest, RiskLevel};
pub use command::ClientCommand;
pub use error::ErrorCode;
pub use event::{HistoryEntry, ServerEvent};
pub use session::{SessionInfo, SessionState};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn serde_roundtrip_client_command_user_message() {
        let cmd = ClientCommand::UserMessage {
            request_id: "req-1".to_string(),
            session_id: "sess-1".to_string(),
            content: "Hello, world!".to_string(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: ClientCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd.request_id(), deserialized.request_id());
        assert_eq!(cmd.session_id(), deserialized.session_id());
    }

    #[test]
    fn serde_roundtrip_server_event_assistant_delta() {
        let event = ServerEvent::AssistantDelta {
            session_id: "sess-1".to_string(),
            content: "Hello".to_string(),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ServerEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            ServerEvent::AssistantDelta {
                session_id,
                content,
                ..
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected AssistantDelta variant"),
        }
    }

    #[test]
    fn serde_roundtrip_approval_request() {
        let req = ApprovalRequest {
            id: "apr-1".to_string(),
            session_id: "sess-1".to_string(),
            tool_name: "file_write".to_string(),
            risk_level: RiskLevel::High,
            summary: "Write to /etc/passwd".to_string(),
            arguments: serde_json::json!({"path": "/etc/passwd"}),
            expires_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ApprovalRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "apr-1");
        assert_eq!(deserialized.tool_name, "file_write");
        assert_eq!(deserialized.risk_level, RiskLevel::High);
    }

    #[test]
    fn serde_roundtrip_session_info() {
        let now = Utc::now();
        let info = SessionInfo {
            id: "sess-1".to_string(),
            title: Some("Test Session".to_string()),
            state: SessionState::Idle,
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "sess-1");
        assert_eq!(deserialized.title, Some("Test Session".to_string()));
        assert_eq!(deserialized.state, SessionState::Idle);
    }
}
