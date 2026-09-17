import type { CardCategory } from '../../types/statistics'

/** 卡片分类配色：统一取自应用蓝色主题（--learning-primary: #287df5）的冷色阶。 */
export const CATEGORY_COLORS: Record<CardCategory, string> = {
  new: '#1d4ed8',
  learning: '#5b9cf8',
  review: '#287df5',
  relearning: '#0f7fb8',
}

/** 复习次数柱状图的渐变色（上浅下深）。 */
export const REVIEW_BAR_GRADIENT: [string, string] = ['#5b9cf8', '#287df5']

/** 新增卡片柱状图的渐变色（上浅下深）。 */
export const ADDED_BAR_GRADIENT: [string, string] = ['#8ab8fb', '#5b9cf8']

/** 今日环形图：已完成段。 */
export const TODAY_DONE_COLOR = '#287df5'

/** 今日环形图：待完成段。 */
export const TODAY_PENDING_COLOR = '#9db2cf'