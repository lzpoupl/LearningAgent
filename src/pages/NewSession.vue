<template>
  <main class="new-session-page">
    <button class="back-button" type="button" @click="emit('back')">
      <el-icon>
        <ArrowLeft />
      </el-icon>
      返回对话
    </button>

    <section class="new-session-content">
      <div class="welcome-mark">✦</div>
      <span class="eyebrow">NEW LEARNING SESSION</span>
      <h1>开始新的学习</h1>
      <p class="subtitle">选择一个学习助手，然后输入你想学习的问题</p>

      <el-card class="start-card" shadow="never">
        <el-form label-position="top" @submit.prevent="startChat">
          <el-form-item label="学习助手">
            <el-select
              v-model="selectedAgent"
              class="agent-select"
              popper-class="agent-select-popper"
              :disabled="loadingAgents || agents.length === 0"
              placeholder="选择学习助手"
            >
              <el-option v-for="agent in agents" :key="agent.id" :label="agent.name" :value="agent.id">
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
            <span v-if="agentError" class="field-error" role="alert">{{ agentError }}</span>
          </el-form-item>

          <el-form-item label="学习问题">
            <el-input v-model="question" :rows="5" maxlength="2000" placeholder="输入你想学习的问题..." resize="none"
              show-word-limit type="textarea" @keydown.enter.exact.prevent="startChat" />
          </el-form-item>

          <div class="form-footer">
            <span>Agent 会结合已启用的学习资料回答</span>
            <el-button :disabled="!question.trim() || !selectedAgent || loadingAgents" native-type="submit" type="primary">
              开始学习
              <el-icon>
                <ArrowRight />
              </el-icon>
            </el-button>
          </div>
        </el-form>
      </el-card>

      <p class="shortcut-tip">Enter 开始 · Shift + Enter 换行</p>
    </section>
  </main>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { ArrowLeft, ArrowRight } from '@element-plus/icons-vue'

import { listAgents } from '../services/agent'
import type { AgentInfo, AgentType } from '../types/chat'

const props = defineProps<{
  initialAgent?: AgentType
}>()

const emit = defineEmits<{
  start: [agent: AgentType, question: string]
  back: []
}>()

const agents = ref<AgentInfo[]>([])
const selectedAgent = ref<AgentType>(props.initialAgent ?? '')
const question = ref('')
const loadingAgents = ref(false)
const agentError = ref('')

watch(
  () => props.initialAgent,
  agent => {
    if (agent) {
      selectedAgent.value = agent
    }
  },
)

async function loadAgents() {
  loadingAgents.value = true
  agentError.value = ''

  try {
    agents.value = (await listAgents()).filter(agent => agent.enabled)
    if (!selectedAgent.value || !agents.value.some(agent => agent.id === selectedAgent.value)) {
      selectedAgent.value = agents.value[0]?.id ?? ''
    }
  } catch (error) {
    console.error(error)
    agentError.value = '学习助手加载失败，请稍后重试。'
  } finally {
    loadingAgents.value = false
  }
}

function startChat() {
  const content = question.value.trim()

  if (!content || !selectedAgent.value) {
    return
  }

  emit('start', selectedAgent.value, content)
}

onMounted(loadAgents)
</script>

<style scoped>
.new-session-page {
  position: relative;
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

.back-button {
  position: absolute;
  top: 20px;
  left: 24px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border: 1px solid var(--learning-border);
  border-radius: 999px;
  background: var(--learning-surface);
  box-shadow: 0 2px 10px rgba(23, 35, 59, 0.05);
  color: var(--learning-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: border-color 140ms ease, background 140ms ease, color 140ms ease;
}

.back-button:hover {
  border-color: #b7d0f8;
  background: #eef5ff;
  color: var(--learning-primary);
}

.back-button:focus-visible {
  outline: 2px solid var(--learning-primary);
  outline-offset: 2px;
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

/* 下拉选项：抬高每行高度，让助手图标与文字有足够呼吸空间 */
:global(.agent-select-popper .el-select-dropdown__item) {
  height: auto;
  min-height: 56px;
  padding: 9px 12px;
  line-height: 1.35;
}

.field-error {
  display: block;
  margin-top: 6px;
  color: var(--el-color-danger);
  font-size: 11px;
}

.agent-option {
  display: flex;
  align-items: center;
  gap: 9px;
}

.agent-option>div {
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

.form-footer>span {
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

  .back-button {
    top: 14px;
    left: 14px;
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
