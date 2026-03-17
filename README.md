# 🦐 皮皮虾助手 (Pipixia Assistant)

<p align="center">
  <strong>基于 Rust + Qdrant + Tauri 的 AI 智能体桌面应用</strong>
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#技术架构">技术架构</a> •
  <a href="#开发指南">开发指南</a> •
  <a href="#license">License</a>
</p>

---

## 功能特性

### 🧠 三层记忆系统

基于 [Rig 记忆架构](https://book.rig.rs/playbook/memory.html) 设计的三层记忆系统：

| 层级 | 类型 | 说明 |
|------|------|------|
| **短期记忆** | EphemeralMemory | 当前会话对话历史，支持自动压缩摘要 |
| **长期记忆** | UserObservation | 用户偏好、上下文、目标，跨会话持久化 |
| **语义记忆** | Qdrant Vector | 向量存储，支持语义搜索相似记忆 |

### 💬 智能对话

- **自动信息提取** - AI 自动识别用户偏好、上下文、目标
- **跨会话记忆** - 记住用户信息，下次对话继续
- **对话压缩** - 超过 20 条消息自动生成摘要
- **语义搜索** - 基于向量相似度检索相关记忆

### 🖥️ 桌面应用

- **跨平台** - Windows / macOS / Linux
- **原生性能** - Rust + Tauri，内存占用 ~50MB
- **自适应主题** - 深色/浅色模式自动切换

---

## 快速开始

### 前置要求

| 依赖 | 版本 | 安装方式 |
|------|------|----------|
| Rust | 1.70+ | [rustup.rs](https://rustup.rs) |
| Node.js | 18+ | [nodejs.org](https://nodejs.org) |
| Qdrant | 最新 | `docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant` |
| Dashscope API Key | - | [dashscope.console.aliyun.com](https://dashscope.console.aliyun.com/) |

### 安装步骤

```bash
# 1. 克隆仓库
git clone https://github.com/你的用户名/pipixia-assistant.git
cd pipixia-assistant

# 2. 配置 API Key
cp backend/.env.example backend/.env
# 编辑 backend/.env，填入你的 Dashscope API Key

# 3. 启动 Qdrant（新终端）
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant

# 4. 启动服务
# Windows PowerShell
.\start.ps1

# 或手动启动
cd backend && cargo run
cd frontend && npm install && npm run tauri dev
```

### 停止服务

```powershell
# Windows PowerShell
.\stop.ps1
```

---

## 技术架构

### 后端 (Backend)

```
backend/
├── src/
│   ├── main.rs          # 入口 + HTTP 服务器
│   ├── agent.rs         # Agent 管理器
│   ├── memory.rs        # 三层记忆系统
│   ├── workflow.rs      # 工作流引擎
│   ├── tools.rs         # 工具系统
│   └── api_key.rs       # API 密钥轮换
├── agents/              # Agent YAML 配置
│   ├── general-agent.yml
│   ├── code-agent.yml
│   └── drawio-agent.yml
└── Cargo.toml
```

**技术栈：**
- **Rig-core** - AI Agent 框架
- **Qdrant** - 向量数据库（语义搜索）
- **Axum** - HTTP 服务器
- **Dashscope** - 通义千问 LLM + Embedding

### 前端 (Frontend)

```
frontend/
├── src/
│   └── App.vue          # 主界面
├── src-tauri/           # Tauri 配置
│   ├── src/
│   ├── tauri.conf.json
│   └── capabilities/
└── package.json
```

**技术栈：**
- **Tauri 2.0** - 桌面应用框架
- **Vue 3 + TypeScript** - 前端框架
- **@tauri-apps/plugin-http** - HTTP 客户端

### API 接口

| 端点 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/chat` | POST | 发送聊天消息 |
| `/memory` | GET | 获取记忆摘要 |

#### 聊天请求示例

```json
POST /chat
{
  "agent_id": "100001_GeneralAssistant",
  "message": "你好，我喜欢跑步",
  "user_id": "wenchang",
  "session_id": "session-001"
}
```

---

## 开发指南

### 后端开发

```bash
cd backend

# 开发模式
cargo run

# 构建
cargo build --release

# 运行测试
cargo test
```

### 前端开发

```bash
cd frontend

# 安装依赖
npm install

# 开发模式（仅前端）
npm run dev

# 开发模式（完整应用）
npm run tauri dev

# 构建发布
npm run tauri build
```

### 项目结构

```
pipixia-assistant/
├── backend/             # Rust 后端
│   ├── src/            # 源代码
│   ├── agents/         # Agent 配置
│   ├── .env.example    # 环境变量模板
│   └── Cargo.toml
│
├── frontend/            # Tauri 前端
│   ├── src/            # Vue 组件
│   ├── src-tauri/      # Tauri 配置
│   └── package.json
│
├── start.ps1           # 一键启动脚本
├── stop.ps1            # 停止脚本
├── .gitignore
├── LICENSE
└── README.md
```

---

## 配置

### 环境变量 (backend/.env)

```env
# API Key
OPENAI_API_KEY=your-dashscope-api-key

# Qdrant
QDRANT_URL=http://localhost:6334

# Embedding API
EMBEDDING_API_URL=https://dashscope.aliyuncs.com/compatible-mode

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

### Agent 配置 (backend/agents/*.yml)

```yaml
agent_id: "100001"
agent_name: "通用助手"
agent_desc: "能够回答用户一般问题的智能助手"

module:
  ai_api:
    base_url: "https://dashscope.aliyuncs.com/compatible-mode/"
  chat_model:
    model: "qwen-plus"

agents:
  - name: "GeneralAssistant"
    instruction: |
      你是一位友好、专业的通用智能助手...
```

---

## 记忆系统设计

### 架构图

```
┌─────────────────────────────────────────────────────────┐
│                     用户输入                             │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  Ephemeral Memory (短期记忆)                            │
│  - 当前会话对话历史                                      │
│  - 自动压缩（超过 20 条消息）                            │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  Long-term Memory (长期记忆 - Qdrant)                   │
│  ┌────────────────┬────────────────┬────────────────┐  │
│  │ UserObservation│ ConversationObs│ GroundedFact   │  │
│  │ 用户偏好/上下文 │ 对话洞察       │ 已验证事实      │  │
│  └────────────────┴────────────────┴────────────────┘  │
│                    ↓ 语义搜索 ↓                          │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│              Agent (带记忆上下文的 Prompt)               │
└─────────────────────────────────────────────────────────┘
```

### 记忆类型

| 类型 | 字段 | 说明 |
|------|------|------|
| **UserObservation** | preferences, context, goals | 用户偏好、上下文、目标 |
| **ConversationObservation** | topic, insight, importance | 对话主题、洞察、重要性 |
| **GroundedFact** | fact, source, confidence | 已验证事实、来源、置信度 |

---

## 性能指标

| 指标 | 数值 |
|------|------|
| 后端启动时间 | ~0.5s |
| 前端启动时间 | ~1s |
| 内存占用 | ~50MB |
| 对话响应 | 1-3s |

---

## 参考文档

- [Rig 记忆系统](https://book.rig.rs/playbook/memory.html)
- [Qdrant 文档](https://qdrant.tech/documentation/)
- [Tauri 文档](https://v2.tauri.app/)
- [Dashscope API](https://help.aliyun.com/zh/dashscope/)

---

## 致谢

- [Rig](https://github.com/0xPlaygrounds/rig) - AI Agent 框架
- [Qdrant](https://qdrant.tech/) - 向量数据库
- [Tauri](https://tauri.app/) - 桌面应用框架
- [Dashscope](https://dashscope.aliyun.com/) - 通义千问 API

---

## License

MIT License - 详见 [LICENSE](LICENSE) 文件

---

<p align="center">
  Made with 🦐
</p>