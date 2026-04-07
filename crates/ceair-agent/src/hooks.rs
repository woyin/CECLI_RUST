//! # 钩子系统
//!
//! 提供生命周期钩子机制，在代理执行的关键节点拦截并处理事件。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 钩子事件类型
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    /// 会话开始前
    PreSession,
    /// 会话结束后
    PostSession,
    /// 消息发送前
    PreMessage,
    /// 消息接收后
    PostMessage,
    /// 工具调用前
    PreToolCall,
    /// 工具调用后
    PostToolCall,
    /// 上下文压缩前
    PreCompaction,
    /// 上下文压缩后
    PostCompaction,
}

/// 钩子动作
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookAction {
    /// 继续执行
    Continue,
    /// 修改内容后继续
    Modify(String),
    /// 阻止执行
    Block(String),
    /// 跳过后续钩子
    Skip,
}

/// 钩子定义
#[derive(Clone, Debug)]
pub struct HookDef {
    /// 钩子名称
    pub name: String,
    /// 触发事件
    pub event: HookEvent,
    /// 优先级（数值越小优先级越高）
    pub priority: i32,
    /// 处理器
    pub handler: HookHandler,
}

/// 钩子处理器
#[derive(Clone, Debug)]
pub enum HookHandler {
    /// 内联动作（用于内置钩子）
    Inline(String),
    /// 外部脚本路径
    Script(std::path::PathBuf),
}

/// 钩子上下文
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HookContext {
    /// 触发事件
    pub event: HookEvent,
    /// 上下文数据
    pub data: HashMap<String, serde_json::Value>,
}

/// 钩子注册表
pub struct HookRegistry {
    hooks: Vec<HookDef>,
}

impl HookRegistry {
    /// 创建空的钩子注册表
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// 注册钩子
    pub fn register(&mut self, hook: HookDef) {
        self.hooks.push(hook);
    }

    /// 注销指定名称的钩子，返回是否成功移除
    pub fn unregister(&mut self, name: &str) -> bool {
        let before = self.hooks.len();
        self.hooks.retain(|h| h.name != name);
        self.hooks.len() < before
    }

    /// 获取指定事件的所有钩子（按优先级升序排列）
    pub fn get_hooks_for(&self, event: &HookEvent) -> Vec<&HookDef> {
        let mut matched: Vec<&HookDef> = self.hooks.iter().filter(|h| &h.event == event).collect();
        matched.sort_by_key(|h| h.priority);
        matched
    }

    /// 执行钩子链并返回最终动作
    ///
    /// 内联处理器格式：
    /// - `"continue"` → Continue
    /// - `"block:<原因>"` → Block
    /// - `"modify:<内容>"` → Modify
    /// - `"skip"` → Skip
    pub fn execute_hooks(&self, ctx: &HookContext) -> HookAction {
        let hooks = self.get_hooks_for(&ctx.event);
        if hooks.is_empty() {
            return HookAction::Continue;
        }

        for hook in &hooks {
            let action = match &hook.handler {
                HookHandler::Inline(cmd) => parse_inline_action(cmd),
                HookHandler::Script(_) => HookAction::Continue,
            };
            match &action {
                HookAction::Block(_) | HookAction::Skip | HookAction::Modify(_) => return action,
                HookAction::Continue => continue,
            }
        }

        HookAction::Continue
    }

    /// 返回已注册钩子数量
    pub fn count(&self) -> usize {
        self.hooks.len()
    }

    /// 清空所有钩子
    pub fn clear(&mut self) {
        self.hooks.clear();
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析内联动作字符串为 `HookAction`
fn parse_inline_action(cmd: &str) -> HookAction {
    if cmd == "continue" {
        HookAction::Continue
    } else if cmd == "skip" {
        HookAction::Skip
    } else if let Some(reason) = cmd.strip_prefix("block:") {
        HookAction::Block(reason.to_string())
    } else if let Some(content) = cmd.strip_prefix("modify:") {
        HookAction::Modify(content.to_string())
    } else {
        HookAction::Continue
    }
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助函数：构造内联钩子
    fn make_hook(name: &str, event: HookEvent, priority: i32, action: &str) -> HookDef {
        HookDef {
            name: name.to_string(),
            event,
            priority,
            handler: HookHandler::Inline(action.to_string()),
        }
    }

    #[test]
    fn test_register_hook() {
        let mut reg = HookRegistry::new();
        reg.register(make_hook("h1", HookEvent::PreSession, 0, "continue"));
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn test_unregister_hook() {
        let mut reg = HookRegistry::new();
        reg.register(make_hook("h1", HookEvent::PreSession, 0, "continue"));
        assert!(reg.unregister("h1"));
        assert_eq!(reg.count(), 0);
        // 注销不存在的钩子应返回 false
        assert!(!reg.unregister("nonexistent"));
    }

    #[test]
    fn test_get_hooks_for_event() {
        let mut reg = HookRegistry::new();
        reg.register(make_hook("h1", HookEvent::PreSession, 0, "continue"));
        reg.register(make_hook("h2", HookEvent::PostSession, 0, "continue"));
        reg.register(make_hook("h3", HookEvent::PreSession, 1, "continue"));

        let hooks = reg.get_hooks_for(&HookEvent::PreSession);
        assert_eq!(hooks.len(), 2);
        assert!(hooks.iter().all(|h| h.event == HookEvent::PreSession));
    }

    #[test]
    fn test_hooks_sorted_by_priority() {
        let mut reg = HookRegistry::new();
        reg.register(make_hook("low", HookEvent::PreMessage, 10, "continue"));
        reg.register(make_hook("high", HookEvent::PreMessage, 1, "continue"));
        reg.register(make_hook("mid", HookEvent::PreMessage, 5, "continue"));

        let hooks = reg.get_hooks_for(&HookEvent::PreMessage);
        assert_eq!(hooks[0].name, "high");
        assert_eq!(hooks[1].name, "mid");
        assert_eq!(hooks[2].name, "low");
    }

    #[test]
    fn test_execute_continue() {
        let mut reg = HookRegistry::new();
        reg.register(make_hook("h1", HookEvent::PreSession, 0, "continue"));

        let ctx = HookContext {
            event: HookEvent::PreSession,
            data: HashMap::new(),
        };
        assert_eq!(reg.execute_hooks(&ctx), HookAction::Continue);
    }

    #[test]
    fn test_execute_block() {
        let mut reg = HookRegistry::new();
        reg.register(make_hook("blocker", HookEvent::PreToolCall, 0, "block:denied"));

        let ctx = HookContext {
            event: HookEvent::PreToolCall,
            data: HashMap::new(),
        };
        assert_eq!(
            reg.execute_hooks(&ctx),
            HookAction::Block("denied".to_string())
        );
    }

    #[test]
    fn test_no_hooks_returns_continue() {
        let reg = HookRegistry::new();
        let ctx = HookContext {
            event: HookEvent::PreSession,
            data: HashMap::new(),
        };
        assert_eq!(reg.execute_hooks(&ctx), HookAction::Continue);
    }

    #[test]
    fn test_clear() {
        let mut reg = HookRegistry::new();
        reg.register(make_hook("h1", HookEvent::PreSession, 0, "continue"));
        reg.register(make_hook("h2", HookEvent::PostSession, 0, "continue"));
        reg.clear();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_count() {
        let mut reg = HookRegistry::new();
        assert_eq!(reg.count(), 0);
        reg.register(make_hook("h1", HookEvent::PreSession, 0, "continue"));
        assert_eq!(reg.count(), 1);
        reg.register(make_hook("h2", HookEvent::PostSession, 0, "continue"));
        assert_eq!(reg.count(), 2);
    }

    #[test]
    fn test_hook_context_serialization() {
        let mut data = HashMap::new();
        data.insert("key".to_string(), serde_json::json!("value"));
        let ctx = HookContext {
            event: HookEvent::PreMessage,
            data,
        };

        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: HookContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event, ctx.event);
        assert_eq!(deserialized.data["key"], serde_json::json!("value"));
    }
}
