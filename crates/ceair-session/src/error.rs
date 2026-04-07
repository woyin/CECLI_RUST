//! 会话错误类型模块
//!
//! 定义了会话管理过程中可能出现的所有错误类型。

/// 会话管理错误类型
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// IO 操作错误
    #[error("会话 IO 错误: {0}")]
    Io(String),

    /// 序列化/反序列化错误
    #[error("会话序列化错误: {0}")]
    Serialization(String),

    /// 文件格式错误
    #[error("会话文件格式错误: {0}")]
    Format(String),

    /// 会话未找到
    #[error("会话未找到: {0}")]
    NotFound(String),

    /// 条目未找到
    #[error("条目未找到: {0}")]
    EntryNotFound(String),
}

/// 会话模块的 Result 类型别名
pub type SessionResult<T> = std::result::Result<T, SessionError>;

impl From<std::io::Error> for SessionError {
    fn from(err: std::io::Error) -> Self {
        SessionError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for SessionError {
    fn from(err: serde_json::Error) -> Self {
        SessionError::Serialization(err.to_string())
    }
}
