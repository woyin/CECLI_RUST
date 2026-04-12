use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chengcoding_control_protocol::ApprovalDecision;
use chengcoding_worker::WorkerRuntime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApprovalResponse {
    pub decision: ApprovalDecision,
}

pub async fn respond_approval(
    State(runtime): State<Arc<WorkerRuntime>>,
    Path(id): Path<String>,
    Json(req): Json<ApprovalResponse>,
) -> impl IntoResponse {
    if runtime.approval.resolve(&id, req.decision) {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
