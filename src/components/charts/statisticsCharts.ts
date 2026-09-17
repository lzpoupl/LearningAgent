import type { CardBreakdown, DailyCount, TodayProgress } from '../../types/statistics'

import type { EChartsOption } from './echarts'
import {
  ADDED_BAR_GRADIENT,
  CATEGORY_COLORS,
  REVIEW_BAR_GRADIENT,
  TODAY_DONE_COLOR,
  TODAY_PENDING_COLOR,
} from './statisticsColors'

/** 图表用色，与 `src/styles/app.css` 的 --learning-* 变量保持一致。 */
interface ChartPalette {
  text: string
  muted: string
  split: string
  axis: string
  surface: string
  tooltipBackground: string
  tooltipBorder: string
}

const LIGHT_PALETTE: ChartPalette = {
  text: '#17233b',
  muted: '#94a3b8',
  split: '#eef2f7',
  axis: '#e5ebf3',
  surface: '#ffffff',
  tooltipBackground: 'rgba(255, 255, 255, 0.96)',
  tooltipBorder: '#e5ebf3',
}

const DARK_PALETTE: ChartPalette = {
  text: '#e6eaf1',
  muted: '#6d7686',
  split: '#232830',
  axis: '#2a3038',
  surface: '#171b21',
  tooltipBackground: 'rgba(23, 27, 33, 0.96)',
  tooltipBorder: '#2a3038',
}

function palette(dark: boolean): ChartPalette {
  return dark ? DARK_PALETTE : LIGHT_PALETTE
}

/** 浮层样式：跟随深浅色主题。 */
function tooltipStyle(palette: ChartPalette) {
  return {
    backgroundColor: palette.tooltipBackground,
    borderColor: palette.tooltipBorder,
    borderWidth: 1,
    padding: [6, 10],
    textStyle: { color: palette.text, fontSize: 12 },
  }
}

/**
 * 相邻扇形的分隔描边。
 *
 * 这里不使用 `borderRadius`：ECharts 在圆角半径超过弧长时会把扇形压成圆点，
 * 占比很小的分类就不再是按比例画出的弧，环形图会失真。描边也只取 1px，
 * 避免盖住占比极低的细扇形。
 */
function sectorSeparator(palette: ChartPalette) {
  return { borderColor: palette.surface, borderWidth: 1 }
}

/** 环形图的底色轨道，保证没有数据时也能看到圆环。 */
function donutTrack(palette: ChartPalette, radius: [string, string]) {
  return {
    type: 'pie' as const,
    radius,
    center: ['50%', '50%'],
    silent: true,
    tooltip: { show: false },
    label: { show: false },
    labelLine: { show: false },
    data: [{ value: 1, name: '', itemStyle: { color: palette.split } }],
  }
}

/** 今日学习统计：已完成与待复习两段，扇形角度严格等于两者之比。 */
export function todayDonutOption(progress: TodayProgress | null, dark: boolean): EChartsOption {
  const active = palette(dark)
  const radius: [string, string] = ['70%', '94%']

  return {
    tooltip: { ...tooltipStyle(active), trigger: 'item', formatter: '{b}：{c} 张' },
    series: [
      donutTrack(active, radius),
      {
        type: 'pie',
        radius,
        center: ['50%', '50%'],
        label: { show: false },
        labelLine: { show: false },
        emphasis: { scale: true, scaleSize: 5 },
        data: [
          {
            name: '今日已完成',
            value: progress?.reviewedCards ?? 0,
            itemStyle: { color: TODAY_DONE_COLOR },
          },
          {
            name: '今日待复习',
            value: progress?.pendingCards ?? 0,
            itemStyle: { color: TODAY_PENDING_COLOR },
          },
        ],
      },
    ],
  }
}

/** 卡片数量：四个状态按占比着色。 */
export function cardBreakdownOption(
  breakdown: CardBreakdown | null,
  dark: boolean,
): EChartsOption {
  const active = palette(dark)
  const radius: [string, string] = ['64%', '92%']
  const data = (breakdown?.categories ?? [])
    .filter(item => item.count > 0)
    .map(item => ({
      name: item.label,
      value: item.count,
      itemStyle: { color: CATEGORY_COLORS[item.category] },
    }))

  return {
    tooltip: { ...tooltipStyle(active), trigger: 'item', formatter: '{b}：{c} 张（{d}%）' },
    series: [
      donutTrack(active, radius),
      {
        type: 'pie',
        radius,
        center: ['50%', '50%'],
        label: { show: false },
        labelLine: { show: false },
        itemStyle: sectorSeparator(active),
        emphasis: { scale: true, scaleSize: 5 },
        data,
      },
    ],
  }
}

/** 柱状图渐变：`from` 在顶部，`to` 在底部。 */
function barGradient(from: string, to: string) {
  return {
    type: 'linear' as const,
    x: 0,
    y: 0,
    x2: 0,
    y2: 1,
    colorStops: [
      { offset: 0, color: from },
      { offset: 1, color: to },
    ],
  }
}

/** 按天计数的柱状图：天数较多时提供缩放，避免柱子被压成细线。 */
function dailyBarOption(
  days: DailyCount[],
  dark: boolean,
  unit: string,
  gradient: [string, string],
): EChartsOption {
  const active = palette(dark)
  const zoomable = days.length > 60

  return {
    grid: { left: 44, right: 18, top: 18, bottom: zoomable ? 48 : 28 },
    tooltip: {
      ...tooltipStyle(active),
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      formatter: (params: unknown) => {
        const first = (Array.isArray(params) ? params[0] : params) as
          | { dataIndex: number; value: number }
          | undefined
        if (!first) {
          return ''
        }
        const day = days[first.dataIndex]
        return `${day ? day.day : ''}<br/>${first.value}${unit}`
      },
    },
    xAxis: {
      type: 'category',
      data: days.map(day => day.daysAgo),
      axisTick: { show: false },
      axisLine: { lineStyle: { color: active.axis } },
      axisLabel: { color: active.muted, fontSize: 10, hideOverlap: true },
    },
    yAxis: {
      type: 'value',
      minInterval: 1,
      splitLine: { lineStyle: { color: active.split } },
      axisLabel: { color: active.muted, fontSize: 10 },
    },
    dataZoom: zoomable
      ? [
          { type: 'inside', throttle: 60 },
          {
            type: 'slider',
            height: 14,
            bottom: 8,
            borderColor: 'transparent',
            backgroundColor: active.split,
            fillerColor: 'rgba(40, 125, 245, 0.16)',
            handleStyle: { color: active.axis },
            moveHandleSize: 0,
            textStyle: { color: active.muted, fontSize: 9 },
          },
        ]
      : [],
    series: [
      {
        type: 'bar',
        data: days.map(day => day.count),
        barCategoryGap: '20%',
        itemStyle: { borderRadius: [3, 3, 0, 0], color: barGradient(gradient[0], gradient[1]) },
        emphasis: { itemStyle: { color: gradient[0] } },
      },
    ],
  }
}

/** 复习次数柱状图。 */
export function reviewBarOption(days: DailyCount[], dark: boolean): EChartsOption {
  return dailyBarOption(days, dark, ' 次复习', REVIEW_BAR_GRADIENT)
}

/** 新增卡片柱状图。 */
export function addedBarOption(days: DailyCount[], dark: boolean): EChartsOption {
  return dailyBarOption(days, dark, ' 张卡片', ADDED_BAR_GRADIENT)
}