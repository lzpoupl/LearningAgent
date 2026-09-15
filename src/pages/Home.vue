<template>
  <main class="home-page">
    <div class="home-scroll">
      <div class="home-inner">
        <section class="card hero">
          <h1>
            {{ greeting }}，<br />
            开启你的终身学习之旅
          </h1>

          <p>AI Agent × 学习资产 × 间隔重复 · 让学习更高效</p>
        </section>

        <div class="quick-grid">
          <button
            v-for="action in quickActions"
            :key="action.title"
            class="card quick-card"
            type="button"
            @click="openQuickAction(action)"
          >
            <div class="quick-icon" :class="action.theme">
              <AgentGlyph v-if="action.agentIcon" :icon="action.agentIcon" :size="22" />
              <template v-else>{{ action.icon }}</template>
            </div>

            <strong>{{ action.title }}</strong>

            <span>{{ action.description }}</span>
          </button>
        </div>

        <div class="home-columns">
          <section class="card section-card">
            <div class="section-title">
              <strong>今日学习计划</strong>
              <span>{{ todayLabel }}</span>
            </div>

            <div
              v-for="task in tasks"
              :key="task.id"
              class="task"
            >
              <input
                v-model="task.done"
                class="checkbox"
                type="checkbox"
                :disabled="taskUpdating === task.id"
                @change="toggleTask(task)"
              />

              <div class="task-main">
                {{ task.title }}
                <small>{{ task.detail }}</small>
              </div>

              <span class="task-time">{{ taskTime(task.minutes) }}</span>
            </div>
          </section>

          <section class="card section-card">
            <div class="section-title">
              <strong>今日学习统计</strong>
              <button
                class="section-link"
                type="button"
                @click="emit('open-anki')"
              >
                进入复习 →
              </button>
            </div>

            <div
              class="stats-circle"
              :style="circleStyle"
            >
              <div class="stats-circle-content">
                <strong>{{ statsLoading ? '…' : dueTodayCount }}</strong>
                <span>今日待复习卡片</span>
              </div>
            </div>

            <p
              v-if="statsError"
              class="stats-error"
              role="alert"
            >
              {{ statsError }}
            </p>

            <div class="mini-stats">
              <div>
                <strong>{{ doneTaskCount }}/{{ tasks.length }}</strong>
                <small>完成任务</small>
              </div>

              <div>
                <strong>{{ statsLoading ? '…' : totalCardCount }}</strong>
                <small>卡片总数</small>
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import AgentGlyph from '../components/agent/AgentGlyph.vue'
import { listAgents } from '../services/agent'
import { getTodayOverview, updateStudyTask } from '../services/study'
import type { AgentInfo, AgentType } from '../types/chat'
import type { StudyTask, TodayOverview } from '../types/study'

const emit = defineEmits<{
  'start-chat': [agent: AgentType]
  'open-agents': []
  'open-assets': []
  'open-anki': []
}>()

interface QuickAction {
  icon?: string
  agentIcon?: string
  title: string
  description: string
  theme: 'blue' | 'green' | 'purple' | 'orange'
  agent?: AgentType
  target?: 'agents' | 'assets'
}

const agents = ref<AgentInfo[]>([])
const tasks = ref<StudyTask[]>([])
const overview = ref<TodayOverview | null>(null)
const statsLoading = ref(false)
const statsError = ref('')
const taskUpdating = ref('')

const quickActions = computed<QuickAction[]>(() => {
  const agentActions = agents.value
    .slice(0, 2)
    .map((agent, index) => ({
      agentIcon: agent.icon,
      title: agent.name,
      description: agent.description,
      theme: index === 0 ? ('blue' as const) : ('green' as const),
      agent: agent.id,
    }))

  return [
    ...agentActions,
    {
      icon: '♙',
      title: '自定义 Agent',
      description: '配置你的专属学习助手',
      theme: 'purple',
      target: 'agents',
    },
    {
      icon: '▣',
      title: '添加学习资料',
      description: 'PDF · 笔记 · 课件',
      theme: 'orange',
      target: 'assets',
    },
  ]
})

const greeting = computed(() => {
  const hour = new Date().getHours()

  if (hour < 6) {
    return '夜深了'
  }

  if (hour < 12) {
    return '早上好'
  }

  if (hour < 14) {
    return '中午好'
  }

  if (hour < 18) {
    return '下午好'
  }

  return '晚上好'
})

const todayLabel = computed(() => {
  const now = overview.value?.date ? new Date(overview.value.date) : new Date()
  const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']

  return `${now.getMonth() + 1}月${now.getDate()}日 · ${weekdays[now.getDay()]}`
})

const doneTaskCount = computed(
  () => tasks.value.filter(task => task.done).length
)

const totalCardCount = computed(() => overview.value?.totalCardCount ?? 0)
const dueTodayCount = computed(() => overview.value?.dueCardCount ?? 0)

const circleStyle = computed(() => {
  const percent =
    totalCardCount.value > 0
      ? Math.min(
          100,
          Math.round((dueTodayCount.value / totalCardCount.value) * 100)
        )
      : 0

  return {
    background: `conic-gradient(var(--primary) 0 ${percent}%, var(--track) ${percent}% 100%)`
  }
})

function taskTime(minutes: number): string {
  return `${(minutes / 60).toFixed(1)}h`
}

async function toggleTask(task: StudyTask) {
  const previous = !task.done
  if (taskUpdating.value) {
    task.done = previous
    return
  }

  taskUpdating.value = task.id
  statsError.value = ''
  try {
    const updatedTask = await updateStudyTask(task.id, task.done)
    Object.assign(task, updatedTask)
  } catch (error) {
    console.error(error)
    task.done = previous
    statsError.value = '学习计划更新失败，请重试。'
  } finally {
    taskUpdating.value = ''
  }
}

function openQuickAction(action: QuickAction) {
  if (action.agent) {
    emit('start-chat', action.agent)
  } else if (action.target === 'agents') {
    emit('open-agents')
  } else if (action.target === 'assets') {
    emit('open-assets')
  }
}

async function loadStats() {
  statsLoading.value = true
  statsError.value = ''

  try {
    const [loadedOverview, loadedAgents] = await Promise.all([
      getTodayOverview(),
      listAgents(),
    ])
    overview.value = loadedOverview
    tasks.value = loadedOverview.tasks
    agents.value = loadedAgents
  } catch (error) {
    console.error(error)
    statsError.value = '首页数据加载失败，请稍后重试。'
  } finally {
    statsLoading.value = false
  }
}

onMounted(loadStats)
</script>

<style scoped>
.home-page {
  --primary: #287df5;
  --primary-light: #eaf3ff;
  --primary-dark: #1764d1;

  --bg: #f5f8fc;
  --surface: #ffffff;

  --text: #17233b;
  --text-2: #64748b;
  --text-3: #94a3b8;

  --border: #e5ebf3;
  --track: #e8eff8;

  --radius: 14px;
  --shadow: 0 8px 30px rgba(35, 75, 130, 0.07);

  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow: hidden;

  display: flex;
  flex-direction: column;

  background: var(--bg);
  color: var(--text);
}

.home-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 28px 32px 40px;
}

.home-inner {
  max-width: 1240px;
  margin: 0 auto;
}

.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.hero {
  position: relative;
  min-height: 205px;
  padding: 35px;
  overflow: hidden;
  color: #17355e;
  background: linear-gradient(105deg, #edf6ff 0%, #e9f4ff 52%, #e8f0ff 100%);
}

.hero::after {
  content: '';
  position: absolute;
  right: -40px;
  top: -80px;
  width: 420px;
  height: 320px;
  border-radius: 50%;
  background: radial-gradient(
    circle,
    rgba(90, 145, 235, 0.25),
    rgba(90, 145, 235, 0) 68%
  );
}

.hero h1 {
  position: relative;
  z-index: 1;
  margin: 0;
  font-size: 25px;
  line-height: 1.5;
}

.hero p {
  position: relative;
  z-index: 1;
  margin: 8px 0 0;
  color: #6b83a1;
  font-size: 12px;
}

.quick-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
  margin-top: 18px;
}

.quick-card {
  padding: 18px;
  min-height: 125px;
  text-align: left;
  color: var(--text);
  cursor: pointer;
  transition: 0.2s;
}

.quick-card:hover {
  transform: translateY(-2px);
}

.quick-icon {
  width: 38px;
  height: 38px;
  border-radius: 11px;
  display: grid;
  place-items: center;
  font-size: 18px;
  margin-bottom: 13px;
}

.quick-icon.blue {
  background: #e9f3ff;
  color: #3d8df5;
}

.quick-icon.green {
  background: #e9faf1;
  color: #2d9a6d;
}

.quick-icon.purple {
  background: #f0ebff;
  color: #8764e8;
}

.quick-icon.orange {
  background: #fff4e4;
  color: #d98b13;
}

.quick-card strong {
  display: block;
  font-size: 13px;
}

.quick-card span {
  display: block;
  color: var(--text-3);
  font-size: 10px;
  margin-top: 6px;
}

.home-columns {
  display: grid;
  grid-template-columns: 1.5fr 1fr;
  gap: 16px;
  margin-top: 16px;
}

.section-card {
  padding: 20px;
}

.section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 17px;
}

.section-title strong {
  font-size: 14px;
}

.section-title span {
  color: var(--text-3);
  font-size: 11px;
}

.section-link {
  border: none;
  background: transparent;
  color: var(--text-3);
  font-size: 11px;
  cursor: pointer;
  transition: 0.2s;
}

.section-link:hover {
  color: var(--primary);
}

.task {
  display: flex;
  align-items: center;
  min-height: 45px;
  border-bottom: 1px solid #f0f3f7;
}

.task:last-child {
  border-bottom: 0;
}

.checkbox {
  width: 17px;
  height: 17px;
  margin-right: 10px;
  accent-color: var(--primary);
}

.task-main {
  flex: 1;
  font-size: 12px;
}

.task-main small {
  display: block;
  color: var(--text-3);
  margin-top: 3px;
  font-size: 10px;
}

.task-time {
  color: var(--text-2);
  font-size: 10px;
}

.stats-circle {
  position: relative;
  width: 125px;
  height: 125px;
  margin: 12px auto 15px;
  border-radius: 50%;
  display: grid;
  place-items: center;
}

.stats-circle::before {
  content: '';
  position: absolute;
  inset: 0;
  margin: auto;
  width: 96px;
  height: 96px;
  border-radius: 50%;
  background: var(--surface);
}

.stats-circle-content {
  position: relative;
  z-index: 1;
  text-align: center;
}

.stats-circle strong {
  display: block;
  font-size: 23px;
}

.stats-circle span {
  font-size: 10px;
  color: var(--text-3);
}

.stats-error {
  margin: 0 0 12px;
  text-align: center;
  color: #ef6a6a;
  font-size: 10px;
}

.mini-stats {
  display: flex;
  justify-content: space-around;
  text-align: center;
}

.mini-stats strong {
  font-size: 15px;
}

.mini-stats small {
  display: block;
  color: var(--text-3);
  margin-top: 4px;
  font-size: 10px;
}

.home-scroll::-webkit-scrollbar {
  width: 6px;
}

.home-scroll::-webkit-scrollbar-thumb {
  border-radius: 10px;
  background: #dddddd;
}

.home-scroll::-webkit-scrollbar-track {
  background: transparent;
}

html.dark .home-page {
  --primary-light: #17263d;
  --bg: #0f1216;
  --surface: #171b21;
  --text: #e6eaf1;
  --text-2: #a7aebc;
  --text-3: #6d7686;
  --border: #2a3038;
  --track: #232832;
  --shadow: 0 10px 34px rgba(0, 0, 0, 0.45);
}

html.dark .home-page .hero {
  color: #e6eaf1;
  background: linear-gradient(105deg, #17233a 0%, #182538 52%, #1a2540 100%);
}

html.dark .home-page .hero p {
  color: #9fb0c8;
}

html.dark .home-page .quick-icon.blue {
  background: #17263d;
  color: #6fa8f8;
}

html.dark .home-page .quick-icon.green {
  background: #16281f;
  color: #5fbf92;
}

html.dark .home-page .quick-icon.purple {
  background: #221c38;
  color: #a98ff0;
}

html.dark .home-page .quick-icon.orange {
  background: #2c2314;
  color: #e0a94a;
}

html.dark .home-page .task {
  border-bottom-color: var(--learning-border);
}

html.dark .home-page .home-scroll::-webkit-scrollbar-thumb {
  background: var(--learning-border);
}

@media (max-width: 1000px) {
  .quick-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .home-columns {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 650px) {
  .home-scroll {
    padding: 18px;
  }

  .quick-grid {
    grid-template-columns: 1fr;
  }
}
</style>
