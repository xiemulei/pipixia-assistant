use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(flatten)]
    pub agent: AgentConfig,
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(rename = "agent_id")]
    pub agent_id: String,
    #[serde(rename = "agent_name")]
    pub agent_name: String,
    #[serde(rename = "agent_desc")]
    pub agent_desc: String,
    pub module: ModuleConfig,
    pub agents: Vec<AgentDefinition>,
    #[serde(default, rename = "agent_workflows")]
    pub agent_workflows: Vec<WorkflowConfig>,
    #[serde(default, rename = "skill_locations")]
    pub skill_locations: Vec<String>,
}

/// 模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub ai_api: AiApiConfig,
    pub chat_model: ChatModelConfig,
}

/// AI API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiApiConfig {
    pub base_url: String,
    pub api_key: String,
    pub completions_path: String,
    #[serde(default)]
    pub embeddings_path: String,
}

/// 聊天模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatModelConfig {
    pub model: String,
    #[serde(default)]
    pub tool_mcp_list: Vec<ToolConfig>,
}

/// 工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    #[serde(default)]
    pub stdio: Option<StdioTool>,
    #[serde(default)]
    pub sse: Option<SseTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioTool {
    pub name: String,
    pub server_parameters: ServerParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseTool {
    pub name: String,
    pub base_uri: String,
    pub sse_endpoint: String,
    #[serde(default)]
    pub request_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerParameters {
    pub command: String,
    pub args: Vec<String>,
}

/// Agent 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    pub instruction: String,
    #[serde(default)]
    pub output_key: Option<String>,
}

/// 工作流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    #[serde(default)]
    #[serde(rename = "type")]
    pub workflow_type: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub sub_agents: Vec<String>,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // 简单测试：直接加载一个配置文件
        let config_path = Path::new("agents/general-agent.yml");
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config: AppConfig = serde_yaml::from_str(&content)?;
            return Ok(config);
        }

        anyhow::bail!("配置文件不存在：agents/general-agent.yml")
    }
}
