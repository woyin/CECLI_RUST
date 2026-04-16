use serde::{Deserialize, Serialize};

use crate::approval::ApprovalDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientCommand {
    SessionAttach {
        request_id: String,
        session_id: String,
    },
    SessionCreate {
        request_id: String,
        title: Option<String>,
        working_directory: Option<String>,
    },
    UserMessage {
        request_id: String,
        session_id: String,
        content: String,
    },
    TaskCancel {
        request_id: String,
        session_id: String,
    },
    ApprovalRespond {
        request_id: String,
        approval_id: String,
        decision: ApprovalDecision,
    },
    SessionRename {
        request_id: String,
        session_id: String,
        title: String,
    },
    SessionClose {
        request_id: String,
        session_id: String,
    },
    Ping {
        request_id: String,
    },
}

impl ClientCommand {
    pub fn request_id(&self) -> &str {
        match self {
            ClientCommand::SessionAttach { request_id, .. } => request_id,
            ClientCommand::SessionCreate { request_id, .. } => request_id,
            ClientCommand::UserMessage { request_id, .. } => request_id,
            ClientCommand::TaskCancel { request_id, .. } => request_id,
            ClientCommand::ApprovalRespond { request_id, .. } => request_id,
            ClientCommand::SessionRename { request_id, .. } => request_id,
            ClientCommand::SessionClose { request_id, .. } => request_id,
            ClientCommand::Ping { request_id, .. } => request_id,
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            ClientCommand::SessionAttach { session_id, .. } => Some(session_id),
            ClientCommand::SessionCreate { .. } => None,
            ClientCommand::UserMessage { session_id, .. } => Some(session_id),
            ClientCommand::TaskCancel { session_id, .. } => Some(session_id),
            ClientCommand::ApprovalRespond { .. } => None,
            ClientCommand::SessionRename { session_id, .. } => Some(session_id),
            ClientCommand::SessionClose { session_id, .. } => Some(session_id),
            ClientCommand::Ping { .. } => None,
        }
    }
}
