use anyhow::{Result, Context};
use rig::{completion::Prompt, providers::openai::Client};
use rig::providers::openai;
use rig::client::{CompletionClient, ProviderClient};
use tracing::info;

use crate::config::{AppConfig, AgentConfig};

/// Agent 管理器
pub struct AgentManager {
    config: AppConfig,
}

impl AgentManager {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let agent_config = &config.agent;
        info!("初始化 Agent: {} ({})", agent_config.agent_name, agent_config.agent_id);
        Ok(Self { config })
    }

    pub fn list_agents(&self) -> Vec<String> {
        self.config.agent.agents.iter()
            .map(|agent| format!("{}_{}", self.config.agent.agent_id, agent.name))
            .collect()
    }

    pub async fn chat(&self, agent_id: &str, message: &str) -> Result<String> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY 环境变量未设置")?;
        
        // 找到对应的 Agent 配置
        let agent_def = self.config.agent.agents.iter()
            .find(|a| format!("{}_{}", self.config.agent.agent_id, a.name) == agent_id)
            .with_context(|| format!("Agent 未找到：{}", agent_id))?;
        
        let model_name = &self.config.agent.module.chat_model.model;
        
        // 直接调用 Dashscope API
        let client = reqwest::Client::new();
        let response = client
            .post("https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": model_name,
                "messages": [
                    {"role": "system", "content": agent_def.instruction},
                    {"role": "user", "content": message}
                ]
            }))
            .send()
            .await
            .context("HTTP 请求失败")?;

        let json: serde_json::Value = response.json().await.context("解析响应失败")?;
        
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("响应格式错误: {:?}", json))?
            .to_string();

        Ok(content)
    }

    pub fn get_agent_config(&self, _agent_id: &str) -> Option<&AgentConfig> {
        Some(&self.config.agent)
    }
}