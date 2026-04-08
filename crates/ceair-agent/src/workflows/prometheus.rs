//! # Prometheus 战略规划工作流
//!
//! 实现从用户意图到可执行计划的完整规划流程。
//!
//! ## 状态机流转
//!
//! ```text
//! Interview → ClearanceCheck → PlanGeneration → MetisConsult → MomusReview → Done
//! ```
//!
//! ## 核心原则
//!
//! - Prometheus 为 **只读** 角色，仅在 `.sisyphus/` 目录下创建/修改文件
//! - 规划输出目录：`.sisyphus/plans/`
//! - 所有计划须经 Momus 审查后才算完成

use serde::{Deserialize, Serialize};

// ============================================================
// 状态机定义
// ============================================================

/// Prometheus 工作流状态机阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrometheusPhase {
    /// 访谈阶段：收集需求与约束
    Interview,
    /// 放行检查：验证访谈信息完整性
    ClearanceCheck,
    /// 计划生成：基于访谈结果生成结构化计划
    PlanGeneration,
    /// Metis 咨询：向 Metis 征求规划建议
    MetisConsult,
    /// Momus 审查：由 Momus 对计划进行批判性审查
    MomusReview,
    /// 完成：计划已通过审查
    Done,
}

// ============================================================
// 意图策略
// ============================================================

/// 意图驱动的规划策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentStrategy {
    /// 重构：对现有代码进行结构性改进
    Refactoring,
    /// 从零构建：全新功能或项目
    BuildFromScratch,
    /// 中等规模任务：常规功能开发
    MidSizedTask,
    /// 架构设计：系统级架构变更
    Architecture,
}

// ============================================================
// 访谈状态
// ============================================================

/// 访谈阶段的完成度追踪
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterviewState {
    /// 核心目标是否已明确定义
    pub core_objective_defined: bool,
    /// 作用域边界是否已确定
    pub scope_boundaries_established: bool,
    /// 是否没有关键歧义
    pub no_critical_ambiguities: bool,
    /// 技术方案是否已决定
    pub technical_approach_decided: bool,
    /// 测试策略是否已确认
    pub test_strategy_confirmed: bool,
}

impl InterviewState {
    /// 创建一个全部未完成的初始访谈状态
    pub fn new() -> Self {
        Self {
            core_objective_defined: false,
            scope_boundaries_established: false,
            no_critical_ambiguities: false,
            technical_approach_decided: false,
            test_strategy_confirmed: false,
        }
    }

    /// 检查所有字段是否都已确认（放行条件）
    pub fn all_confirmed(&self) -> bool {
        self.core_objective_defined
            && self.scope_boundaries_established
            && self.no_critical_ambiguities
            && self.technical_approach_decided
            && self.test_strategy_confirmed
    }

    /// 返回未完成的字段名称列表
    pub fn pending_items(&self) -> Vec<&'static str> {
        let mut items = Vec::new();
        if !self.core_objective_defined {
            items.push("core_objective_defined");
        }
        if !self.scope_boundaries_established {
            items.push("scope_boundaries_established");
        }
        if !self.no_critical_ambiguities {
            items.push("no_critical_ambiguities");
        }
        if !self.technical_approach_decided {
            items.push("technical_approach_decided");
        }
        if !self.test_strategy_confirmed {
            items.push("test_strategy_confirmed");
        }
        items
    }

    /// 已完成的字段数量
    pub fn confirmed_count(&self) -> usize {
        5 - self.pending_items().len()
    }
}

impl Default for InterviewState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 计划文档
// ============================================================

/// 计划任务定义
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanTask {
    /// 任务唯一标识
    pub id: String,
    /// 任务标题
    pub title: String,
    /// 任务详细描述
    pub description: String,
    /// 涉及的文件引用
    pub file_references: Vec<String>,
    /// 验收标准
    pub acceptance_criteria: Vec<String>,
    /// 依赖的前置任务 ID 列表
    pub depends_on: Vec<String>,
}

/// 计划文档 — Prometheus 的最终输出
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDocument {
    /// 计划名称
    pub name: String,
    /// 任务列表
    pub tasks: Vec<PlanTask>,
    /// 总体验收标准
    pub acceptance_criteria: Vec<String>,
    /// 是否已通过 Momus 审查
    pub verified_by_momus: bool,
}

impl PlanDocument {
    /// 创建新的计划文档
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tasks: Vec::new(),
            acceptance_criteria: Vec::new(),
            verified_by_momus: false,
        }
    }

    /// 添加任务到计划
    pub fn add_task(&mut self, task: PlanTask) {
        self.tasks.push(task);
    }

    /// 获取没有依赖的根任务
    pub fn root_tasks(&self) -> Vec<&PlanTask> {
        self.tasks
            .iter()
            .filter(|t| t.depends_on.is_empty())
            .collect()
    }

    /// 获取指定任务的直接后继任务
    pub fn dependents_of(&self, task_id: &str) -> Vec<&PlanTask> {
        self.tasks
            .iter()
            .filter(|t| t.depends_on.iter().any(|dep| dep == task_id))
            .collect()
    }

    /// 计划输出目录
    pub fn output_dir() -> &'static str {
        ".sisyphus/plans/"
    }
}

// ============================================================
// Prometheus 工作流
// ============================================================

/// Prometheus 战略规划工作流
///
/// 状态机驱动的规划流程，从访谈到审查完成。
/// Prometheus 为只读角色，仅在 `.sisyphus/` 目录下操作。
#[derive(Debug, Clone)]
pub struct PrometheusWorkflow {
    /// 当前阶段
    phase: PrometheusPhase,
    /// 访谈状态追踪
    interview_state: InterviewState,
    /// 意图策略
    strategy: Option<IntentStrategy>,
    /// 生成的计划文档
    plan: Option<PlanDocument>,
}

impl PrometheusWorkflow {
    /// 创建新的 Prometheus 工作流实例
    pub fn new() -> Self {
        Self {
            phase: PrometheusPhase::Interview,
            interview_state: InterviewState::new(),
            strategy: None,
            plan: None,
        }
    }

    /// 获取当前阶段
    pub fn phase(&self) -> PrometheusPhase {
        self.phase
    }

    /// 获取访谈状态的引用
    pub fn interview_state(&self) -> &InterviewState {
        &self.interview_state
    }

    /// 获取访谈状态的可变引用
    pub fn interview_state_mut(&mut self) -> &mut InterviewState {
        &mut self.interview_state
    }

    /// 设置意图策略
    pub fn set_strategy(&mut self, strategy: IntentStrategy) {
        self.strategy = Some(strategy);
    }

    /// 获取当前策略
    pub fn strategy(&self) -> Option<IntentStrategy> {
        self.strategy
    }

    /// 获取计划文档引用
    pub fn plan(&self) -> Option<&PlanDocument> {
        self.plan.as_ref()
    }

    /// 执行放行检查，若访谈完整则自动推进到 PlanGeneration
    pub fn perform_clearance_check(&mut self) -> Result<(), Vec<&'static str>> {
        if self.phase != PrometheusPhase::Interview
            && self.phase != PrometheusPhase::ClearanceCheck
        {
            return Ok(());
        }

        self.phase = PrometheusPhase::ClearanceCheck;

        if self.interview_state.all_confirmed() {
            self.phase = PrometheusPhase::PlanGeneration;
            Ok(())
        } else {
            let pending = self.interview_state.pending_items();
            Err(pending)
        }
    }

    /// 设置计划文档并推进到 MetisConsult 阶段
    pub fn submit_plan(&mut self, plan: PlanDocument) -> bool {
        if self.phase != PrometheusPhase::PlanGeneration {
            return false;
        }
        self.plan = Some(plan);
        self.phase = PrometheusPhase::MetisConsult;
        true
    }

    /// 完成 Metis 咨询，推进到 MomusReview
    pub fn complete_metis_consult(&mut self) -> bool {
        if self.phase != PrometheusPhase::MetisConsult {
            return false;
        }
        self.phase = PrometheusPhase::MomusReview;
        true
    }

    /// 完成 Momus 审查，标记计划为已验证并推进到 Done
    pub fn complete_momus_review(&mut self, approved: bool) -> bool {
        if self.phase != PrometheusPhase::MomusReview {
            return false;
        }

        if approved {
            if let Some(ref mut plan) = self.plan {
                plan.verified_by_momus = true;
            }
            self.phase = PrometheusPhase::Done;
            true
        } else {
            // 审查未通过，回退到 PlanGeneration 重新制定
            self.phase = PrometheusPhase::PlanGeneration;
            false
        }
    }

    /// 工作流是否已完成
    pub fn is_done(&self) -> bool {
        self.phase == PrometheusPhase::Done
    }

    /// 推进到下一个阶段（通用方法，需满足前置条件）
    pub fn advance(&mut self) -> PrometheusPhase {
        match self.phase {
            PrometheusPhase::Interview => {
                if self.interview_state.all_confirmed() {
                    self.phase = PrometheusPhase::ClearanceCheck;
                }
            }
            PrometheusPhase::ClearanceCheck => {
                if self.interview_state.all_confirmed() {
                    self.phase = PrometheusPhase::PlanGeneration;
                }
            }
            PrometheusPhase::PlanGeneration => {
                if self.plan.is_some() {
                    self.phase = PrometheusPhase::MetisConsult;
                }
            }
            PrometheusPhase::MetisConsult => {
                self.phase = PrometheusPhase::MomusReview;
            }
            PrometheusPhase::MomusReview => {
                if self
                    .plan
                    .as_ref()
                    .map_or(false, |p| p.verified_by_momus)
                {
                    self.phase = PrometheusPhase::Done;
                }
            }
            PrometheusPhase::Done => {}
        }
        self.phase
    }
}

impl Default for PrometheusWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助函数：创建全部确认的访谈状态
    fn fully_confirmed_interview() -> InterviewState {
        InterviewState {
            core_objective_defined: true,
            scope_boundaries_established: true,
            no_critical_ambiguities: true,
            technical_approach_decided: true,
            test_strategy_confirmed: true,
        }
    }

    /// 辅助函数：创建示例计划文档
    fn sample_plan() -> PlanDocument {
        let mut plan = PlanDocument::new("测试计划");
        plan.add_task(PlanTask {
            id: "task-1".to_string(),
            title: "基础设施搭建".to_string(),
            description: "初始化项目结构".to_string(),
            file_references: vec!["src/lib.rs".to_string()],
            acceptance_criteria: vec!["编译通过".to_string()],
            depends_on: vec![],
        });
        plan.add_task(PlanTask {
            id: "task-2".to_string(),
            title: "核心逻辑实现".to_string(),
            description: "实现核心业务逻辑".to_string(),
            file_references: vec!["src/core.rs".to_string()],
            acceptance_criteria: vec!["单元测试通过".to_string()],
            depends_on: vec!["task-1".to_string()],
        });
        plan
    }

    // --- 访谈状态测试 ---

    #[test]
    fn 新建访谈状态全部为未确认() {
        let state = InterviewState::new();
        assert!(!state.all_confirmed());
        assert_eq!(state.pending_items().len(), 5);
        assert_eq!(state.confirmed_count(), 0);
    }

    #[test]
    fn 部分确认的访谈状态() {
        let mut state = InterviewState::new();
        state.core_objective_defined = true;
        state.scope_boundaries_established = true;

        assert!(!state.all_confirmed());
        assert_eq!(state.confirmed_count(), 2);
        assert_eq!(state.pending_items().len(), 3);
    }

    #[test]
    fn 全部确认的访谈状态() {
        let state = fully_confirmed_interview();
        assert!(state.all_confirmed());
        assert!(state.pending_items().is_empty());
        assert_eq!(state.confirmed_count(), 5);
    }

    #[test]
    fn 默认访谈状态等同于新建() {
        let default_state = InterviewState::default();
        let new_state = InterviewState::new();
        assert_eq!(default_state, new_state);
    }

    // --- 放行检查测试 ---

    #[test]
    fn 放行检查_访谈未完成时返回错误() {
        let mut wf = PrometheusWorkflow::new();
        let result = wf.perform_clearance_check();
        assert!(result.is_err());
        let pending = result.unwrap_err();
        assert_eq!(pending.len(), 5);
        assert_eq!(wf.phase(), PrometheusPhase::ClearanceCheck);
    }

    #[test]
    fn 放行检查_访谈完成后自动推进() {
        let mut wf = PrometheusWorkflow::new();
        *wf.interview_state_mut() = fully_confirmed_interview();
        let result = wf.perform_clearance_check();
        assert!(result.is_ok());
        assert_eq!(wf.phase(), PrometheusPhase::PlanGeneration);
    }

    // --- 状态机流转测试 ---

    #[test]
    fn 初始阶段为访谈() {
        let wf = PrometheusWorkflow::new();
        assert_eq!(wf.phase(), PrometheusPhase::Interview);
        assert!(!wf.is_done());
    }

    #[test]
    fn 完整工作流流转() {
        let mut wf = PrometheusWorkflow::new();

        // 访谈阶段 → 填写所有字段
        *wf.interview_state_mut() = fully_confirmed_interview();

        // 放行检查通过 → PlanGeneration
        assert!(wf.perform_clearance_check().is_ok());
        assert_eq!(wf.phase(), PrometheusPhase::PlanGeneration);

        // 提交计划 → MetisConsult
        assert!(wf.submit_plan(sample_plan()));
        assert_eq!(wf.phase(), PrometheusPhase::MetisConsult);

        // 完成 Metis 咨询 → MomusReview
        assert!(wf.complete_metis_consult());
        assert_eq!(wf.phase(), PrometheusPhase::MomusReview);

        // Momus 审查通过 → Done
        assert!(wf.complete_momus_review(true));
        assert_eq!(wf.phase(), PrometheusPhase::Done);
        assert!(wf.is_done());

        // 验证计划已标记为审查通过
        assert!(wf.plan().unwrap().verified_by_momus);
    }

    #[test]
    fn 审查未通过时回退到计划生成() {
        let mut wf = PrometheusWorkflow::new();
        *wf.interview_state_mut() = fully_confirmed_interview();
        wf.perform_clearance_check().unwrap();
        wf.submit_plan(sample_plan());
        wf.complete_metis_consult();

        // 审查不通过
        let result = wf.complete_momus_review(false);
        assert!(!result);
        assert_eq!(wf.phase(), PrometheusPhase::PlanGeneration);
        assert!(!wf.is_done());
    }

    #[test]
    fn 在错误阶段提交计划无效() {
        let mut wf = PrometheusWorkflow::new();
        assert!(!wf.submit_plan(sample_plan()));
        assert_eq!(wf.phase(), PrometheusPhase::Interview);
    }

    #[test]
    fn 在错误阶段完成咨询无效() {
        let mut wf = PrometheusWorkflow::new();
        assert!(!wf.complete_metis_consult());
        assert_eq!(wf.phase(), PrometheusPhase::Interview);
    }

    #[test]
    fn 在错误阶段完成审查无效() {
        let mut wf = PrometheusWorkflow::new();
        assert!(!wf.complete_momus_review(true));
        assert_eq!(wf.phase(), PrometheusPhase::Interview);
    }

    // --- 计划文档测试 ---

    #[test]
    fn 计划文档根任务查询() {
        let plan = sample_plan();
        let roots = plan.root_tasks();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, "task-1");
    }

    #[test]
    fn 计划文档后继任务查询() {
        let plan = sample_plan();
        let deps = plan.dependents_of("task-1");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].id, "task-2");
    }

    #[test]
    fn 计划输出目录正确() {
        assert_eq!(PlanDocument::output_dir(), ".sisyphus/plans/");
    }

    // --- 策略与杂项测试 ---

    #[test]
    fn 设置和获取策略() {
        let mut wf = PrometheusWorkflow::new();
        assert!(wf.strategy().is_none());
        wf.set_strategy(IntentStrategy::Refactoring);
        assert_eq!(wf.strategy(), Some(IntentStrategy::Refactoring));
    }

    #[test]
    fn 通用推进方法_在访谈阶段未完成时不变() {
        let mut wf = PrometheusWorkflow::new();
        let phase = wf.advance();
        assert_eq!(phase, PrometheusPhase::Interview);
    }

    #[test]
    fn 默认工作流等同于新建() {
        let default_wf = PrometheusWorkflow::default();
        assert_eq!(default_wf.phase(), PrometheusPhase::Interview);
        assert!(default_wf.plan().is_none());
        assert!(default_wf.strategy().is_none());
    }

    #[test]
    fn 序列化反序列化计划文档() {
        let plan = sample_plan();
        let json = serde_json::to_string(&plan).expect("序列化失败");
        let deserialized: PlanDocument =
            serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(plan, deserialized);
    }
}
