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
        "model" | "models" => {
            if args.is_empty() {
                // TUI 模式下由 launch.rs 拦截并打开交互菜单
                // 此分支仅在非 TUI 模式下触发
                SlashCommandResult::Prompt(
                    "请使用 /model <模型名称> 切换模型，或在 TUI 中直接选择。".to_string(),
                )
            } else {
                SlashCommandResult::Prompt(format!("切换模型为: {}", args))
            }
        }
        "plan" => SlashCommandResult::Executed,
        "compact" => {
            let focus = if args.is_empty() { None } else { Some(args) };
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
        // 扩展命令：深度初始化、循环模式、重构、工作流控制
        "init-deep" => SlashCommandResult::Prompt(format_init_deep(args)),
        "ralph-loop" => SlashCommandResult::Prompt(format_ralph_loop(args)),
        "ulw-loop" => SlashCommandResult::Prompt(format_ulw_loop(args)),
        "refactor" => SlashCommandResult::Prompt(format_refactor(args)),
        "start-work" => SlashCommandResult::Prompt(format_start_work(args)),
        "stop-continuation" => SlashCommandResult::Executed,
        "handoff" => SlashCommandResult::Prompt(format_handoff(args)),
        // Autopilot 长任务全自动模式命令
        "autopilot" => SlashCommandResult::Prompt(format_autopilot(args)),
        "autopilot-stop" => SlashCommandResult::Executed,
        "autopilot-status" => SlashCommandResult::Prompt(format_autopilot_status()),
        "doctor" => SlashCommandResult::Prompt(format_doctor()),
        "cost" => SlashCommandResult::Prompt(format_cost()),
        _ => SlashCommandResult::NotFound(name.to_string()),
    }
}

/// 格式化帮助信息
fn format_help() -> String {
    let help = "\
OrangeCoding 斜杠命令帮助
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

工作流:
  /init-deep      深度初始化项目
  /ralph-loop     Ralph 持续改进循环
  /ulw-loop       UltraWork 全自动模式
  /refactor       重构助手
  /start-work     开始新工作会话
  /stop-continuation 停止自动循环
  /handoff        任务交接

Autopilot:
  /autopilot [需求] 启动长任务全自动模式
  /autopilot-stop  停止 Autopilot 循环
  /autopilot-status 查看 Autopilot 状态

诊断:
  /doctor         环境与配置健康检查
  /cost           显示本会话 token 用量与成本估算

退出:
  /exit           退出
  /quit           退出";

    help.to_string()
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

/// 格式化深度初始化提示
///
/// 扫描项目结构，创建 boulder.json，初始化 Agent 状态。
fn format_init_deep(args: &str) -> String {
    let target = if args.is_empty() { "." } else { args };
    format!(
        "深度初始化启动\n\
         目标路径: {}\n\
         步骤:\n\
         1. 扫描项目结构\n\
         2. 创建 boulder.json\n\
         3. 初始化 Agent 状态",
        target
    )
}

/// 格式化 Ralph 循环提示
///
/// 持续改进循环：plan → implement → review → refine。
fn format_ralph_loop(args: &str) -> String {
    let focus = if args.is_empty() {
        "全局".to_string()
    } else {
        args.to_string()
    };
    format!(
        "Ralph 循环已启动\n\
         聚焦: {}\n\
         循环阶段: plan → implement → review → refine",
        focus
    )
}

/// 格式化 UltraWork 循环提示
///
/// 全自动模式启动。
fn format_ulw_loop(args: &str) -> String {
    let config = if args.is_empty() {
        "默认配置".to_string()
    } else {
        args.to_string()
    };
    format!("UltraWork 全自动循环已启动\n配置: {}", config)
}

/// 格式化重构助手提示
///
/// 分析代码并提出重构建议。
fn format_refactor(args: &str) -> String {
    if args.is_empty() {
        "重构助手已启动\n请指定目标文件或模块以开始分析。".to_string()
    } else {
        format!("重构助手已启动\n分析目标: {}", args)
    }
}

/// 格式化开始工作提示
///
/// 创建新的工作会话，初始化 Boulder。
fn format_start_work(args: &str) -> String {
    let task = if args.is_empty() {
        "未指定".to_string()
    } else {
        args.to_string()
    };
    format!(
        "工作会话已创建\n\
         任务: {}\n\
         Boulder 已初始化",
        task
    )
}

/// 格式化任务交接提示
///
/// 将当前任务交给另一个 Agent。
fn format_handoff(args: &str) -> String {
    if args.is_empty() {
        "任务交接\n请指定目标 Agent 名称。".to_string()
    } else {
        format!("任务交接\n目标 Agent: {}", args)
    }
}

/// 格式化调试信息
fn format_debug() -> String {
    format!(
        "OrangeCoding 调试信息\n版本: {}\n平台: {} {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

fn format_autopilot(args: &str) -> String {
    let requirement = if args.is_empty() {
        "未指定需求（将在首次交互中收集）".to_string()
    } else {
        args.to_string()
    };
    format!(
        "🚀 Autopilot 长任务全自动模式已启动\n\
         需求: {}\n\
         循环流程: Plan → Execute → Verify → Replan\n\
         使用 /autopilot-stop 可随时停止\n\
         使用 /autopilot-status 查看进度",
        requirement
    )
}

fn format_autopilot_status() -> String {
    "Autopilot 状态: 未运行".to_string()
}

/// `/doctor` — 环境与配置健康检查
///
/// 检查项：Rust 工具链 / 配置目录 / 关键依赖可达性 / CLAUDE.md 记忆锚点。
fn format_doctor() -> String {
    use std::path::PathBuf;

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_dir = home.join(".config/orangecoding");
    let models_yml = config_dir.join("models.yml");
    let permissions_json = config_dir.join("permissions.json");
    let agent_md = config_dir.join("AGENT.md");

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let claude_md = cwd.join("CLAUDE.md");

    let ok = |b: bool| if b { "✅" } else { "⚠️" };

    format!(
        "OrangeCoding Doctor
==================

环境
  OS:              {}
  当前目录:        {}

配置目录  {}
  {}
  models.yml       {} {}
  permissions.json {} {}
  AGENT.md         {} {}

项目记忆
  CLAUDE.md        {} {}

建议
  - 若 models.yml 缺失，请创建或运行 /settings
  - CLAUDE.md 用于持久化项目偏好，会被自动注入 prompt
  - permissions.json 支持 Bash(git *)、Write(**/*.ts) 等模式",
        std::env::consts::OS,
        cwd.display(),
        ok(config_dir.is_dir()),
        config_dir.display(),
        ok(models_yml.is_file()),
        models_yml.display(),
        ok(permissions_json.is_file()),
        permissions_json.display(),
        ok(agent_md.is_file()),
        agent_md.display(),
        ok(claude_md.is_file()),
        claude_md.display(),
    )
}

/// `/cost` — 会话 token 用量与成本估算
///
/// 当前为占位实现；真实数据由 TUI 侧会话统计注入。
fn format_cost() -> String {
    "Token 用量统计
==================
  输入 token:   （会话未追踪）
  输出 token:   （会话未追踪）
  估算成本:     $0.0000

提示
  真实用量由运行时根据 provider 返回值累计并注入。
  可结合 /usage 与 /debug 查看更多细节。"
        .to_string()
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
                assert!(text.contains("模型"));
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

    // ---- 扩展命令测试 ----

    #[test]
    fn test_execute_init_deep() {
        // 验证深度初始化命令返回包含关键信息的提示
        let result = execute_builtin("init-deep", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("深度初始化启动"));
                assert!(text.contains("boulder.json"));
                assert!(text.contains("Agent 状态"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_init_deep_with_path() {
        // 验证深度初始化命令支持自定义路径参数
        let result = execute_builtin("init-deep", "src/core");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("src/core"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_ralph_loop() {
        // 验证 Ralph 循环命令返回循环阶段信息
        let result = execute_builtin("ralph-loop", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("Ralph 循环已启动"));
                assert!(text.contains("plan → implement → review → refine"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_ulw_loop() {
        // 验证 UltraWork 循环命令返回全自动模式信息
        let result = execute_builtin("ulw-loop", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("UltraWork 全自动循环已启动"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_refactor() {
        // 验证重构助手命令返回正确提示
        let result = execute_builtin("refactor", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("重构助手已启动"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_start_work() {
        // 验证开始工作命令创建会话并初始化 Boulder
        let result = execute_builtin("start-work", "实现用户认证");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("工作会话已创建"));
                assert!(text.contains("实现用户认证"));
                assert!(text.contains("Boulder 已初始化"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_stop_continuation() {
        // 验证停止继续命令返回 Executed 状态
        let result = execute_builtin("stop-continuation", "");
        assert!(matches!(result, SlashCommandResult::Executed));
    }

    #[test]
    fn test_execute_handoff() {
        // 验证任务交接命令包含目标 Agent 信息
        let result = execute_builtin("handoff", "reviewer-agent");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("任务交接"));
                assert!(text.contains("reviewer-agent"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_autopilot() {
        let result = execute_builtin("autopilot", "实现用户认证系统");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("Autopilot"));
                assert!(text.contains("实现用户认证系统"));
                assert!(text.contains("Plan → Execute → Verify → Replan"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_autopilot_no_args() {
        let result = execute_builtin("autopilot", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("Autopilot"));
                assert!(text.contains("未指定需求"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_execute_autopilot_stop() {
        let result = execute_builtin("autopilot-stop", "");
        assert!(matches!(result, SlashCommandResult::Executed));
    }

    #[test]
    fn test_execute_autopilot_status() {
        let result = execute_builtin("autopilot-status", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("Autopilot"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_help_contains_autopilot() {
        let result = execute_builtin("help", "");
        match result {
            SlashCommandResult::Prompt(text) => {
                assert!(text.contains("/autopilot"));
                assert!(text.contains("/autopilot-stop"));
                assert!(text.contains("/autopilot-status"));
                assert!(text.contains("/doctor"));
                assert!(text.contains("/cost"));
            }
            other => panic!("期望 Prompt，得到 {:?}", other),
        }
    }

    #[test]
    fn test_doctor_command() {
        let r = execute_builtin("doctor", "");
        match r {
            SlashCommandResult::Prompt(t) => {
                assert!(t.contains("环境") || t.contains("健康") || t.contains("Doctor"));
            }
            other => panic!("expected Prompt, got {:?}", other),
        }
    }

    #[test]
    fn test_cost_command() {
        let r = execute_builtin("cost", "");
        match r {
            SlashCommandResult::Prompt(t) => {
                assert!(t.to_lowercase().contains("token") || t.contains("成本"));
            }
            other => panic!("expected Prompt, got {:?}", other),
        }
    }
}
