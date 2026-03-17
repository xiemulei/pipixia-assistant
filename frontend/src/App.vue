<script setup lang="ts">
import { ref, nextTick } from "vue";
import { fetch } from "@tauri-apps/plugin-http";

// API 配置
const API_BASE = "http://localhost:3000";

// 状态
const messages = ref<Array<{ role: "user" | "assistant"; content: string }>>([]);
const inputMessage = ref("");
const isLoading = ref(false);
const userId = ref("wenchang");
const agentId = ref("100001_GeneralAssistant");
const memorySummary = ref("");
const showMemory = ref(false);

// 消息容器引用
const messagesContainer = ref<HTMLElement | null>(null);

// 发送消息
async function sendMessage() {
  if (!inputMessage.value.trim() || isLoading.value) return;

  const userMessage = inputMessage.value.trim();
  messages.value.push({ role: "user", content: userMessage });
  inputMessage.value = "";
  isLoading.value = true;

  try {
    // 使用 Tauri HTTP 插件的 fetch
    const response = await fetch(`${API_BASE}/chat`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        agent_id: agentId.value,
        message: userMessage,
        user_id: userId.value,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();

    messages.value.push({
      role: "assistant",
      content: data.response,
    });

    if (data.memory_summary) {
      memorySummary.value = data.memory_summary;
    }
  } catch (error: any) {
    console.error("Fetch error:", error);
    messages.value.push({
      role: "assistant",
      content: `错误: ${error.message}`,
    });
  } finally {
    isLoading.value = false;
    await nextTick();
    scrollToBottom();
  }
}

// 滚动到底部
function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

// 清空对话
function clearChat() {
  messages.value = [];
}

// 格式化消息（简单的 markdown 支持）
function formatMessage(content: string): string {
  return content
    .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.*?)\*/g, "<em>$1</em>")
    .replace(/`(.*?)`/g, "<code>$1</code>")
    .replace(/\n/g, "<br>");
}
</script>

<template>
  <div class="app">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <h2>🦐 皮皮虾助手</h2>
      </div>
      
      <div class="sidebar-section">
        <label>用户 ID</label>
        <input v-model="userId" placeholder="用户 ID" />
      </div>

      <div class="sidebar-section">
        <label>Agent</label>
        <select v-model="agentId">
          <option value="100001_GeneralAssistant">通用助手</option>
        </select>
      </div>

      <div class="sidebar-actions">
        <button @click="showMemory = !showMemory" class="btn-secondary">
          {{ showMemory ? "隐藏记忆" : "查看记忆" }}
        </button>
        <button @click="clearChat" class="btn-secondary">清空对话</button>
      </div>

      <!-- 记忆面板 -->
      <div v-if="showMemory" class="memory-panel">
        <h3>📝 用户记忆</h3>
        <pre v-if="memorySummary">{{ memorySummary }}</pre>
        <p v-else class="no-memory">暂无记忆数据</p>
      </div>
    </aside>

    <!-- 主聊天区域 -->
    <main class="chat-main">
      <!-- 消息列表 -->
      <div class="messages" ref="messagesContainer">
        <div v-if="messages.length === 0" class="empty-state">
          <h2>👋 你好，我是皮皮虾助手</h2>
          <p>有什么我可以帮你的吗？</p>
        </div>

        <div
          v-for="(msg, index) in messages"
          :key="index"
          :class="['message', msg.role]"
        >
          <div class="message-avatar">
            {{ msg.role === "user" ? "👤" : "🦐" }}
          </div>
          <div class="message-content" v-html="formatMessage(msg.content)"></div>
        </div>

        <div v-if="isLoading" class="message assistant">
          <div class="message-avatar">🦐</div>
          <div class="message-content loading">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="input-area">
        <textarea
          v-model="inputMessage"
          @keydown.enter.exact.prevent="sendMessage"
          placeholder="输入消息... (Enter 发送)"
          rows="1"
        ></textarea>
        <button @click="sendMessage" :disabled="isLoading || !inputMessage.trim()">
          发送
        </button>
      </div>
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --primary-color: #4f46e5;
  --primary-hover: #4338ca;
  --bg-color: #f3f4f6;
  --sidebar-bg: #ffffff;
  --message-user: #4f46e5;
  --message-assistant: #f9fafb;
  --text-color: #1f2937;
  --text-secondary: #6b7280;
  --border-color: #e5e7eb;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-color: #111827;
    --sidebar-bg: #1f2937;
    --message-assistant: #1f2937;
    --text-color: #f9fafb;
    --text-secondary: #9ca3af;
    --border-color: #374151;
  }
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background-color: var(--bg-color);
  color: var(--text-color);
}

.app {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 280px;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  padding: 1rem;
}

.sidebar-header {
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 1rem;
}

.sidebar-header h2 {
  font-size: 1.25rem;
  color: var(--text-color);
}

.sidebar-section {
  margin-bottom: 1rem;
}

.sidebar-section label {
  display: block;
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
}

.sidebar-section input,
.sidebar-section select {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-color);
  color: var(--text-color);
}

.sidebar-actions {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.btn-secondary {
  padding: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-color);
  color: var(--text-color);
  cursor: pointer;
  transition: background 0.2s;
}

.btn-secondary:hover {
  background: var(--border-color);
}

.memory-panel {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  background: var(--bg-color);
  border-radius: 0.5rem;
  font-size: 0.875rem;
}

.memory-panel h3 {
  margin-bottom: 0.5rem;
}

.memory-panel pre {
  white-space: pre-wrap;
  font-family: inherit;
  color: var(--text-secondary);
}

.no-memory {
  color: var(--text-secondary);
}

.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
}

.empty-state h2 {
  margin-bottom: 0.5rem;
  color: var(--text-color);
}

.message {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 1rem;
  max-width: 80%;
}

.message.user {
  margin-left: auto;
  flex-direction: row-reverse;
}

.message-avatar {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
  flex-shrink: 0;
}

.message-content {
  padding: 0.75rem 1rem;
  border-radius: 1rem;
  background: var(--message-assistant);
  line-height: 1.5;
}

.message.user .message-content {
  background: var(--message-user);
  color: white;
}

.message-content code {
  background: rgba(0, 0, 0, 0.1);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 0.875em;
}

.message.user .message-content code {
  background: rgba(255, 255, 255, 0.2);
}

.loading {
  display: flex;
  gap: 0.25rem;
}

.loading span {
  width: 0.5rem;
  height: 0.5rem;
  background: var(--text-secondary);
  border-radius: 50%;
  animation: bounce 1s infinite ease-in-out;
}

.loading span:nth-child(1) {
  animation-delay: -0.32s;
}

.loading span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}

.input-area {
  display: flex;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  background: var(--sidebar-bg);
  border-top: 1px solid var(--border-color);
}

.input-area textarea {
  flex: 1;
  padding: 0.75rem 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  resize: none;
  font-size: 1rem;
  font-family: inherit;
  background: var(--bg-color);
  color: var(--text-color);
  outline: none;
}

.input-area textarea:focus {
  border-color: var(--primary-color);
}

.input-area button {
  padding: 0.75rem 1.5rem;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 0.75rem;
  font-size: 1rem;
  cursor: pointer;
  transition: background 0.2s;
}

.input-area button:hover:not(:disabled) {
  background: var(--primary-hover);
}

.input-area button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>