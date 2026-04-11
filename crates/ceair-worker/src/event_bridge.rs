use ceair_control_protocol::ServerEvent;
use ceair_core::AgentEvent;
use chrono::Utc;

/// Convert an [`AgentEvent`] into an optional [`ServerEvent`].
///
/// Some agent events (e.g. `MessageReceived`) have no corresponding server
/// event and return `None`.
pub fn agent_event_to_server_event(event: &AgentEvent, session_id: &str) -> Option<ServerEvent> {
    let sid = session_id.to_string();
    let now = Utc::now();

    match event {
        AgentEvent::Started { .. } => Some(ServerEvent::AgentStatus {
            session_id: sid,
            status: "started".to_string(),
            message: None,
            timestamp: now,
        }),

        AgentEvent::StreamChunk { content, .. } => Some(ServerEvent::AssistantDelta {
            session_id: sid,
            content: content.clone(),
            timestamp: now,
        }),

        AgentEvent::ToolCallRequested { tool_call, .. } => Some(ServerEvent::ToolCallStarted {
            session_id: sid,
            tool_call_id: tool_call.id.clone(),
            tool_name: tool_call.function_name.clone(),
            arguments_preview: tool_call.arguments.clone(),
            timestamp: now,
        }),

        AgentEvent::ToolCallCompleted {
            tool_name,
            success,
            duration_ms,
            ..
        } => Some(ServerEvent::ToolCallCompleted {
            session_id: sid,
            tool_call_id: String::new(),
            tool_name: tool_name.as_str().to_string(),
            success: *success,
            result_preview: String::new(),
            duration_ms: *duration_ms,
            timestamp: now,
        }),

        AgentEvent::TokenUsageUpdated { usage, .. } => Some(ServerEvent::UsageDelta {
            session_id: sid,
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            timestamp: now,
        }),

        AgentEvent::Completed { summary, .. } => Some(ServerEvent::AssistantDone {
            session_id: sid,
            full_content: summary.clone(),
            timestamp: now,
        }),

        AgentEvent::Error { error_message, .. } => Some(ServerEvent::Error {
            session_id: Some(sid),
            code: "agent_error".to_string(),
            message: error_message.clone(),
            timestamp: now,
        }),

        AgentEvent::MessageReceived { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ceair_core::{AgentEvent, AgentId, SessionId, TokenUsage};

    #[test]
    fn stream_chunk_to_assistant_delta() {
        let event =
            AgentEvent::stream_chunk(AgentId::new(), SessionId::new(), "hello world".to_string());
        let server = agent_event_to_server_event(&event, "sess-1").unwrap();
        match server {
            ServerEvent::AssistantDelta {
                session_id,
                content,
                ..
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(content, "hello world");
            }
            other => panic!("expected AssistantDelta, got {:?}", other),
        }
    }

    #[test]
    fn completed_to_assistant_done() {
        let event = AgentEvent::completed(
            AgentId::new(),
            SessionId::new(),
            "task finished".to_string(),
        );
        let server = agent_event_to_server_event(&event, "sess-2").unwrap();
        match server {
            ServerEvent::AssistantDone {
                session_id,
                full_content,
                ..
            } => {
                assert_eq!(session_id, "sess-2");
                assert_eq!(full_content, "task finished");
            }
            other => panic!("expected AssistantDone, got {:?}", other),
        }
    }

    #[test]
    fn token_usage_updated_to_usage_delta() {
        let usage = TokenUsage::new(100, 50);
        let event = AgentEvent::token_usage_updated(AgentId::new(), SessionId::new(), usage);
        let server = agent_event_to_server_event(&event, "sess-3").unwrap();
        match server {
            ServerEvent::UsageDelta {
                prompt_tokens,
                completion_tokens,
                total_tokens,
                ..
            } => {
                assert_eq!(prompt_tokens, 100);
                assert_eq!(completion_tokens, 50);
                assert_eq!(total_tokens, 150);
            }
            other => panic!("expected UsageDelta, got {:?}", other),
        }
    }

    #[test]
    fn message_received_returns_none() {
        let event = AgentEvent::message_received(AgentId::new(), SessionId::new(), "hi");
        let server = agent_event_to_server_event(&event, "sess-4");
        assert!(server.is_none());
    }
}
