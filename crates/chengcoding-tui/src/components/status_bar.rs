//! 增强型状态栏组件
//!
//! 提供比基础 `StatusBar` 更丰富的状态展示，包括分项 token 用量统计、
//! 费用估算和思考中指示器。支持主题系统的颜色方案。

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::app::AppMode;
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Token 用量
// ---------------------------------------------------------------------------

/// Token 用量 — 详细的输入/输出/缓存 token 统计
#[derive(Clone, Debug, Default)]
pub struct TokenUsage {
    /// 输入 token 数量
    pub input_tokens: u64,
    /// 输出 token 数量
    pub output_tokens: u64,
    /// 缓存读取的 token 数量
    pub cache_read_tokens: u64,
    /// 缓存写入的 token 数量
    pub cache_write_tokens: u64,
}

// ---------------------------------------------------------------------------
// 增强型状态栏
// ---------------------------------------------------------------------------

/// 增强型状态栏 — 显示模型、token 用量、会话信息
///
/// 布局: `[Model] | [Tokens: in/out] | [Cost] | [Session] | [Mode]`
pub struct EnhancedStatusBar {
    /// 当前模型名称
    pub model_name: String,
    /// 详细 token 用量
    pub tokens_used: TokenUsage,
    /// 当前会话名称
    pub session_name: Option<String>,
    /// 当前应用模式
    pub mode: AppMode,
    /// 当前费用（美元）
    pub cost: Option<f64>,
    /// 思考中指示器文本
    pub thinking_indicator: Option<String>,
}

impl EnhancedStatusBar {
    /// 创建默认状态栏
    pub fn new() -> Self {
        Self {
            model_name: String::new(),
            tokens_used: TokenUsage::default(),
            session_name: None,
            mode: AppMode::Normal,
            cost: None,
            thinking_indicator: None,
        }
    }

    /// 更新模型信息
    pub fn set_model(&mut self, name: &str) {
        self.model_name = name.to_string();
    }

    /// 更新 token 用量
    pub fn update_tokens(&mut self, usage: TokenUsage) {
        self.tokens_used = usage;
    }

    /// 设置成本
    pub fn set_cost(&mut self, cost: f64) {
        self.cost = Some(cost);
    }

    /// 格式化 token 显示
    ///
    /// 使用 k/M 后缀提高可读性：
    /// - `500 / 200` — 原样显示小数值
    /// - `12.5k / 3.2k` — 千级使用 k 后缀
    /// - `1.2M / 456.8k` — 百万级使用 M 后缀
    pub fn format_tokens(&self) -> String {
        let input = Self::format_number(self.tokens_used.input_tokens);
        let output = Self::format_number(self.tokens_used.output_tokens);
        format!("{input} / {output}")
    }

    /// 格式化成本显示
    ///
    /// 以美元格式输出，固定三位小数：`$0.023`
    pub fn format_cost(&self) -> String {
        match self.cost {
            Some(c) => format!("${:.3}", c),
            None => "$0.000".to_string(),
        }
    }

    /// 渲染状态栏到缓冲区
    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // 构建状态栏各段
        let mut spans: Vec<Span<'static>> = Vec::new();

        // 模型名称
        spans.push(Span::styled(
            format!(" 🤖 {} ", self.model_name),
            Style::default()
                .fg(theme.colors.primary)
                .add_modifier(Modifier::BOLD),
        ));

        // 分隔符
        spans.push(Span::styled(
            "│".to_string(),
            Style::default().fg(theme.colors.divider),
        ));

        // Token 用量
        spans.push(Span::styled(
            format!(" 📊 {} ", self.format_tokens()),
            Style::default().fg(theme.colors.foreground),
        ));

        // 分隔符
        spans.push(Span::styled(
            "│".to_string(),
            Style::default().fg(theme.colors.divider),
        ));

        // 费用
        spans.push(Span::styled(
            format!(" {} ", self.format_cost()),
            Style::default().fg(theme.colors.accent),
        ));

        // 会话名称（如果有）
        if let Some(ref session) = self.session_name {
            spans.push(Span::styled(
                "│".to_string(),
                Style::default().fg(theme.colors.divider),
            ));
            spans.push(Span::styled(
                format!(" 📁 {} ", session),
                Style::default().fg(theme.colors.secondary),
            ));
        }

        // 分隔符
        spans.push(Span::styled(
            "│".to_string(),
            Style::default().fg(theme.colors.divider),
        ));

        // 模式指示器
        let mode_color = match self.mode {
            AppMode::Normal => theme.colors.status_info,
            AppMode::Input => theme.colors.status_success,
            AppMode::Command => theme.colors.status_warning,
            AppMode::Help => theme.colors.secondary,
        };
        spans.push(Span::styled(
            format!(" {} ", self.mode),
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ));

        let line = Line::from(spans);

        // 填充背景
        let bg_style = Style::default()
            .bg(theme.colors.surface)
            .fg(theme.colors.foreground);
        for x in area.x..area.x + area.width {
            buf.get_mut(x, area.y).set_style(bg_style).set_symbol(" ");
        }

        // 渲染文本行
        let line_width = area.width as usize;
        buf.set_line(area.x, area.y, &line, line_width as u16);
    }

    // -----------------------------------------------------------------------
    // 私有辅助方法
    // -----------------------------------------------------------------------

    /// 格式化数字：小值原样，千级用 k，百万级用 M
    fn format_number(n: u64) -> String {
        if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}k", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }
}

impl Default for EnhancedStatusBar {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 单元测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens_small() {
        // 小数值应原样显示
        let mut bar = EnhancedStatusBar::new();
        bar.tokens_used = TokenUsage {
            input_tokens: 500,
            output_tokens: 200,
            ..Default::default()
        };
        assert_eq!(bar.format_tokens(), "500 / 200");
    }

    #[test]
    fn test_format_tokens_k() {
        // 千级应使用 k 后缀
        let mut bar = EnhancedStatusBar::new();
        bar.tokens_used = TokenUsage {
            input_tokens: 12500,
            output_tokens: 3200,
            ..Default::default()
        };
        assert_eq!(bar.format_tokens(), "12.5k / 3.2k");
    }

    #[test]
    fn test_format_tokens_m() {
        // 百万级应使用 M 后缀，混合时分别格式化
        let mut bar = EnhancedStatusBar::new();
        bar.tokens_used = TokenUsage {
            input_tokens: 1_234_567,
            output_tokens: 456_789,
            ..Default::default()
        };
        assert_eq!(bar.format_tokens(), "1.2M / 456.8k");
    }

    #[test]
    fn test_format_cost() {
        // 正常费用显示三位小数
        let mut bar = EnhancedStatusBar::new();
        bar.set_cost(0.0234);
        assert_eq!(bar.format_cost(), "$0.023");
    }

    #[test]
    fn test_format_cost_zero() {
        // 无费用时显示 $0.000
        let bar = EnhancedStatusBar::new();
        assert_eq!(bar.format_cost(), "$0.000");
    }

    #[test]
    fn test_set_model() {
        // 验证设置模型名称
        let mut bar = EnhancedStatusBar::new();
        bar.set_model("claude-3.5-sonnet");
        assert_eq!(bar.model_name, "claude-3.5-sonnet");
    }

    #[test]
    fn test_update_tokens() {
        // 验证更新 token 用量
        let mut bar = EnhancedStatusBar::new();
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: 30,
            cache_write_tokens: 20,
        };
        bar.update_tokens(usage.clone());
        assert_eq!(bar.tokens_used.input_tokens, 100);
        assert_eq!(bar.tokens_used.output_tokens, 50);
        assert_eq!(bar.tokens_used.cache_read_tokens, 30);
        assert_eq!(bar.tokens_used.cache_write_tokens, 20);
    }

    #[test]
    fn test_new_defaults() {
        // 验证默认值
        let bar = EnhancedStatusBar::new();
        assert_eq!(bar.model_name, "");
        assert_eq!(bar.tokens_used.input_tokens, 0);
        assert_eq!(bar.tokens_used.output_tokens, 0);
        assert!(bar.session_name.is_none());
        assert_eq!(bar.mode, AppMode::Normal);
        assert!(bar.cost.is_none());
        assert!(bar.thinking_indicator.is_none());
    }
}
