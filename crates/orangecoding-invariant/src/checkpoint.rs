//! # Checkpoint / 文件快照
//!
//! 对齐规范第六章 6.4：
//! 在关键事件（TODO 完成、任务里程碑、用户主动触发）前后对受影响文件做内容快照，
//! 支持按 checkpoint ID 回滚到先前状态。
//!
//! 与 `rollback.rs`（git 级）互补：本模块是文件级、轻量、无需 git 仓库，
//! 适合覆盖未提交的中间态。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// 文件快照条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileSnapshot {
    pub path: PathBuf,
    /// 快照时文件内容（None 表示文件当时不存在）
    pub content: Option<String>,
    /// 内容 SHA-256（十六进制）
    pub sha256: String,
}

impl FileSnapshot {
    /// 从磁盘读取文件创建快照；若文件不存在则记录为 None
    pub fn capture(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self {
                path: path.to_path_buf(),
                content: None,
                sha256: hex_sha256(b""),
            });
        }
        let content = std::fs::read_to_string(path)?;
        let sha = hex_sha256(content.as_bytes());
        Ok(Self {
            path: path.to_path_buf(),
            content: Some(content),
            sha256: sha,
        })
    }

    /// 恢复快照到磁盘
    pub fn restore(&self) -> std::io::Result<()> {
        match &self.content {
            Some(c) => {
                if let Some(parent) = self.path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&self.path, c)?;
            }
            None => {
                if self.path.exists() {
                    std::fs::remove_file(&self.path)?;
                }
            }
        }
        Ok(())
    }
}

/// 触发快照的事件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckpointTrigger {
    /// TODO 状态变化
    TodoCompleted(String),
    /// 里程碑
    Milestone(String),
    /// 手动触发
    Manual,
    /// 定时
    Scheduled,
}

/// Checkpoint — 一组文件的快照集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub trigger: CheckpointTrigger,
    pub description: String,
    pub snapshots: Vec<FileSnapshot>,
}

impl Checkpoint {
    /// 创建 checkpoint（对指定文件列表拍照）
    pub fn capture(
        id: impl Into<String>,
        trigger: CheckpointTrigger,
        description: impl Into<String>,
        files: &[PathBuf],
    ) -> std::io::Result<Self> {
        let mut snaps = Vec::with_capacity(files.len());
        for f in files {
            snaps.push(FileSnapshot::capture(f)?);
        }
        Ok(Self {
            id: id.into(),
            created_at: Utc::now(),
            trigger,
            description: description.into(),
            snapshots: snaps,
        })
    }

    /// 恢复所有快照
    pub fn restore_all(&self) -> std::io::Result<usize> {
        let mut count = 0;
        for s in &self.snapshots {
            s.restore()?;
            count += 1;
        }
        Ok(count)
    }

    pub fn file_count(&self) -> usize {
        self.snapshots.len()
    }
}

/// Checkpoint 存储（内存型）
#[derive(Debug, Default)]
pub struct CheckpointStore {
    entries: Vec<Checkpoint>,
}

impl CheckpointStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, c: Checkpoint) {
        self.entries.push(c);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn find(&self, id: &str) -> Option<&Checkpoint> {
        self.entries.iter().find(|c| c.id == id)
    }

    pub fn latest(&self) -> Option<&Checkpoint> {
        self.entries.last()
    }

    pub fn list(&self) -> &[Checkpoint] {
        &self.entries
    }

    /// 回滚到指定 checkpoint
    pub fn rollback_to(&self, id: &str) -> Result<usize, String> {
        let cp = self
            .find(id)
            .ok_or_else(|| format!("checkpoint {} not found", id))?;
        cp.restore_all()
            .map_err(|e| format!("restore failed: {}", e))
    }

    /// 持久化到 JSON 文件
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)
    }

    /// 从 JSON 文件加载
    pub fn load_from(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let data = std::fs::read_to_string(path)?;
        let entries: Vec<Checkpoint> = serde_json::from_str(&data)?;
        Ok(Self { entries })
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(out.len() * 2);
    for b in out {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_snapshot_capture_existing() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("a.txt");
        std::fs::write(&p, "hello").unwrap();

        let snap = FileSnapshot::capture(&p).unwrap();
        assert_eq!(snap.content.as_deref(), Some("hello"));
        assert_eq!(snap.sha256.len(), 64);
    }

    #[test]
    fn test_file_snapshot_capture_missing() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("nope.txt");
        let snap = FileSnapshot::capture(&p).unwrap();
        assert!(snap.content.is_none());
    }

    #[test]
    fn test_file_snapshot_restore_writes_content() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("x.txt");
        std::fs::write(&p, "v1").unwrap();

        let snap = FileSnapshot::capture(&p).unwrap();
        std::fs::write(&p, "v2").unwrap();

        snap.restore().unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "v1");
    }

    #[test]
    fn test_restore_removes_file_that_didnt_exist() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("ghost.txt");
        let snap = FileSnapshot::capture(&p).unwrap();
        std::fs::write(&p, "created later").unwrap();
        snap.restore().unwrap();
        assert!(!p.exists());
    }

    #[test]
    fn test_checkpoint_capture_and_restore() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.rs");
        let b = dir.path().join("b.rs");
        std::fs::write(&a, "A1").unwrap();
        std::fs::write(&b, "B1").unwrap();

        let cp = Checkpoint::capture(
            "cp-1",
            CheckpointTrigger::Manual,
            "before refactor",
            &[a.clone(), b.clone()],
        )
        .unwrap();

        assert_eq!(cp.file_count(), 2);

        std::fs::write(&a, "A2").unwrap();
        std::fs::write(&b, "B2").unwrap();

        let n = cp.restore_all().unwrap();
        assert_eq!(n, 2);
        assert_eq!(std::fs::read_to_string(&a).unwrap(), "A1");
        assert_eq!(std::fs::read_to_string(&b).unwrap(), "B1");
    }

    #[test]
    fn test_checkpoint_store_find_and_rollback() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("f.txt");
        std::fs::write(&p, "orig").unwrap();

        let mut store = CheckpointStore::new();
        let cp = Checkpoint::capture(
            "cp-a",
            CheckpointTrigger::TodoCompleted("t1".into()),
            "todo t1 done",
            &[p.clone()],
        )
        .unwrap();
        store.push(cp);

        std::fs::write(&p, "changed").unwrap();
        assert_eq!(store.len(), 1);
        assert!(store.find("cp-a").is_some());
        assert!(store.find("nope").is_none());

        store.rollback_to("cp-a").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "orig");
    }

    #[test]
    fn test_checkpoint_store_rollback_unknown() {
        let store = CheckpointStore::new();
        let err = store.rollback_to("ghost").unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_checkpoint_store_persistence() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("watched.txt");
        std::fs::write(&file, "v1").unwrap();

        let mut store = CheckpointStore::new();
        store.push(
            Checkpoint::capture("cp-p", CheckpointTrigger::Manual, "first", &[file.clone()])
                .unwrap(),
        );

        let persist = dir.path().join("ckpts.json");
        store.save_to(&persist).unwrap();

        let loaded = CheckpointStore::load_from(&persist).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.find("cp-p").unwrap().description, "first");
    }

    #[test]
    fn test_sha256_changes_with_content() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("h.txt");
        std::fs::write(&p, "a").unwrap();
        let s1 = FileSnapshot::capture(&p).unwrap().sha256;
        std::fs::write(&p, "b").unwrap();
        let s2 = FileSnapshot::capture(&p).unwrap().sha256;
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_latest_returns_last_pushed() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("a.txt");
        std::fs::write(&p, "x").unwrap();

        let mut store = CheckpointStore::new();
        store.push(Checkpoint::capture("a", CheckpointTrigger::Manual, "1", &[p.clone()]).unwrap());
        store.push(Checkpoint::capture("b", CheckpointTrigger::Manual, "2", &[p.clone()]).unwrap());
        assert_eq!(store.latest().unwrap().id, "b");
    }
}
