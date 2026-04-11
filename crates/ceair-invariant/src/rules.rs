use serde::{Deserialize, Serialize};

/// 不变量严重性级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

/// 不变量类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InvariantCategory {
    Auth,
    Cancellation,
    Session,
    ToolPermission,
    Context,
    Audit,
    Approval,
    Event,
}

/// 单条不变量规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantRule {
    pub id: String,
    pub category: InvariantCategory,
    pub name: String,
    pub description: String,
    pub severity: Severity,
}

impl InvariantRule {
    pub fn new(
        id: impl Into<String>,
        category: InvariantCategory,
        name: impl Into<String>,
        severity: Severity,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            name: name.into(),
            description: String::new(),
            severity,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// 所有已知的系统不变量
pub fn system_invariants() -> Vec<InvariantRule> {
    vec![
        InvariantRule::new(
            "INV-AUTH-01",
            InvariantCategory::Auth,
            "WebSocket 连接必须鉴权",
            Severity::Critical,
        )
        .with_description("每个 WS 升级请求必须在握手阶段验证 token"),
        InvariantRule::new(
            "INV-AUTH-02",
            InvariantCategory::Auth,
            "HTTP API 必须通过认证中间件",
            Severity::Critical,
        )
        .with_description("除 /health 外所有端点必须要求 Bearer token"),
        InvariantRule::new(
            "INV-AUTH-03",
            InvariantCategory::Auth,
            "Token 不得出现在日志中",
            Severity::Critical,
        )
        .with_description("认证 token 不得以明文出现在日志输出中"),
        InvariantRule::new(
            "INV-CANCEL-01",
            InvariantCategory::Cancellation,
            "取消信号必须向下传播",
            Severity::High,
        )
        .with_description("cancel 调用必须传播到所有子 token"),
        InvariantRule::new(
            "INV-CANCEL-02",
            InvariantCategory::Cancellation,
            "取消后必须可重置",
            Severity::High,
        )
        .with_description("reset_cancel_token 必须产生新的未取消 token"),
        InvariantRule::new(
            "INV-SESSION-01",
            InvariantCategory::Session,
            "会话上下文必须跨 turn 持久化",
            Severity::High,
        )
        .with_description("同一 session_id 的多次 turn 必须共享上下文"),
        InvariantRule::new(
            "INV-SESSION-02",
            InvariantCategory::Session,
            "关闭的会话不可继续使用",
            Severity::High,
        )
        .with_description("close 后 get/cancel/update 必须返回失败"),
        InvariantRule::new(
            "INV-SESSION-03",
            InvariantCategory::Session,
            "会话 ID 必须全局唯一",
            Severity::Medium,
        )
        .with_description("create_session 生成的 ID 必须是唯一 UUID"),
        InvariantRule::new(
            "INV-TOOL-01",
            InvariantCategory::ToolPermission,
            "高危工具执行前必须权限检查",
            Severity::Critical,
        )
        .with_description("destructive 工具必须先 check_permissions"),
        InvariantRule::new(
            "INV-TOOL-02",
            InvariantCategory::ToolPermission,
            "Deny 决策必须阻止执行",
            Severity::Critical,
        )
        .with_description("check_permissions 返回 Deny 时 execute 不得被调用"),
        InvariantRule::new(
            "INV-TOOL-03",
            InvariantCategory::ToolPermission,
            "输入验证必须在执行前完成",
            Severity::High,
        )
        .with_description("validate_input Err 时 execute 不得被调用"),
        InvariantRule::new(
            "INV-CTX-01",
            InvariantCategory::Context,
            "压缩后系统提示不得丢失",
            Severity::High,
        )
        .with_description("压缩操作后 System 消息必须保留"),
        InvariantRule::new(
            "INV-CTX-02",
            InvariantCategory::Context,
            "Token 预算不得为负",
            Severity::Medium,
        )
        .with_description("remaining 值在任何操作后不得为负"),
        InvariantRule::new(
            "INV-AUDIT-01",
            InvariantCategory::Audit,
            "高危操作必须有审计记录",
            Severity::High,
        )
        .with_description("bash/edit/delete/审批等操作必须记录审计"),
        InvariantRule::new(
            "INV-AUDIT-02",
            InvariantCategory::Audit,
            "审计链哈希必须连续",
            Severity::Medium,
        )
        .with_description("每条记录的 hash 基于 previous_hash 计算"),
        InvariantRule::new(
            "INV-APPROVAL-01",
            InvariantCategory::Approval,
            "审批请求必须可等待",
            Severity::High,
        )
        .with_description("request_approval 返回的 Receiver 必须在 resolve 后产出结果"),
        InvariantRule::new(
            "INV-APPROVAL-02",
            InvariantCategory::Approval,
            "审批结果必须送达请求方",
            Severity::High,
        )
        .with_description("resolve 必须通过 channel 发送决策"),
        InvariantRule::new(
            "INV-EVENT-01",
            InvariantCategory::Event,
            "事件序列必须保持时间顺序",
            Severity::Medium,
        )
        .with_description("publish 顺序必须被 subscriber 保持"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn system_invariants_returns_18_rules() {
        let rules = system_invariants();
        assert_eq!(rules.len(), 18);
    }

    #[test]
    fn all_categories_covered() {
        let rules = system_invariants();
        let categories: HashSet<_> = rules.iter().map(|r| r.category).collect();

        assert!(categories.contains(&InvariantCategory::Auth));
        assert!(categories.contains(&InvariantCategory::Cancellation));
        assert!(categories.contains(&InvariantCategory::Session));
        assert!(categories.contains(&InvariantCategory::ToolPermission));
        assert!(categories.contains(&InvariantCategory::Context));
        assert!(categories.contains(&InvariantCategory::Audit));
        assert!(categories.contains(&InvariantCategory::Approval));
        assert!(categories.contains(&InvariantCategory::Event));
        assert_eq!(categories.len(), 8);
    }

    #[test]
    fn all_severities_used() {
        let rules = system_invariants();
        let severities: HashSet<_> = rules.iter().map(|r| r.severity).collect();

        assert!(severities.contains(&Severity::Critical));
        assert!(severities.contains(&Severity::High));
        assert!(severities.contains(&Severity::Medium));
        // Low is defined but not used in system invariants — that's fine
    }

    #[test]
    fn ids_are_unique() {
        let rules = system_invariants();
        let ids: HashSet<_> = rules.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids.len(), rules.len(), "duplicate rule IDs found");
    }
}
