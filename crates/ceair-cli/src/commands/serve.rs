//! # Serve 命令
//!
//! 实现 `ceair serve` 子命令，启动本地 HTTP + WebSocket 控制服务器。
//! 浏览器通过 localhost 连接，可创建会话、发送消息、查看流式输出和审批工具调用。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use tokio::sync::mpsc;
use tracing::info;

use ceair_agent::{AgentContext, AgentLoop, AgentLoopConfig};
use ceair_agent::executor::ToolExecutor;
use ceair_ai::{AiProvider, ChatOptions, ProviderConfig, ProviderFactory};
use ceair_config::CeairConfig;
use ceair_control_server::{start_server, ControlServerConfig};
use ceair_core::{AgentEvent, AgentId, SessionId};
use ceair_tools::{create_default_registry, SecurityPolicy};
use ceair_worker::{AgentExecutor, WorkerRuntime};

/// Serve 命令的参数
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// 绑定地址（默认 127.0.0.1:3200）
    #[arg(long, default_value = "127.0.0.1:3200")]
    pub bind: SocketAddr,
}

/// Real agent executor that creates an AgentLoop per turn.
struct LocalAgentExecutor {
    provider: Arc<dyn AiProvider>,
    chat_options: ChatOptions,
    config: CeairConfig,
}

#[async_trait::async_trait]
impl AgentExecutor for LocalAgentExecutor {
    async fn execute_turn(
        &self,
        session_id: String,
        user_message: String,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Result<(), String> {
        let sid = SessionId::new();
        let agent_id = AgentId::new();
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let mut context = AgentContext::new(sid, working_dir);
        context.add_user_message(&user_message);

        let policy = SecurityPolicy::default_policy();
        let registry = Arc::new(create_default_registry(policy));
        let tool_executor = ToolExecutor::new(registry);

        let loop_config = AgentLoopConfig {
            max_iterations: self.config.agent.max_iterations,
            auto_approve_tools: false,
            ..Default::default()
        };

        let mut agent_loop = AgentLoop::new(
            agent_id,
            Arc::clone(&self.provider),
            tool_executor,
            context,
            loop_config,
        );

        let cancel_token = tokio_util::sync::CancellationToken::new();

        agent_loop
            .run(&self.chat_options, cancel_token, event_tx)
            .await
            .map_err(|e| format!("{}", e))?;

        Ok(())
    }
}

/// 执行 serve 命令
///
/// 启动本地控制服务器，打印访问地址和认证令牌，然后等待关闭信号。
pub async fn execute(args: ServeArgs, config: CeairConfig) -> Result<()> {
    info!("正在启动控制服务器...");

    // Create AI provider
    let provider = setup_provider(&config)?;
    let chat_options = ChatOptions::with_model(&config.ai.model)
        .temperature(config.ai.temperature)
        .max_tokens(config.ai.max_tokens);

    let executor: Arc<dyn AgentExecutor> = Arc::new(LocalAgentExecutor {
        provider: Arc::from(provider),
        chat_options,
        config: config.clone(),
    });

    let runtime = Arc::new(WorkerRuntime::with_executor(executor));

    let server_config = ControlServerConfig {
        bind_addr: args.bind,
    };

    let token = start_server(server_config, runtime).await?;

    println!();
    println!("🌐 CEAIR 控制服务器已启动");
    println!("   地址: http://{}", args.bind);
    println!("   令牌: {}", token);
    println!();
    println!("在浏览器中打开上方地址，使用令牌进行认证。");
    println!("按 Ctrl+C 停止服务器。");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("收到中断信号，正在关闭控制服务器...");
    println!("\n正在关闭...");

    Ok(())
}

fn setup_provider(config: &CeairConfig) -> Result<Box<dyn AiProvider>> {
    let provider_name = &config.ai.provider;

    let api_key = config.ai.api_key.clone().or_else(|| {
        let env_key = format!("{}_API_KEY", provider_name.to_uppercase());
        std::env::var(&env_key).ok()
            .or_else(|| std::env::var("CEAIR_API_KEY").ok())
    });

    let api_key = api_key.unwrap_or_default();

    let provider_config = ProviderConfig {
        api_key,
        api_secret: config.ai.api_secret.clone().or_else(|| {
            let env_secret = format!("{}_API_SECRET", provider_name.to_uppercase());
            std::env::var(&env_secret).ok()
        }),
        base_url: config.ai.base_url.clone(),
        default_model: Some(config.ai.model.clone()),
        timeout_secs: config.agent.timeout_secs,
        extra: HashMap::new(),
    };

    ProviderFactory::create_provider(provider_name, provider_config)
        .map_err(|e| anyhow::anyhow!("创建 AI 提供商 '{}' 失败: {}", provider_name, e))
}
