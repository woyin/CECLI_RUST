//! # 命令模块
//!
//! 本模块包含 CEAIR CLI 的所有子命令实现：
//! - [`launch`] - 启动 AI 智能体（交互模式或单次任务模式）
//! - [`config`] - 管理配置项（查看、设置、获取、初始化）
//! - [`status`] - 显示系统运行状态
//! - [`serve`] - 启动本地 Web 控制服务器

/// 启动命令 - 启动 AI 智能体执行任务
pub mod launch;

/// 配置命令 - 查看和管理配置
pub mod config;

/// 初始化命令 - 初始化项目 CEAIR 配置
pub mod init;

/// 状态命令 - 显示系统运行状态
pub mod status;

/// 控制服务器命令 - 启动 HTTP + WebSocket 控制服务
pub mod serve;
