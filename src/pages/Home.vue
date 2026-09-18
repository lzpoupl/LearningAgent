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

        <StatisticsOverview
          class="home-stats"
          :today="todayStats"
          :breakdown="breakdown"
        />

        <p
          v-if="statsError"
          class="stats-error"
          role="alert"
        >
          {{ statsError }}
        </p>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import AgentGlyph from '../components/agent/AgentGlyph.vue'
import StatisticsOverview from '../components/statistics/StatisticsOverview.vue'
import { listAgents } from '../services/agent'
import { getCardBreakdown, getTodayProgress } from '../services/statistics'
import type { AgentInfo, AgentType } from '../types/chat'
import type { CardBreakdown, TodayProgress } from '../types/statistics'

const emit = defineEmits<{
  'start-chat': [agent: AgentType]
  'open-agents': []
  'open-assets': []
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
const todayStats = ref<TodayProgress | null>(null)
const breakdown = ref<CardBreakdown | null>(null)
const statsError = ref('')

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
  statsError.value = ''

  try {
    const [loadedAgents, loadedStats, loadedBreakdown] = await Promise.all([
      listAgents(),
      getTodayProgress(),
      getCardBreakdown(),
    ])
    agents.value = loadedAgents
    todayStats.value = loadedStats
    breakdown.value = loadedBreakdown
  } catch (error) {
    console.error(error)
    statsError.value = '首页数据加载失败，请稍后重试。'
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

.home-stats {
  margin-top: 16px;
}

.stats-error {
  margin: 12px 0 0;
  color: #ef6a6a;
  font-size: 12px;
  text-align: center;
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

html.dark .home-page .home-scroll::-webkit-scrollbar-thumb {
  background: var(--learning-border);
}

@media (max-width: 1000px) {
  .quick-grid {
    grid-template-columns: repeat(2, 1fr);
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
