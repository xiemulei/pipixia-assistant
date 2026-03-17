//! API 密钥管理 - 实现密钥轮换和负载均衡
//! 
//! 功能：
//! - 多密钥管理
//! - 密钥状态跟踪
//! - 自动轮换
//! - 使用统计

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::RwLock;
use chrono::Local;

/// API 密钥状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub key: String,
    pub enabled: bool,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub usage_count: u64,
    pub error_count: u64,
}

/// API 密钥管理器
pub struct ApiKeyManager {
    keys: RwLock<VecDeque<ApiKeyEntry>>,
    current_index: RwLock<usize>,
}

impl ApiKeyManager {
    pub fn new(keys: Vec<String>) -> Self {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let key_entries: VecDeque<ApiKeyEntry> = keys
            .into_iter()
            .map(|key| ApiKeyEntry {
                key,
                enabled: true,
                created_at: now.clone(),
                last_used_at: None,
                usage_count: 0,
                error_count: 0,
            })
            .collect();

        Self {
            keys: RwLock::new(key_entries),
            current_index: RwLock::new(0),
        }
    }

    /// 获取下一个可用的密钥（轮询）
    pub async fn get_next_key(&self) -> Result<String> {
        let mut keys = self.keys.write().await;
        let mut index = self.current_index.write().await;

        let total = keys.len();
        if total == 0 {
            anyhow::bail!("没有可用的 API 密钥");
        }

        // 轮询查找可用密钥
        let start = *index;
        loop {
            if let Some(key) = keys.get_mut(*index) {
                if key.enabled {
                    key.usage_count += 1;
                    key.last_used_at = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                    
                    let key_value = key.key.clone();
                    
                    // 更新索引
                    *index = (*index + 1) % total;
                    
                    return Ok(key_value);
                }
            }

            *index = (*index + 1) % total;
            if *index == start {
                anyhow::bail!("所有 API 密钥都已禁用");
            }
        }
    }

    /// 报告密钥使用错误
    pub async fn report_error(&self, key: &str) -> Result<()> {
        let mut keys = self.keys.write().await;

        if let Some(key_entry) = keys.iter_mut().find(|k| k.key == key) {
            key_entry.error_count += 1;

            // 错误次数过多时自动禁用
            if key_entry.error_count >= 10 {
                key_entry.enabled = false;
            }
        }

        Ok(())
    }

    /// 获取密钥统计信息
    pub async fn get_status(&self) -> ApiKeyStatus {
        let keys = self.keys.read().await;

        let total = keys.len();
        let available = keys.iter().filter(|k| k.enabled).count();
        let total_usage: u64 = keys.iter().map(|k| k.usage_count).sum();
        let total_errors: u64 = keys.iter().map(|k| k.error_count).sum();

        ApiKeyStatus {
            total_keys: total,
            available_keys: available,
            total_usage,
            total_errors,
        }
    }

    /// 重置所有密钥状态
    pub async fn reset_all(&self) {
        let mut keys = self.keys.write().await;

        for key in keys.iter_mut() {
            key.enabled = true;
            key.error_count = 0;
        }
    }

    /// 添加新密钥
    pub async fn add_key(&self, key: &str) {
        let mut keys = self.keys.write().await;

        keys.push_back(ApiKeyEntry {
            key: key.to_string(),
            enabled: true,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            last_used_at: None,
            usage_count: 0,
            error_count: 0,
        });
    }
}

/// API 密钥状态报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStatus {
    pub total_keys: usize,
    pub available_keys: usize,
    pub total_usage: u64,
    pub total_errors: u64,
}
