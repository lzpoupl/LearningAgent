<template>
  <div class="new-session-page">

    <!-- 顶部 -->
    <header class="page-header">
      <div class="logo">
        <div class="logo-icon">
          L
        </div>

        <span>
          LearningAgent
        </span>
      </div>
    </header>

    <!-- 中央内容 -->
    <main class="new-session-content">

      <div class="welcome-icon">
        ✦
      </div>

      <h1>
        开始新的学习
      </h1>

      <p class="subtitle">
        选择一个学习助手，然后输入你想学习的问题
      </p>

      <div class="agent-select-wrap">
        <label class="agent-label">学习助手</label>

        <div
          class="agent-select"
          @click="agentMenuOpen = !agentMenuOpen"
        >
          <div class="agent-selected">
            <span class="agent-icon">{{ selectedAgentInfo.icon }}</span>
            <span>{{ selectedAgentInfo.name }}</span>
          </div>
          <span class="agent-caret">▾</span>
        </div>

        <div
          v-if="agentMenuOpen"
          class="agent-menu"
        >
          <button
            v-for="agent in agents"
            :key="agent.id"
            class="agent-option"
            :class="{ active: selectedAgent === agent.id }"
            type="button"
            @click="selectAgent(agent.id)"
          >
            <span class="agent-icon small">{{ agent.icon }}</span>
            <span class="agent-option-text">
              <span class="agent-option-name">{{ agent.name }}</span>
              <span class="agent-option-desc">{{ agent.description }}</span>
            </span>
            <span v-if="selectedAgent === agent.id" class="selected-dot">✓</span>
          </button>
        </div>
      </div>

      <!-- 搜索框 -->
      <div class="search-area">

        <textarea
          v-model="question"
          placeholder="输入你想学习的问题..."
          rows="3"
          @keydown.enter.exact.prevent="startChat"
        />

        <button
          class="search-button"
          :disabled="!question.trim()"
          @click="startChat"
        >
          <span>开始学习</span>
          <span class="arrow">→</span>
        </button>

      </div>

      <div class="search-tip">
        选择学习助手后，输入问题即可开始新的学习会话
      </div>

    </main>

  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { AgentType } from '../types/chat'

const emit = defineEmits<{
  start: [
    agent: AgentType,
    question: string
  ]
}>()

const selectedAgent = ref<AgentType>('math')
const agentMenuOpen = ref(false)

const question = ref('')

const agents = [
  {
    id: 'math' as AgentType,
    name: '数学老师',
    description: '数学问题、公式推导与解题思路',
    icon: '∑'
  },
  {
    id: 'english' as AgentType,
    name: '英语老师',
    description: '英语语法、单词与语言表达',
    icon: 'A'
  }
]

const selectedAgentInfo = computed(
  () => agents.find((agent) => agent.id === selectedAgent.value) ?? agents[0]
)

function selectAgent(agentId: AgentType) {
  selectedAgent.value = agentId
  agentMenuOpen.value = false
}

function startChat() {
  const text = question.value.trim()

  if (!text) {
    return
  }

  emit('start', selectedAgent.value, text)
}
</script>

<style scoped>
.new-session-page {
  width: 100%;
  height: 100vh;
  flex: 1;
  min-width: 0;
  overflow: hidden;

  display: flex;
  flex-direction: column;

  background: #ffffff;
}

.page-header {
  height: 64px;
  flex-shrink: 0;

  display: flex;
  align-items: center;

  padding: 0 28px;

  border-bottom: 1px solid #eeeeee;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;

  font-size: 17px;
  font-weight: 600;
}

.logo-icon {
  width: 24px;
  height: 24px;

  display: flex;
  align-items: center;
  justify-content: center;

  border-radius: 7px;

  background: #111111;
  color: #ffffff;
  font-size: 12px;
}

.new-session-content {
  width: min(760px, 82vw);
  max-width: 760px;
  height: calc(100vh - 64px);

  margin: 0 auto;

  padding: 18px 0 10px;

  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

@media (max-width: 700px) {
  .new-session-content {
    width: calc(100vw - 24px);
    max-width: 100%;
  }
}

.welcome-icon {
  width: 48px;
  height: 48px;

  display: flex;
  align-items: center;
  justify-content: center;

  margin-bottom: 18px;

  border-radius: 14px;

  background: #111111;
  color: #ffffff;

  font-size: 22px;
}

h1 {
  margin: 0;

  font-size: 30px;
  font-weight: 600;
  color: #222222;
}

.subtitle {
  margin: 12px 0 32px;

  font-size: 14px;
  color: #999999;
}

/* Agent */

.agent-select-wrap {
  position: relative;
  width: 100%;
  margin-top: 12px;
}

.agent-label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  color: #666666;
}

.agent-select {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  min-height: 34px;
  padding: 6px 10px;
  border: 1px solid #d9d9d9;
  border-radius: 10px;
  background: #ffffff;
  cursor: pointer;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.04);
}

.agent-selected {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #222222;
  font-size: 14px;
  font-weight: 500;
}

.agent-caret {
  color: #666666;
  font-size: 12px;
}

.agent-menu {
  position: absolute;
  z-index: 20;
  left: 0;
  right: 0;
  top: calc(100% + 8px);
  max-height: 180px;
  border: 1px solid #e5e5e5;
  border-radius: 10px;
  overflow-y: auto;
  background: #ffffff;
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.08);
}

.agent-option {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 10px;
  padding: 10px 12px;
  border: none;
  background: #ffffff;
  text-align: left;
  cursor: pointer;
}

.agent-option + .agent-option {
  border-top: 1px solid #f0f0f0;
}

.agent-option.active {
  background: #f7f7f7;
}

.agent-option-text {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 2px;
}

.agent-option-name {
  font-size: 13px;
  font-weight: 600;
  color: #222222;
}

.agent-option-desc {
  font-size: 11px;
  color: #999999;
}

.agent-icon {
  width: 26px;
  height: 26px;
  flex-shrink: 0;

  display: flex;
  align-items: center;
  justify-content: center;

  border-radius: 8px;

  background: #eeeeee;

  font-size: 14px;
  font-weight: 600;
}

.agent-icon.small {
  width: 22px;
  height: 22px;
  font-size: 12px;
}

.selected-dot {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #111111;
  color: #ffffff;
  font-size: 10px;
}

/* 搜索框 */

.search-area {
  width: min(100%, 760px);
  height: clamp(140px, 22vh, 240px);

  margin-top: 18px;

  padding: 10px 12px;

  border: 1px solid #d9d9d9;
  border-radius: 14px;

  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.06);
}

@media (max-width: 1000px) {
  .agent-select-wrap {
    width: min(100%, 380px);
  }

  .search-area {
    width: min(100%, 380px);
  }
}

textarea {
  width: 100%;
  height: calc(100% - 50px);

  display: block;

  padding: 6px 6px 8px;

  border: none;
  outline: none;

  resize: none;

  font-size: 15px;
  line-height: 1.5;

  color: #222222;
}

textarea::placeholder {
  color: #aaaaaa;
}

.search-button {
  height: 34px;

  display: flex;
  align-items: center;
  gap: 8px;

  margin-left: auto;
  padding: 0 14px;

  border: none;
  border-radius: 8px;

  background: #111111;
  color: #ffffff;

  cursor: pointer;

  font-size: 12px;
}

.search-button:disabled {
  background: #dddddd;
  cursor: not-allowed;
}

.arrow {
  font-size: 17px;
}

.search-tip {
  margin-top: 12px;

  font-size: 11px;
  color: #aaaaaa;
}
</style>