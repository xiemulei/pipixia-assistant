//! 工作流引擎 - 实现 Agent 编排模式
//! 
//! 支持的工作流类型：
//! - Chain: 串行执行（提示链模式）
//! - Parallel: 并行执行（并行化模式）
//! - Router: 条件路由（路由模式）
//! - HumanInTheLoop: 人工审核（人类参与模式）

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::agent::AgentManager;

/// 工作流执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_name: String,
    pub outputs: Vec<OutputItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputItem {
    pub agent_name: String,
    pub output: String,
    pub output_key: Option<String>,
}

/// 链式工作流 - 提示链模式
pub struct ChainWorkflow {
    pub name: String,
    pub agent_ids: Vec<String>,
}

impl ChainWorkflow {
    pub fn new(name: &str, agent_ids: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            agent_ids,
        }
    }

    pub async fn execute(&self, manager: &AgentManager, input: &str) -> Result<WorkflowResult> {
        let mut current_input = input.to_string();
        let mut outputs = Vec::new();

        for agent_id in &self.agent_ids {
            let output = manager.chat(agent_id, &current_input).await?;
            
            outputs.push(OutputItem {
                agent_name: agent_id.clone(),
                output: output.clone(),
                output_key: None,
            });

            // 将输出作为下一个 Agent 的输入
            current_input = output;
        }

        Ok(WorkflowResult {
            workflow_name: self.name.clone(),
            outputs,
        })
    }
}

/// 并行工作流 - 并行化模式
pub struct ParallelWorkflow {
    pub name: String,
    pub agent_ids: Vec<String>,
}

impl ParallelWorkflow {
    pub fn new(name: &str, agent_ids: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            agent_ids,
        }
    }

    pub async fn execute(&self, manager: Arc<AgentManager>, input: &str) -> Result<WorkflowResult> {
        let mut outputs = Vec::new();
        let mut join_set = JoinSet::new();

        // 并发执行所有 Agent
        for agent_id in &self.agent_ids {
            let manager = Arc::clone(&manager);
            let agent_id = agent_id.clone();
            let input = input.to_string();

            join_set.spawn(async move {
                manager.chat(&agent_id, &input).await
            });
        }

        // 收集结果
        while let Some(result) = join_set.join_next().await {
            let output = result??;
            outputs.push(OutputItem {
                agent_name: "parallel".to_string(),
                output,
                output_key: None,
            });
        }

        Ok(WorkflowResult {
            workflow_name: self.name.clone(),
            outputs,
        })
    }
}

/// 路由工作流 - 路由模式
pub struct RouterWorkflow {
    pub name: String,
    pub routes: Vec<Route>,
}

pub struct Route {
    pub condition: Box<dyn Fn(&str) -> bool + Send + Sync>,
    pub agent_id: String,
}

impl RouterWorkflow {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            routes: Vec::new(),
        }
    }

    pub fn add_route<F>(&mut self, condition: F, agent_id: &str)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.routes.push(Route {
            condition: Box::new(condition),
            agent_id: agent_id.to_string(),
        });
    }

    pub async fn execute(&self, manager: &AgentManager, input: &str) -> Result<WorkflowResult> {
        // 找到匹配的路由
        for route in &self.routes {
            if (route.condition)(input) {
                let output = manager.chat(&route.agent_id, input).await?;
                
                return Ok(WorkflowResult {
                    workflow_name: self.name.clone(),
                    outputs: vec![OutputItem {
                        agent_name: route.agent_id.clone(),
                        output,
                        output_key: None,
                    }],
                });
            }
        }

        anyhow::bail!("没有匹配的路由")
    }
}

/// 人类参与工作流 - HITL 模式
pub struct HumanInTheLoopWorkflow {
    pub name: String,
    pub pre_agent_id: String,
    pub post_agent_id: Option<String>,
}

impl HumanInTheLoopWorkflow {
    pub fn new(name: &str, pre_agent_id: &str, post_agent_id: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            pre_agent_id: pre_agent_id.to_string(),
            post_agent_id: post_agent_id.map(|s| s.to_string()),
        }
    }

    pub async fn execute<F>(
        &self,
        manager: &AgentManager,
        input: &str,
        human_approve: F,
    ) -> Result<WorkflowResult>
    where
        F: FnOnce(&str) -> bool,
    {
        // 第一步：Agent 生成结果
        let pre_output = manager.chat(&self.pre_agent_id, input).await?;

        // 第二步：人类审核
        if !human_approve(&pre_output) {
            anyhow::bail!("人类审核未通过");
        }

        // 第三步：可选的后处理 Agent
        let final_output = if let Some(ref post_agent_id) = self.post_agent_id {
            manager.chat(post_agent_id, &pre_output).await?
        } else {
            pre_output.clone()
        };

        Ok(WorkflowResult {
            workflow_name: self.name.clone(),
            outputs: vec![
                OutputItem {
                    agent_name: self.pre_agent_id.clone(),
                    output: pre_output,
                    output_key: None,
                },
                OutputItem {
                    agent_name: self.post_agent_id.clone().unwrap_or_default(),
                    output: final_output,
                    output_key: None,
                },
            ],
        })
    }
}
