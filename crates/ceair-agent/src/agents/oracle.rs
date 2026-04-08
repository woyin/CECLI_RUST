//! # Oracle Agent — 架构顾问
//!
//! Oracle 是系统的架构顾问 Agent，提供只读的代码分析、架构审查
//! 和技术咨询服务。它使用 GPT-5.4 的 high 变体以获得最佳分析质量。
//! Oracle 不可修改文件，也不可委派任务。

use super::{AgentDefinition, AgentKind};
use std::collections::HashSet;

/// 系统提示词常量 — 描述 Oracle 的角色和行为准则
const SYSTEM_PROMPT: &str = "\
你是 Oracle，系统的架构顾问 Agent。你的核心职责是：

1. **架构分析**：深入分析代码库的架构设计，识别模式、反模式和潜在的架构债务。
   评估系统的可扩展性、可维护性和性能特征。
2. **代码审查**：对代码变更进行专业审查，关注：
   - 设计模式的正确使用
   - SOLID 原则的遵守情况
   - 错误处理的完整性
   - 安全漏洞和潜在风险
   - 性能瓶颈和优化机会
3. **技术咨询**：针对技术选型、架构决策提供专业建议，
   给出利弊分析和推荐方案。
4. **依赖分析**：评估项目依赖的健康状况，识别过时、不安全
   或冗余的依赖项。
5. **最佳实践**：推荐行业最佳实践，帮助团队提升代码质量和开发效率。

**约束**：
- 你是严格只读的 Agent，不能写入或编辑任何文件
- 你不能委派任务给其他 Agent
- 你的输出仅为分析报告和建议，不包含直接的代码修改
- write、edit、task、call_omo_agent 工具均被阻止

**分析原则**：
- 基于事实和数据，避免主观臆断
- 提供可操作的具体建议，而非泛泛而谈
- 优先关注高影响的问题
- 考虑项目的具体上下文和约束条件";

/// Oracle Agent — 架构顾问实例
///
/// 只读分析 Agent，使用 GPT-5.4 high 变体。不可写入、编辑或委派。
pub struct OracleAgent;

impl OracleAgent {
    /// 创建新的 Oracle Agent 实例
    pub fn new() -> Self {
        Self
    }
}

impl AgentDefinition for OracleAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Oracle
    }

    /// 默认使用 GPT-5.4 — 强大的分析和推理能力
    fn default_model(&self) -> &str {
        "gpt-5.4"
    }

    /// 使用 high 变体 — 获得最高质量的分析输出
    fn default_variant(&self) -> Option<&str> {
        Some("high")
    }

    fn fallback_models(&self) -> Vec<String> {
        vec![
            "claude-opus-4-6".to_string(),
            "claude-sonnet-4-6".to_string(),
        ]
    }

    /// 阻止写入、编辑和委派相关工具
    fn blocked_tools(&self) -> HashSet<String> {
        let mut blocked = HashSet::new();
        blocked.insert("write".to_string());
        blocked.insert("edit".to_string());
        blocked.insert("task".to_string());
        blocked.insert("call_omo_agent".to_string());
        blocked
    }

    fn system_prompt(&self) -> &str {
        SYSTEM_PROMPT
    }

    fn display_name(&self) -> &str {
        "Oracle"
    }

    fn description(&self) -> &str {
        "架构顾问 — 只读分析、代码审查"
    }

    /// Oracle 是只读 Agent
    fn is_read_only(&self) -> bool {
        true
    }

    /// Oracle 不可委派任务
    fn can_delegate(&self) -> bool {
        false
    }
}
