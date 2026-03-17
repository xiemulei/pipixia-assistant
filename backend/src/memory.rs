//! 记忆系统 - 基于 Rig 官方最佳实践
//! 
//! 参考: https://book.rig.rs/playbook/memory.html
//!
//! 三层记忆架构:
//! 1. Ephemeral Memory (短期记忆) - 当前对话历史，支持压缩
//! 2. Long-term Memory (长期记忆) - 跨会话持久化
//!    - ConversationObservation: 对话洞察
//!    - UserObservation: 用户偏好/上下文
//!    - GroundedFact: 已验证事实
//! 3. Semantic Memory (语义记忆) - Qdrant 向量存储

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, QueryPointsBuilder, 
        UpsertPointsBuilder, VectorParamsBuilder, Filter, Condition, Value,
    },
    Qdrant,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 向量维度
const VECTOR_SIZE: usize = 1024;
const COLLECTION_NAME: &str = "agent_memories";

// ============================================================================
// 短期记忆 - Ephemeral Memory
// ============================================================================

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

/// 短期记忆 - 管理当前会话的对话历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralMemory {
    pub session_id: String,
    pub messages: Vec<Message>,
    pub summary: Option<String>,
    pub max_messages: usize,
    pub created_at: String,
    pub updated_at: String,
}

impl EphemeralMemory {
    pub fn new(session_id: &str, max_messages: usize) -> Self {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            session_id: session_id.to_string(),
            messages: Vec::new(),
            summary: None,
            max_messages,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 添加用户消息
    pub fn add_user_message(&mut self, content: &str) {
        let message = Message {
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        self.messages.push(message);
        self.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    /// 添加助手消息
    pub fn add_assistant_message(&mut self, content: &str) {
        let message = Message {
            role: "assistant".to_string(),
            content: content.to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        self.messages.push(message);
        self.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    /// 检查是否需要压缩
    pub fn needs_compaction(&self) -> bool {
        self.messages.len() > self.max_messages
    }

    /// 获取格式化的消息用于摘要生成
    pub fn format_messages_for_summary(&self) -> String {
        self.messages
            .iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 清空消息（压缩后）
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// 设置摘要
    pub fn set_summary(&mut self, summary: String) {
        self.summary = Some(summary);
        self.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    /// 获取带摘要的消息
    pub fn get_messages_with_summary(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        if let Some(summary) = &self.summary {
            messages.push(Message {
                role: "user".to_string(),
                content: format!("之前的对话摘要:\n{}", summary),
                timestamp: self.updated_at.clone(),
            });
        }

        messages.extend(self.messages.clone());
        messages
    }
}

// ============================================================================
// 长期记忆 - Long-term Memory
// ============================================================================

/// 对话观察 - 从对话中提取的洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationObservation {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub topic: String,
    pub insight: String,
    pub importance: f32,
    pub conversation_id: String,
}

/// 用户观察 - 用户的持久信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserObservation {
    pub preferences: HashMap<String, String>,
    pub context: Vec<String>,
    pub communication_style: Option<String>,
    pub goals: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

/// 已验证事实 - 客观可验证的信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedFact {
    pub id: String,
    pub fact: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f32,
    pub conversation_id: String,
}

/// 记忆类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    ConversationObservation,
    UserObservation,
    GroundedFact,
}

/// 记忆搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub id: String,
    pub user_id: String,
    pub memory_type: String,
    pub content: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// 记忆管理器
// ============================================================================

/// 记忆管理器 - 整合三层记忆架构
pub struct MemoryManager {
    // Qdrant 向量存储
    qdrant: Arc<Qdrant>,
    
    // 短期记忆 (按 session_id 索引)
    ephemeral_memories: RwLock<HashMap<String, EphemeralMemory>>,
    
    // 用户观察缓存 (按 user_id 索引)
    user_observations: RwLock<HashMap<String, UserObservation>>,
    
    // API 配置
    embedding_api_url: String,
    embedding_api_key: String,
    llm_api_key: String,
}

impl MemoryManager {
    pub async fn new(qdrant_url: &str, embedding_api_url: &str, embedding_api_key: &str) -> Result<Self> {
        let llm_api_key = embedding_api_key.to_string();
        
        tracing::info!("连接 Qdrant：{}", qdrant_url);
        
        let qdrant = Qdrant::from_url(qdrant_url).build()?;
        
        let _ = qdrant.create_collection(
            CreateCollectionBuilder::new(COLLECTION_NAME)
                .vectors_config(VectorParamsBuilder::new(VECTOR_SIZE as u64, Distance::Cosine)),
        ).await;
        
        tracing::info!("✅ Qdrant 连接成功");

        Ok(Self {
            qdrant: Arc::new(qdrant),
            ephemeral_memories: RwLock::new(HashMap::new()),
            user_observations: RwLock::new(HashMap::new()),
            embedding_api_url: embedding_api_url.to_string(),
            embedding_api_key: embedding_api_key.to_string(),
            llm_api_key,
        })
    }

    // ========================================================================
    // 短期记忆操作
    // ========================================================================

    /// 获取或创建短期记忆
    pub async fn get_or_create_ephemeral(&self, session_id: &str, max_messages: usize) -> EphemeralMemory {
        let mut memories = self.ephemeral_memories.write().await;

        if let Some(memory) = memories.get(session_id) {
            memory.clone()
        } else {
            let memory = EphemeralMemory::new(session_id, max_messages);
            memories.insert(session_id.to_string(), memory.clone());
            memory
        }
    }

    /// 添加对话到短期记忆
    pub async fn add_to_ephemeral(
        &self,
        session_id: &str,
        user_message: &str,
        assistant_response: &str,
        max_messages: usize,
    ) -> Result<()> {
        let mut memories = self.ephemeral_memories.write().await;

        let memory = memories
            .entry(session_id.to_string())
            .or_insert_with(|| EphemeralMemory::new(session_id, max_messages));

        memory.add_user_message(user_message);
        memory.add_assistant_message(assistant_response);

        Ok(())
    }

    /// 压缩短期记忆（生成摘要）
    pub async fn compact_ephemeral(&self, session_id: &str) -> Result<Option<String>> {
        let mut memories = self.ephemeral_memories.write().await;

        if let Some(memory) = memories.get_mut(session_id) {
            if !memory.needs_compaction() {
                return Ok(None);
            }

            // 生成摘要
            let summary_prompt = format!(
                "请为以下对话生成简洁的摘要，保留关键信息、决策和上下文:\n\n{}",
                memory.format_messages_for_summary()
            );

            let summary = self.call_llm(&summary_prompt).await?;
            
            memory.set_summary(summary.clone());
            memory.clear_messages();

            tracing::info!("✅ 短期记忆已压缩，摘要: {}...", &summary[..summary.len().min(100)]);

            return Ok(Some(summary));
        }

        Ok(None)
    }

    /// 获取短期记忆的带摘要消息
    pub async fn get_ephemeral_messages(&self, session_id: &str) -> Vec<Message> {
        let memories = self.ephemeral_memories.read().await;

        if let Some(memory) = memories.get(session_id) {
            memory.get_messages_with_summary()
        } else {
            Vec::new()
        }
    }

    // ========================================================================
    // 长期记忆操作
    // ========================================================================

    /// 存储对话观察
    pub async fn store_conversation_observation(
        &self,
        user_id: &str,
        session_id: &str,
        topic: &str,
        insight: &str,
        importance: f32,
    ) -> Result<String> {
        let content = format!("[{}] {}", topic, insight);
        
        let mut metadata = HashMap::new();
        metadata.insert("topic".to_string(), topic.to_string());
        metadata.insert("importance".to_string(), importance.to_string());
        metadata.insert("conversation_id".to_string(), session_id.to_string());

        self.store_to_qdrant(user_id, "conversation_observation", &content, metadata).await
    }

    /// 存储用户观察
    pub async fn store_user_observation(
        &self,
        user_id: &str,
        category: &str,
        observation: &str,
    ) -> Result<String> {
        // 更新内存缓存
        {
            let mut observations = self.user_observations.write().await;
            let user_obs = observations
                .entry(user_id.to_string())
                .or_insert_with(|| UserObservation {
                    last_updated: Utc::now(),
                    ..Default::default()
                });

            match category {
                "preference" => {
                    // 格式: "key:value"
                    if let Some((key, value)) = observation.split_once(':') {
                        user_obs.preferences.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
                "context" => {
                    if !user_obs.context.contains(&observation.to_string()) {
                        user_obs.context.push(observation.to_string());
                    }
                }
                "communication_style" => {
                    user_obs.communication_style = Some(observation.to_string());
                }
                "goal" => {
                    if !user_obs.goals.contains(&observation.to_string()) {
                        user_obs.goals.push(observation.to_string());
                    }
                }
                _ => {}
            }

            user_obs.last_updated = Utc::now();
        }

        // 存储到 Qdrant
        let content = format!("[{}] {}", category, observation);
        
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), category.to_string());

        self.store_to_qdrant(user_id, "user_observation", &content, metadata).await
    }

    /// 存储已验证事实
    pub async fn store_grounded_fact(
        &self,
        user_id: &str,
        session_id: &str,
        fact: &str,
        source: &str,
        confidence: f32,
    ) -> Result<String> {
        let content = fact.to_string();
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), source.to_string());
        metadata.insert("confidence".to_string(), confidence.to_string());
        metadata.insert("conversation_id".to_string(), session_id.to_string());

        self.store_to_qdrant(user_id, "grounded_fact", &content, metadata).await
    }

    /// 从对话中自动提取观察
    pub async fn extract_observations_from_conversation(
        &self,
        user_id: &str,
        session_id: &str,
        user_message: &str,
        assistant_response: &str,
    ) -> Result<Vec<String>> {
        let extraction_prompt = format!(
            r#"分析以下对话，提取关键观察信息。返回 JSON 数组格式。

对话:
用户: {}
助手: {}

提取规则:
1. 用户偏好: 用户表达的好恶、习惯 (如 "运动:跑步")
2. 用户上下文: 用户的背景信息 (如 "职业:程序员")
3. 沟通风格: 用户的交流偏好 (如 "喜欢简洁的回答")
4. 目标: 用户提到的目标或计划

返回格式 (JSON 数组):
[{{"type": "preference", "content": "运动:跑步"}}, ...]

如果没有观察到重要信息，返回空数组 []"#,
            user_message, assistant_response
        );

        let response = self.call_llm(&extraction_prompt).await?;

        // 解析 JSON
        let content = response.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let observations: Vec<serde_json::Value> = serde_json::from_str(content)
            .unwrap_or_default();

        let mut stored_ids = Vec::new();

        for obs in observations {
            if let (Some(obs_type), Some(obs_content)) = (
                obs.get("type").and_then(|v| v.as_str()),
                obs.get("content").and_then(|v| v.as_str()),
            ) {
                let id = self.store_user_observation(user_id, obs_type, obs_content).await?;
                stored_ids.push(id);
            }
        }

        if !stored_ids.is_empty() {
            tracing::info!("🔍 提取并存储了 {} 条用户观察", stored_ids.len());
        }

        Ok(stored_ids)
    }

    // ========================================================================
    // 记忆检索
    // ========================================================================

    /// 语义搜索相似记忆
    pub async fn search_similar(
        &self,
        query: &str,
        user_id: &str,
        memory_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<MemorySearchResult>> {
        let query_vector = self.get_embedding(query).await?;

        // 构建过滤器
        let mut conditions = vec![Condition::matches("user_id", user_id.to_string())];
        
        if let Some(mt) = memory_type {
            conditions.push(Condition::matches("memory_type", mt.to_string()));
        }

        let filter = Filter::all(conditions);

        let response = self.qdrant.query(
            QueryPointsBuilder::new(COLLECTION_NAME)
                .query(query_vector)
                .limit(limit)
                .with_payload(true)
                .filter(filter),
        ).await?;

        let results = response.result.into_iter().filter_map(|p| {
            let payload = p.payload;
            Some(MemorySearchResult {
                id: format!("{:?}", p.id),
                user_id: payload.get("user_id")?.as_str()?.to_string(),
                memory_type: payload.get("memory_type")?.as_str()?.to_string(),
                content: payload.get("content")?.as_str()?.to_string(),
                score: p.score,
                metadata: payload.into_iter()
                    .filter_map(|(k, v)| Some((k, v.as_str()?.to_string())))
                    .collect(),
            })
        }).collect();

        Ok(results)
    }

    /// 获取用户观察摘要
    pub async fn get_user_observation_summary(&self, user_id: &str) -> Result<String> {
        // 先检查内存缓存
        {
            let observations = self.user_observations.read().await;
            if let Some(user_obs) = observations.get(user_id) {
                if !user_obs.preferences.is_empty() || !user_obs.context.is_empty() {
                    return Ok(self.format_user_observation(user_obs));
                }
            }
        }

        // 从 Qdrant 检索
        let results = self.search_similar("用户偏好 上下文 目标", user_id, Some("user_observation"), 10).await?;

        let mut summary = String::from("=== 用户记忆 ===\n");

        // 按类型分组
        let mut preferences: Vec<&MemorySearchResult> = Vec::new();
        let mut contexts: Vec<&MemorySearchResult> = Vec::new();
        let mut goals: Vec<&MemorySearchResult> = Vec::new();

        for result in &results {
            if let Some(category) = result.metadata.get("category") {
                match category.as_str() {
                    "preference" => preferences.push(result),
                    "context" => contexts.push(result),
                    "goal" => goals.push(result),
                    _ => {}
                }
            }
        }

        if !preferences.is_empty() {
            summary.push_str("\n【偏好】\n");
            for p in &preferences {
                summary.push_str(&format!("  • {}\n", p.content));
            }
        }

        if !contexts.is_empty() {
            summary.push_str("\n【上下文】\n");
            for c in &contexts {
                summary.push_str(&format!("  • {}\n", c.content));
            }
        }

        if !goals.is_empty() {
            summary.push_str("\n【目标】\n");
            for g in &goals {
                summary.push_str(&format!("  • {}\n", g.content));
            }
        }

        if preferences.is_empty() && contexts.is_empty() && goals.is_empty() {
            summary.push_str("（暂无记忆）\n");
        }

        Ok(summary)
    }

    /// 获取完整的记忆上下文（用于构建 Agent prompt）
    pub async fn build_memory_context(&self, user_id: &str, session_id: &str) -> Result<String> {
        let mut context = String::new();

        // 1. 用户观察
        let user_obs_summary = self.get_user_observation_summary(user_id).await?;
        context.push_str(&user_obs_summary);

        // 2. 相关的已验证事实
        let facts = self.search_similar("相关事实 重要信息", user_id, Some("grounded_fact"), 5).await?;
        if !facts.is_empty() {
            context.push_str("\n【相关事实】\n");
            for fact in facts {
                context.push_str(&format!("  • {} (来源: {})\n", 
                    fact.content, 
                    fact.metadata.get("source").unwrap_or(&"未知".to_string())
                ));
            }
        }

        // 3. 短期记忆摘要（如果有）
        let messages = self.get_ephemeral_messages(session_id).await;
        if !messages.is_empty() {
            context.push_str("\n【当前对话上下文】\n");
            for msg in messages.iter().take(5) {
                let preview: String = msg.content.chars().take(100).collect();
                context.push_str(&format!("  [{}] {}...\n", msg.role, preview));
            }
        }

        Ok(context)
    }

    // ========================================================================
    // 内部辅助方法
    // ========================================================================

    /// 获取文本的 embedding 向量
    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/v1/embeddings", self.embedding_api_url))
            .header("Authorization", format!("Bearer {}", self.embedding_api_key))
            .json(&serde_json::json!({"model": "text-embedding-v3", "input": text}))
            .send().await?;

        let json: serde_json::Value = response.json().await?;
        Ok(json["data"][0]["embedding"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid embedding"))?
            .iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
    }

    /// 调用 LLM
    async fn call_llm(&self, prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.llm_api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "qwen-plus",
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.1
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    /// 存储到 Qdrant
    async fn store_to_qdrant(
        &self,
        user_id: &str,
        memory_type: &str,
        content: &str,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let vector = self.get_embedding(content).await?;

        let mut payload: HashMap<String, Value> = metadata
            .into_iter()
            .map(|(k, v)| (k, Value::from(v)))
            .collect();

        payload.insert("user_id".to_string(), Value::from(user_id));
        payload.insert("memory_type".to_string(), Value::from(memory_type));
        payload.insert("content".to_string(), Value::from(content));
        payload.insert("created_at".to_string(), Value::from(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()));

        let point = PointStruct::new(id.clone(), vector, payload);
        self.qdrant.upsert_points(UpsertPointsBuilder::new(COLLECTION_NAME, vec![point])).await?;

        tracing::debug!("✅ 记忆已存储: [{}] {}", memory_type, content);

        Ok(id)
    }

    /// 格式化用户观察
    fn format_user_observation(&self, user_obs: &UserObservation) -> String {
        let mut summary = String::from("=== 用户记忆 ===\n");

        if !user_obs.preferences.is_empty() {
            summary.push_str("\n【偏好】\n");
            for (key, value) in &user_obs.preferences {
                summary.push_str(&format!("  • {}: {}\n", key, value));
            }
        }

        if !user_obs.context.is_empty() {
            summary.push_str("\n【上下文】\n");
            for ctx in &user_obs.context {
                summary.push_str(&format!("  • {}\n", ctx));
            }
        }

        if let Some(style) = &user_obs.communication_style {
            summary.push_str(&format!("\n【沟通风格】\n  • {}\n", style));
        }

        if !user_obs.goals.is_empty() {
            summary.push_str("\n【目标】\n");
            for goal in &user_obs.goals {
                summary.push_str(&format!("  • {}\n", goal));
            }
        }

        summary
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}