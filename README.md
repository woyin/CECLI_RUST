# CEAIR — AI 编程助手 (Rust 实现)

> 基于公开文档的 Clean-room 重新实现，使用 Rust 构建的终端 AI 编程代理。

## 📊 项目状态

| 指标 | 数值 |
|------|------|
| **Crates** | 11 个工作空间成员 |
| **源文件** | 98 个 .rs 文件 |
| **代码行数** | ~44,000 行 |
| **测试数量** | 974 个单元测试 |
| **测试状态** | ✅ 全部通过 |
| **许可证** | Apache-2.0 |

## 🏗️ 架构

| 模块 | 说明 | 核心功能 |
|------|------|---------|
| **ceair-core** | 核心类型 | 消息系统、错误处理、事件总线 |
| **ceair-session** | 会话管理 | JSONL 树形存储、分支、Blob 存储 |
| **ceair-ai** | AI 提供者 | OpenAI/Anthropic/DeepSeek/通义千问/文心一言、模型角色路由 |
| **ceair-agent** | 代理引擎 | Agent 循环、管道编排、上下文压缩、TTSR、Hashline 编辑、记忆系统、钩子/技能/自定义工具 |
| **ceair-tools** | 工具集 | 18 个内置工具（见下方） |
| **ceair-config** | 配置管理 | 多工具配置发现、加密存储、模型配置 |
| **ceair-audit** | 审计安全 | 日志链、数据脱敏、密钥混淆 |
| **ceair-mesh** | 多代理 | 共享状态、消息总线、任务编排 |
| **ceair-mcp** | MCP 协议 | JSON-RPC 2.0、stdio/SSE 传输 |
| **ceair-tui** | 终端界面 | Markdown 渲染、主题系统、会话选择器 |
| **ceair-cli** | 命令行 | 斜杠命令、RPC 模式、项目初始化 |

## 🔧 内置工具（18 个）

| 工具 | 说明 |
|------|------|
| `bash` | Shell 命令执行、超时控制、输出截断 |
| `read` | 文件读取 |
| `write` | 文件写入 |
| `edit` | 精确字符串匹配编辑 |
| `grep` | 正则搜索、glob 过滤、上下文行 |
| `find` | 文件查找、glob 匹配、类型过滤 |
| `python` | Python REPL、语法校验、安全检查 |
| `notebook` | Jupyter Notebook 操作 |
| `browser` | 网页交互与截图 |
| `ssh` | 远程命令执行 |
| `lsp` | 语言服务器协议集成 |
| `ask` | 结构化用户交互 |
| `todo` | 分阶段任务跟踪 |
| `task` | 子任务代理委派 |
| `fetch` | URL 内容抓取、HTML 转文本 |
| `web_search` | 多引擎搜索（Brave/Jina） |
| `calc` | 数学表达式求值 |
| `ast_grep` | AST 代码搜索与编辑 |

## 🤖 AI 提供者

- **OpenAI 兼容** — GPT-4o 等，可配置 base_url（支持 Ollama/LM Studio/vLLM）
- **Anthropic** — Claude 系列，Messages API 格式
- **DeepSeek** — DeepSeek Chat/Coder
- **通义千问** — 阿里云 DashScope
- **文心一言** — 百度 ERNIE
- **模型角色** — Default/Smol/Slow/Plan/Commit + 思考级别

## ⚡ 核心特性

- **TTSR 引擎** — 基于正则触发的零成本流式规则注入
- **Hashline 编辑** — SHA-256 内容哈希锚点精确定位
- **上下文压缩** — 自动/手动对话摘要，保持上下文窗口可控
- **记忆系统** — 跨会话知识提取与整合（默认关闭）
- **密钥混淆** — Placeholder/Redact 两种模式保护敏感数据
- **配置发现** — 兼容 .ceair/.claude/.codex/.gemini 多工具配置
- **扩展系统** — 钩子（生命周期拦截）、技能包、自定义工具
- **斜杠命令** — 18 个内置命令 + Markdown 自定义命令
- **RPC 模式** — JSONL stdio 协议，支持编程式访问
- **MCP 协议** — Model Context Protocol 客户端/服务器
- **多代理** — 共享状态、消息总线、角色系统、任务编排

## 🚀 快速开始

```bash
# 构建
cargo build --release

# 运行
./target/release/ceair --help

# 初始化项目
ceair init

# 启动会话
ceair launch

# 查看状态
ceair status
```

## 📋 开发

```bash
# 运行所有测试
cargo test --workspace

# 检查编译
cargo check --workspace

# 构建发布版
cargo build --release
```

## 📜 许可证

Apache-2.0

---

> 本项目为 Clean-room 重新实现，仅参考公开文档，未参考任何原始源代码。
