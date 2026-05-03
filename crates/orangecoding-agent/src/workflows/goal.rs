//! # Goal 自主迭代循环
//!
//! 实现 Planning → Executing → Verifying → Replan/Done 的循环执行模式。
//! 用户输入目标需求，系统自动规划、执行、验证，直到目标完成。
//!
//! ## 触发方式
//!
//! - CLI: `orangecoding launch --goal "需求描述"`
//! - 斜杠命令: `/goal 需求描述`
//!
//! ## 循环流程
//!
//! ```text
//! Planning → Executing → Verifying → Done
//!     ▲                         │
//!     │      验证失败           │
//!     └─────────────────────────┘
//!                        │ 验证通过
//!                        ▼
//!                      Done
//! ```

use serde::{Deserialize, Serialize};

// ============================================================
// Goal 阶段
// ============================================================

/// Goal 执行阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GoalPhase {
    /// 生成/重新生成执行计划
    Planning,
    /// 按计划执行任务
    Executing,
    /// 验证执行结果
    Verifying,
    /// 全部完成
    Done,
}

impl GoalPhase {
    /// 返回阶段的中文显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Planning => "制定计划",
            Self::Executing => "执行任务",
            Self::Verifying => "验证结果",
            Self::Done => "已完成",
        }
    }
}

// ============================================================
// Goal 配置
// ============================================================

/// Goal 模式配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalConfig {
    /// 最大循环次数
    pub max_cycles: u32,
    /// 每个任务内部的 AI 初始迭代软预算
    pub task_max_iterations: u32,
    /// 自定义验证命令
    pub verify_commands: Vec<String>,
    /// 每轮完成后是否自动 git commit
    pub auto_commit_per_cycle: bool,
    /// 完成标记字符串
    pub completion_promise: String,
    /// 是否启用漂移检测
    pub enable_drift_detection: bool,
}

impl Default for GoalConfig {
    fn default() -> Self {
        Self {
            max_cycles: 20,
            task_max_iterations: 30,
            verify_commands: vec!["cargo test".to_string()],
            auto_commit_per_cycle: true,
            completion_promise: "GOAL_COMPLETE".to_string(),
            enable_drift_detection: true,
        }
    }
}

// ============================================================
// Goal 任务与计划
// ============================================================

/// 任务执行状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalTaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed { reason: String },
    Skipped { reason: String },
}

/// 单个 Goal 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub target_files: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub depends_on: Vec<String>,
    pub status: GoalTaskStatus,
}

/// Goal 执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalPlan {
    pub requirement: String,
    pub tasks: Vec<GoalTask>,
    pub acceptance_criteria: Vec<String>,
    pub cycle: u32,
    pub forbidden_detours: Vec<String>,
    pub context: String,
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod type_tests {
    use super::*;

    #[test]
    fn 默认配置合理() {
        let config = GoalConfig::default();
        assert_eq!(config.max_cycles, 20);
        assert_eq!(config.task_max_iterations, 30);
        assert_eq!(config.verify_commands, vec!["cargo test"]);
        assert!(config.auto_commit_per_cycle);
        assert_eq!(config.completion_promise, "GOAL_COMPLETE");
        assert!(config.enable_drift_detection);
    }

    #[test]
    fn 阶段显示名称正确() {
        assert_eq!(GoalPhase::Planning.display_name(), "制定计划");
        assert_eq!(GoalPhase::Executing.display_name(), "执行任务");
        assert_eq!(GoalPhase::Verifying.display_name(), "验证结果");
        assert_eq!(GoalPhase::Done.display_name(), "已完成");
    }

    #[test]
    fn 任务状态序列化往返() {
        let status = GoalTaskStatus::Failed {
            reason: "编译错误".to_string(),
        };
        let json = serde_json::to_string(&status).expect("序列化失败");
        let deserialized: GoalTaskStatus =
            serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(status, deserialized);
    }

    #[test]
    fn 计划可序列化() {
        let plan = GoalPlan {
            requirement: "实现用户登录功能".to_string(),
            tasks: vec![GoalTask {
                id: "task-1".to_string(),
                title: "创建登录页面".to_string(),
                description: "实现用户登录界面".to_string(),
                target_files: vec!["src/login.rs".to_string()],
                acceptance_criteria: vec!["页面可渲染".to_string()],
                depends_on: vec![],
                status: GoalTaskStatus::Pending,
            }],
            acceptance_criteria: vec!["用户可以成功登录".to_string()],
            cycle: 1,
            forbidden_detours: vec!["不要修改数据库模式".to_string()],
            context: "项目使用 Actix-web 框架".to_string(),
        };
        let json = serde_json::to_string(&plan).expect("序列化失败");
        let deserialized: GoalPlan =
            serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(plan.requirement, deserialized.requirement);
        assert_eq!(plan.tasks.len(), deserialized.tasks.len());
        assert_eq!(plan.tasks[0].id, deserialized.tasks[0].id);
        assert_eq!(plan.cycle, deserialized.cycle);
        assert_eq!(
            plan.forbidden_detours,
            deserialized.forbidden_detours
        );
        assert_eq!(plan.context, deserialized.context);
    }
}
