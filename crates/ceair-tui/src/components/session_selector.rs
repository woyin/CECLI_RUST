//! 会话选择器组件
//!
//! 提供历史会话的列表浏览、搜索过滤和选择功能，
//! 允许用户在多个会话之间快速切换。

// ---------------------------------------------------------------------------
// 会话信息
// ---------------------------------------------------------------------------

/// 会话信息 — 描述一个历史会话的摘要数据
#[derive(Clone, Debug)]
pub struct SessionInfo {
    /// 会话唯一标识
    pub id: String,
    /// 创建时间戳（ISO 8601 格式）
    pub timestamp: String,
    /// 会话内容预览（首条消息的缩略文本）
    pub preview: String,
    /// 消息总数
    pub message_count: usize,
    /// 工作目录路径
    pub working_dir: String,
}

// ---------------------------------------------------------------------------
// 会话选择器
// ---------------------------------------------------------------------------

/// 会话选择器 — 列出和选择历史会话
///
/// 支持上下键导航、文本过滤和显隐切换。
/// 选择索引在过滤后的列表范围内循环。
pub struct SessionSelector {
    /// 所有会话列表
    pub sessions: Vec<SessionInfo>,
    /// 当前选中索引
    pub selected_index: usize,
    /// 过滤文本（按预览和工作目录匹配）
    pub filter: String,
    /// 是否可见
    pub visible: bool,
}

impl SessionSelector {
    /// 创建空的会话选择器
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected_index: 0,
            filter: String::new(),
            visible: false,
        }
    }

    /// 设置会话列表并重置选择索引
    pub fn set_sessions(&mut self, sessions: Vec<SessionInfo>) {
        self.sessions = sessions;
        self.selected_index = 0;
    }

    /// 向上移动选择（到达顶部时循环到底部）
    pub fn select_previous(&mut self) {
        let count = self.filtered_sessions().len();
        if count == 0 {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = count - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// 向下移动选择（到达底部时循环到顶部）
    pub fn select_next(&mut self) {
        let count = self.filtered_sessions().len();
        if count == 0 {
            return;
        }
        if self.selected_index >= count - 1 {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }
    }

    /// 获取当前选中的会话
    ///
    /// 返回过滤后列表中 `selected_index` 对应的会话，
    /// 列表为空时返回 `None`。
    pub fn selected(&self) -> Option<&SessionInfo> {
        let filtered = self.filtered_sessions();
        filtered.into_iter().nth(self.selected_index)
    }

    /// 设置过滤文本并重置选择索引
    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_string();
        self.selected_index = 0;
    }

    /// 获取过滤后的会话列表
    ///
    /// 按预览文本和工作目录进行模糊匹配（大小写不敏感）。
    /// 过滤器为空时返回所有会话。
    pub fn filtered_sessions(&self) -> Vec<&SessionInfo> {
        if self.filter.is_empty() {
            return self.sessions.iter().collect();
        }

        let filter_lower = self.filter.to_lowercase();
        self.sessions
            .iter()
            .filter(|s| {
                s.preview.to_lowercase().contains(&filter_lower)
                    || s.working_dir.to_lowercase().contains(&filter_lower)
            })
            .collect()
    }

    /// 切换显示/隐藏
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl Default for SessionSelector {
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

    /// 创建测试用的会话列表
    fn sample_sessions() -> Vec<SessionInfo> {
        vec![
            SessionInfo {
                id: "s1".into(),
                timestamp: "2025-01-01T10:00:00Z".into(),
                preview: "实现用户认证模块".into(),
                message_count: 12,
                working_dir: "/home/user/project-alpha".into(),
            },
            SessionInfo {
                id: "s2".into(),
                timestamp: "2025-01-02T14:30:00Z".into(),
                preview: "修复数据库连接问题".into(),
                message_count: 8,
                working_dir: "/home/user/project-beta".into(),
            },
            SessionInfo {
                id: "s3".into(),
                timestamp: "2025-01-03T09:15:00Z".into(),
                preview: "重构前端组件".into(),
                message_count: 20,
                working_dir: "/home/user/project-alpha".into(),
            },
        ]
    }

    #[test]
    fn test_new_empty() {
        // 新建的选择器应为空且不可见
        let selector = SessionSelector::new();
        assert!(selector.sessions.is_empty());
        assert_eq!(selector.selected_index, 0);
        assert!(selector.filter.is_empty());
        assert!(!selector.visible);
    }

    #[test]
    fn test_set_sessions() {
        // 设置会话列表后应包含所有会话并重置索引
        let mut selector = SessionSelector::new();
        selector.selected_index = 5; // 故意设置一个非零值
        selector.set_sessions(sample_sessions());
        assert_eq!(selector.sessions.len(), 3);
        assert_eq!(selector.selected_index, 0); // 应已重置
    }

    #[test]
    fn test_select_next_wraps() {
        // 到达底部时应循环到顶部
        let mut selector = SessionSelector::new();
        selector.set_sessions(sample_sessions());

        selector.select_next(); // 0 -> 1
        assert_eq!(selector.selected_index, 1);

        selector.select_next(); // 1 -> 2
        assert_eq!(selector.selected_index, 2);

        selector.select_next(); // 2 -> 0（循环）
        assert_eq!(selector.selected_index, 0);
    }

    #[test]
    fn test_select_previous_wraps() {
        // 到达顶部时应循环到底部
        let mut selector = SessionSelector::new();
        selector.set_sessions(sample_sessions());

        selector.select_previous(); // 0 -> 2（循环）
        assert_eq!(selector.selected_index, 2);

        selector.select_previous(); // 2 -> 1
        assert_eq!(selector.selected_index, 1);
    }

    #[test]
    fn test_selected_empty() {
        // 空列表时应返回 None
        let selector = SessionSelector::new();
        assert!(selector.selected().is_none());
    }

    #[test]
    fn test_filter_by_preview() {
        // 按预览文本过滤
        let mut selector = SessionSelector::new();
        selector.set_sessions(sample_sessions());
        selector.set_filter("数据库");

        let filtered = selector.filtered_sessions();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "s2");
    }

    #[test]
    fn test_filter_by_working_dir() {
        // 按工作目录过滤
        let mut selector = SessionSelector::new();
        selector.set_sessions(sample_sessions());
        selector.set_filter("project-alpha");

        let filtered = selector.filtered_sessions();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "s1");
        assert_eq!(filtered[1].id, "s3");
    }

    #[test]
    fn test_toggle_visibility() {
        // 切换可见性
        let mut selector = SessionSelector::new();
        assert!(!selector.visible);

        selector.toggle();
        assert!(selector.visible);

        selector.toggle();
        assert!(!selector.visible);
    }

    #[test]
    fn test_filtered_preserves_selection() {
        // 设置过滤器后选择索引应重置
        let mut selector = SessionSelector::new();
        selector.set_sessions(sample_sessions());
        selector.selected_index = 2;

        selector.set_filter("用户");
        assert_eq!(selector.selected_index, 0);

        // 选中项应为过滤结果的第一项
        let selected = selector.selected().unwrap();
        assert_eq!(selected.id, "s1");
    }
}
