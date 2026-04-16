use std::sync::Arc;

use dashmap::DashMap;
use orangecoding_control_protocol::{ApprovalDecision, ApprovalRequest, RiskLevel};
use tokio::sync::oneshot;
use uuid::Uuid;

/// Manages pending approval requests, pairing each with a oneshot channel so
/// the caller can `await` the decision.
pub struct ApprovalBridge {
    pending: Arc<DashMap<String, oneshot::Sender<ApprovalDecision>>>,
}

impl ApprovalBridge {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(DashMap::new()),
        }
    }

    /// Create an approval request and return it together with a receiver that
    /// will resolve once [`resolve`] is called with the matching approval id.
    pub async fn request_approval(
        &self,
        session_id: String,
        tool_name: String,
        risk_level: RiskLevel,
        summary: String,
        arguments: serde_json::Value,
    ) -> (ApprovalRequest, oneshot::Receiver<ApprovalDecision>) {
        let id = Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();

        let request = ApprovalRequest {
            id: id.clone(),
            session_id,
            tool_name,
            risk_level,
            summary,
            arguments,
            expires_at: None,
        };

        self.pending.insert(id, tx);
        (request, rx)
    }

    /// Resolve a pending approval. Returns `true` if the approval was found
    /// and the decision was sent, `false` otherwise.
    pub fn resolve(&self, approval_id: &str, decision: ApprovalDecision) -> bool {
        if let Some((_, tx)) = self.pending.remove(approval_id) {
            tx.send(decision).is_ok()
        } else {
            false
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for ApprovalBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn approve_flow() {
        let bridge = ApprovalBridge::new();
        let (req, rx) = bridge
            .request_approval(
                "s1".into(),
                "write_file".into(),
                RiskLevel::High,
                "writing config".into(),
                serde_json::json!({"path": "/etc/conf"}),
            )
            .await;

        assert_eq!(bridge.pending_count(), 1);
        let resolved = bridge.resolve(&req.id, ApprovalDecision::Approved);
        assert!(resolved);
        assert_eq!(rx.await.unwrap(), ApprovalDecision::Approved);
        assert_eq!(bridge.pending_count(), 0);
    }

    #[tokio::test]
    async fn deny_flow() {
        let bridge = ApprovalBridge::new();
        let (req, rx) = bridge
            .request_approval(
                "s2".into(),
                "delete_file".into(),
                RiskLevel::Critical,
                "removing data".into(),
                serde_json::json!({}),
            )
            .await;

        let decision = ApprovalDecision::Denied {
            reason: Some("not allowed".into()),
        };
        let resolved = bridge.resolve(&req.id, decision.clone());
        assert!(resolved);
        assert_eq!(rx.await.unwrap(), decision);
    }

    #[tokio::test]
    async fn resolve_nonexistent_returns_false() {
        let bridge = ApprovalBridge::new();
        let resolved = bridge.resolve("does-not-exist", ApprovalDecision::Approved);
        assert!(!resolved);
    }
}
