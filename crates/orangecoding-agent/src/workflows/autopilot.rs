//! # Autopilot 长任务全自动模式
//!
//! 实现 Plan → Execute → Verify → Replan 的循环执行模式。
//! 用户输入大需求，系统自动完成所有工作。
//!
//! ## 触发方式
//!
//! - CLI: `orangecoding launch --autopilot "需求描述"`
//! - 斜杠命令: `/autopilot 需求描述`
//!
//! ## 循环流程
//!
//! ```text
//! Analyzing → Planning → Executing → Verifying
//!                ▲                        │
//!                │    验证失败             │
//!                └────────────────────────┘
//!                                 │ 验证通过
//!                                 ▼
//!                              Done
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================
// Autopilot 阶段
// ============================================================

/// Autopilot 执行阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AutopilotPhase {
    /// 分析需求，准备第一轮计划
    Analyzing,
    /// 生成/重新生成执行计划
    Planning,
    /// 按计划执行任务
    Executing,
    /// 验证执行结果（测试运行 + AI 评估）
    Verifying,
    /// 全部完成
    Done,
    /// 不可恢复的错误
    Failed,
}

impl AutopilotPhase {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Analyzing => "分析需求",
            Self::Planning => "制定计划",
            Self::Executing => "执行任务",
            Self::Verifying => "验证结果",
            Self::Done => "已完成",
            Self::Failed => "失败",
        }
    }
}

// ============================================================
// Autopilot 配置
// ============================================================

/// Autopilot 模式配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutopilotConfig {
    /// 初始循环软预算（默认 10，耗尽后自动扩展）
    pub max_cycles: u32,
    /// 严格验证模式：所有验收标准必须通过 + 测试全部通过
    pub verify_strict: bool,
    /// 每个任务内部的 AI 初始迭代软预算
    pub task_max_iterations: u32,
    /// 每个任务超时秒数
    pub task_timeout_secs: u64,
    /// 每轮结束后暂停等待用户确认
    pub pause_between_cycles: bool,
    /// 是否自动运行测试
    pub auto_run_tests: bool,
    /// 自定义验证命令（覆盖默认的 cargo test）
    pub verify_commands: Vec<String>,
    /// 每轮完成后是否自动 git commit
    pub auto_commit_per_cycle: bool,
}

impl Default for AutopilotConfig {
    fn default() -> Self {
        Self {
            max_cycles: 10,
            verify_strict: false,
            task_max_iterations: 30,
            task_timeout_secs: 600,
            pause_between_cycles: false,
            auto_run_tests: true,
            verify_commands: vec![],
            auto_commit_per_cycle: false,
        }
    }
}

// ============================================================
// Autopilot 任务与计划
// ============================================================

/// 任务执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// 单个 Autopilot 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutopilotTask {
    pub id: String,
    pub description: String,
    pub target_files: Vec<PathBuf>,
    pub acceptance_criteria: Vec<String>,
    pub depends_on: Vec<String>,
    pub status: TaskStatus,
}

/// Autopilot 执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutopilotPlan {
    pub requirement: String,
    pub tasks: Vec<AutopilotTask>,
    pub acceptance_criteria: Vec<String>,
    pub cycle: u32,
}

// ============================================================
// 验证报告
// ============================================================

/// 测试运行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunResult {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub failed_details: Vec<String>,
    pub build_passed: bool,
}

impl Default for TestRunResult {
    fn default() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            failed_details: vec![],
            build_passed: true,
        }
    }
}

/// 单条验收标准评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    pub criterion: String,
    pub passed: bool,
    pub evidence: String,
}

/// 验证报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub cycle: u32,
    pub test_results: TestRunResult,
    pub criteria_results: Vec<CriterionResult>,
    pub passed: bool,
    pub failure_summary: Option<String>,
    pub suggestions: Vec<String>,
}

// ============================================================
// Autopilot 状态机
// ============================================================

/// Autopilot 长任务模式控制器
#[derive(Debug, Clone)]
pub struct AutopilotMode {
    is_active: bool,
    phase: AutopilotPhase,
    config: AutopilotConfig,
    current_cycle: u32,
    plan: Option<AutopilotPlan>,
    last_verification: Option<VerificationReport>,
    requirement: String,
}

impl AutopilotMode {
    pub fn new(requirement: String) -> Self {
        Self {
            is_active: false,
            phase: AutopilotPhase::Analyzing,
            config: AutopilotConfig::default(),
            current_cycle: 0,
            plan: None,
            last_verification: None,
            requirement,
        }
    }

    pub fn with_config(requirement: String, config: AutopilotConfig) -> Self {
        Self {
            is_active: false,
            phase: AutopilotPhase::Analyzing,
            config,
            current_cycle: 0,
            plan: None,
            last_verification: None,
            requirement,
        }
    }

    /// 激活 Autopilot 模式
    pub fn activate(&mut self) {
        self.is_active = true;
        self.phase = AutopilotPhase::Analyzing;
        self.current_cycle = 0;
    }

    /// 停止 Autopilot
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// 推进到下一阶段，返回是否成功
    pub fn advance(&mut self) -> bool {
        if !self.is_active {
            return false;
        }

        let next = match self.phase {
            AutopilotPhase::Analyzing => Some(AutopilotPhase::Planning),
            AutopilotPhase::Planning => Some(AutopilotPhase::Executing),
            AutopilotPhase::Executing => Some(AutopilotPhase::Verifying),
            AutopilotPhase::Verifying => {
                if let Some(ref report) = self.last_verification {
                    if report.passed {
                        Some(AutopilotPhase::Done)
                    } else {
                        if self.current_cycle >= self.config.max_cycles {
                            self.extend_cycle_budget();
                        }
                        self.current_cycle += 1;
                        Some(AutopilotPhase::Planning)
                    }
                } else {
                    Some(AutopilotPhase::Done)
                }
            }
            AutopilotPhase::Done | AutopilotPhase::Failed => None,
        };

        match next {
            Some(phase) => {
                self.phase = phase;
                if phase == AutopilotPhase::Done || phase == AutopilotPhase::Failed {
                    self.is_active = false;
                }
                true
            }
            None => false,
        }
    }

    // -- Getters & Setters --

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn current_phase(&self) -> AutopilotPhase {
        self.phase
    }

    pub fn current_cycle(&self) -> u32 {
        self.current_cycle
    }

    pub fn config(&self) -> &AutopilotConfig {
        &self.config
    }

    pub fn requirement(&self) -> &str {
        &self.requirement
    }

    pub fn plan(&self) -> Option<&AutopilotPlan> {
        self.plan.as_ref()
    }

    pub fn set_plan(&mut self, plan: AutopilotPlan) {
        self.plan = Some(plan);
    }

    pub fn last_verification(&self) -> Option<&VerificationReport> {
        self.last_verification.as_ref()
    }

    pub fn set_verification(&mut self, report: VerificationReport) {
        self.last_verification = Some(report);
    }

    fn extend_cycle_budget(&mut self) {
        let extension = (self.config.max_cycles.saturating_add(1) / 2).max(1);
        self.config.max_cycles = self.config.max_cycles.saturating_add(extension);
    }

    /// 生成状态摘要（用于事件上报）
    pub fn status_summary(&self) -> String {
        format!(
            "Autopilot [{}] Cycle {}/{} — {}",
            self.phase.display_name(),
            self.current_cycle,
            self.config.max_cycles,
            if let Some(ref plan) = self.plan {
                let done = plan
                    .tasks
                    .iter()
                    .filter(|t| t.status == TaskStatus::Completed)
                    .count();
                let total = plan.tasks.len();
                format!("Tasks: {done}/{total}")
            } else {
                "No plan yet".to_string()
            }
        )
    }

    /// 是否已完成
    pub fn is_complete(&self) -> bool {
        matches!(self.phase, AutopilotPhase::Done)
    }

    /// 是否失败
    pub fn is_failed(&self) -> bool {
        matches!(self.phase, AutopilotPhase::Failed)
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 新建模式默认未激活() {
        let mode = AutopilotMode::new("test".into());
        assert!(!mode.is_active());
        assert_eq!(mode.current_phase(), AutopilotPhase::Analyzing);
        assert_eq!(mode.current_cycle(), 0);
    }

    #[test]
    fn 激活后可推进() {
        let mut mode = AutopilotMode::new("test".into());
        mode.activate();
        assert!(mode.is_active());
        assert!(mode.advance()); // Analyzing → Planning
        assert_eq!(mode.current_phase(), AutopilotPhase::Planning);
    }

    #[test]
    fn 未激活时无法推进() {
        let mut mode = AutopilotMode::new("test".into());
        assert!(!mode.advance());
    }

    #[test]
    fn 完整循环_验证通过() {
        let mut mode = AutopilotMode::new("test".into());
        mode.activate();

        mode.advance(); // → Planning
        mode.advance(); // → Executing
        mode.advance(); // → Verifying

        // 设置验证通过
        mode.set_verification(VerificationReport {
            cycle: 1,
            test_results: TestRunResult::default(),
            criteria_results: vec![],
            passed: true,
            failure_summary: None,
            suggestions: vec![],
        });

        mode.advance(); // → Done
        assert!(mode.is_complete());
        assert!(!mode.is_active());
    }

    #[test]
    fn 验证失败触发重计划() {
        let mut mode = AutopilotMode::new("test".into());
        mode.activate();

        mode.advance(); // → Planning
        mode.advance(); // → Executing
        mode.advance(); // → Verifying

        mode.set_verification(VerificationReport {
            cycle: 1,
            test_results: TestRunResult {
                total: 10,
                passed: 8,
                failed: 2,
                failed_details: vec!["test_a failed".into()],
                build_passed: true,
            },
            criteria_results: vec![],
            passed: false,
            failure_summary: Some("2 tests failed".into()),
            suggestions: vec!["Fix test_a".into()],
        });

        mode.advance(); // → Planning (replan)
        assert_eq!(mode.current_phase(), AutopilotPhase::Planning);
        assert_eq!(mode.current_cycle(), 1);
        assert!(mode.is_active());
    }

    #[test]
    fn 超过初始循环预算会扩展并继续() {
        let mut config = AutopilotConfig::default();
        config.max_cycles = 2;
        let mut mode = AutopilotMode::with_config("test".into(), config);
        mode.activate();

        // Cycle 1
        mode.advance(); // → Planning
        mode.advance(); // → Executing
        mode.advance(); // → Verifying
        mode.set_verification(VerificationReport {
            cycle: 1,
            test_results: TestRunResult::default(),
            criteria_results: vec![],
            passed: false,
            failure_summary: Some("fail".into()),
            suggestions: vec![],
        });
        mode.advance(); // → Planning (replan, cycle=1)

        // Cycle 2
        mode.advance(); // → Executing
        mode.advance(); // → Verifying
        mode.set_verification(VerificationReport {
            cycle: 2,
            test_results: TestRunResult::default(),
            criteria_results: vec![],
            passed: false,
            failure_summary: Some("fail again".into()),
            suggestions: vec![],
        });
        mode.advance(); // → Planning (replan, cycle=2)

        assert_eq!(mode.current_phase(), AutopilotPhase::Planning);
        mode.advance(); // → Executing
        mode.advance(); // → Verifying
        mode.set_verification(VerificationReport {
            cycle: 2,
            test_results: TestRunResult::default(),
            criteria_results: vec![],
            passed: false,
            failure_summary: Some("still failing".into()),
            suggestions: vec![],
        });
        mode.advance(); // → Planning after extending cycle budget
        assert!(mode.is_active());
        assert_eq!(mode.current_phase(), AutopilotPhase::Planning);
        assert_eq!(mode.current_cycle(), 3);
        assert!(mode.config().max_cycles > 2);
    }

    #[test]
    fn 循环预算耗尽时扩展而不是失败() {
        let mut config = AutopilotConfig::default();
        config.max_cycles = 1;
        let mut mode = AutopilotMode::with_config("test".into(), config);
        mode.activate();

        mode.advance(); // → Planning
        mode.advance(); // → Executing
        mode.advance(); // → Verifying
        mode.set_verification(VerificationReport {
            cycle: 0,
            test_results: TestRunResult::default(),
            criteria_results: vec![],
            passed: false,
            failure_summary: Some("需要继续重试".into()),
            suggestions: vec![],
        });

        mode.advance(); // → Planning, cycle=1
        mode.advance(); // → Executing
        mode.advance(); // → Verifying
        mode.advance(); // budget exhausted, should extend and replan

        assert!(mode.is_active());
        assert_eq!(mode.current_phase(), AutopilotPhase::Planning);
        assert_eq!(mode.current_cycle(), 2);
        assert!(mode.config().max_cycles > 1);
    }

    #[test]
    fn 默认配置合理() {
        let config = AutopilotConfig::default();
        assert_eq!(config.max_cycles, 10);
        assert!(!config.verify_strict);
        assert_eq!(config.task_max_iterations, 30);
        assert_eq!(config.task_timeout_secs, 600);
        assert!(!config.pause_between_cycles);
        assert!(config.auto_run_tests);
        assert!(config.verify_commands.is_empty());
        assert!(!config.auto_commit_per_cycle);
    }

    #[test]
    fn 状态摘要包含关键信息() {
        let mut mode = AutopilotMode::new("Build auth system".into());
        mode.activate();
        mode.advance(); // → Planning

        let summary = mode.status_summary();
        assert!(summary.contains("制定计划"));
        assert!(summary.contains("0/10"));
    }

    #[test]
    fn 停止后不再推进() {
        let mut mode = AutopilotMode::new("test".into());
        mode.activate();
        mode.deactivate();
        assert!(!mode.advance());
    }
}
