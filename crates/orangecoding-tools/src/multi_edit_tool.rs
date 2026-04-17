//! # MultiEdit 工具
//!
//! 在单个文件上原子地应用多处 `(old_str, new_str)` 替换。
//! 所有替换必须全部成功，否则文件保持不变。
//!
//! 设计对齐规范：
//! - 每个 old_str 必须在当前内容中唯一匹配
//! - 所有编辑按顺序作用，每步基于前一步的结果
//! - 任何一步失败则整体回滚，文件不会被写入
//! - 写入前可选校验 `expected_sha256` 检测并发修改

use crate::edit_tool::{EditOperation, EditTool};
use crate::{Tool, ToolError, ToolMetadata, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

/// MultiEdit 工具 — 单文件多处原子替换
#[derive(Debug, Default)]
pub struct MultiEditTool;

impl MultiEditTool {
    pub fn new() -> Self {
        Self
    }

    /// 计算内容的 SHA-256 哈希（16 进制字符串）
    pub fn content_sha256(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[async_trait]
impl Tool for MultiEditTool {
    fn name(&self) -> &str {
        "multi_edit"
    }

    fn description(&self) -> &str {
        "在单个文件上原子地应用多处字符串替换，全部成功或全部失败。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要编辑的文件路径"
                },
                "edits": {
                    "type": "array",
                    "description": "编辑操作列表，每项包含 old_text 和 new_text",
                    "items": {
                        "type": "object",
                        "properties": {
                            "old_text": { "type": "string" },
                            "new_text": { "type": "string" }
                        },
                        "required": ["old_text", "new_text"]
                    },
                    "minItems": 1
                },
                "expected_sha256": {
                    "type": "string",
                    "description": "可选：期望的文件内容 SHA-256，用于检测并发修改"
                }
            },
            "required": ["path", "edits"]
        })
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            is_read_only: false,
            is_concurrency_safe: false,
            is_destructive: true,
            is_enabled: true,
        }
    }

    async fn execute(&self, params: Value) -> ToolResult<String> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParams("缺少必要参数: path".to_string()))?;

        let edits_json = params
            .get("edits")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::InvalidParams("缺少必要参数: edits".to_string()))?;

        if edits_json.is_empty() {
            return Err(ToolError::InvalidParams("edits 不能为空".to_string()));
        }

        let mut edits: Vec<EditOperation> = Vec::with_capacity(edits_json.len());
        for (i, item) in edits_json.iter().enumerate() {
            let old_text = item
                .get("old_text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidParams(format!("edits[{}] 缺少 old_text", i)))?;
            let new_text = item
                .get("new_text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidParams(format!("edits[{}] 缺少 new_text", i)))?;
            if old_text.is_empty() {
                return Err(ToolError::InvalidParams(format!(
                    "edits[{}].old_text 不能为空",
                    i
                )));
            }
            edits.push(EditOperation {
                path: path.to_string(),
                old_text: old_text.to_string(),
                new_text: new_text.to_string(),
            });
        }

        if !Path::new(path).exists() {
            return Err(ToolError::ExecutionError(format!("文件不存在: {}", path)));
        }

        let original = fs::read_to_string(path).await?;

        // 并发修改检测
        if let Some(expected) = params.get("expected_sha256").and_then(|v| v.as_str()) {
            let actual = Self::content_sha256(&original);
            if !expected.eq_ignore_ascii_case(&actual) {
                return Err(ToolError::ExecutionError(format!(
                    "文件哈希不匹配，可能已被并发修改: 期望 {}, 实际 {}",
                    expected, actual
                )));
            }
        }

        // 原子地在内存中应用所有编辑
        let new_content = EditTool::apply_edits(&original, &edits)?;

        // 全部成功后写回文件
        fs::write(path, &new_content).await?;

        Ok(format!("已对文件 {} 原子应用 {} 处编辑", path, edits.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_multi_edit_all_succeed() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "foo\nbar\nbaz\n").unwrap();

        let tool = MultiEditTool::new();
        let result = tool
            .execute(json!({
                "path": file.to_str().unwrap(),
                "edits": [
                    { "old_text": "foo", "new_text": "FOO" },
                    { "old_text": "bar", "new_text": "BAR" }
                ]
            }))
            .await
            .unwrap();

        assert!(result.contains("2 处编辑"));
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "FOO\nBAR\nbaz\n");
    }

    #[tokio::test]
    async fn test_multi_edit_atomic_rollback_on_failure() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "foo\nbar\n").unwrap();
        let original_content = fs::read_to_string(&file).unwrap();

        let tool = MultiEditTool::new();
        let result = tool
            .execute(json!({
                "path": file.to_str().unwrap(),
                "edits": [
                    { "old_text": "foo", "new_text": "FOO" },
                    { "old_text": "notfound", "new_text": "xxx" }
                ]
            }))
            .await;

        assert!(result.is_err(), "应在第二步失败");
        // 文件应未被修改
        let after = fs::read_to_string(&file).unwrap();
        assert_eq!(after, original_content);
    }

    #[tokio::test]
    async fn test_multi_edit_expected_sha256_mismatch() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();

        let tool = MultiEditTool::new();
        let result = tool
            .execute(json!({
                "path": file.to_str().unwrap(),
                "expected_sha256": "deadbeef",
                "edits": [ { "old_text": "hello", "new_text": "world" } ]
            }))
            .await;
        assert!(result.is_err());
        let after = fs::read_to_string(&file).unwrap();
        assert_eq!(after, "hello");
    }

    #[tokio::test]
    async fn test_multi_edit_expected_sha256_match() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();
        let expected = MultiEditTool::content_sha256("hello");

        let tool = MultiEditTool::new();
        let result = tool
            .execute(json!({
                "path": file.to_str().unwrap(),
                "expected_sha256": expected,
                "edits": [ { "old_text": "hello", "new_text": "world" } ]
            }))
            .await
            .unwrap();
        assert!(result.contains("1 处编辑"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "world");
    }

    #[tokio::test]
    async fn test_multi_edit_empty_edits_rejected() {
        let tool = MultiEditTool::new();
        let result = tool.execute(json!({ "path": "/tmp/x", "edits": [] })).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multi_edit_missing_path() {
        let tool = MultiEditTool::new();
        let result = tool
            .execute(json!({ "edits": [{"old_text":"a","new_text":"b"}] }))
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_content_sha256_stable() {
        let a = MultiEditTool::content_sha256("hello");
        let b = MultiEditTool::content_sha256("hello");
        assert_eq!(a, b);
        assert_ne!(a, MultiEditTool::content_sha256("world"));
    }
}
