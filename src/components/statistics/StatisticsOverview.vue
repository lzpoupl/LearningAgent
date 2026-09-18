<template>
  <section class="panel-grid">
    <article class="panel">
      <div class="section-heading">
        <div>
          <span class="panel-kicker">TODAY</span>
          <h2>今日学习统计</h2>
        </div>
      </div>

      <div class="panel-body">
        <div class="donut-wrap">
          <EChart :option="todayOption" height="176px" aria-label="今日卡片完成比例" />
          <div class="donut-center">
            <strong class="donut-value">{{ today?.pendingCards ?? 0 }}</strong>
            <span class="donut-label">今日待复习卡片</span>
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

      <div class="panel-body">
        <div class="donut-wrap">
          <EChart :option="breakdownOption" height="176px" aria-label="卡片数量占比" />
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
</template>

<script setup lang="ts">
import { computed } from 'vue'

import EChart from '../charts/EChart.vue'
import { cardBreakdownOption, todayDonutOption } from '../charts/statisticsCharts'
import { CATEGORY_COLORS } from '../charts/statisticsColors'
import { useTheme } from '../../composables/useTheme'
import type { CardBreakdown, TodayProgress } from '../../types/statistics'

const props = defineProps<{
  today: TodayProgress | null
  breakdown: CardBreakdown | null
}>()

const { isDark } = useTheme()

const todayOption = computed(() => todayDonutOption(props.today, isDark.value))
const breakdownOption = computed(() => cardBreakdownOption(props.breakdown, isDark.value))
</script>

<style scoped>
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

.panel-body {
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

@media (max-width: 980px) {
  .panel-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 620px) {
  .panel {
    padding: 20px 16px;
  }

  .panel-body {
    flex-direction: column;
  }
}
</style>
