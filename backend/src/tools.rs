//! 工具系统 - 实现工具使用模式
//! 
//! 支持的工具类型：
//! - 内置工具：get_current_date, get_user_memory
//! - HTTP 工具：自定义 API 调用

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// 工具定义
pub struct Tool {
    pub name: String,
    pub description: String,
    pub handler: Box<dyn Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>> + Send + Sync>,
}

/// 工具注册表
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// 注册工具
    pub fn register<F, Fut>(&mut self, name: &str, description: &str, handler: F)
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<String>> + Send + 'static,
    {
        let tool = Tool {
            name: name.to_string(),
            description: description.to_string(),
            handler: Box::new(move |input| Box::pin(handler(input))),
        };

        self.tools.get_mut().insert(name.to_string(), tool);
    }

    /// 执行工具
    pub async fn execute(&self, name: &str, input: &str) -> Result<String> {
        let tools = self.tools.read().await;
        
        let tool = tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("工具未找到：{}", name))?;

        (tool.handler)(input.to_string()).await
    }

    /// 列出所有工具
    pub async fn list_tools(&self) -> Vec<String> {
        self.tools.read().await.keys().cloned().collect()
    }
}

/// 创建默认工具集
pub fn create_default_tools() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // 内置工具：获取当前日期
    registry.register("get_current_date", "获取当前日期和时间", |_| async {
        let now: DateTime<Local> = Local::now();
        Ok(now.format("%Y-%m-%d %H:%M:%S").to_string())
    });

    // 内置工具：获取用户记忆（占位实现）
    registry.register("get_user_memory", "获取用户的个人信息、偏好和历史记忆", |input| async move {
        // TODO: 连接到实际的记忆存储
        Ok(format!("用户记忆：{}", input))
    });

    // HTTP 工具：Bing 搜索（占位实现）
    registry.register("bing_search", "必应搜索引擎，用于搜索互联网上的最新信息", |query| async move {
        // TODO: 实现实际的 Bing 搜索 API 调用
        Ok(format!("Bing 搜索结果：{}", query))
    });

    // HTTP 工具：天气查询（占位实现）
    registry.register("weather_query", "查询指定城市的天气", |city| async move {
        // TODO: 连接到天气 API
        Ok(format!("{} 的天气：晴朗，25°C", city))
    });

    registry
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub input: String,
    pub output: String,
    pub success: bool,
}

impl ToolCallResult {
    pub fn success(tool_name: &str, input: &str, output: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            input: input.to_string(),
            output: output.to_string(),
            success: true,
        }
    }

    pub fn failure(tool_name: &str, input: &str, error: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            input: input.to_string(),
            output: error.to_string(),
            success: false,
        }
    }
}
