//! # 任务交接文档（Handoff Artifact）
//!
//! 对齐规范第六章 6.5：
//! 当 Agent 达到上下文上限、会话结束或用户主动请求交接时，
//! 生成结构化的交接文档以便下一个会话（人工或自动）无缝接手。
//!
//! 五个核心字段：
//! - `task_status` - 任务状态概览（In Progress / Blocked / Completed）
//! - `completed_summary` - 已完成工作摘要
//! - `current_state` - 当前上下文状态（文件/分支/TODO 进度）
//! - `gotchas` - 已知陷阱与注意事项
//! - `next_steps` - 下一步行动列表

use serde::{Deserialize, Serialize};

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 进行中
    InProgress,
    /// 受阻
    Blocked(String),
    /// 已完成
    Completed,
    /// 已取消
    Cancelled(String),
}

impl TaskStatus {
    pub fn as_label(&self) -> String {
        match self {
            TaskStatus::InProgress => "进行中".to_string(),
            TaskStatus::Blocked(r) => format!("受阻：{}", r),
            TaskStatus::Completed => "已完成".to_string(),
            TaskStatus::Cancelled(r) => format!("已取消：{}", r),
        }
    }
}

/// 交接文档
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandoffArtifact {
    /// 任务标题
    pub title: String,
    /// 任务状态
    pub task_status: Option<TaskStatus>,
    /// 已完成工作摘要（Markdown 项目符号）
    pub completed_summary: Vec<String>,
    /// 当前状态条目
    pub current_state: Vec<StateEntry>,
    /// 已知陷阱
    pub gotchas: Vec<String>,
    /// 下一步行动
    pub next_steps: Vec<String>,
    /// 生成时间戳（ISO8601）
    pub generated_at: Option<String>,
}

/// 当前状态条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    pub key: String,
    pub value: String,
}

impl HandoffArtifact {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            generated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.task_status = Some(status);
        self
    }

    pub fn add_completed(&mut self, item: impl Into<String>) -> &mut Self {
        self.completed_summary.push(item.into());
        self
    }

    pub fn add_state(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.current_state.push(StateEntry {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    pub fn add_gotcha(&mut self, item: impl Into<String>) -> &mut Self {
        self.gotchas.push(item.into());
        self
    }

    pub fn add_next_step(&mut self, item: impl Into<String>) -> &mut Self {
        self.next_steps.push(item.into());
        self
    }

    /// 渲染为 Markdown
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Handoff: {}\n\n", self.title));
        if let Some(ts) = &self.generated_at {
            out.push_str(&format!("> 生成时间：{}\n\n", ts));
        }
        if let Some(s) = &self.task_status {
            out.push_str(&format!("**任务状态**：{}\n\n", s.as_label()));
        }

        out.push_str("## 已完成工作\n\n");
        if self.completed_summary.is_empty() {
            out.push_str("_（无）_\n\n");
        } else {
            for item in &self.completed_summary {
                out.push_str(&format!("- {}\n", item));
            }
            out.push('\n');
        }

        out.push_str("## 当前状态\n\n");
        if self.current_state.is_empty() {
            out.push_str("_（无）_\n\n");
        } else {
            for e in &self.current_state {
                out.push_str(&format!("- **{}**: {}\n", e.key, e.value));
            }
            out.push('\n');
        }

        out.push_str("## 注意事项（Gotchas）\n\n");
        if self.gotchas.is_empty() {
            out.push_str("_（无）_\n\n");
        } else {
            for g in &self.gotchas {
                out.push_str(&format!("- ⚠️ {}\n", g));
            }
            out.push('\n');
        }

        out.push_str("## 下一步行动\n\n");
        if self.next_steps.is_empty() {
            out.push_str("_（无）_\n\n");
        } else {
            for (i, n) in self.next_steps.iter().enumerate() {
                out.push_str(&format!("{}. {}\n", i + 1, n));
            }
            out.push('\n');
        }

        out
    }

    /// 校验：至少应包含 title 与 next_steps 或 status，以保证可交接
    pub fn is_valid(&self) -> bool {
        !self.title.is_empty() && (self.task_status.is_some() || !self.next_steps.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sets_title_and_timestamp() {
        let h = HandoffArtifact::new("Task A");
        assert_eq!(h.title, "Task A");
        assert!(h.generated_at.is_some());
    }

    #[test]
    fn test_builder_methods() {
        let mut h = HandoffArtifact::new("Feature X").with_status(TaskStatus::InProgress);
        h.add_completed("wrote tests")
            .add_state("branch", "feat/x")
            .add_gotcha("test fixtures need cleanup")
            .add_next_step("implement handler");

        assert_eq!(h.completed_summary.len(), 1);
        assert_eq!(h.current_state.len(), 1);
        assert_eq!(h.gotchas.len(), 1);
        assert_eq!(h.next_steps.len(), 1);
        assert_eq!(h.task_status, Some(TaskStatus::InProgress));
    }

    #[test]
    fn test_to_markdown_includes_sections() {
        let mut h = HandoffArtifact::new("Sample").with_status(TaskStatus::InProgress);
        h.add_completed("A").add_next_step("B").add_gotcha("C");
        let md = h.to_markdown();
        assert!(md.contains("# Handoff: Sample"));
        assert!(md.contains("## 已完成工作"));
        assert!(md.contains("## 下一步行动"));
        assert!(md.contains("## 注意事项"));
        assert!(md.contains("- A"));
        assert!(md.contains("1. B"));
        assert!(md.contains("⚠️ C"));
    }

    #[test]
    fn test_is_valid() {
        let empty = HandoffArtifact::default();
        assert!(!empty.is_valid());

        let mut h = HandoffArtifact::new("X");
        assert!(!h.is_valid());
        h.add_next_step("do something");
        assert!(h.is_valid());
    }

    #[test]
    fn test_task_status_label() {
        assert_eq!(TaskStatus::InProgress.as_label(), "进行中");
        assert_eq!(TaskStatus::Completed.as_label(), "已完成");
        assert!(TaskStatus::Blocked("dep".into()).as_label().contains("dep"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut h = HandoffArtifact::new("T").with_status(TaskStatus::Completed);
        h.add_completed("ok");
        let json = serde_json::to_string(&h).unwrap();
        let back: HandoffArtifact = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "T");
        assert_eq!(back.task_status, Some(TaskStatus::Completed));
    }

    #[test]
    fn test_empty_sections_show_placeholder() {
        let h = HandoffArtifact::new("E").with_status(TaskStatus::InProgress);
        let md = h.to_markdown();
        assert!(md.contains("_（无）_"));
    }
}
