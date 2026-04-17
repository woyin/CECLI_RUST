//! # 模式化权限规则
//!
//! 对齐规范第三章的规则语法：
//!   - `Bash(git *)`
//!   - `Write(**/*.ts)`
//!   - `WebFetch(docs.python.org)`
//!   - `Read(*)`
//!
//! 以及 deny/allow/ask 三级优先顺序：
//! DENY > ASK > ALLOW > 默认 ASK
//!
//! 同时支持规则的持久化：
//!   - 用户级：~/.config/orangecoding/permissions.json
//!   - 项目级：<project>/.orangecoding/permissions.json

use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::permissions::PermissionLevel;

/// 规则的解析结果：`ToolName(pattern)`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionRule {
    /// 工具名（大小写不敏感，例如 `Bash`/`bash`）
    pub tool: String,
    /// 要匹配的模式（glob）
    pub pattern: String,
}

impl PermissionRule {
    /// 解析 `Tool(pattern)` 或 `Tool` 形式的规则
    pub fn parse(raw: &str) -> Result<Self, String> {
        let raw = raw.trim();
        if let Some(lparen) = raw.find('(') {
            if !raw.ends_with(')') {
                return Err(format!("规则格式错误，缺少结尾 ')': {}", raw));
            }
            let tool = raw[..lparen].trim().to_string();
            let pattern = raw[lparen + 1..raw.len() - 1].trim().to_string();
            if tool.is_empty() {
                return Err(format!("规则缺少 Tool 名: {}", raw));
            }
            if pattern.is_empty() {
                return Err(format!("规则模式为空: {}", raw));
            }
            // 预校验 pattern 合法性
            Pattern::new(&pattern).map_err(|e| format!("无效的 glob 模式 `{}`: {}", pattern, e))?;
            Ok(Self { tool, pattern })
        } else if raw.is_empty() {
            Err("规则为空".to_string())
        } else {
            Ok(Self {
                tool: raw.to_string(),
                pattern: "*".to_string(),
            })
        }
    }

    /// 检查规则是否匹配某次工具调用
    ///
    /// - 工具名大小写不敏感
    /// - pattern 按 glob 匹配目标（由调用方选择：例如文件路径、命令字符串、域名等）
    pub fn matches(&self, tool: &str, target: &str) -> bool {
        if !self.tool.eq_ignore_ascii_case(tool) {
            return false;
        }
        match Pattern::new(&self.pattern) {
            Ok(p) => p.matches(target),
            Err(_) => false,
        }
    }
}

/// 规则集合：deny / ask / allow 三段
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionRuleSet {
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub ask: Vec<String>,
    #[serde(default)]
    pub allow: Vec<String>,
}

impl PermissionRuleSet {
    /// 评估工具调用的权限级别
    ///
    /// 优先级：DENY > ASK > ALLOW > 默认 Ask
    pub fn evaluate(&self, tool: &str, target: &str) -> PermissionLevel {
        if self.any_match(&self.deny, tool, target) {
            return PermissionLevel::Deny;
        }
        if self.any_match(&self.ask, tool, target) {
            return PermissionLevel::Ask;
        }
        if self.any_match(&self.allow, tool, target) {
            return PermissionLevel::Allow;
        }
        PermissionLevel::Ask
    }

    fn any_match(&self, raws: &[String], tool: &str, target: &str) -> bool {
        for raw in raws {
            if let Ok(rule) = PermissionRule::parse(raw) {
                if rule.matches(tool, target) {
                    return true;
                }
            }
        }
        false
    }

    /// 合并两个规则集（项目级覆盖用户级时仅做追加，不删除）
    pub fn merge(mut self, other: PermissionRuleSet) -> Self {
        self.deny.extend(other.deny);
        self.ask.extend(other.ask);
        self.allow.extend(other.allow);
        self
    }

    /// 从 JSON 文件加载
    pub fn load_from<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let parsed: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(parsed)
    }

    /// 写回 JSON 文件（会自动创建父目录）
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        let pretty = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        fs::write(path, pretty)
    }
}

/// 约定的存放位置
pub fn user_rules_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/orangecoding/permissions.json")
}

/// 项目级规则路径
pub fn project_rules_path(project_root: &Path) -> PathBuf {
    project_root.join(".orangecoding/permissions.json")
}

/// 按优先级加载全局 + 项目规则并合并
pub fn load_effective_rules(project_root: Option<&Path>) -> PermissionRuleSet {
    let mut set = PermissionRuleSet::load_from(user_rules_path()).unwrap_or_default();
    if let Some(root) = project_root {
        if let Ok(extra) = PermissionRuleSet::load_from(project_rules_path(root)) {
            set = set.merge(extra);
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_tool_and_pattern() {
        let r = PermissionRule::parse("Bash(git *)").unwrap();
        assert_eq!(r.tool, "Bash");
        assert_eq!(r.pattern, "git *");
    }

    #[test]
    fn test_parse_tool_only_defaults_to_star() {
        let r = PermissionRule::parse("Read").unwrap();
        assert_eq!(r.tool, "Read");
        assert_eq!(r.pattern, "*");
    }

    #[test]
    fn test_parse_rejects_invalid() {
        assert!(PermissionRule::parse("").is_err());
        assert!(PermissionRule::parse("Bash(foo").is_err());
        assert!(PermissionRule::parse("Bash()").is_err());
        assert!(PermissionRule::parse("(hello)").is_err());
    }

    #[test]
    fn test_rule_matches_command() {
        let r = PermissionRule::parse("Bash(git *)").unwrap();
        assert!(r.matches("Bash", "git status"));
        assert!(r.matches("bash", "git commit -m 'x'"));
        assert!(!r.matches("Bash", "rm -rf /"));
        assert!(!r.matches("Write", "git status"));
    }

    #[test]
    fn test_rule_matches_path_glob() {
        let r = PermissionRule::parse("Write(**/*.ts)").unwrap();
        assert!(r.matches("Write", "src/foo.ts"));
        assert!(r.matches("Write", "a/b/c/x.ts"));
        assert!(!r.matches("Write", "src/foo.rs"));
    }

    #[test]
    fn test_rule_matches_domain() {
        let r = PermissionRule::parse("WebFetch(docs.python.org)").unwrap();
        assert!(r.matches("WebFetch", "docs.python.org"));
        assert!(!r.matches("WebFetch", "evil.com"));
    }

    #[test]
    fn test_evaluate_deny_wins() {
        let set = PermissionRuleSet {
            deny: vec!["Bash(rm -rf *)".into()],
            allow: vec!["Bash(*)".into()],
            ask: vec![],
        };
        assert_eq!(set.evaluate("Bash", "rm -rf /"), PermissionLevel::Deny);
        assert_eq!(set.evaluate("Bash", "ls -la"), PermissionLevel::Allow);
    }

    #[test]
    fn test_evaluate_default_is_ask() {
        let set = PermissionRuleSet::default();
        assert_eq!(set.evaluate("Bash", "whatever"), PermissionLevel::Ask);
    }

    #[test]
    fn test_evaluate_ask_over_allow() {
        let set = PermissionRuleSet {
            deny: vec![],
            ask: vec!["Write(**/*.env)".into()],
            allow: vec!["Write(*)".into()],
        };
        assert_eq!(set.evaluate("Write", "foo/.env"), PermissionLevel::Ask);
        assert_eq!(set.evaluate("Write", "foo/bar.rs"), PermissionLevel::Allow);
    }

    #[test]
    fn test_persistence_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/permissions.json");
        let set = PermissionRuleSet {
            deny: vec!["Bash(rm -rf /)".into()],
            ask: vec!["Write(**/.env)".into()],
            allow: vec!["Read(*)".into()],
        };
        set.save_to(&path).unwrap();
        let loaded = PermissionRuleSet::load_from(&path).unwrap();
        assert_eq!(loaded.deny, set.deny);
        assert_eq!(loaded.ask, set.ask);
        assert_eq!(loaded.allow, set.allow);
    }

    #[test]
    fn test_merge_combines_rules() {
        let a = PermissionRuleSet {
            deny: vec!["Bash(rm -rf /)".into()],
            ask: vec![],
            allow: vec!["Read(*)".into()],
        };
        let b = PermissionRuleSet {
            deny: vec![],
            ask: vec!["Write(**/*.ts)".into()],
            allow: vec![],
        };
        let merged = a.merge(b);
        assert_eq!(merged.deny.len(), 1);
        assert_eq!(merged.ask.len(), 1);
        assert_eq!(merged.allow.len(), 1);
    }

    #[test]
    fn test_invalid_rule_in_set_is_skipped() {
        let set = PermissionRuleSet {
            deny: vec!["not_a_rule(".into()],
            ask: vec![],
            allow: vec!["Read(*)".into()],
        };
        // 非法 deny 规则不应导致 panic；默认仍为 Allow（被 allow Read(*) 命中）
        assert_eq!(set.evaluate("Read", "foo"), PermissionLevel::Allow);
    }
}
