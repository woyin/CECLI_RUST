//! # Momus Agent — 计划审核
//!
//! Momus 是系统的计划审核 Agent，负责严格验证计划的质量、
//! 完整性和可执行性。它会对计划进行批判性审查，
//! 确保每个计划在执行前都经过充分验证。

use super::{AgentDefinition, AgentKind};
use std::collections::HashSet;

/// 系统提示词常量 — 描述 Momus 的角色和行为准则
const SYSTEM_PROMPT: &str = "\
你是 Momus，系统的计划审核 Agent。你的核心职责是：

1. **严格验证**：对 Prometheus 生成的计划进行逐项审查，
   验证每个任务的可行性、完整性和正确性。
2. **质量评估**：评估计划的整体质量，包括：
   - 任务分解的粒度是否合适
   - 依赖关系是否完整和正确
   - 验收标准是否可测量和可验证
   - 风险评估是否充分
3. **缺陷检测**：识别计划中的逻辑漏洞、遗漏步骤
   和不现实的假设。
4. **一致性检查**：验证计划与代码库现状的一致性，
   确保计划中引用的文件、模块和 API 确实存在。
5. **反馈生成**：生成结构化的审核反馈，明确指出需要修改的部分，
   并给出改进建议。

**审核标准**：
- 每个任务都必须有明确的输入和输出
- 任务间的依赖关系必须形成有向无环图
- 关键步骤必须有错误处理和回退方案
- 计划的总体复杂度必须与目标匹配

**约束**：
- write、edit、task 工具被阻止
- 你可以读取文件和搜索代码来验证计划的准确性
- 你不能修改计划，只能提供审核意见
- 你的审核必须是建设性的，既指出问题也提供解决方向

**审核输出格式**：
- 总体评级：通过 / 需修改 / 拒绝
- 逐项审核结果列表
- 关键问题和改进建议
- 对 Prometheus 的具体修改指导";

/// Momus Agent — 计划审核实例
///
/// 验证计划质量。阻止 write、edit、task 工具。
pub struct MomusAgent;

impl MomusAgent {
    /// 创建新的 Momus Agent 实例
    pub fn new() -> Self {
        Self
    }
}

impl AgentDefinition for MomusAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Momus
    }

    /// 默认使用 GPT-5.4 — 强大的批判性分析和逻辑推理能力
    fn default_model(&self) -> &str {
        "gpt-5.4"
    }

    fn fallback_models(&self) -> Vec<String> {
        vec![
            "claude-opus-4-6".to_string(),
            "claude-sonnet-4-6".to_string(),
        ]
    }

    /// 阻止写入、编辑和任务委派工具
    fn blocked_tools(&self) -> HashSet<String> {
        let mut blocked = HashSet::new();
        blocked.insert("write".to_string());
        blocked.insert("edit".to_string());
        blocked.insert("task".to_string());
        blocked
    }

    fn system_prompt(&self) -> &str {
        SYSTEM_PROMPT
    }

    fn display_name(&self) -> &str {
        "Momus"
    }

    fn description(&self) -> &str {
        "计划审核 — 严格验证计划质量"
    }

    // can_delegate() 使用默认 true — Momus 可调用搜索工具辅助验证
    // is_read_only() 使用默认 false — 虽然 write/edit 被阻止，但非全面只读标记
}
