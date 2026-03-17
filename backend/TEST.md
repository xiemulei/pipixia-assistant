# 快速测试指南

## 1. 配置环境变量

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑 .env 文件，填入你的 API Key
# DASHSCOPE_API_KEY=sk-your-actual-key-here
```

## 2. 运行服务

```bash
cargo run
```

服务会启动在 `http://localhost:3000`

## 3. 测试 API

### 健康检查

```bash
curl http://localhost:3000/health
```

预期响应：
```json
{
  "status": "ok",
  "agents": []
}
```

### 发送聊天请求

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "100001_GeneralAssistant",
    "message": "你好，介绍一下你自己"
  }'
```

预期响应：
```json
{
  "response": "你好！我是一个友好、专业的通用智能助手..."
}
```

## 4. 添加更多 Agent

在 `agents/` 目录下创建新的 YAML 配置文件：

```yaml
agent:
  agent-id: "100004"
  agent-name: "自定义助手"
  agent-desc: "我的自定义助手"
  
  module:
    ai-api:
      base-url: "https://dashscope.aliyuncs.com/compatible-mode/"
      api-key: "${DASHSCOPE_API_KEY}"
      completions-path: "v1/chat/completions"
    
    chat-model:
      model: "qwen-plus"

  agents:
    - name: "CustomAssistant"
      description: "自定义助手"
      instruction: |
        你是一个自定义助手，请...
```

## 5. 常见问题

### Q: 提示 "Agent 未找到"
A: 确保 agent_id 格式正确：`{agent-id}_{agent-name}`，例如 `100001_GeneralAssistant`

### Q: API 调用失败
A: 检查 `.env` 文件中的 API Key 是否正确

### Q: 编译错误
A: 确保 Rust 版本 >= 1.70，运行 `rustup update` 更新

## 6. 下一步

- [ ] 实现真实的工具调用（Bing 搜索、天气查询）
- [ ] 连接记忆系统（SQLite/Redis）
- [ ] 添加 MCP 协议支持
- [ ] 实现完整的工作流引擎
- [ ] Docker 容器化部署
