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

    <StatisticsOverview
      :today="today"
      :breakdown="breakdown"
    />

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
  reviewBarOption,
} from '../components/charts/statisticsCharts'
import PageHeader from '../components/common/PageHeader.vue'
import StatisticsOverview from '../components/statistics/StatisticsOverview.vue'
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

@media (max-width: 620px) {
  .stats-page {
    padding: 28px 16px 48px;
  }

  .panel {
    padding: 20px 16px;
  }
}
</style>
