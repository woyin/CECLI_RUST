//! # Token 预算状态机
//!
//! 跟踪每轮 token 用量，基于使用率和收益递减检测做出续写决策。
//!
//! # 设计思想
//! 参考 reference 中 checkTokenBudget() 的实现：
//! - 预算耗尽时立即停止
//! - 连续多轮 token 增量很小时检测收益递减
//! - 使用率未达阈值时提示继续
//! - 支持多 Agent 上下文的特殊处理

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 常量
// ---------------------------------------------------------------------------

/// 使用率完成阈值（90%），超过此比例时建议停止
const COMPLETION_THRESHOLD: f64 = 0.9;

/// 收益递减检测阈值（每轮新增 token 数），低于此值视为收益不足
const DIMINISHING_THRESHOLD: usize = 500;

/// 触发收益递减检测所需的最小连续轮数
const DIMINISHING_MIN_ROUNDS: usize = 3;

// ---------------------------------------------------------------------------
// 数据结构
// ---------------------------------------------------------------------------

/// Token 预算配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenBudgetConfig {
    /// 模型的上下文窗口大小（tokens）
    pub context_window: usize,
    /// 为系统提示和工具定义预留的 token 数
    pub reserved_tokens: usize,
    /// 使用率完成阈值（0.0 ~ 1.0）
    pub completion_threshold: f64,
    /// 收益递减检测的 token 增量阈值
    pub diminishing_threshold: usize,
    /// 触发收益递减的最小连续轮数
    pub diminishing_min_rounds: usize,
}

impl Default for TokenBudgetConfig {
    fn default() -> Self {
        Self {
            context_window: 200_000,
            reserved_tokens: 10_000,
            completion_threshold: COMPLETION_THRESHOLD,
            diminishing_threshold: DIMINISHING_THRESHOLD,
            diminishing_min_rounds: DIMINISHING_MIN_ROUNDS,
        }
    }
}

/// 预算决策
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetDecision {
    /// 继续执行，附带使用率提示
    Continue(String),
    /// 停止执行，附带停止原因
    Stop(String),
}

/// Token 预算状态机
///
/// 跟踪每轮 token 用量，检测收益递减和预算耗尽。
///
/// # 工作流程
/// 1. 初始化时设置上下文窗口和预留 token
/// 2. 每轮调用 `record_usage(tokens)` 记录用量
/// 3. 调用 `check_budget()` 获取续写决策
///
/// # 收益递减检测
/// 当连续 N 轮的 token 增量都小于阈值时，视为收益递减，建议停止。
/// 这避免了 Agent 陷入无意义的循环。
pub struct TokenBudget {
    config: TokenBudgetConfig,
    /// 每轮的累计 token 用量历史
    usage_history: Vec<usize>,
    /// 是否处于多 Agent 上下文中
    is_multi_agent: bool,
}

impl TokenBudget {
    /// 创建新的 Token 预算实例
    pub fn new(config: TokenBudgetConfig) -> Self {
        Self {
            config,
            usage_history: Vec::new(),
            is_multi_agent: false,
        }
    }

    /// 设置是否处于多 Agent 上下文
    pub fn set_multi_agent(&mut self, multi: bool) {
        self.is_multi_agent = multi;
    }

    /// 记录当前轮次的 token 用量（累计值）
    pub fn record_usage(&mut self, total_tokens: usize) {
        self.usage_history.push(total_tokens);
    }

    /// 获取可用预算（context_window - reserved_tokens）
    pub fn available_budget(&self) -> usize {
        self.config
            .context_window
            .saturating_sub(self.config.reserved_tokens)
    }

    /// 获取当前 token 用量（最后一次记录）
    pub fn current_usage(&self) -> usize {
        self.usage_history.last().copied().unwrap_or(0)
    }

    /// 计算当前使用率（0.0 ~ 1.0）
    pub fn usage_ratio(&self) -> f64 {
        let budget = self.available_budget();
        if budget == 0 {
            return 1.0;
        }
        self.current_usage() as f64 / budget as f64
    }

    /// 检查预算，返回续写决策
    ///
    /// # 决策逻辑（按优先级）
    /// 1. 预算 ≤ 0 → Stop
    /// 2. 多 Agent 上下文 → Stop（子 Agent 应尽快完成）
    /// 3. 连续 N 轮增量 < 阈值 → Stop（收益递减）
    /// 4. 使用率 < 完成阈值 → Continue
    /// 5. 否则 → Stop
    pub fn check_budget(&self) -> BudgetDecision {
        let budget = self.available_budget();
        let current = self.current_usage();

        // 规则 1: 预算耗尽
        if current >= budget {
            return BudgetDecision::Stop("token 预算已耗尽".to_string());
        }

        // 规则 2: 多 Agent 上下文应尽快完成
        if self.is_multi_agent {
            return BudgetDecision::Stop("多 Agent 上下文，子任务应尽快完成".to_string());
        }

        // 规则 3: 收益递减检测
        if self.is_diminishing_returns() {
            return BudgetDecision::Stop(format!(
                "收益递减：连续 {} 轮 token 增量均低于 {} tokens",
                self.config.diminishing_min_rounds, self.config.diminishing_threshold
            ));
        }

        // 规则 4: 使用率检查
        let ratio = self.usage_ratio();
        if ratio < self.config.completion_threshold {
            let percent = (ratio * 100.0) as u32;
            BudgetDecision::Continue(format!("已使用 {}% 的上下文预算", percent))
        } else {
            // 规则 5: 接近上限
            BudgetDecision::Stop(format!(
                "上下文使用率已达 {:.0}%，建议压缩或停止",
                ratio * 100.0
            ))
        }
    }

    /// 检测是否存在收益递减
    ///
    /// 当连续 diminishing_min_rounds 轮的 token 增量均低于 diminishing_threshold 时返回 true
    fn is_diminishing_returns(&self) -> bool {
        let n = self.config.diminishing_min_rounds;
        if self.usage_history.len() < n + 1 {
            return false;
        }

        // 检查最近 n 轮的增量
        let recent = &self.usage_history[self.usage_history.len() - n - 1..];
        for window in recent.windows(2) {
            let delta = window[1].saturating_sub(window[0]);
            if delta >= self.config.diminishing_threshold {
                return false;
            }
        }
        true
    }

    /// 获取历史记录长度（轮数）
    pub fn rounds(&self) -> usize {
        self.usage_history.len()
    }

    /// 清空使用历史
    pub fn reset(&mut self) {
        self.usage_history.clear();
    }
}

// ===========================================================================
// 单元测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建测试用的默认预算
    fn test_budget() -> TokenBudget {
        TokenBudget::new(TokenBudgetConfig {
            context_window: 100_000,
            reserved_tokens: 10_000,
            ..Default::default()
        })
    }

    // -----------------------------------------------------------------------
    // 基础功能测试
    // -----------------------------------------------------------------------

    /// 测试充足预算时返回 Continue
    #[test]
    fn test_budget_continue_when_plenty() {
        let mut budget = test_budget();
        budget.record_usage(10_000); // 10K / 90K = 11%

        match budget.check_budget() {
            BudgetDecision::Continue(msg) => {
                assert!(msg.contains("11%"), "应显示使用率百分比: {}", msg);
            }
            BudgetDecision::Stop(msg) => {
                panic!("预算充足时不应停止: {}", msg);
            }
        }
    }

    /// 测试预算耗尽时返回 Stop
    #[test]
    fn test_budget_stop_when_exhausted() {
        let mut budget = test_budget();
        budget.record_usage(95_000); // 超过 available_budget (90K)

        match budget.check_budget() {
            BudgetDecision::Stop(msg) => {
                assert!(msg.contains("耗尽"), "应提示预算耗尽: {}", msg);
            }
            BudgetDecision::Continue(_) => {
                panic!("预算耗尽时不应继续");
            }
        }
    }

    /// 测试使用率接近阈值时返回 Stop
    #[test]
    fn test_budget_stop_near_threshold() {
        let mut budget = test_budget();
        // 可用预算 90K，90% 阈值 = 81K
        budget.record_usage(85_000); // 94% > 90%

        match budget.check_budget() {
            BudgetDecision::Stop(msg) => {
                assert!(msg.contains("使用率"), "应提示使用率过高: {}", msg);
            }
            BudgetDecision::Continue(_) => {
                panic!("接近阈值时不应继续");
            }
        }
    }

    // -----------------------------------------------------------------------
    // 收益递减检测测试
    // -----------------------------------------------------------------------

    /// 测试收益递减检测（连续 3 轮小增量）
    #[test]
    fn test_diminishing_returns_detection() {
        let mut budget = test_budget();

        // 模拟连续 4 轮用量，最后 3 轮增量 < 500
        budget.record_usage(10_000);
        budget.record_usage(10_100); // delta=100
        budget.record_usage(10_200); // delta=100
        budget.record_usage(10_300); // delta=100

        match budget.check_budget() {
            BudgetDecision::Stop(msg) => {
                assert!(msg.contains("收益递减"), "应提示收益递减: {}", msg);
            }
            BudgetDecision::Continue(_) => {
                panic!("收益递减时不应继续");
            }
        }
    }

    /// 测试非收益递减（增量足够大）
    #[test]
    fn test_no_diminishing_returns() {
        let mut budget = test_budget();

        budget.record_usage(10_000);
        budget.record_usage(11_000); // delta=1000 > 500
        budget.record_usage(12_000); // delta=1000 > 500
        budget.record_usage(13_000); // delta=1000 > 500

        match budget.check_budget() {
            BudgetDecision::Continue(_) => {} // 正确
            BudgetDecision::Stop(msg) => {
                panic!("增量足够大时不应停止: {}", msg);
            }
        }
    }

    /// 测试轮数不足时不触发收益递减
    #[test]
    fn test_not_enough_rounds_for_diminishing() {
        let mut budget = test_budget();

        // 只有 2 轮（需要 4 轮才能检测 3 轮增量）
        budget.record_usage(10_000);
        budget.record_usage(10_100);

        match budget.check_budget() {
            BudgetDecision::Continue(_) => {} // 正确
            BudgetDecision::Stop(msg) => {
                panic!("轮数不足时不应检测收益递减: {}", msg);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 多 Agent 上下文测试
    // -----------------------------------------------------------------------

    /// 测试多 Agent 上下文时返回 Stop
    #[test]
    fn test_multi_agent_stops() {
        let mut budget = test_budget();
        budget.set_multi_agent(true);
        budget.record_usage(10_000); // 预算充足但多 Agent

        match budget.check_budget() {
            BudgetDecision::Stop(msg) => {
                assert!(msg.contains("多 Agent"), "应提示多 Agent 上下文: {}", msg);
            }
            BudgetDecision::Continue(_) => {
                panic!("多 Agent 上下文时不应继续");
            }
        }
    }

    // -----------------------------------------------------------------------
    // 辅助方法测试
    // -----------------------------------------------------------------------

    /// 测试 available_budget 计算
    #[test]
    fn test_available_budget() {
        let budget = test_budget();
        assert_eq!(budget.available_budget(), 90_000);
    }

    /// 测试 current_usage 和 rounds
    #[test]
    fn test_usage_tracking() {
        let mut budget = test_budget();

        assert_eq!(budget.current_usage(), 0);
        assert_eq!(budget.rounds(), 0);

        budget.record_usage(5000);
        assert_eq!(budget.current_usage(), 5000);
        assert_eq!(budget.rounds(), 1);

        budget.record_usage(8000);
        assert_eq!(budget.current_usage(), 8000);
        assert_eq!(budget.rounds(), 2);
    }

    /// 测试 usage_ratio 计算
    #[test]
    fn test_usage_ratio() {
        let mut budget = test_budget();
        budget.record_usage(45_000); // 45K / 90K = 50%

        let ratio = budget.usage_ratio();
        assert!((ratio - 0.5).abs() < 0.001);
    }

    /// 测试空历史时的 usage_ratio
    #[test]
    fn test_usage_ratio_empty() {
        let budget = test_budget();
        assert_eq!(budget.usage_ratio(), 0.0);
    }

    /// 测试 reset 清空历史
    #[test]
    fn test_reset() {
        let mut budget = test_budget();
        budget.record_usage(5000);
        budget.record_usage(10000);
        assert_eq!(budget.rounds(), 2);

        budget.reset();
        assert_eq!(budget.rounds(), 0);
        assert_eq!(budget.current_usage(), 0);
    }

    /// 测试零可用预算时的边界情况
    #[test]
    fn test_zero_available_budget() {
        let budget = TokenBudget::new(TokenBudgetConfig {
            context_window: 1000,
            reserved_tokens: 2000, // reserved > window
            ..Default::default()
        });

        // saturating_sub 确保不下溢
        assert_eq!(budget.available_budget(), 0);
        assert_eq!(budget.usage_ratio(), 1.0);
    }

    /// 测试默认配置值
    #[test]
    fn test_default_config() {
        let config = TokenBudgetConfig::default();
        assert_eq!(config.context_window, 200_000);
        assert_eq!(config.reserved_tokens, 10_000);
        assert!((config.completion_threshold - 0.9).abs() < 0.001);
        assert_eq!(config.diminishing_threshold, 500);
        assert_eq!(config.diminishing_min_rounds, 3);
    }

    /// 测试 BudgetDecision 的 PartialEq
    #[test]
    fn test_budget_decision_equality() {
        let a = BudgetDecision::Continue("ok".to_string());
        let b = BudgetDecision::Continue("ok".to_string());
        assert_eq!(a, b);

        let c = BudgetDecision::Stop("done".to_string());
        assert_ne!(a, c);
    }
}
