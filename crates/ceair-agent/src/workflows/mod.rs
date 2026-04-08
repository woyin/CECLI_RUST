//! # 编排工作流模块
//!
//! 实现多Agent协作的工作流引擎，包含规划、执行、会话连续性等核心流程。

/// Prometheus 规划工作流
pub mod prometheus;
/// Atlas 执行编排
pub mod atlas;
/// Boulder 会话连续性系统
pub mod boulder;
/// UltraWork 全自动模式
pub mod ultrawork;
