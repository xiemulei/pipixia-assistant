# 皮皮虾助手 - AI 智能体桌面应用

基于 Tauri + Vue3 + TypeScript 构建的 AI 智能体桌面客户端。

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **桌面框架**: Tauri 2.0 (Rust)
- **后端**: Rust + Axum + Qdrant

## 项目结构

```
rig-agent-ui/
├── src/                    # Vue 前端代码
│   ├── App.vue            # 主应用组件
│   └── main.ts            # 入口文件
├── src-tauri/             # Tauri Rust 代码
│   └── tauri.conf.json    # Tauri 配置
└── package.json
```

## 快速开始

### 1. 启动后端服务

```bash
# 在另一个终端启动后端
cd ../rig-agent-scaffold
$env:OPENAI_API_KEY='your-api-key'
cargo run
```

### 2. 启动前端开发服务器

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev
```

### 3. 构建生产版本

```bash
npm run tauri build
```

## 功能特性

### ✅ 聊天界面
- 实时对话
- 消息格式化（粗体、斜体、代码）
- 加载动画

### ✅ 记忆系统
- 查看用户记忆
- AI 自动提取关键信息
- 跨会话记忆持久化

### ✅ 用户配置
- 自定义用户 ID
- 选择不同 Agent

## API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/chat` | POST | 发送聊天消息 |
| `/memory` | GET | 获取记忆摘要 |

## 开发说明

### 前端开发

```bash
# 仅启动前端开发服务器
npm run dev
```

### Tauri 开发

```bash
# 启动 Tauri 开发模式（包含热重载）
npm run tauri dev
```

### 构建发布

```bash
# 构建桌面应用
npm run tauri build
```

## 配置

### 环境变量

在 `rig-agent-scaffold` 目录创建 `.env` 文件：

```env
OPENAI_API_KEY=your-dashscope-api-key
QDRANT_URL=http://localhost:6334
```

### Tauri 安全策略

已配置 CSP 允许连接到：
- `http://localhost:3000` (后端 API)
- `http://localhost:6333` (Qdrant HTTP)
- `http://localhost:6334` (Qdrant gRPC)
- `https://dashscope.aliyuncs.com` (AI API)

## 截图

_待添加_

## License

MIT