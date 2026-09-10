<template>
  <main class="new-session-page">
    <section class="new-session-content">
      <div class="welcome-mark">✦</div>
      <span class="eyebrow">NEW LEARNING SESSION</span>
      <h1>开始新的学习</h1>
      <p class="subtitle">选择一个学习助手，然后输入你想学习的问题</p>

      <el-card class="start-card" shadow="never">
        <el-form label-position="top" @submit.prevent="startChat">
          <el-form-item label="学习助手">
            <el-select v-model="selectedAgent" class="agent-select" placeholder="选择学习助手">
              <el-option
                v-for="agent in agentCatalog"
                :key="agent.id"
                :label="agent.name"
                :value="agent.id"
              >
                <div class="agent-option">
                  <el-avatar :size="28" :style="{ background: agent.color }">
                    {{ agent.icon }}
                  </el-avatar>
                  <div>
                    <strong>{{ agent.name }}</strong>
                    <span>{{ agent.description }}</span>
                  </div>
                </div>
              </el-option>
            </el-select>
          </el-form-item>

          <el-form-item label="学习问题">
            <el-input
              v-model="question"
              :rows="5"
              maxlength="2000"
              placeholder="输入你想学习的问题..."
              resize="none"
              show-word-limit
              type="textarea"
              @keydown.enter.exact.prevent="startChat"
            />
          </el-form-item>

          <div class="form-footer">
            <span>Agent 会结合已启用的学习资料回答</span>
            <el-button :disabled="!question.trim()" native-type="submit" type="primary">
              开始学习
              <el-icon><ArrowRight /></el-icon>
            </el-button>
          </div>
        </el-form>
      </el-card>

      <p class="shortcut-tip">Enter 开始 · Shift + Enter 换行</p>
    </section>
  </main>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ArrowRight } from '@element-plus/icons-vue'

import { agentCatalog } from '../data/agents'
import type { AgentType } from '../types/chat'

const props = defineProps<{
  initialAgent?: AgentType
}>()

const emit = defineEmits<{
  start: [agent: AgentType, question: string]
}>()

const selectedAgent = ref<AgentType>(props.initialAgent ?? 'math')
const question = ref('')

watch(
  () => props.initialAgent,
  agent => {
    if (agent) {
      selectedAgent.value = agent
    }
  },
)

function startChat() {
  const content = question.value.trim()

  if (!content) {
    return
  }

  emit('start', selectedAgent.value, content)
}
</script>

<style scoped>
.new-session-page {
  display: flex;
  width: 100%;
  height: 100%;
  min-height: 0;
  align-items: center;
  justify-content: center;
  padding: 24px;
  overflow-y: auto;
  background:
    radial-gradient(circle at 70% 20%, rgba(40, 125, 245, 0.1), transparent 32%),
    var(--learning-bg);
}

.new-session-content {
  width: min(720px, 100%);
  text-align: center;
}

.welcome-mark {
  display: grid;
  width: 48px;
  height: 48px;
  margin: 0 auto 16px;
  place-items: center;
  border-radius: 15px;
  background: linear-gradient(135deg, #438fff, #5c6df5);
  box-shadow: 0 10px 25px rgba(50, 120, 240, 0.22);
  color: #fff;
  font-size: 22px;
}

.eyebrow {
  color: var(--el-color-primary);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

h1,
p {
  margin: 0;
}

h1 {
  margin-top: 9px;
  color: var(--learning-text);
  font-family: Georgia, 'Times New Roman', serif;
  font-size: clamp(30px, 5vw, 42px);
  font-weight: 500;
}

.subtitle {
  margin-top: 10px;
  color: var(--learning-text-secondary);
  font-size: 13px;
}

.start-card {
  margin-top: 28px;
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
  text-align: left;
}

.start-card :deep(.el-card__body) {
  padding: clamp(18px, 4vw, 30px);
}

.start-card :deep(.el-form-item) {
  margin-bottom: 20px;
}

.agent-select {
  width: 100%;
}

.agent-option {
  display: flex;
  align-items: center;
  gap: 9px;
}

.agent-option > div {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.agent-option strong {
  color: var(--learning-text);
  font-size: 12px;
}

.agent-option span {
  color: var(--learning-text-muted);
  font-size: 10px;
}

.form-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.form-footer > span {
  color: var(--learning-text-muted);
  font-size: 10px;
}

.shortcut-tip {
  margin-top: 12px;
  color: var(--learning-text-muted);
  font-size: 10px;
}

@media (max-width: 520px) {
  .new-session-page {
    align-items: flex-start;
    padding: 28px 14px;
  }

  .form-footer {
    align-items: flex-start;
    flex-direction: column;
  }

  .form-footer :deep(.el-button) {
    width: 100%;
  }
}
</style>
