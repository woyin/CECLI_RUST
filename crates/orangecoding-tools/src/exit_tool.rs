//! # Exit 工具
//!
//! 标记任务完成或请求退出当前 Agent 循环。
//! 对齐 Claude Code 规范中的 ExitTool。

use crate::{Tool, ToolError, ToolMetadata, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Exit 工具 — 标记任务完成
///
/// 调用时由 Agent 主循环识别并终止当前会话的 Execute 阶段，
/// 配合的 reason/summary 将作为 Agent 最后的输出。
#[derive(Debug, Default)]
pub struct ExitTool;

impl ExitTool {
    pub fn new() -> Self {
        Self
    }
}

/// 退出事件的结构化结果
pub const EXIT_MARKER: &str = "__AGENT_EXIT__";

#[async_trait]
impl Tool for ExitTool {
    fn name(&self) -> &str {
        "exit"
    }

    fn description(&self) -> &str {
        "标记当前任务已完成并请求退出 Agent 循环，可附带 summary 简要说明。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "summary": {
                    "type": "string",
                    "description": "任务完成的简要摘要"
                },
                "success": {
                    "type": "boolean",
                    "description": "是否成功完成任务，默认 true",
                    "default": true
                }
            },
            "required": []
        })
    }

    fn metadata(&self) -> ToolMetadata {
        // Exit 不修改文件系统，但会改变 Agent 状态
        ToolMetadata {
            is_read_only: false,
            is_concurrency_safe: false,
            is_destructive: false,
            is_enabled: true,
        }
    }

    async fn execute(&self, params: Value) -> ToolResult<String> {
        let summary = params
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("任务已完成");
        let success = params
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let status = if success { "success" } else { "failure" };
        Ok(format!("{} {} {}", EXIT_MARKER, status, summary))
    }

    fn validate_input(&self, params: &Value) -> ToolResult<()> {
        if !params.is_object() {
            return Err(ToolError::InvalidParams("参数必须是 JSON 对象".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_exit_defaults() {
        let tool = ExitTool::new();
        let result = tool.execute(json!({})).await.unwrap();
        assert!(result.starts_with(EXIT_MARKER));
        assert!(result.contains("success"));
        assert!(result.contains("任务已完成"));
    }

    #[tokio::test]
    async fn test_exit_with_summary() {
        let tool = ExitTool::new();
        let result = tool
            .execute(json!({
                "summary": "修复了登录问题",
                "success": true
            }))
            .await
            .unwrap();
        assert!(result.contains("修复了登录问题"));
        assert!(result.contains("success"));
    }

    #[tokio::test]
    async fn test_exit_failure_mode() {
        let tool = ExitTool::new();
        let result = tool
            .execute(json!({ "success": false, "summary": "无法继续" }))
            .await
            .unwrap();
        assert!(result.contains("failure"));
        assert!(result.contains("无法继续"));
    }

    #[test]
    fn test_exit_tool_name() {
        let tool = ExitTool::new();
        assert_eq!(tool.name(), "exit");
        assert!(!tool.description().is_empty());
    }
}
