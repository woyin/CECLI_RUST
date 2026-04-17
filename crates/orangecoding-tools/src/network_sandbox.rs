//! # 网络沙箱
//!
//! 对齐规范第六章 6.1 的 NetworkSandbox：
//! - `allowed_domains` 白名单
//! - `blocked_domains` 黑名单（优先生效）
//! - `allowed_ports` 端口白名单（空表示不限制）
//! - `default_allow_list` 常见开发域名默认放行
//!
//! 判断顺序（Deny-First）：
//!   1. 命中黑名单 → Deny
//!   2. 命中白名单 / 默认放行 → Allow
//!   3. 否则 → Ask（保守策略，返回 Unknown）

use serde::{Deserialize, Serialize};
use url::Url;

/// 网络访问决策
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDecision {
    Allow,
    Deny,
    /// 未命中任何规则，需要询问用户
    Unknown,
}

/// 网络沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSandbox {
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub blocked_domains: Vec<String>,
    #[serde(default)]
    pub allowed_ports: Vec<u16>,
    /// 启用默认放行列表
    #[serde(default = "default_true")]
    pub use_default_allow_list: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NetworkSandbox {
    fn default() -> Self {
        Self {
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            allowed_ports: Vec::new(),
            use_default_allow_list: true,
        }
    }
}

/// 常见开发域名的默认放行列表
pub const DEFAULT_ALLOW_LIST: &[&str] = &[
    "pypi.org",
    "files.pythonhosted.org",
    "registry.npmjs.org",
    "npmjs.com",
    "github.com",
    "raw.githubusercontent.com",
    "api.github.com",
    "crates.io",
    "static.crates.io",
    "docs.rs",
    "docs.python.org",
    "developer.mozilla.org",
    "rust-lang.org",
    "golang.org",
];

impl NetworkSandbox {
    pub fn new() -> Self {
        Self::default()
    }

    /// 判定是否匹配域名模式（支持 `*.example.com` 这样的通配符前缀）
    fn host_matches(pattern: &str, host: &str) -> bool {
        let pattern = pattern.trim().to_ascii_lowercase();
        let host = host.trim().to_ascii_lowercase();
        if pattern.is_empty() || host.is_empty() {
            return false;
        }
        if let Some(suffix) = pattern.strip_prefix("*.") {
            return host == suffix || host.ends_with(&format!(".{}", suffix));
        }
        pattern == host
    }

    /// 判断主机名是否被默认放行
    fn is_default_allowed(host: &str) -> bool {
        DEFAULT_ALLOW_LIST
            .iter()
            .any(|d| Self::host_matches(d, host))
    }

    /// 对单个 URL 做权限判定
    pub fn check_url(&self, url: &str) -> NetworkDecision {
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return NetworkDecision::Deny,
        };
        let host = match parsed.host_str() {
            Some(h) => h,
            None => return NetworkDecision::Deny,
        };

        // 端口检查：若有白名单，则非白名单端口直接拒绝
        if !self.allowed_ports.is_empty() {
            let port = parsed.port_or_known_default().unwrap_or(0);
            if !self.allowed_ports.contains(&port) {
                return NetworkDecision::Deny;
            }
        }

        // Deny 优先
        if self
            .blocked_domains
            .iter()
            .any(|d| Self::host_matches(d, host))
        {
            return NetworkDecision::Deny;
        }

        // 用户白名单
        if self
            .allowed_domains
            .iter()
            .any(|d| Self::host_matches(d, host))
        {
            return NetworkDecision::Allow;
        }

        // 默认开发域名
        if self.use_default_allow_list && Self::is_default_allowed(host) {
            return NetworkDecision::Allow;
        }

        NetworkDecision::Unknown
    }

    /// 便捷布尔方法
    pub fn is_allowed(&self, url: &str) -> bool {
        matches!(self.check_url(url), NetworkDecision::Allow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allow_list_works() {
        let s = NetworkSandbox::default();
        assert_eq!(
            s.check_url("https://pypi.org/project/foo/"),
            NetworkDecision::Allow
        );
        assert_eq!(
            s.check_url("https://docs.python.org/3/"),
            NetworkDecision::Allow
        );
    }

    #[test]
    fn test_unknown_returns_unknown() {
        let s = NetworkSandbox::default();
        assert_eq!(s.check_url("https://evil.com/x"), NetworkDecision::Unknown);
    }

    #[test]
    fn test_deny_overrides_allow() {
        let s = NetworkSandbox {
            allowed_domains: vec!["example.com".into()],
            blocked_domains: vec!["example.com".into()],
            ..Default::default()
        };
        assert_eq!(s.check_url("https://example.com/x"), NetworkDecision::Deny);
    }

    #[test]
    fn test_wildcard_match() {
        let s = NetworkSandbox {
            allowed_domains: vec!["*.internal.corp".into()],
            ..Default::default()
        };
        assert_eq!(
            s.check_url("https://api.internal.corp/v1"),
            NetworkDecision::Allow
        );
        assert_eq!(
            s.check_url("https://internal.corp/v1"),
            NetworkDecision::Allow
        );
        assert_eq!(s.check_url("https://corp.com/v1"), NetworkDecision::Unknown);
    }

    #[test]
    fn test_port_whitelist() {
        let s = NetworkSandbox {
            allowed_domains: vec!["example.com".into()],
            allowed_ports: vec![443],
            use_default_allow_list: false,
            ..Default::default()
        };
        assert_eq!(s.check_url("https://example.com/x"), NetworkDecision::Allow);
        assert_eq!(s.check_url("http://example.com/x"), NetworkDecision::Deny);
    }

    #[test]
    fn test_invalid_url_denied() {
        let s = NetworkSandbox::default();
        assert_eq!(s.check_url("not-a-url"), NetworkDecision::Deny);
    }

    #[test]
    fn test_disable_default_list() {
        let s = NetworkSandbox {
            use_default_allow_list: false,
            ..Default::default()
        };
        assert_eq!(s.check_url("https://pypi.org/"), NetworkDecision::Unknown);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let s = NetworkSandbox {
            allowed_domains: vec!["example.com".into()],
            blocked_domains: vec!["evil.com".into()],
            allowed_ports: vec![443, 8443],
            use_default_allow_list: true,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: NetworkSandbox = serde_json::from_str(&json).unwrap();
        assert_eq!(back.allowed_domains, s.allowed_domains);
        assert_eq!(back.blocked_domains, s.blocked_domains);
        assert_eq!(back.allowed_ports, s.allowed_ports);
    }
}
