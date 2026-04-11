use crate::rules::Severity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 单条不变量检查的结果
pub enum CheckResult {
    Pass,
    Fail(String),
    Skip(String),
}

/// 单条违规记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// 违规报告 — 一次完整检查的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationReport {
    pub total_rules: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub violations: Vec<Violation>,
    pub checked_at: DateTime<Utc>,
    pub has_critical: bool,
}

impl ViolationReport {
    /// 是否全部通过
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    /// 按严重性过滤违规
    pub fn violations_by_severity(&self, severity: Severity) -> Vec<&Violation> {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .collect()
    }

    /// 生成人类可读的报告文本
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# 不变量检查报告\n\n");
        md.push_str(&format!(
            "检查时间: {}\n\n",
            self.checked_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        md.push_str("| 指标 | 数值 |\n|------|------|\n");
        md.push_str(&format!("| 总规则数 | {} |\n", self.total_rules));
        md.push_str(&format!("| 通过 | {} |\n", self.passed));
        md.push_str(&format!("| 失败 | {} |\n", self.failed));
        md.push_str(&format!("| 跳过 | {} |\n\n", self.skipped));

        if self.violations.is_empty() {
            md.push_str("✅ 所有不变量检查通过\n");
        } else {
            md.push_str("## 违规列表\n\n");
            for v in &self.violations {
                md.push_str(&format!("### {} — {}\n\n", v.rule_id, v.rule_name));
                md.push_str(&format!("- **严重性**: {:?}\n", v.severity));
                md.push_str(&format!("- **详情**: {}\n\n", v.message));
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn clean_report() -> ViolationReport {
        ViolationReport {
            total_rules: 18,
            passed: 18,
            failed: 0,
            skipped: 0,
            violations: vec![],
            checked_at: Utc::now(),
            has_critical: false,
        }
    }

    fn report_with_violations() -> ViolationReport {
        ViolationReport {
            total_rules: 18,
            passed: 15,
            failed: 3,
            skipped: 0,
            violations: vec![
                Violation {
                    rule_id: "INV-AUTH-01".into(),
                    rule_name: "WebSocket 连接必须鉴权".into(),
                    severity: Severity::Critical,
                    message: "认证未启用".into(),
                    timestamp: Utc::now(),
                },
                Violation {
                    rule_id: "INV-AUTH-02".into(),
                    rule_name: "HTTP API 必须通过认证中间件".into(),
                    severity: Severity::Critical,
                    message: "认证未启用".into(),
                    timestamp: Utc::now(),
                },
                Violation {
                    rule_id: "INV-TOOL-01".into(),
                    rule_name: "高危工具执行前必须权限检查".into(),
                    severity: Severity::High,
                    message: "工具权限检查未强制执行".into(),
                    timestamp: Utc::now(),
                },
            ],
            checked_at: Utc::now(),
            has_critical: true,
        }
    }

    #[test]
    fn is_clean_when_no_violations() {
        let report = clean_report();
        assert!(report.is_clean());
    }

    #[test]
    fn is_clean_false_when_violations_exist() {
        let report = report_with_violations();
        assert!(!report.is_clean());
    }

    #[test]
    fn violations_by_severity_filters_correctly() {
        let report = report_with_violations();

        let critical = report.violations_by_severity(Severity::Critical);
        assert_eq!(critical.len(), 2);

        let high = report.violations_by_severity(Severity::High);
        assert_eq!(high.len(), 1);

        let medium = report.violations_by_severity(Severity::Medium);
        assert!(medium.is_empty());
    }

    #[test]
    fn to_markdown_renders_clean_report() {
        let report = clean_report();
        let md = report.to_markdown();

        assert!(md.contains("# 不变量检查报告"));
        assert!(md.contains("✅ 所有不变量检查通过"));
        assert!(md.contains("| 总规则数 | 18 |"));
        assert!(md.contains("| 通过 | 18 |"));
    }

    #[test]
    fn to_markdown_renders_violations() {
        let report = report_with_violations();
        let md = report.to_markdown();

        assert!(md.contains("## 违规列表"));
        assert!(md.contains("### INV-AUTH-01"));
        assert!(md.contains("- **严重性**: Critical"));
        assert!(md.contains("| 失败 | 3 |"));
    }
}
