//! # AI 驱动的记忆召回
//!
//! 基于语义相关性的记忆检索系统。
//!
//! # 设计思想
//! 参考 reference 中的记忆召回机制：
//! - 通过 AI 模型进行语义匹配，而非简单关键词搜索
//! - 使用 MemorySelector trait 抽象 AI 调用，便于测试
//! - 新鲜度机制：超过阈值的记忆附加过期警告
//! - 最多返回 5 条最相关记忆，避免 context 膨胀

use std::time::{Duration, SystemTime};

// ---------------------------------------------------------------------------
// 记忆条目
// ---------------------------------------------------------------------------

/// 待选择的记忆摘要
#[derive(Clone, Debug)]
pub struct MemorySummary {
    /// 文件路径
    pub path: String,
    /// 记忆名称
    pub name: String,
    /// 记忆类型
    pub entry_type: String,
    /// 前 30 行内容（用于 AI 判断相关性）
    pub preview: String,
    /// 最后修改时间
    pub modified_at: SystemTime,
}

/// AI 选中的记忆
#[derive(Clone, Debug)]
pub struct SelectedMemory {
    /// 文件路径
    pub path: String,
    /// 记忆名称
    pub name: String,
    /// 完整内容
    pub content: String,
    /// 新鲜度警告（如果记忆可能过期）
    pub freshness_warning: Option<String>,
}

// ---------------------------------------------------------------------------
// MemorySelector trait
// ---------------------------------------------------------------------------

/// AI 记忆选择器 trait
///
/// 为什么使用 trait 而不是直接调用 AI：
/// - 单元测试可以注入 Mock 实现
/// - 生产环境可以替换不同的 AI 后端
/// - 便于控制返回结果的格式
pub trait MemorySelector {
    /// 从候选记忆中选择最相关的
    ///
    /// 参数:
    /// - query: 用户查询
    /// - candidates: 候选记忆列表
    ///
    /// 返回: 选中的索引列表（从 0 开始）
    fn select(
        &self,
        query: &str,
        candidates: &[MemorySummary],
    ) -> Vec<usize>;
}

// ---------------------------------------------------------------------------
// 记忆召回器
// ---------------------------------------------------------------------------

/// 新鲜度阈值：超过此时间的记忆附加警告
const FRESHNESS_THRESHOLD: Duration = Duration::from_secs(24 * 60 * 60); // 1 天

/// 最大返回数量
const MAX_RESULTS: usize = 5;

/// 执行记忆召回
///
/// 流程：
/// 1. 按修改时间排序（最新优先）
/// 2. 调用 AI 选择相关记忆
/// 3. 截断到 MAX_RESULTS
/// 4. 附加新鲜度警告
pub fn recall_memories(
    selector: &dyn MemorySelector,
    query: &str,
    candidates: &[MemorySummary],
    contents: &std::collections::HashMap<String, String>,
    already_shown: &[String],
) -> Vec<SelectedMemory> {
    if candidates.is_empty() || query.trim().is_empty() {
        return Vec::new();
    }

    // 排除已展示的记忆
    let filtered: Vec<&MemorySummary> = candidates
        .iter()
        .filter(|c| !already_shown.contains(&c.path))
        .collect();

    if filtered.is_empty() {
        return Vec::new();
    }

    // 构建过滤后的候选列表
    let filtered_summaries: Vec<MemorySummary> =
        filtered.iter().map(|s| (*s).clone()).collect();

    // AI 选择
    let selected_indices = selector.select(query, &filtered_summaries);

    let now = SystemTime::now();

    selected_indices
        .into_iter()
        .filter(|&idx| idx < filtered_summaries.len())
        .take(MAX_RESULTS)
        .map(|idx| {
            let summary = &filtered_summaries[idx];
            let content = contents
                .get(&summary.path)
                .cloned()
                .unwrap_or_default();

            // 检查新鲜度
            let freshness_warning = check_freshness(&summary.modified_at, &now);

            SelectedMemory {
                path: summary.path.clone(),
                name: summary.name.clone(),
                content,
                freshness_warning,
            }
        })
        .collect()
}

/// 检查记忆新鲜度
///
/// 超过 1 天的记忆返回过期警告
fn check_freshness(modified_at: &SystemTime, now: &SystemTime) -> Option<String> {
    match now.duration_since(*modified_at) {
        Ok(age) if age > FRESHNESS_THRESHOLD => {
            let days = age.as_secs() / (24 * 60 * 60);
            Some(format!("[⚠ 此记忆已 {} 天未更新，可能已过期]", days))
        }
        _ => None,
    }
}

/// 构建 AI 查询 manifest
///
/// 格式:
/// ```text
/// Query: {query}
///
/// Available memories:
/// [0] [{type}] {name}: {preview_first_line}
/// [1] [{type}] {name}: {preview_first_line}
/// ```
pub fn build_manifest(query: &str, candidates: &[MemorySummary]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Query: {}", query));
    lines.push(String::new());
    lines.push("Available memories:".to_string());

    for (i, c) in candidates.iter().enumerate() {
        let first_line = c
            .preview
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("");
        lines.push(format!("[{}] [{}] {}: {}", i, c.entry_type, c.name, first_line));
    }

    lines.join("\n")
}

// ===========================================================================
// 单元测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Mock 选择器：总是选择前 N 个
    struct MockSelector {
        select_count: usize,
    }

    impl MemorySelector for MockSelector {
        fn select(&self, _query: &str, candidates: &[MemorySummary]) -> Vec<usize> {
            (0..self.select_count.min(candidates.len())).collect()
        }
    }

    /// Mock 选择器：选择特定索引
    struct IndexSelector {
        indices: Vec<usize>,
    }

    impl MemorySelector for IndexSelector {
        fn select(&self, _query: &str, _candidates: &[MemorySummary]) -> Vec<usize> {
            self.indices.clone()
        }
    }

    fn make_summary(name: &str, path: &str, age_secs: u64) -> MemorySummary {
        MemorySummary {
            path: path.to_string(),
            name: name.to_string(),
            entry_type: "user".to_string(),
            preview: format!("{} 的详细内容", name),
            modified_at: SystemTime::now() - Duration::from_secs(age_secs),
        }
    }

    fn make_contents(paths: &[&str]) -> HashMap<String, String> {
        paths
            .iter()
            .map(|p| (p.to_string(), format!("{} 完整内容", p)))
            .collect()
    }

    #[test]
    fn test_empty_candidates_returns_empty() {
        let selector = MockSelector { select_count: 5 };
        let result = recall_memories(&selector, "query", &[], &HashMap::new(), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_empty_query_returns_empty() {
        let selector = MockSelector { select_count: 5 };
        let candidates = vec![make_summary("A", "a.md", 0)];
        let result = recall_memories(&selector, "  ", &candidates, &HashMap::new(), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_mock_selector_correct_return() {
        let selector = MockSelector { select_count: 2 };
        let candidates = vec![
            make_summary("A", "a.md", 0),
            make_summary("B", "b.md", 0),
            make_summary("C", "c.md", 0),
        ];
        let contents = make_contents(&["a.md", "b.md", "c.md"]);
        let result = recall_memories(&selector, "test", &candidates, &contents, &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "A");
        assert_eq!(result[1].name, "B");
    }

    #[test]
    fn test_max_5_results() {
        let selector = MockSelector { select_count: 10 };
        let candidates: Vec<MemorySummary> = (0..10)
            .map(|i| make_summary(&format!("M{}", i), &format!("{}.md", i), 0))
            .collect();
        let contents: HashMap<String, String> = candidates
            .iter()
            .map(|c| (c.path.clone(), "content".to_string()))
            .collect();
        let result = recall_memories(&selector, "test", &candidates, &contents, &[]);
        assert!(result.len() <= MAX_RESULTS);
    }

    #[test]
    fn test_freshness_warning_recent() {
        let selector = MockSelector { select_count: 1 };
        let candidates = vec![make_summary("A", "a.md", 60)]; // 1 分钟前
        let contents = make_contents(&["a.md"]);
        let result = recall_memories(&selector, "test", &candidates, &contents, &[]);
        assert_eq!(result.len(), 1);
        assert!(result[0].freshness_warning.is_none());
    }

    #[test]
    fn test_freshness_warning_old() {
        let selector = MockSelector { select_count: 1 };
        let candidates = vec![make_summary("A", "a.md", 3 * 24 * 60 * 60)]; // 3 天前
        let contents = make_contents(&["a.md"]);
        let result = recall_memories(&selector, "test", &candidates, &contents, &[]);
        assert_eq!(result.len(), 1);
        assert!(result[0].freshness_warning.is_some());
        let warning = result[0].freshness_warning.as_ref().unwrap();
        assert!(warning.contains("3"));
        assert!(warning.contains("过期"));
    }

    #[test]
    fn test_exclude_already_shown() {
        let selector = MockSelector { select_count: 5 };
        let candidates = vec![
            make_summary("A", "a.md", 0),
            make_summary("B", "b.md", 0),
        ];
        let contents = make_contents(&["a.md", "b.md"]);
        let already_shown = vec!["a.md".to_string()];
        let result =
            recall_memories(&selector, "test", &candidates, &contents, &already_shown);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "B");
    }

    #[test]
    fn test_all_already_shown() {
        let selector = MockSelector { select_count: 5 };
        let candidates = vec![make_summary("A", "a.md", 0)];
        let already_shown = vec!["a.md".to_string()];
        let result = recall_memories(
            &selector,
            "test",
            &candidates,
            &HashMap::new(),
            &already_shown,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn test_invalid_index_filtered() {
        let selector = IndexSelector {
            indices: vec![0, 99, 1],
        };
        let candidates = vec![
            make_summary("A", "a.md", 0),
            make_summary("B", "b.md", 0),
        ];
        let contents = make_contents(&["a.md", "b.md"]);
        let result = recall_memories(&selector, "test", &candidates, &contents, &[]);
        assert_eq!(result.len(), 2); // 99 被过滤
    }

    #[test]
    fn test_build_manifest() {
        let candidates = vec![
            make_summary("Rust 规范", "rust.md", 0),
            make_summary("测试指南", "test.md", 0),
        ];
        let manifest = build_manifest("如何写测试", &candidates);
        assert!(manifest.contains("Query: 如何写测试"));
        assert!(manifest.contains("[0]"));
        assert!(manifest.contains("[1]"));
        assert!(manifest.contains("Rust 规范"));
    }

    #[test]
    fn test_selected_memory_content() {
        let selector = MockSelector { select_count: 1 };
        let candidates = vec![make_summary("A", "a.md", 0)];
        let mut contents = HashMap::new();
        contents.insert("a.md".to_string(), "完整的 A 内容".to_string());
        let result = recall_memories(&selector, "test", &candidates, &contents, &[]);
        assert_eq!(result[0].content, "完整的 A 内容");
    }

    #[test]
    fn test_missing_content_returns_empty_string() {
        let selector = MockSelector { select_count: 1 };
        let candidates = vec![make_summary("A", "a.md", 0)];
        let result =
            recall_memories(&selector, "test", &candidates, &HashMap::new(), &[]);
        assert_eq!(result[0].content, "");
    }

    #[test]
    fn test_check_freshness_boundary() {
        let now = SystemTime::now();
        // 刚好 1 天
        let at_boundary = now - FRESHNESS_THRESHOLD;
        assert!(check_freshness(&at_boundary, &now).is_none());

        // 超过 1 天
        let over_boundary = now - FRESHNESS_THRESHOLD - Duration::from_secs(1);
        assert!(check_freshness(&over_boundary, &now).is_some());
    }
}
