use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use ceair_control_protocol::{ClientCommand, ErrorCode, ServerEvent, SessionState};
use ceair_worker::WorkerRuntime;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(_query): Query<WsQuery>,
    State(runtime): State<Arc<WorkerRuntime>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, runtime))
}

async fn handle_socket(socket: WebSocket, runtime: Arc<WorkerRuntime>) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_rx = runtime.subscribe_events();

    // Forward server events to the WebSocket client
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to serialize event: {}", e);
                }
            }
        }
    });

    // Receive client commands from the WebSocket
    let recv_runtime = runtime.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    handle_client_message(&recv_runtime, &text);
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

fn handle_client_message(runtime: &WorkerRuntime, text: &str) {
    let command: ClientCommand = match serde_json::from_str(text) {
        Ok(cmd) => cmd,
        Err(e) => {
            let event = ServerEvent::Error {
                session_id: None,
                code: ErrorCode::InvalidCommand.as_str().to_string(),
                message: format!("Failed to parse command: {}", e),
                timestamp: chrono::Utc::now(),
            };
            runtime.publish_event(event);
            return;
        }
    };

    match command {
        ClientCommand::Ping { .. } => {
            runtime.publish_event(ServerEvent::Pong {
                timestamp: chrono::Utc::now(),
            });
        }
        ClientCommand::SessionCreate {
            title,
            working_directory,
            ..
        } => {
            let info = runtime
                .sessions
                .create_session(title, working_directory);
            runtime.publish_event(ServerEvent::SessionCreated {
                session_id: info.id.clone(),
                title: info.title.clone(),
                timestamp: chrono::Utc::now(),
            });
        }
        ClientCommand::TaskCancel { session_id, .. } => {
            runtime.sessions.cancel_task(&session_id);
            runtime.publish_event(ServerEvent::AgentStatus {
                session_id,
                status: "cancelled".to_string(),
                message: None,
                timestamp: chrono::Utc::now(),
            });
        }
        ClientCommand::ApprovalRespond {
            approval_id,
            decision,
            ..
        } => {
            runtime.approval.resolve(&approval_id, decision.clone());
            runtime.publish_event(ServerEvent::ApprovalResolved {
                session_id: String::new(),
                approval_id,
                decision,
                timestamp: chrono::Utc::now(),
            });
        }
        ClientCommand::SessionClose { session_id, .. } => {
            runtime.sessions.close_session(&session_id);
        }
        ClientCommand::UserMessage {
            session_id,
            content,
            ..
        } => {
            runtime
                .sessions
                .update_state(&session_id, SessionState::Running);
            runtime.publish_event(ServerEvent::AgentStatus {
                session_id: session_id.clone(),
                status: "running".to_string(),
                message: Some(content.clone()),
                timestamp: chrono::Utc::now(),
            });
            if !runtime.run_agent_turn(session_id, content) {
                tracing::warn!("No agent executor configured; message not processed");
            }
        }
        ClientCommand::SessionAttach { session_id, .. } => {
            if let Some(info) = runtime.sessions.get_session(&session_id) {
                runtime.publish_event(ServerEvent::SessionSnapshot {
                    session_id: info.id.clone(),
                    info,
                    history: vec![],
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        ClientCommand::SessionRename {
            session_id, title, ..
        } => {
            tracing::info!(
                "Session rename requested: {} -> {}",
                session_id,
                title
            );
        }
    }
}
