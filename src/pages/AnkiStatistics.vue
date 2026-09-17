<template>
  <main class="stats-page">
    <PageHeader
      eyebrow="ANKI STATISTICS"
      title="Anki 卡片统计数据"
      description="复习进度、卡片分布与历史趋势，全部来自你的 Anki 卡片库。"
    >
      <el-button :icon="Refresh" :loading="loading" @click="reloadAll">刷新</el-button>
    </PageHeader>

    <div v-if="errorMessage" class="load-error" role="alert">{{ errorMessage }}</div>

    <section class="panel-grid">
      <article class="panel">
        <div class="section-heading">
          <div>
            <span class="panel-kicker">TODAY</span>
            <h2>今日学习统计</h2>
          </div>
        </div>

        <div class="today-body">
          <div class="donut-wrap">
            <EChart
              :option="todayOption"
              height="176px"
              aria-label="今日卡片完成比例"
            />
            <div class="donut-center">
              <strong class="donut-value">{{ today?.pendingCards ?? 0 }}</strong>
              <span class="donut-label">今日待复习卡片</span>
            </div>
          </div>

          <div class="today-metrics">
            <div>
              <strong>{{ today?.reviewedCards ?? 0 }}/{{ today?.totalCards ?? 0 }}</strong>
              <small>完成任务</small>
            </div>
            <div>
              <strong>{{ today?.totalCards ?? 0 }}</strong>
              <small>卡片总数</small>
            </div>
          </div>
        </div>
      </article>

      <article class="panel">
        <div class="section-heading">
          <div>
            <span class="panel-kicker">CARDS</span>
            <h2>卡片数量</h2>
          </div>
        </div>

        <div class="breakdown-body">
          <div class="donut-wrap">
            <EChart
              :option="breakdownOption"
              height="176px"
              aria-label="卡片数量占比"
            />
            <div class="donut-center">
              <strong class="donut-value">{{ breakdown?.total ?? 0 }}</strong>
              <span class="donut-label">卡片总数</span>
            </div>
          </div>

          <table class="legend">
            <tbody>
              <tr v-for="item in breakdown?.categories ?? []" :key="item.category">
                <td class="legend-name">
                  <i class="legend-dot" :style="{ background: CATEGORY_COLORS[item.category] }" />
                  {{ item.label }}
                </td>
                <td class="legend-count">{{ item.count }}</td>
                <td class="legend-percent">{{ item.percent }}%</td>
              </tr>
              <tr class="legend-total">
                <td class="legend-name">总计</td>
                <td class="legend-count">{{ breakdown?.total ?? 0 }}</td>
                <td class="legend-percent" />
              </tr>
            </tbody>
          </table>
        </div>
      </article>
    </section>

    <section class="panel chart-panel">
      <div class="section-heading">
        <div>
          <span class="panel-kicker">REVIEWS</span>
          <h2>复习</h2>
        </div>
        <el-radio-group v-model="reviewRange" size="small" @change="reloadReviewHistory">
          <el-radio-button
            v-for="option in RANGE_OPTIONS"
            :key="option.value"
            :label="option.value"
          >
            {{ option.label }}
          </el-radio-button>
        </el-radio-group>
      </div>

      <p class="panel-caption">已经回答的问题的数量。</p>

      <EChart
        :option="reviewOption"
        height="248px"
        aria-label="每日复习次数"
      />

      <div class="chart-summary">
        <span>
          学习天数 {{ reviewHistory?.studiedDays ?? 0 }}/{{ reviewHistory?.elapsedDays ?? 0 }}
          ({{ reviewHistory?.studiedDayPercent ?? 0 }}%)
        </span>
        <span>总计：{{ reviewHistory?.totalReviews ?? 0 }} 次复习</span>
        <span>
          平均值（包含未学习天数）：{{ reviewHistory?.averagePerElapsedDay ?? 0 }} 次复习/天
        </span>
        <span>
          平均值（只计实际学习天数）：{{ reviewHistory?.averagePerStudiedDay ?? 0 }} 次复习/天
        </span>
      </div>
    </section>

    <section class="panel chart-panel">
      <div class="section-heading">
        <div>
          <span class="panel-kicker">ADDED</span>
          <h2>新增</h2>
        </div>
        <el-radio-group v-model="addedRange" size="small" @change="reloadAddedCards">
          <el-radio-button
            v-for="option in RANGE_OPTIONS"
            :key="option.value"
            :label="option.value"
          >
            {{ option.label }}
          </el-radio-button>
        </el-radio-group>
      </div>

      <p class="panel-caption">新增的卡片数量。</p>

      <EChart
        :option="addedOption"
        height="248px"
        aria-label="每日新增卡片数量"
      />

      <div class="chart-summary">
        <span>总计 {{ addedCards?.total ?? 0 }} 张卡片</span>
        <span>平均 {{ addedCards?.averagePerDay ?? 0 }} 张/天</span>
      </div>
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Refresh } from '@element-plus/icons-vue'

import EChart from '../components/charts/EChart.vue'
import {
  addedBarOption,
  cardBreakdownOption,
  reviewBarOption,
  todayDonutOption,
} from '../components/charts/statisticsCharts'
import { CATEGORY_COLORS } from '../components/charts/statisticsColors'
import PageHeader from '../components/common/PageHeader.vue'
import { useTheme } from '../composables/useTheme'
import {
  getAddedCards,
  getCardBreakdown,
  getReviewHistory,
  getTodayProgress,
} from '../services/statistics'
import type {
  AddedCardsStats,
  CardBreakdown,
  ReviewHistoryStats,
  TimeRange,
  TodayProgress,
} from '../types/statistics'

const RANGE_OPTIONS: Array<{ value: TimeRange; label: string }> = [
  { value: 'last_month', label: '1 个月' },
  { value: 'last_three_months', label: '3 个月' },
  { value: 'last_year', label: '1 年' },
  { value: 'all', label: '全部时间' },
]

const today = ref<TodayProgress | null>(null)
const breakdown = ref<CardBreakdown | null>(null)
const reviewHistory = ref<ReviewHistoryStats | null>(null)
const addedCards = ref<AddedCardsStats | null>(null)
const reviewRange = ref<TimeRange>('last_year')
const addedRange = ref<TimeRange>('last_year')
const loading = ref(false)
const errorMessage = ref('')

const { isDark } = useTheme()

const todayOption = computed(() => todayDonutOption(today.value, isDark.value))
const breakdownOption = computed(() => cardBreakdownOption(breakdown.value, isDark.value))
const reviewOption = computed(() =>
  reviewBarOption(reviewHistory.value?.days ?? [], isDark.value),
)
const addedOption = computed(() =>
  addedBarOption(addedCards.value?.days ?? [], isDark.value),
)


async function reloadAll() {
  loading.value = true
  errorMessage.value = ''
  try {
    const [progress, cards, reviews, added] = await Promise.all([
      getTodayProgress(),
      getCardBreakdown(),
      getReviewHistory(reviewRange.value),
      getAddedCards(addedRange.value),
    ])
    today.value = progress
    breakdown.value = cards
    reviewHistory.value = reviews
    addedCards.value = added
  } catch (error) {
    console.error(error)
    errorMessage.value = '统计数据加载失败，请稍后重试。'
  } finally {
    loading.value = false
  }
}

async function reloadReviewHistory() {
  try {
    reviewHistory.value = await getReviewHistory(reviewRange.value)
  } catch (error) {
    console.error(error)
    errorMessage.value = '复习历史加载失败，请稍后重试。'
  }
}

async function reloadAddedCards() {
  try {
    addedCards.value = await getAddedCards(addedRange.value)
  } catch (error) {
    console.error(error)
    errorMessage.value = '新增卡片加载失败，请稍后重试。'
  }
}

onMounted(reloadAll)
</script>

<style scoped>
.stats-page {
  flex: 1;
  min-width: 0;
  height: 100%;
  min-height: 0;
  padding: 32px 36px 48px;
  overflow-y: auto;
  background: var(--learning-bg);
}

.panel-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 18px;
}

.panel {
  padding: 22px 26px 26px;
  border: 1px solid var(--learning-border);
  border-radius: 16px;
  background: var(--learning-surface);
  box-shadow: var(--learning-shadow);
}

.chart-panel {
  margin-top: 18px;
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.panel-kicker {
  color: var(--learning-text-muted);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

.section-heading h2 {
  margin: 4px 0 0;
  color: var(--learning-text);
  font-size: 17px;
}


.panel-caption {
  margin: 12px 0 0;
  color: var(--learning-text-muted);
  font-size: 12px;
  text-align: center;
}

.today-body,
.breakdown-body {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 28px;
  margin-top: 18px;
}

.donut-wrap {
  position: relative;
  width: 176px;
  height: 176px;
  flex: none;
}

.donut-center {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  color: var(--learning-text);
  text-align: center;
  pointer-events: none;
}

.donut-value {
  font-size: 30px;
  font-weight: 700;
  line-height: 1.1;
}

.donut-label {
  max-width: 90px;
  color: var(--learning-text-muted);
  font-size: 11px;
  line-height: 1.4;
}

.today-metrics {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.today-metrics strong {
  display: block;
  color: var(--learning-text);
  font-size: 22px;
}

.today-metrics small {
  color: var(--learning-text-muted);
  font-size: 11px;
}

.legend {
  flex: 1;
  min-width: 0;
  border-collapse: collapse;
  font-size: 12px;
}

.legend td {
  padding: 3px 0;
  color: var(--learning-text-secondary);
}

.legend-name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 2px;
}

.legend-count,
.legend-percent {
  width: 52px;
  color: var(--learning-text-muted);
  text-align: right;
}

.legend-total td {
  padding-top: 8px;
  color: var(--learning-text);
  font-weight: 600;
}

.chart-summary {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 6px 20px;
  margin-top: 16px;
  color: var(--learning-text-muted);
  font-size: 12px;
}

.load-error {
  margin-bottom: 16px;
  padding: 12px 16px;
  border: 1px solid var(--el-color-danger-light-7);
  border-radius: 12px;
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
  font-size: 13px;
}

@media (max-width: 980px) {
  .panel-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 620px) {
  .stats-page {
    padding: 28px 16px 48px;
  }

  .panel {
    padding: 20px 16px;
  }

  .today-body,
  .breakdown-body {
    flex-direction: column;
  }
}
</style>