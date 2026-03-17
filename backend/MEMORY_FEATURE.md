# 长短期记忆功能说明

## 功能概述

Rust + Rig 智能体脚手架现在支持**长短期记忆**功能，实现了类似人类的记忆系统：

### 🧠 短期记忆 (Short-Term Memory)
- **存储位置**: 内存（会话级别）
- **作用范围**: 当前会话内的对话历史
- **容量限制**: 最近 20 轮对话
- **生命周期**: 会话结束后自动清除

### 🗄️ 长期记忆 (Long-Term Memory)
- **存储位置**: SQLite 数据库（持久化）
- **作用范围**: 跨会话、跨时间
- **内容包括**:
  - 用户画像（姓名、职业、位置等）
  - 重要事实（AI 提取的关键信息）
  - 偏好设置（用户喜好、习惯）
  - 完整对话历史
- **生命周期**: 永久保存，除非手动删除

## API 接口

### 1. 聊天接口（带记忆）

```bash
POST /chat
Content-Type: application/json

{
  "agent_id": "100001_GeneralAssistant",
  "message": "你好，还记得我叫什么吗？",
  "user_id": "xiemulei",       # 可选，用于关联长期记忆
  "session_id": "session-001"  # 可选，用于关联短期记忆
}
```

**响应**:
```json
{
  "response": "你好！根据我的记忆，你叫谢文昌...",
  "memory_summary": "=== 用户记忆 ===\n姓名：谢文昌\n职业：Java 程序员\n..."
}
```

### 2. 获取记忆摘要

```bash
GET /memory
Content-Type: application/json

{
  "user_id": "xiemulei"
}
```

**响应**:
```json
{
  "summary": "=== 用户记忆 ===\n姓名：谢文昌\n职业：Java 程序员\n位置：上海\n\n重要事实:\n  - 喜欢用 Rust 开发\n  - 在微信读书上看书\n\n偏好:\n  - 编程语言：Rust (技术)\n  - 阅读平台：微信读书 (学习)\n\n最近对话:\n  [2026-03-17 16:40:00] 用户：你好，还记得我叫什么吗？ ...\n"
}
```

### 3. 清除短期记忆

```bash
POST /memory/clear
Content-Type: application/json

{
  "user_id": "xiemulei"
}
```

## 工作流程

```
用户发送消息
    ↓
1. 加载长期记忆（SQLite）
    ↓
2. 加载短期记忆（内存）
    ↓
3. 构建带记忆的 Prompt → Agent
    ↓
4. Agent 生成回复
    ↓
5. 保存对话到短期记忆
    ↓
6. 保存对话到长期记忆（跨会话）
    ↓
返回回复 + 记忆摘要
```

## 使用示例

### 示例 1: 记住用户信息

**第一次对话**:
```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "100001_GeneralAssistant",
    "message": "我叫谢文昌，是一名 Java 程序员",
    "user_id": "xiemulei"
  }'
```

**第二次对话（跨会话）**:
```bash
# 即使重启服务，记忆依然存在
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "100001_GeneralAssistant",
    "message": "你还记得我是做什么的吗？",
    "user_id": "xiemulei"
  }'
```

**响应**:
```json
{
  "response": "当然记得！你是一名 Java 程序员，叫谢文昌。",
  "memory_summary": "=== 用户记忆 ===\n姓名：谢文昌\n职业：Java 程序员\n..."
}
```

### 示例 2: 查看完整记忆

```bash
curl -X GET "http://localhost:3000/memory" \
  -H "Content-Type: application/json" \
  -d '{"user_id": "xiemulei"}'
```

## 数据库结构

### long_term_memories 表
```sql
CREATE TABLE long_term_memories (
    user_id TEXT PRIMARY KEY,
    profile_json TEXT NOT NULL,          -- 用户画像 JSON
    important_facts_json TEXT NOT NULL,  -- 重要事实 JSON
    preferences_json TEXT NOT NULL,      -- 偏好设置 JSON
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)
```

### conversation_history 表
```sql
CREATE TABLE conversation_history (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    user_message TEXT NOT NULL,
    assistant_response TEXT NOT NULL,
    summary TEXT,                        -- AI 生成的摘要
    timestamp TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES long_term_memories(user_id)
)
```

## 高级功能

### 1. 记忆巩固（Memory Consolidation）

定期将短期记忆中的重要信息提取到长期记忆：

```rust
// AI 辅助提取关键信息
memory_manager.consolidate_memory(
    "xiemulei",
    "session-001",
    "用户表示喜欢 Rust 语言，计划用它开发智能体系统"
).await?;
```

### 2. 偏好管理

```rust
// 添加用户偏好
memory_manager.add_preference(
    "xiemulei",
    "programming_language",
    "Rust",
    "technical"
).await?;
```

### 3. 重要事实

```rust
// 添加重要事实
memory_manager.add_important_fact(
    "xiemulei",
    "用户正在学习使用 Rust 构建 AI 智能体"
).await?;
```

## 性能优化

| 操作 | 短期记忆 | 长期记忆 |
|------|---------|---------|
| 读取速度 | ⚡ 极快（内存） | 🐢 中等（SQLite） |
| 写入速度 | ⚡ 极快 | 🐢 中等 |
| 存储容量 | 20 轮对话 | 无限制 |
| 跨会话 | ❌ 否 | ✅ 是 |

## 注意事项

1. **隐私保护**: 长期记忆包含用户敏感信息，请妥善保管数据库文件
2. **性能考虑**: 大量对话历史可能影响查询速度，建议定期清理或归档
3. **数据备份**: 定期备份 `memory.db` 文件以防数据丢失
4. **内存使用**: 短期记忆限制为 20 轮，避免内存泄漏

## 下一步

- [ ] 实现 AI 自动提取重要信息
- [ ] 添加记忆搜索功能
- [ ] 支持记忆编辑和删除
- [ ] 多用户记忆隔离
- [ ] 记忆过期策略

---

**数据库文件**: `memory.db`（SQLite 格式）
**当前状态**: ✅ 短期记忆 + ✅ 长期记忆（内存模式）
