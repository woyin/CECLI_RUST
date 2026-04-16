//! # Multimodal Agent — 视觉分析
//!
//! Multimodal（即 Multimodal-Looker）是系统的多模态分析 Agent，
//! 专注于 PDF、图片和图表等视觉内容的分析与理解。
//! 它使用白名单模式，仅允许 `read` 工具以确保安全的只读访问。

use super::{AgentDefinition, AgentKind};
use std::collections::HashSet;

/// 系统提示词常量 — 描述 Multimodal 的角色和行为准则
const SYSTEM_PROMPT: &str = "\
你是 Multimodal（Multimodal-Looker），系统的多模态分析 Agent。你的核心职责是：

1. **图片分析**：分析截图、UI 设计稿、架构图和流程图，
   提取其中的结构信息和设计意图。
2. **PDF 处理**：读取和分析 PDF 文档中的内容，
   包括技术规格书、API 文档和设计文档。
3. **图表解读**：理解性能图表、依赖关系图、UML 图等，
   提取关键数据点和趋势信息。
4. **UI 审查**：分析用户界面截图，识别布局问题、
   可访问性问题和设计一致性问题。
5. **视觉对比**：比较设计稿与实际实现的差异，
   识别不一致的地方。

**分析能力**：
- 识别图片中的文字（OCR）
- 理解图表和数据可视化
- 分析代码截图中的逻辑
- 解读架构图和流程图

**约束**：
- 你使用白名单模式，仅允许使用 `read` 工具
- 所有未列入白名单的工具均被拒绝
- 你只能读取和分析内容，不能修改任何文件
- 你的输出是分析报告和结构化数据

**输出格式**：
- 对每个分析的视觉元素提供结构化描述
- 标注关键发现和需要注意的问题
- 提供文字转录（如适用）
- 给出可操作的建议和改进方向";

/// Multimodal Agent — 视觉分析实例
///
/// 白名单模式 Agent，仅允许 `read` 工具。用于视觉内容分析。
pub struct MultimodalAgent;

impl MultimodalAgent {
    /// 创建新的 Multimodal Agent 实例
    pub fn new() -> Self {
        Self
    }
}

impl AgentDefinition for MultimodalAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Multimodal
    }

    /// 默认使用 GPT-5.4 — 最佳多模态理解能力
    fn default_model(&self) -> &str {
        "gpt-5.4"
    }

    fn fallback_models(&self) -> Vec<String> {
        vec![
            "claude-opus-4-6".to_string(),
            "claude-sonnet-4-6".to_string(),
        ]
    }

    /// 使用白名单模式 — 仅允许 `read` 工具
    fn allowed_tools_only(&self) -> Option<HashSet<String>> {
        let mut allowed = HashSet::new();
        allowed.insert("read".to_string());
        Some(allowed)
    }

    fn system_prompt(&self) -> &str {
        SYSTEM_PROMPT
    }

    fn display_name(&self) -> &str {
        "Multimodal"
    }

    fn description(&self) -> &str {
        "多模态分析 — PDF/图片/图表分析"
    }

    /// Multimodal 是只读 Agent
    fn is_read_only(&self) -> bool {
        true
    }

    /// Multimodal 不可委派任务（白名单不含 task 工具）
    fn can_delegate(&self) -> bool {
        false
    }
}
