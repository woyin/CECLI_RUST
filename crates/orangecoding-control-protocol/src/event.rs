use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::approval::{ApprovalDecision, ApprovalRequest};
use crate::session::SessionInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HistoryEntry {
    UserMessage {
        content: String,
        timestamp: DateTime<Utc>,
    },
    AssistantMessage {
        content: String,
        model: Option<String>,
        timestamp: DateTime<Utc>,
    },
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
        result: Option<String>,
        success: Option<bool>,
        duration_ms: Option<u64>,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    SessionCreated {
        session_id: String,
        title: Option<String>,
        timestamp: DateTime<Utc>,
    },
    SessionSnapshot {
        session_id: String,
        info: SessionInfo,
        history: Vec<HistoryEntry>,
        timestamp: DateTime<Utc>,
    },
    AssistantDelta {
        session_id: String,
        content: String,
        timestamp: DateTime<Utc>,
    },
    AssistantDone {
        session_id: String,
        full_content: String,
        timestamp: DateTime<Utc>,
    },
    ToolCallStarted {
        session_id: String,
        tool_call_id: String,
        tool_name: String,
        arguments_preview: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    ToolCallCompleted {
        session_id: String,
        tool_call_id: String,
        tool_name: String,
        success: bool,
        result_preview: String,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },
    ToolCallFailed {
        session_id: String,
        tool_call_id: String,
        tool_name: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    ApprovalRequired {
        session_id: String,
        approval: ApprovalRequest,
        timestamp: DateTime<Utc>,
    },
    ApprovalResolved {
        session_id: String,
        approval_id: String,
        decision: ApprovalDecision,
        timestamp: DateTime<Utc>,
    },
    AgentStatus {
        session_id: String,
        status: String,
        message: Option<String>,
        timestamp: DateTime<Utc>,
    },
    UsageDelta {
        session_id: String,
        prompt_tokens: u64,
        completion_tokens: u64,
        total_tokens: u64,
        timestamp: DateTime<Utc>,
    },
    Error {
        session_id: Option<String>,
        code: String,
        message: String,
        timestamp: DateTime<Utc>,
    },
    Pong {
        timestamp: DateTime<Utc>,
    },
}
