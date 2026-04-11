//! # 自动压缩触发器与断路器
//!
//! 检测上下文 token 是否接近上限，自动触发压缩。
//! 配备断路器机制：连续失败过多时暂停自动压缩，防止无效重试。
//!
//! # 设计思想
//! 参考 reference 中 autoCompact 的设计：
//! - buffer_tokens 预留缓冲区，防止在精确边界上反复触发
//! - 阈值 = context_window - reserved - buffer_tokens
//! - 断路器模式：连续失败达到上限时禁止自动压缩，
//!   避免在压缩反复失败时浪费 token 和时间

/// 自动压缩配置
#[derive(Clone, Debug)]
pub struct AutoCompactConfig {
    /// 缓冲 token 数，预留给新内容生成
    pub buffer_tokens: usize,
    /// 连续失败最大次数，超过后断路
    pub max_consecutive_failures: u32,
}

impl Default for AutoCompactConfig {
    fn default() -> Self {
        Self {
            buffer_tokens: 13_000,
            max_consecutive_failures: 3,
        }
    }
}

/// 自动压缩触发器
///
/// 跟踪压缩状态，判断何时触发自动压缩，
/// 并在连续失败时通过断路器阻止进一步尝试。
pub struct AutoCompactTrigger {
    config: AutoCompactConfig,
    /// 连续失败计数
    consecutive_failures: u32,
    /// 断路器是否打开（打开 = 禁止自动压缩）
    circuit_open: bool,
}

impl AutoCompactTrigger {
    /// 使用指定配置创建触发器
    pub fn new(config: AutoCompactConfig) -> Self {
        Self {
            config,
            consecutive_failures: 0,
            circuit_open: false,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(AutoCompactConfig::default())
    }

    /// 判断是否应触发自动压缩
    ///
    /// 计算公式:
    /// threshold = context_window - reserved - buffer_tokens
    /// 当 current_tokens > threshold 时触发
    ///
    /// 如果断路器打开，始终返回 false
    pub fn should_compact(
        &self,
        current_tokens: usize,
        context_window: usize,
        reserved: usize,
    ) -> bool {
        // 断路器打开时禁止自动压缩
        if self.circuit_open {
            return false;
        }

        // 防止下溢：如果 reserved + buffer >= context_window，阈值为 0
        let threshold = context_window
            .saturating_sub(reserved)
            .saturating_sub(self.config.buffer_tokens);

        // 阈值为 0 时意味着空间极小，应该触发
        if threshold == 0 && current_tokens > 0 {
            return true;
        }

        current_tokens > threshold
    }

    /// 报告压缩成功
    ///
    /// 重置失败计数和断路器
    pub fn report_success(&mut self) {
        self.consecutive_failures = 0;
        self.circuit_open = false;
    }

    /// 报告压缩失败
    ///
    /// 递增失败计数，达到上限时打开断路器
    pub fn report_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.config.max_consecutive_failures {
            self.circuit_open = true;
        }
    }

    /// 手动重置断路器
    pub fn reset_circuit(&mut self) {
        self.consecutive_failures = 0;
        self.circuit_open = false;
    }

    /// 断路器是否打开
    pub fn is_circuit_open(&self) -> bool {
        self.circuit_open
    }

    /// 当前连续失败次数
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }
}

impl Default for AutoCompactTrigger {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ===========================================================================
// 单元测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_below_threshold_no_compact() {
        let trigger = AutoCompactTrigger::with_defaults();
        // context_window=100000, reserved=5000, buffer=13000
        // threshold = 100000 - 5000 - 13000 = 82000
        assert!(!trigger.should_compact(50_000, 100_000, 5_000));
    }

    #[test]
    fn test_above_threshold_should_compact() {
        let trigger = AutoCompactTrigger::with_defaults();
        // threshold = 100000 - 5000 - 13000 = 82000
        assert!(trigger.should_compact(90_000, 100_000, 5_000));
    }

    #[test]
    fn test_exact_threshold_no_compact() {
        let trigger = AutoCompactTrigger::with_defaults();
        // threshold = 82000, current = 82000, not > threshold
        assert!(!trigger.should_compact(82_000, 100_000, 5_000));
    }

    #[test]
    fn test_just_above_threshold() {
        let trigger = AutoCompactTrigger::with_defaults();
        assert!(trigger.should_compact(82_001, 100_000, 5_000));
    }

    #[test]
    fn test_circuit_breaker_blocks_after_failures() {
        let mut trigger = AutoCompactTrigger::with_defaults();
        // 3次失败后断路
        trigger.report_failure();
        trigger.report_failure();
        assert!(!trigger.is_circuit_open());

        trigger.report_failure(); // 第3次
        assert!(trigger.is_circuit_open());

        // 即使超过阈值也不触发
        assert!(!trigger.should_compact(90_000, 100_000, 5_000));
    }

    #[test]
    fn test_success_resets_failures() {
        let mut trigger = AutoCompactTrigger::with_defaults();
        trigger.report_failure();
        trigger.report_failure();
        assert_eq!(trigger.consecutive_failures(), 2);

        trigger.report_success();
        assert_eq!(trigger.consecutive_failures(), 0);
        assert!(!trigger.is_circuit_open());
    }

    #[test]
    fn test_success_after_circuit_open_resets() {
        let mut trigger = AutoCompactTrigger::with_defaults();
        trigger.report_failure();
        trigger.report_failure();
        trigger.report_failure();
        assert!(trigger.is_circuit_open());

        trigger.report_success();
        assert!(!trigger.is_circuit_open());
        // 恢复后可以正常触发
        assert!(trigger.should_compact(90_000, 100_000, 5_000));
    }

    #[test]
    fn test_custom_buffer() {
        let config = AutoCompactConfig {
            buffer_tokens: 5_000,
            max_consecutive_failures: 3,
        };
        let trigger = AutoCompactTrigger::new(config);
        // threshold = 100000 - 5000 - 5000 = 90000
        assert!(!trigger.should_compact(85_000, 100_000, 5_000));
        assert!(trigger.should_compact(91_000, 100_000, 5_000));
    }

    #[test]
    fn test_custom_max_failures() {
        let config = AutoCompactConfig {
            buffer_tokens: 13_000,
            max_consecutive_failures: 1,
        };
        let mut trigger = AutoCompactTrigger::new(config);
        trigger.report_failure();
        assert!(trigger.is_circuit_open());
    }

    #[test]
    fn test_manual_reset_circuit() {
        let mut trigger = AutoCompactTrigger::with_defaults();
        trigger.report_failure();
        trigger.report_failure();
        trigger.report_failure();
        assert!(trigger.is_circuit_open());

        trigger.reset_circuit();
        assert!(!trigger.is_circuit_open());
        assert_eq!(trigger.consecutive_failures(), 0);
    }

    #[test]
    fn test_saturating_sub_no_panic() {
        let trigger = AutoCompactTrigger::with_defaults();
        // reserved + buffer > context_window → threshold 应为 0
        // 有任何 token 都应触发
        assert!(trigger.should_compact(1, 1000, 20_000));
    }

    #[test]
    fn test_zero_tokens_no_compact() {
        let trigger = AutoCompactTrigger::with_defaults();
        assert!(!trigger.should_compact(0, 100_000, 5_000));
    }

    #[test]
    fn test_zero_context_window() {
        let trigger = AutoCompactTrigger::with_defaults();
        // threshold = 0 - 0 - 13000 → saturating → 0
        // current=0 → false
        assert!(!trigger.should_compact(0, 0, 0));
        // current>0 when threshold=0 → true
        assert!(trigger.should_compact(1, 0, 0));
    }
}
