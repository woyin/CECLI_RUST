//! # 工具权限模块
//!
//! 为工具调用提供权限控制，支持 ask/allow/deny 三种权限级别。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 权限类型枚举
// ============================================================

/// 权限种类，标识需要授权的操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionKind {
    /// 文件编辑操作
    Edit,
    /// Bash 命令执行
    Bash,
    /// 网络请求操作
    WebFetch,
    /// 循环调用检测
    DoomLoop,
    /// 外部目录访问
    ExternalDirectory,
}

/// 权限级别，决定操作是否允许
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// 需要询问用户确认
    Ask,
    /// 直接允许执行
    Allow,
    /// 禁止执行
    Deny,
}

// ============================================================
// 权限策略
// ============================================================

/// 权限策略，管理各类操作的权限级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    /// 各权限种类对应的权限级别
    levels: HashMap<PermissionKind, PermissionLevel>,
}

impl Default for PermissionPolicy {
    /// 默认策略：所有权限种类均为 Ask
    fn default() -> Self {
        let mut levels = HashMap::new();
        levels.insert(PermissionKind::Edit, PermissionLevel::Ask);
        levels.insert(PermissionKind::Bash, PermissionLevel::Ask);
        levels.insert(PermissionKind::WebFetch, PermissionLevel::Ask);
        levels.insert(PermissionKind::DoomLoop, PermissionLevel::Ask);
        levels.insert(PermissionKind::ExternalDirectory, PermissionLevel::Ask);
        Self { levels }
    }
}

impl PermissionPolicy {
    /// 查询指定权限种类的权限级别，未找到时默认返回 Ask
    pub fn check(&self, kind: PermissionKind) -> PermissionLevel {
        self.levels
            .get(&kind)
            .copied()
            .unwrap_or(PermissionLevel::Ask)
    }

    /// 设置指定权限种类的权限级别
    pub fn set(&mut self, kind: PermissionKind, level: PermissionLevel) {
        self.levels.insert(kind, level);
    }

    /// 判断指定权限种类是否为 Allow
    pub fn is_allowed(&self, kind: PermissionKind) -> bool {
        self.check(kind) == PermissionLevel::Allow
    }

    /// 判断指定权限种类是否为 Deny
    pub fn is_denied(&self, kind: PermissionKind) -> bool {
        self.check(kind) == PermissionLevel::Deny
    }
}

// ============================================================
// 权限检查器
// ============================================================

/// 权限检查器，将工具名称与权限策略绑定
#[derive(Debug, Clone)]
pub struct PermissionChecker {
    /// 工具名称
    tool_name: String,
    /// 关联的权限策略
    policy: PermissionPolicy,
}

impl PermissionChecker {
    /// 创建新的权限检查器
    pub fn new(tool_name: impl Into<String>, policy: PermissionPolicy) -> Self {
        Self {
            tool_name: tool_name.into(),
            policy,
        }
    }

    /// 检查指定权限种类是否被允许执行
    ///
    /// - Allow 或 Ask 时返回 Ok
    /// - Deny 时返回包含拒绝原因的 Err
    pub fn check_permission(&self, kind: PermissionKind) -> Result<(), String> {
        match self.policy.check(kind) {
            PermissionLevel::Deny => Err(format!(
                "工具 '{}' 的 {:?} 权限被拒绝",
                self.tool_name, kind
            )),
            _ => Ok(()),
        }
    }

    /// 返回工具名称
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }

    /// 返回权限策略的不可变引用
    pub fn policy(&self) -> &PermissionPolicy {
        &self.policy
    }

    /// 返回权限策略的可变引用
    pub fn policy_mut(&mut self) -> &mut PermissionPolicy {
        &mut self.policy
    }
}

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    /// 测试所有 5 种权限种类变体存在
    #[test]
    fn test_permission_kind_variants() {
        let kinds = vec![
            PermissionKind::Edit,
            PermissionKind::Bash,
            PermissionKind::WebFetch,
            PermissionKind::DoomLoop,
            PermissionKind::ExternalDirectory,
        ];
        assert_eq!(kinds.len(), 5);
        // 确认各变体互不相等
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    /// 测试所有 3 种权限级别变体存在
    #[test]
    fn test_permission_level_variants() {
        let levels = vec![
            PermissionLevel::Ask,
            PermissionLevel::Allow,
            PermissionLevel::Deny,
        ];
        assert_eq!(levels.len(), 3);
        assert_ne!(levels[0], levels[1]);
        assert_ne!(levels[1], levels[2]);
        assert_ne!(levels[0], levels[2]);
    }

    /// 测试默认策略：所有权限种类均为 Ask
    #[test]
    fn test_default_policy() {
        let policy = PermissionPolicy::default();
        assert_eq!(policy.check(PermissionKind::Edit), PermissionLevel::Ask);
        assert_eq!(policy.check(PermissionKind::Bash), PermissionLevel::Ask);
        assert_eq!(policy.check(PermissionKind::WebFetch), PermissionLevel::Ask);
        assert_eq!(policy.check(PermissionKind::DoomLoop), PermissionLevel::Ask);
        assert_eq!(
            policy.check(PermissionKind::ExternalDirectory),
            PermissionLevel::Ask
        );
    }

    /// 测试设置权限后查询返回正确级别
    #[test]
    fn test_set_and_check() {
        let mut policy = PermissionPolicy::default();
        policy.set(PermissionKind::Edit, PermissionLevel::Allow);
        assert_eq!(policy.check(PermissionKind::Edit), PermissionLevel::Allow);
    }

    /// 测试 is_allowed 仅在 Allow 时返回 true
    #[test]
    fn test_is_allowed() {
        let mut policy = PermissionPolicy::default();
        assert!(!policy.is_allowed(PermissionKind::Edit));

        policy.set(PermissionKind::Edit, PermissionLevel::Allow);
        assert!(policy.is_allowed(PermissionKind::Edit));

        policy.set(PermissionKind::Edit, PermissionLevel::Deny);
        assert!(!policy.is_allowed(PermissionKind::Edit));
    }

    /// 测试 is_denied 仅在 Deny 时返回 true
    #[test]
    fn test_is_denied() {
        let mut policy = PermissionPolicy::default();
        assert!(!policy.is_denied(PermissionKind::Bash));

        policy.set(PermissionKind::Bash, PermissionLevel::Deny);
        assert!(policy.is_denied(PermissionKind::Bash));

        policy.set(PermissionKind::Bash, PermissionLevel::Allow);
        assert!(!policy.is_denied(PermissionKind::Bash));
    }

    /// 测试查询未设置的权限种类时默认返回 Ask
    #[test]
    fn test_check_unknown_defaults_to_ask() {
        let mut policy = PermissionPolicy::default();
        // 移除一个条目后验证默认值
        policy.levels.remove(&PermissionKind::WebFetch);
        assert_eq!(
            policy.check(PermissionKind::WebFetch),
            PermissionLevel::Ask
        );
    }

    /// 测试 PermissionChecker 的创建与工具名称
    #[test]
    fn test_permission_checker_new() {
        let checker = PermissionChecker::new("test_tool", PermissionPolicy::default());
        assert_eq!(checker.tool_name(), "test_tool");
    }

    /// 测试 check_permission 在 Allow 时返回 Ok
    #[test]
    fn test_permission_checker_allowed() {
        let mut policy = PermissionPolicy::default();
        policy.set(PermissionKind::Edit, PermissionLevel::Allow);
        let checker = PermissionChecker::new("editor", policy);
        assert!(checker.check_permission(PermissionKind::Edit).is_ok());
    }

    /// 测试 check_permission 在 Deny 时返回 Err
    #[test]
    fn test_permission_checker_denied() {
        let mut policy = PermissionPolicy::default();
        policy.set(PermissionKind::Bash, PermissionLevel::Deny);
        let checker = PermissionChecker::new("bash_runner", policy);
        let result = checker.check_permission(PermissionKind::Bash);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bash_runner"));
    }

    /// 测试 check_permission 在 Ask 时返回 Ok
    #[test]
    fn test_permission_checker_ask() {
        let checker = PermissionChecker::new("fetcher", PermissionPolicy::default());
        assert!(checker.check_permission(PermissionKind::WebFetch).is_ok());
    }

    /// 测试通过 policy_mut 修改权限策略
    #[test]
    fn test_permission_checker_policy_mut() {
        let mut checker = PermissionChecker::new("tool", PermissionPolicy::default());
        checker
            .policy_mut()
            .set(PermissionKind::DoomLoop, PermissionLevel::Deny);
        assert!(checker.policy().is_denied(PermissionKind::DoomLoop));
    }

    /// 测试 PermissionKind 的序列化与反序列化
    #[test]
    fn test_permission_kind_serialization() {
        let kind = PermissionKind::Edit;
        let json = serde_json::to_string(&kind).expect("序列化失败");
        let deserialized: PermissionKind = serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(kind, deserialized);

        // 验证所有变体均可正确往返序列化
        let all_kinds = vec![
            PermissionKind::Bash,
            PermissionKind::WebFetch,
            PermissionKind::DoomLoop,
            PermissionKind::ExternalDirectory,
        ];
        for k in all_kinds {
            let s = serde_json::to_string(&k).expect("序列化失败");
            let d: PermissionKind = serde_json::from_str(&s).expect("反序列化失败");
            assert_eq!(k, d);
        }
    }

    /// 测试对多个权限种类分别设置不同级别
    #[test]
    fn test_permission_policy_multiple_sets() {
        let mut policy = PermissionPolicy::default();
        policy.set(PermissionKind::Edit, PermissionLevel::Allow);
        policy.set(PermissionKind::Bash, PermissionLevel::Deny);
        policy.set(PermissionKind::WebFetch, PermissionLevel::Allow);
        policy.set(PermissionKind::DoomLoop, PermissionLevel::Deny);
        policy.set(PermissionKind::ExternalDirectory, PermissionLevel::Ask);

        assert_eq!(policy.check(PermissionKind::Edit), PermissionLevel::Allow);
        assert_eq!(policy.check(PermissionKind::Bash), PermissionLevel::Deny);
        assert_eq!(
            policy.check(PermissionKind::WebFetch),
            PermissionLevel::Allow
        );
        assert_eq!(
            policy.check(PermissionKind::DoomLoop),
            PermissionLevel::Deny
        );
        assert_eq!(
            policy.check(PermissionKind::ExternalDirectory),
            PermissionLevel::Ask
        );

        assert!(policy.is_allowed(PermissionKind::Edit));
        assert!(policy.is_denied(PermissionKind::Bash));
        assert!(policy.is_allowed(PermissionKind::WebFetch));
        assert!(policy.is_denied(PermissionKind::DoomLoop));
        assert!(!policy.is_allowed(PermissionKind::ExternalDirectory));
        assert!(!policy.is_denied(PermissionKind::ExternalDirectory));
    }
}
