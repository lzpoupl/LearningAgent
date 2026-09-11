<template>
  <main class="statistics-page">
    <header class="statistics-header">
      <div>
        <div class="eyebrow">学习进度 / OVERVIEW</div>
        <h1>学习统计</h1>
        <p>看看最近的学习节奏，把稳定的投入变成长期的积累。</p>
      </div>
       <div class="date-label">{{ loading ? '正在更新...' : '最近更新 · 今天' }}</div>
     </header>

    <div v-if="errorMessage" class="load-error" role="alert">{{ errorMessage }}</div>

    <section class="summary-grid" aria-label="学习摘要">
      <article class="summary-card featured-card">
        <div class="summary-label">今日学习</div>
        <div class="summary-value">{{ statistics ? formatDuration(statistics.today.minutes) : '...' }}</div>
        <div class="summary-foot"><span class="trend-up">↑ {{ statistics?.today.changePercent ?? 0 }}%</span> 比昨日多学习</div>
      </article>

      <article class="summary-card">
        <div class="summary-label">本周学习</div>
        <div class="summary-value">{{ statistics ? formatDuration(statistics.week.minutes) : '...' }}</div>
        <div class="progress-track" :aria-label="`本周目标完成 ${weekProgress}%`"><span :style="{ width: `${weekProgress}%` }" /></div>
        <div class="summary-foot">本周目标 {{ formatDuration(statistics?.week.targetMinutes ?? 0) }} · {{ weekProgress }}%</div>
      </article>

      <article class="summary-card">
        <div class="summary-label">Anki 连续使用</div>
        <div class="summary-value">{{ statistics?.streakDays ?? 0 }}<span>天</span></div>
        <div class="streak-dots" :aria-label="`连续使用${statistics?.streakDays ?? 0}天`">
          <i v-for="day in statistics?.streakDays ?? 0" :key="day" class="streak-dot" />
        </div>
        <div class="summary-foot">保持今天的复习节奏</div>
      </article>

      <article class="summary-card">
        <div class="summary-label">已掌握知识点</div>
        <div class="summary-value">{{ statistics?.masteredKnowledgePoints ?? 0 }}<span>个</span></div>
        <div class="summary-foot"><span class="trend-up">↑ {{ statistics?.masteredThisWeek ?? 0 }}</span> 本周新增掌握</div>
      </article>
    </section>

    <section class="chart-section">
      <div class="section-heading">
        <div>
          <span class="panel-kicker">LAST 7 DAYS</span>
          <h2>最近七天每日学习时间</h2>
        </div>
         <div class="chart-total">合计 <strong>{{ formatDuration(statistics?.week.minutes ?? 0) }}</strong></div>
      </div>

      <div class="chart-wrap">
        <div class="y-axis" aria-hidden="true">
          <span v-for="label in axisLabels" :key="label">{{ label }}</span>
        </div>
        <div class="bar-chart" aria-label="最近七天每日学习时间柱状图">
          <div v-for="item in weeklyStudy" :key="item.day" class="bar-column">
            <div class="bar-value">{{ item.minutes ? formatMinutes(item.minutes) : '—' }}</div>
            <div class="bar-track">
             <div class="bar" :class="{ today: item.today }" :style="{ height: `${item.minutes / maxMinutes * 100}%` }" />
            </div>
            <div class="bar-day">{{ item.day }}</div>
          </div>
        </div>
      </div>
    </section>

    <section class="detail-grid">
      <article class="detail-panel">
        <div class="section-heading compact-heading">
          <div>
            <span class="panel-kicker">FOCUS</span>
            <h2>学习构成</h2>
          </div>
        </div>
         <div v-for="subject in statistics?.subjects ?? []" :key="subject.id" class="focus-row">
           <span class="focus-icon" :class="subject.theme">{{ subject.icon }}</span>
           <div class="focus-info"><strong>{{ subject.name }}</strong><span>{{ formatDuration(subject.minutes) }}</span></div>
           <div class="focus-meter"><span :style="{ width: `${subject.percentage}%` }" /></div>
           <b>{{ subject.percentage }}%</b>
        </div>
      </article>

      <article class="detail-panel insight-panel">
         <span class="panel-kicker">THIS WEEK</span>
         <h2>{{ statistics?.insight.title ?? '学习洞察' }}</h2>
         <p>{{ statistics?.insight.description ?? '正在加载本周学习洞察...' }}</p>
         <div class="insight-line"><span>目标完成度</span><strong>{{ statistics?.insight.progressPercent ?? 0 }}%</strong></div>
         <div class="progress-track large"><span :style="{ width: `${statistics?.insight.progressPercent ?? 0}%` }" /></div>
      </article>
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import { getStudyStatistics } from '../services/study'
import type { StudyStatistics } from '../types/study'

const statistics = ref<StudyStatistics | null>(null)
const loading = ref(false)
const errorMessage = ref('')

const weeklyStudy = computed(() => statistics.value?.weeklyStudy ?? [])
const maxMinutes = computed(() => Math.max(1, ...weeklyStudy.value.map(item => item.minutes)))
const weekProgress = computed(() => statistics.value?.week.progressPercent ?? 0)
const axisLabels = computed(() => {
  const step = Math.max(60, Math.ceil(maxMinutes.value / 3 / 60) * 60)
  return [step * 3, step * 2, step, 0].map(minutes => minutes ? formatMinutes(minutes) : '0')
})

function formatDuration(minutes: number) {
  const hours = Math.floor(minutes / 60)
  const remainingMinutes = minutes % 60
  return hours > 0 ? `${hours}小时 ${remainingMinutes}分` : `${remainingMinutes}分`
}

function formatMinutes(minutes: number) {
  const hours = Math.floor(minutes / 60)
  const remainingMinutes = minutes % 60
  return hours > 0 ? `${hours}h ${remainingMinutes}m` : `${remainingMinutes}m`
}

async function loadStatistics() {
  loading.value = true
  errorMessage.value = ''
  try {
    statistics.value = await getStudyStatistics()
  } catch (error) {
    console.error(error)
    errorMessage.value = '学习统计加载失败，请稍后重试。'
  } finally {
    loading.value = false
  }
}

onMounted(loadStatistics)
</script>

<style scoped>
.statistics-page {
  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow-y: auto;
  padding: 42px clamp(20px, 5vw, 70px) 60px;
  background: var(--learning-bg);
  color: var(--learning-text);
}

.statistics-header,
.summary-grid,
.chart-section,
.detail-grid,
.load-error {
  width: min(1220px, 100%);
  margin: 0 auto;
}

.load-error {
  margin-bottom: 16px;
  padding: 9px 11px;
  border: 1px solid #ecd2c8;
  border-radius: 6px;
  background: #fff1ee;
  color: #a34e3f;
  font-size: 12px;
}

.statistics-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 28px;
}

.eyebrow,
.panel-kicker {
  color: var(--learning-primary);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

h1,
h2,
p {
  margin: 0;
}

h1 {
  margin-top: 9px;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: clamp(30px, 4vw, 46px);
  font-weight: 500;
  line-height: 1.1;
}

.statistics-header p {
  margin-top: 10px;
  color: var(--learning-text-secondary);
  font-size: 14px;
}

.date-label {
  color: #9b9389;
  font-size: 12px;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.summary-card,
.chart-section,
.detail-panel {
  border: 1px solid var(--learning-border);
  border-radius: 8px;
  background: #fff;
}

.summary-card {
  min-height: 172px;
  padding: 20px;
}

.featured-card {
  border-color: #c9dbf8;
  background: #edf5ff;
}

.summary-label {
  color: #81786e;
  font-size: 12px;
}

.summary-value {
  margin-top: 16px;
  color: #3c352e;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 35px;
  line-height: 1;
}

.summary-value span {
  margin-left: 5px;
  color: #958b80;
  font-family: inherit;
  font-size: 13px;
}

.summary-foot {
  margin-top: 17px;
  color: #9c948a;
  font-size: 11px;
}

.trend-up {
  color: #68805e;
  font-weight: 700;
}

.progress-track,
.focus-meter {
  height: 5px;
  overflow: hidden;
  border-radius: 99px;
  background: #eee9e2;
}

.progress-track {
  margin-top: 22px;
}

.progress-track span,
.focus-meter span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: #ad8b69;
}

.streak-dots {
  display: flex;
  gap: 5px;
  margin-top: 23px;
}

.streak-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #b89b7b;
}

.chart-section {
  margin-top: 18px;
  padding: 24px 26px 20px;
}

.section-heading {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
}

.compact-heading {
  align-items: flex-start;
}

h2 {
  margin-top: 7px;
  font-size: 20px;
  font-weight: 650;
}

.chart-total {
  color: #9b9389;
  font-size: 12px;
}

.chart-total strong {
  margin-left: 5px;
  color: #655b50;
}

.chart-wrap {
  display: flex;
  height: 245px;
  margin-top: 26px;
}

.y-axis {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  width: 34px;
  padding-bottom: 25px;
  color: #b0a79c;
  font-size: 10px;
}

.bar-chart {
  flex: 1;
  display: flex;
  align-items: stretch;
  justify-content: space-around;
  gap: 14px;
  border-bottom: 1px solid #e7e2da;
}

.bar-column {
  display: flex;
  flex: 1;
  min-width: 28px;
  flex-direction: column;
  align-items: center;
}

.bar-value {
  height: 22px;
  color: #a09588;
  font-size: 10px;
}

.bar-track {
  display: flex;
  width: min(42px, 70%);
  height: 190px;
  align-items: flex-end;
  border-radius: 5px 5px 0 0;
  background: #f4f0ea;
}

.bar {
  width: 100%;
  min-height: 4px;
  border-radius: 5px 5px 0 0;
  background: #c5b19a;
}

.bar.today {
  background: var(--learning-primary);
}

.bar-day {
  height: 25px;
  padding-top: 9px;
  color: #8f867c;
  font-size: 11px;
  white-space: nowrap;
}

.detail-grid {
  display: grid;
  grid-template-columns: 1.15fr 0.85fr;
  gap: 18px;
  margin-top: 18px;
}

.detail-panel {
  min-height: 210px;
  padding: 23px 26px;
}

.focus-row {
  display: grid;
  grid-template-columns: 28px 105px minmax(80px, 1fr) 32px;
  align-items: center;
  gap: 12px;
  margin-top: 22px;
}

.focus-icon {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  font-size: 13px;
  font-weight: 700;
}

.focus-icon.math { background: #eee4d8; color: #896b4e; }
.focus-icon.english { background: #e7ece8; color: #627865; }
.focus-icon.anki { background: #eee9e1; color: #836b50; }

.focus-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.focus-info strong {
  color: #544b42;
  font-size: 13px;
}

.focus-info span {
  color: #a19a91;
  font-size: 11px;
}

.focus-row b {
  color: #897d71;
  font-size: 11px;
  font-weight: 600;
  text-align: right;
}

.insight-panel {
  background: #edf5ff;
}

.insight-panel p {
  margin-top: 16px;
  color: #766d63;
  font-size: 13px;
  line-height: 1.7;
}

.insight-line {
  display: flex;
  justify-content: space-between;
  margin-top: 24px;
  color: #92877b;
  font-size: 12px;
}

.insight-line strong {
  color: var(--learning-primary);
}

.summary-card,
.chart-section,
.detail-panel {
  box-shadow: var(--learning-shadow);
}

.progress-track.large {
  height: 7px;
  margin-top: 9px;
}

@media (max-width: 920px) {
  .summary-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .detail-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 620px) {
  .statistics-page {
    padding: 28px 16px 48px;
  }

  .statistics-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .summary-grid {
    grid-template-columns: 1fr;
  }

  .chart-section,
  .detail-panel {
    padding: 20px 16px;
  }

  .bar-chart {
    gap: 5px;
  }

  .focus-row {
    grid-template-columns: 28px 86px minmax(50px, 1fr) 28px;
    gap: 8px;
  }
}

.statistics-page .summary-card,
.statistics-page .chart-section,
.statistics-page .detail-panel {
  background: var(--learning-surface);
}

.statistics-page .featured-card,
.statistics-page .insight-panel {
  border-color: var(--el-color-primary-light-7);
  background: var(--el-color-primary-light-9);
}

.statistics-page .insight-panel p {
  color: var(--learning-text-secondary);
}

.statistics-page .load-error {
  border-color: var(--el-color-danger-light-7);
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}

.statistics-page .date-label,
.statistics-page .summary-label,
.statistics-page .summary-foot,
.statistics-page .summary-value span,
.statistics-page .chart-total,
.statistics-page .y-axis,
.statistics-page .bar-value,
.statistics-page .bar-day,
.statistics-page .focus-info span,
.statistics-page .focus-row b,
.statistics-page .insight-line {
  color: var(--learning-text-muted);
}

.statistics-page .summary-value,
.statistics-page .chart-total strong,
.statistics-page .focus-info strong {
  color: var(--learning-text);
}

.statistics-page .bar-chart {
  border-color: var(--learning-border);
}

.statistics-page .progress-track,
.statistics-page .focus-meter,
.statistics-page .bar-track {
  background: var(--learning-border-soft);
}

.statistics-page .progress-track span,
.statistics-page .focus-meter span,
.statistics-page .bar,
.statistics-page .bar.today {
  background: var(--learning-primary);
}

.statistics-page .streak-dot {
  background: var(--el-color-primary-light-3);
}

.statistics-page .trend-up,
.statistics-page .insight-line strong {
  color: var(--el-color-success);
}

.statistics-page .focus-icon.math,
.statistics-page .focus-icon.english,
.statistics-page .focus-icon.anki {
  background: var(--el-color-primary-light-9);
  color: var(--learning-primary);
}
</style>
