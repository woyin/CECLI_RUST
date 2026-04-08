//! # UltraWork 全自动模式
//!
//! 提供完全自主的端到端执行模式，自动扫描、探索、规划、执行和验证。
//!
//! ## 激活方式
//!
//! 通过关键词 `"ultrawork"` 或 `"ulw"` 触发激活。
//!
//! ## 阶段流程
//!
//! ```text
//! Scanning → Exploring → Planning → Executing → Verifying → Done
//! ```
//!
//! ## 集成点
//!
//! - IntentGate：意图分类后决定是否启用 UltraWork
//! - CategoryRegistry：根据阶段自动调整 Agent 类别

use serde::{Deserialize, Serialize};

// ============================================================
// UltraWork 阶段
// ============================================================

/// UltraWork 执行阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UltraWorkPhase {
    /// 扫描：分析项目结构和现状
    Scanning,
    /// 探索：深入理解关键代码路径
    Exploring,
    /// 规划：制定执行计划
    Planning,
    /// 执行：自动执行任务
    Executing,
    /// 验证：检验执行结果
    Verifying,
    /// 完成：所有阶段结束
    Done,
}

impl UltraWorkPhase {
    /// 获取下一个阶段
    pub fn next(self) -> Option<UltraWorkPhase> {
        match self {
            Self::Scanning => Some(Self::Exploring),
            Self::Exploring => Some(Self::Planning),
            Self::Planning => Some(Self::Executing),
            Self::Executing => Some(Self::Verifying),
            Self::Verifying => Some(Self::Done),
            Self::Done => None,
        }
    }

    /// 阶段显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Scanning => "扫描",
            Self::Exploring => "探索",
            Self::Planning => "规划",
            Self::Executing => "执行",
            Self::Verifying => "验证",
            Self::Done => "完成",
        }
    }
}

// ============================================================
// UltraWork 配置
// ============================================================

/// UltraWork 模式配置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UltraWorkConfig {
    /// 最大并行 Agent 数
    pub max_parallel_agents: usize,
    /// 是否启用自纠错
    pub enable_self_correction: bool,
    /// 是否启用深度研究
    pub enable_deep_research: bool,
}

impl Default for UltraWorkConfig {
    fn default() -> Self {
        Self {
            max_parallel_agents: 3,
            enable_self_correction: true,
            enable_deep_research: false,
        }
    }
}

// ============================================================
// 关键词检测
// ============================================================

/// 触发 UltraWork 的关键词列表
const ACTIVATION_KEYWORDS: &[&str] = &["ultrawork", "ulw"];

/// 检查输入文本是否包含 UltraWork 激活关键词
pub fn detect_activation(input: &str) -> bool {
    let lower = input.to_lowercase();
    ACTIVATION_KEYWORDS
        .iter()
        .any(|kw| lower.contains(kw))
}

// ============================================================
// UltraWork 模式
// ============================================================

/// UltraWork 全自动模式控制器
///
/// 管理从扫描到验证的完整自动化流程。
/// 与 IntentGate 和 CategoryRegistry 集成，根据阶段自动调整行为。
#[derive(Debug, Clone)]
pub struct UltraWorkMode {
    /// 是否已激活
    is_active: bool,
    /// 当前执行阶段
    phase: UltraWorkPhase,
    /// 模式配置
    config: UltraWorkConfig,
}

impl UltraWorkMode {
    /// 创建新的 UltraWork 模式（未激活）
    pub fn new() -> Self {
        Self {
            is_active: false,
            phase: UltraWorkPhase::Scanning,
            config: UltraWorkConfig::default(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(config: UltraWorkConfig) -> Self {
        Self {
            is_active: false,
            phase: UltraWorkPhase::Scanning,
            config,
        }
    }

    /// 激活 UltraWork 模式
    pub fn activate(&mut self) {
        self.is_active = true;
        self.phase = UltraWorkPhase::Scanning;
    }

    /// 停用 UltraWork 模式
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// 推进到下一个阶段，返回新阶段；若已完成则返回 None
    pub fn advance_phase(&mut self) -> Option<UltraWorkPhase> {
        if !self.is_active {
            return None;
        }
        match self.phase.next() {
            Some(next) => {
                self.phase = next;
                if next == UltraWorkPhase::Done {
                    self.is_active = false;
                }
                Some(next)
            }
            None => None,
        }
    }

    /// 获取当前阶段
    pub fn current_phase(&self) -> UltraWorkPhase {
        self.phase
    }

    /// 是否已激活
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// 是否已完成所有阶段
    pub fn is_complete(&self) -> bool {
        self.phase == UltraWorkPhase::Done
    }

    /// 获取配置引用
    pub fn config(&self) -> &UltraWorkConfig {
        &self.config
    }

    /// 根据当前阶段建议适合的 Agent 类别
    ///
    /// 这是与 CategoryRegistry 的集成点
    pub fn suggested_category(&self) -> &'static str {
        match self.phase {
            UltraWorkPhase::Scanning => "explore",
            UltraWorkPhase::Exploring => "explore",
            UltraWorkPhase::Planning => "strategy",
            UltraWorkPhase::Executing => "code",
            UltraWorkPhase::Verifying => "test",
            UltraWorkPhase::Done => "none",
        }
    }

    /// 根据当前阶段判断是否应该请求 IntentGate 重新分类
    ///
    /// 这是与 IntentGate 的集成点
    pub fn should_reclassify_intent(&self) -> bool {
        matches!(
            self.phase,
            UltraWorkPhase::Planning | UltraWorkPhase::Executing
        )
    }
}

impl Default for UltraWorkMode {
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

    #[test]
    fn 新建模式默认未激活() {
        let mode = UltraWorkMode::new();
        assert!(!mode.is_active());
        assert!(!mode.is_complete());
        assert_eq!(mode.current_phase(), UltraWorkPhase::Scanning);
    }

    #[test]
    fn 激活和停用() {
        let mut mode = UltraWorkMode::new();
        mode.activate();
        assert!(mode.is_active());
        mode.deactivate();
        assert!(!mode.is_active());
    }

    #[test]
    fn 完整阶段流转() {
        let mut mode = UltraWorkMode::new();
        mode.activate();

        assert_eq!(mode.advance_phase(), Some(UltraWorkPhase::Exploring));
        assert_eq!(mode.advance_phase(), Some(UltraWorkPhase::Planning));
        assert_eq!(mode.advance_phase(), Some(UltraWorkPhase::Executing));
        assert_eq!(mode.advance_phase(), Some(UltraWorkPhase::Verifying));
        assert_eq!(mode.advance_phase(), Some(UltraWorkPhase::Done));

        // Done 阶段自动停用
        assert!(!mode.is_active());
        assert!(mode.is_complete());

        // 无法继续推进
        assert_eq!(mode.advance_phase(), None);
    }

    #[test]
    fn 未激活时无法推进() {
        let mut mode = UltraWorkMode::new();
        assert_eq!(mode.advance_phase(), None);
    }

    #[test]
    fn 关键词检测_ultrawork() {
        assert!(detect_activation("请使用 ultrawork 模式"));
        assert!(detect_activation("ULTRAWORK"));
        assert!(detect_activation("开启ulw"));
    }

    #[test]
    fn 关键词检测_无匹配() {
        assert!(!detect_activation("普通请求"));
        assert!(!detect_activation("ultra 和 work"));
    }

    #[test]
    fn 自定义配置() {
        let config = UltraWorkConfig {
            max_parallel_agents: 5,
            enable_self_correction: false,
            enable_deep_research: true,
        };
        let mode = UltraWorkMode::with_config(config.clone());
        assert_eq!(mode.config().max_parallel_agents, 5);
        assert!(!mode.config().enable_self_correction);
        assert!(mode.config().enable_deep_research);
    }

    #[test]
    fn 阶段建议类别() {
        let mut mode = UltraWorkMode::new();
        mode.activate();
        assert_eq!(mode.suggested_category(), "explore");
        mode.advance_phase(); // Exploring
        assert_eq!(mode.suggested_category(), "explore");
        mode.advance_phase(); // Planning
        assert_eq!(mode.suggested_category(), "strategy");
        mode.advance_phase(); // Executing
        assert_eq!(mode.suggested_category(), "code");
        mode.advance_phase(); // Verifying
        assert_eq!(mode.suggested_category(), "test");
    }

    #[test]
    fn 阶段显示名称() {
        assert_eq!(UltraWorkPhase::Scanning.display_name(), "扫描");
        assert_eq!(UltraWorkPhase::Done.display_name(), "完成");
    }

    #[test]
    fn 意图重分类触发时机() {
        let mut mode = UltraWorkMode::new();
        mode.activate();
        assert!(!mode.should_reclassify_intent()); // Scanning
        mode.advance_phase(); // Exploring
        assert!(!mode.should_reclassify_intent());
        mode.advance_phase(); // Planning
        assert!(mode.should_reclassify_intent());
        mode.advance_phase(); // Executing
        assert!(mode.should_reclassify_intent());
    }

    #[test]
    fn 默认配置合理() {
        let config = UltraWorkConfig::default();
        assert_eq!(config.max_parallel_agents, 3);
        assert!(config.enable_self_correction);
        assert!(!config.enable_deep_research);
    }
}
