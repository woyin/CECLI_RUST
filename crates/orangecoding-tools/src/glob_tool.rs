//! # Glob 工具
//!
//! 按模式匹配文件路径，返回匹配的文件列表。
//! 规范对齐 Claude Code 的 GlobTool。

use crate::{Tool, ToolError, ToolMetadata, ToolResult};
use async_trait::async_trait;
use glob::glob;
use serde_json::{json, Value};
use std::path::PathBuf;

/// 默认返回的最大匹配数量
const DEFAULT_LIMIT: usize = 500;

/// Glob 工具 — 按 glob 模式匹配文件路径
///
/// 支持的模式语法：
/// - `*` 匹配任意字符（不含 `/`）
/// - `**` 跨目录匹配
/// - `?` 匹配单个字符
/// - `{a,b}` 匹配 a 或 b
#[derive(Debug, Default)]
pub struct GlobTool;

impl GlobTool {
    pub fn new() -> Self {
        Self
    }

    /// 纯函数版本：按模式收集匹配路径，支持限制数量
    pub fn match_paths(pattern: &str, limit: usize) -> ToolResult<Vec<PathBuf>> {
        let iter = glob(pattern)
            .map_err(|e| ToolError::InvalidParams(format!("无效的 glob 模式: {}", e)))?;
        let mut out = Vec::new();
        for entry in iter {
            match entry {
                Ok(p) => {
                    out.push(p);
                    if out.len() >= limit {
                        break;
                    }
                }
                Err(e) => {
                    tracing::debug!("跳过无法访问的路径: {}", e);
                }
            }
        }
        Ok(out)
    }
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "按 glob 模式匹配文件路径，例如 **/*.rs、src/**/test_*.py"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "glob 模式，支持 *、**、?、{a,b}"
                },
                "cwd": {
                    "type": "string",
                    "description": "模式匹配的相对基准目录（可选）"
                },
                "limit": {
                    "type": "number",
                    "description": "最大返回数量，默认 500",
                    "minimum": 1,
                    "maximum": 10000
                }
            },
            "required": ["pattern"]
        })
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata::read_only()
    }

    async fn execute(&self, params: Value) -> ToolResult<String> {
        let pattern = params
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParams("缺少必要参数: pattern".to_string()))?;

        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(DEFAULT_LIMIT);

        // 若指定 cwd，将相对模式拼接到 cwd 下
        let full_pattern = if let Some(cwd) = params.get("cwd").and_then(|v| v.as_str()) {
            if pattern.starts_with('/') {
                pattern.to_string()
            } else {
                format!("{}/{}", cwd.trim_end_matches('/'), pattern)
            }
        } else {
            pattern.to_string()
        };

        let paths = Self::match_paths(&full_pattern, limit)?;

        if paths.is_empty() {
            return Ok(format!("未匹配任何路径: {}", full_pattern));
        }

        let lines: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
        Ok(format!(
            "匹配到 {} 个路径:\n{}",
            lines.len(),
            lines.join("\n")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_glob_matches_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "").unwrap();
        fs::write(dir.path().join("b.rs"), "").unwrap();
        fs::write(dir.path().join("c.txt"), "").unwrap();

        let pattern = format!("{}/*.rs", dir.path().display());
        let tool = GlobTool::new();
        let result = tool.execute(json!({ "pattern": pattern })).await.unwrap();
        assert!(result.contains("a.rs"));
        assert!(result.contains("b.rs"));
        assert!(!result.contains("c.txt"));
    }

    #[tokio::test]
    async fn test_glob_recursive() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub/deep");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("x.py"), "").unwrap();
        let pattern = format!("{}/**/*.py", dir.path().display());
        let tool = GlobTool::new();
        let result = tool.execute(json!({ "pattern": pattern })).await.unwrap();
        assert!(result.contains("x.py"));
    }

    #[tokio::test]
    async fn test_glob_no_match() {
        let tool = GlobTool::new();
        let result = tool
            .execute(json!({ "pattern": "/nonexistent-root-dir-xyz/*.abc" }))
            .await
            .unwrap();
        assert!(result.contains("未匹配"));
    }

    #[tokio::test]
    async fn test_glob_missing_pattern() {
        let tool = GlobTool::new();
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_glob_invalid_pattern() {
        let tool = GlobTool::new();
        let result = tool.execute(json!({ "pattern": "[" })).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_glob_is_read_only() {
        let tool = GlobTool::new();
        let meta = tool.metadata();
        assert!(meta.is_read_only);
        assert!(meta.is_concurrency_safe);
    }

    #[test]
    fn test_glob_schema() {
        let tool = GlobTool::new();
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "pattern"));
    }
}
