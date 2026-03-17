mod config;
mod agent;
mod workflow;
mod tools;
mod memory;
mod api_key;

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use crate::agent::AgentManager;
use crate::config::AppConfig;
use crate::memory::MemoryManager;

/// 应用状态（共享给所有路由）
pub struct AppState {
    pub agent_manager: Arc<AgentManager>,
    pub memory_manager: Arc<MemoryManager>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 启动 Rig Agent Scaffold");

    // 加载环境变量
    dotenvy::dotenv().ok();

    // 加载配置
    let config = AppConfig::load()?;
    info!("✅ 配置加载完成");

    // 初始化记忆管理器（Qdrant 向量数据库）
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let embedding_api_url = std::env::var("EMBEDDING_API_URL")
        .unwrap_or_else(|_| "https://dashscope.aliyuncs.com/compatible-mode".to_string());
    let embedding_api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set");
    
    let memory_manager = match MemoryManager::new(&qdrant_url, &embedding_api_url, &embedding_api_key).await {
        Ok(mm) => {
            info!("✅ Qdrant 记忆系统初始化完成");
            mm
        }
        Err(e) => {
            warn!("Qdrant 连接失败：{}，记忆功能将不可用", e);
            // 创建一个假的 MemoryManager（或者 panic）
            panic!("无法初始化记忆系统，请确保 Qdrant 正在运行：{}", e);
        }
    };

    // 初始化 Agent 管理器
    let agent_manager = Arc::new(AgentManager::new(config).await?);
    info!("✅ Agent 管理器初始化完成，可用 Agent: {:?}", agent_manager.list_agents());

    // 创建应用状态
    let state = Arc::new(AppState {
        agent_manager,
        memory_manager: memory_manager.into(),
    });

    // 启动 HTTP 服务器
    start_server(state).await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct ChatRequest {
    agent_id: String,
    message: String,
    user_id: Option<String>,      // 可选的用户 ID，用于记忆
    session_id: Option<String>,   // 可选的会话 ID，用于短期记忆
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    memory_summary: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    agents: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct MemoryRequest {
    user_id: String,
}

#[derive(Serialize)]
struct MemoryResponse {
    summary: String,
}

async fn start_server(state: Arc<AppState>) -> Result<()> {
    let app = Router::new()
        // 健康检查
        .route("/health", get(|state: State<Arc<AppState>>| async move {
            let agents = state.agent_manager.list_agents();
            Json(HealthResponse {
                status: "ok".to_string(),
                agents,
            })
        }))
        // 聊天接口
        .route("/chat", post(|state: State<Arc<AppState>>, Json(req): Json<ChatRequest>| async move {
            handle_chat(state, req).await
        }))
        // 获取记忆摘要
        .route("/memory", get(|state: State<Arc<AppState>>, Json(req): Json<MemoryRequest>| async move {
            match state.memory_manager.get_user_observation_summary(&req.user_id).await {
                Ok(summary) => Ok::<_, StatusCode>(Json(MemoryResponse { summary })),
                Err(e) => {
                    tracing::error!("Memory error: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }))
        // 清除短期记忆
        .route("/memory/clear", post(|state: State<Arc<AppState>>, Json(req): Json<MemoryRequest>| async move {
            // 这里可以添加清除短期记忆的逻辑
            Ok::<_, StatusCode>((StatusCode::OK, Json(serde_json::json!({"status": "ok"}))))
        }))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("🌐 服务器启动在 {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 处理聊天请求
async fn handle_chat(
    state: State<Arc<AppState>>,
    req: ChatRequest,
) -> (StatusCode, Json<ChatResponse>) {
    // 生成默认的 user_id 和 session_id（如果没有提供）
    let user_id = req.user_id.unwrap_or_else(|| "default_user".to_string());
    let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // 构建带记忆的 prompt
    let memory_context = state.memory_manager.build_memory_context(&user_id, &session_id).await.ok();
    
    let prompt = if let Some(ref ctx) = memory_context {
        format!("{}\n\n{}\n\n用户问题：{}", 
            state.agent_manager.get_agent_config(&req.agent_id)
                .and_then(|c| c.agents.first())
                .map(|a| a.instruction.clone())
                .unwrap_or_default(),
            ctx,
            req.message
        )
    } else {
        req.message.clone()
    };

    // 调用 Agent
    match state.agent_manager.chat(&req.agent_id, &prompt).await {
        Ok(response) => {
            // 1. 保存到短期记忆
            let _ = state.memory_manager
                .add_to_ephemeral(&session_id, &req.message, &response, 20)
                .await;

            // 2. 从对话中提取观察并存储到长期记忆
            let _ = state.memory_manager
                .extract_observations_from_conversation(&user_id, &session_id, &req.message, &response)
                .await;

            // 3. 检查是否需要压缩短期记忆
            if let Ok(Some(summary)) = state.memory_manager.compact_ephemeral(&session_id).await {
                tracing::info!("对话已压缩: {}...", &summary[..summary.len().min(50)]);
            }

            // 获取记忆摘要用于返回
            let memory_summary = state.memory_manager.get_user_observation_summary(&user_id).await.ok();

            (StatusCode::OK, Json(ChatResponse {
                response,
                memory_summary,
            }))
        }
        Err(e) => {
            tracing::error!("Chat error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ChatResponse {
                response: e.to_string(),
                memory_summary: None,
            }))
        }
    }
}
