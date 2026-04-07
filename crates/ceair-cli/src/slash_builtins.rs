//! # 内置斜杠命令实现
//!
//! 本模块提供所有内置斜杠命令的执行逻辑。
//! 内置命令无需 Markdown 模板，直接由代码处理。

use crate::slash::SlashCommandResult;

/// 执行内置斜杠命令
///
/// 根据命令名称分发到对应的处理函数。
/// 对于未识别的内置命令，返回 `NotFound`。
pub fn execute_builtin(name: &str, args: &str) -> SlashCommandResult {
    match name {
        "help" | "hotkeys" => SlashCommandResult::Prompt(format_help()),
        "model" | "models" => SlashCommandResult::Prompt(format_model_selector()),
        "plan" => SlashCommandResult::Executed,
        "compact" => {
            let focus = if args.is_empty() {
                None
            } else {
                Some(args)
            };
            SlashCommandResult::Prompt(format_compact(focus))
        }
        "new" => SlashCommandResult::Executed,
        "resume" => SlashCommandResult::Executed,
        "export" => SlashCommandResult::Executed,
        "session" => SlashCommandResult::Prompt(format_session_info()),
        "usage" => SlashCommandResult::Prompt(format_usage()),
        "exit" | "quit" => SlashCommandResult::Executed,
        "settings" => SlashCommandResult::Executed,
        "tree" => SlashCommandResult::Executed,
        "branch" => SlashCommandResult::Executed,
        "fork" => SlashCommandResult::Executed,
        "copy" => SlashCommandResult::Executed,
        "debug" => SlashCommandResult::Prompt(format_debug()),
        _ => SlashCommandResult::NotFound(name.to_string()),
    }
}

/// 格式化帮助信息
fn format_help() -> String {
    let help = "\
CEAIR 斜杠命令帮助
==================

会话管理:
  /new            开始新会话
  /resume         打开会话选择器
  /session        显示会话信息
  /export [path]  导出会话为 HTML
  /tree           会话树导航
  /branch         分支选择器
  /fork           从消息分叉

模型与设置:
  /model          模型选择器
  /settings       设置菜单
  /plan           切换计划模式

上下文管理:
  /compact [focus] 手动压缩上下文

实用工具:
  /copy           复制最后一条消息
  /usage          显示用量
  /debug          调试工具
  /help           显示此帮助
  /hotkeys        显示快捷键

退出:
  /exit           退出
  /quit           退出";

    help.to_string()
}

/// 格式化模型选择器信息
fn format_model_selector() -> String {
    "可用模型:\n  1. DeepSeek Chat\n  2. Qianwen (通义千问)\n  3. Wenxin (文心一言)".to_string()
}

/// 格式化压缩上下文提示
fn format_compact(focus: Option<&str>) -> String {
    match focus {
        Some(f) => format!("正在压缩上下文，聚焦于: {}", f),
        None => "正在压缩上下文...".to_string(),
    }
}

/// 格式化会话信息
fn format_session_info() -> String {
    "当前会话信息（待实现）".to_string()
}

/// 格式化用量信息
fn format_usage() -> String {
    "用量统计（待实现）".to_string()
}

/// 格式化调试信息
fn format_debug() -> String {
    format!(
        "CEAIR 调试信息\n版本: {}\n平台: {} {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_help() {
        let result = execute_builtin("help", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("斜杠命令帮助"));
                assert!(text.contains("/new"));
                assert!(text.contains("/exit"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_hotkeys_alias() {
        // hotkeys 应与 help 返回相同内容
        let result = execute_builtin("hotkeys", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("斜杠命令帮助"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_exit() {
        let result = execute_builtin("exit", "");
        assert!(matches!(result, SlashCommandResult::Executed));
    }

    #[test]
    fn test_execute_quit() {
        let result = execute_builtin("quit", "");
        assert!(matches!(result, SlashCommandResult::Executed));
    }

    #[test]
    fn test_execute_compact_no_focus() {
        let result = execute_builtin("compact", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("压缩上下文"));
                assert!(!text.contains("聚焦于"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_compact_with_focus() {
        let result = execute_builtin("compact", "API 性能");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("聚焦于: API 性能"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_model() {
        let result = execute_builtin("model", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("可用模型"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_debug() {
        let result = execute_builtin("debug", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("调试信息"));
                assert!(text.contains("版本"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_unknown() {
        let result = execute_builtin("nonexistent", "");
        assert!(matches!(result, SlashCommandResult::NotFound(_)));
    }

    #[test]
    fn test_execute_new() {
        let result = execute_builtin("new", "");
        assert!(matches!(result, SlashCommandResult::Executed));
    }

    #[test]
    fn test_execute_copy() {
        let result = execute_builtin("copy", "");
        assert!(matches!(result, SlashCommandResult::Executed));
    }

    #[test]
    fn test_execute_plan() {
        let result = execute_builtin("plan", "");
        assert!(matches!(result, SlashCommandResult::Executed));
    }
}
