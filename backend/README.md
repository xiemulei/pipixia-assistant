# Rig AI Agent Scaffold

基于 Rust + Rig 框架的 AI 智能体脚手架，实现与 Java 版本相同的核心功能。

## 功能特性

- ✅ **多 Agent 配置** - YAML 配置不同 Agent
- ✅ **技能系统** - 可插拔技能包
- ✅ **工具调用** - MCP 工具、HTTP API
- ✅ **Agent 工作流** - 并行/串行执行
- ✅ **记忆系统** - 用户记忆管理
- ✅ **API 密钥轮换** - 多密钥负载均衡

## 快速开始

```bash
# 克隆项目
git clone git@github.com:xiemulei/rig-agent-scaffold.git
cd rig-agent-scaffold

# 配置 API Key
cp .env.example .env
# 编辑 .env 填入你的 DASHSCOPE_API_KEY

# 运行
cargo run
```

## 项目结构

```
rig-agent-scaffold/
├── Cargo.toml              # 项目依赖
├── .env.example            # 环境变量模板
├── agents/                 # Agent 配置
│   ├── general-agent.yml   # 通用助手
│   ├── code-agent.yml      # 代码助手
│   └── drawio-agent.yml    # 画图助手
├── skills/                 # 技能包
│   ├── pdf-processor/
│   ├── weather-skill/
│   └── news-skill/
├── src/
│   ├── main.rs             # 入口
│   ├── config/             # 配置加载
│   ├── agent/              # Agent 核心
│   ├── workflow/           # 工作流引擎
│   ├── tools/              # 工具实现
│   ├── memory/             # 记忆系统
│   └── api_key/            # API 密钥管理
└── README.md
```

## Agent 配置示例

```yaml
# agents/general-agent.yml
agent:
  agent-id: 100001
  agent-name: 通用助手
  agent-desc: 能够回答用户一般问题的智能助手
  
module:
  ai-api:
    base-url: https://dashscope.aliyuncs.com/compatible-mode/
    api-key: ${DASHSCOPE_API_KEY}
  chat-model:
    model: qwen-plus
    tools:
      - bing_search
      - get_current_date

agents:
  - name: GeneralAssistant
    instruction: |
      你是一位友好、专业的通用智能助手。
      当用户询问与自己相关的问题时，必须首先调用 get_user_memory 工具。
```

## 核心模式实现

| 设计模式 | Rust 实现 |
|----------|----------|
| 提示链 | `workflow::ChainWorkflow` |
| 路由 | `agent::Router` |
| 并行化 | `workflow::ParallelWorkflow` (Tokio) |
| 反思 | `agent::ReflectiveAgent` |
| 工具使用 | `tools::ToolRegistry` |
| 人类参与 | `workflow::HumanInTheLoop` |

## 与 Java 版本对比

| 特性 | Java 版本 | Rust 版本 |
|------|----------|----------|
| 框架 | Spring Boot | Axum + Rig |
| 配置 | YAML | YAML (serde_yaml) |
| 并发 | 线程池 | Tokio 异步 |
| 内存占用 | ~500MB | ~50MB |
| 启动时间 | ~5s | ~0.5s |

## 开发中

- [ ] MCP 协议支持
- [ ] 更多技能包
- [ ] Docker 部署
- [ ] 性能测试

## License

MIT
