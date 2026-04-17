//! # 项目记忆锚点（CLAUDE.md / AGENT.md）
//!
//! 对齐规范第四章 4.1：
//! Agent 启动时按优先级自动读取以下位置的 Markdown 记忆文件并合并：
//!
//! 1. `~/.config/orangecoding/AGENT.md`（全局用户偏好）
//! 2. `<project_root>/CLAUDE.md`（项目级规范，最重要）
//! 3. `<current_dir>/CLAUDE.md`（子目录级补充）
//!
//! 同时兼容以下别名：`ORANGECODING.md`, `AGENTS.md`, `CLAUDE.md`。
//! 三层内容按顺序拼接，并在每段前增加清晰的 section header。

use std::path::{Path, PathBuf};

/// 可识别的记忆文件名（全局级）
const GLOBAL_MEMORY_NAMES: &[&str] = &["AGENT.md", "ORANGECODING.md", "CLAUDE.md", "AGENTS.md"];

/// 可识别的记忆文件名（项目/当前目录级）
const LOCAL_MEMORY_NAMES: &[&str] = &["CLAUDE.md", "ORANGECODING.md", "AGENT.md", "AGENTS.md"];

/// 一层记忆来源
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryLayer {
    pub label: &'static str,
    pub path: PathBuf,
    pub content: String,
}

/// 聚合后的记忆锚点
#[derive(Debug, Clone, Default)]
pub struct MemoryAnchor {
    pub layers: Vec<MemoryLayer>,
}

impl MemoryAnchor {
    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// 合并后的 Markdown 文本，可直接注入 prompt
    pub fn rendered(&self) -> String {
        if self.layers.is_empty() {
            return String::new();
        }
        let mut out = String::from("# 项目记忆锚点\n\n");
        for layer in &self.layers {
            out.push_str(&format!(
                "## [{}] {}\n\n{}\n\n",
                layer.label,
                layer.path.display(),
                layer.content.trim_end()
            ));
        }
        out
    }

    /// 合并另一个锚点
    pub fn merge(mut self, other: MemoryAnchor) -> Self {
        self.layers.extend(other.layers);
        self
    }
}

fn read_first_existing(dir: &Path, names: &[&str], label: &'static str) -> Option<MemoryLayer> {
    for name in names {
        let p = dir.join(name);
        if p.is_file() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                return Some(MemoryLayer {
                    label,
                    path: p,
                    content,
                });
            }
        }
    }
    None
}

/// 全局用户目录
pub fn user_memory_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/orangecoding")
}

/// 按规范加载三层记忆
///
/// 参数：
/// - `project_root`: 项目根目录（可选，若提供则读取该目录下的 CLAUDE.md）
/// - `current_dir`: 当前工作目录（可选，若与 project_root 不同则额外读取）
pub fn load_memory_anchor(project_root: Option<&Path>, current_dir: Option<&Path>) -> MemoryAnchor {
    let mut layers = Vec::new();

    // 1) 全局
    if let Some(layer) = read_first_existing(&user_memory_dir(), GLOBAL_MEMORY_NAMES, "global") {
        layers.push(layer);
    }

    // 2) 项目根
    if let Some(root) = project_root {
        if let Some(layer) = read_first_existing(root, LOCAL_MEMORY_NAMES, "project") {
            layers.push(layer);
        }

        // 3) 当前目录（避免重复）
        if let Some(cwd) = current_dir {
            if cwd != root {
                if let Some(layer) = read_first_existing(cwd, LOCAL_MEMORY_NAMES, "directory") {
                    layers.push(layer);
                }
            }
        }
    } else if let Some(cwd) = current_dir {
        if let Some(layer) = read_first_existing(cwd, LOCAL_MEMORY_NAMES, "directory") {
            layers.push(layer);
        }
    }

    MemoryAnchor { layers }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_empty_when_nothing_exists() {
        let dir = tempdir().unwrap();
        let anchor = load_memory_anchor(Some(dir.path()), Some(dir.path()));
        assert!(anchor.is_empty());
        assert_eq!(anchor.rendered(), "");
    }

    #[test]
    fn test_project_root_claude_md() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "# Project rules\n- use TDD\n").unwrap();

        let anchor = load_memory_anchor(Some(dir.path()), None);
        assert_eq!(anchor.layers.len(), 1);
        assert_eq!(anchor.layers[0].label, "project");
        assert!(anchor.layers[0].content.contains("use TDD"));
        assert!(anchor.rendered().contains("use TDD"));
    }

    #[test]
    fn test_current_dir_supplement() {
        let root = tempdir().unwrap();
        let sub = root.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(root.path().join("CLAUDE.md"), "root").unwrap();
        fs::write(sub.join("CLAUDE.md"), "local").unwrap();

        let anchor = load_memory_anchor(Some(root.path()), Some(&sub));
        assert_eq!(anchor.layers.len(), 2);
        assert_eq!(anchor.layers[0].label, "project");
        assert_eq!(anchor.layers[1].label, "directory");
    }

    #[test]
    fn test_same_root_and_cwd_no_duplicate() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("CLAUDE.md"), "same").unwrap();

        let anchor = load_memory_anchor(Some(root.path()), Some(root.path()));
        assert_eq!(anchor.layers.len(), 1);
    }

    #[test]
    fn test_prefers_agent_md_at_global_level() {
        // 这里无法直接写 $HOME 下的文件，但可以测试 read_first_existing 的优先序
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("AGENT.md"), "agent").unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "claude").unwrap();

        let layer = read_first_existing(dir.path(), GLOBAL_MEMORY_NAMES, "global").unwrap();
        assert_eq!(layer.content, "agent");
    }

    #[test]
    fn test_local_prefers_claude_over_agent() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("AGENT.md"), "agent").unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "claude").unwrap();

        let layer = read_first_existing(dir.path(), LOCAL_MEMORY_NAMES, "project").unwrap();
        assert_eq!(layer.content, "claude");
    }

    #[test]
    fn test_rendered_includes_section_headers() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "hello").unwrap();
        let anchor = load_memory_anchor(Some(dir.path()), None);
        let rendered = anchor.rendered();
        assert!(rendered.contains("# 项目记忆锚点"));
        assert!(rendered.contains("[project]"));
    }

    #[test]
    fn test_merge_combines_layers() {
        let dir1 = tempdir().unwrap();
        fs::write(dir1.path().join("CLAUDE.md"), "a").unwrap();
        let dir2 = tempdir().unwrap();
        fs::write(dir2.path().join("CLAUDE.md"), "b").unwrap();

        let m1 = load_memory_anchor(Some(dir1.path()), None);
        let m2 = load_memory_anchor(Some(dir2.path()), None);
        let merged = m1.merge(m2);
        assert_eq!(merged.layers.len(), 2);
    }
}
