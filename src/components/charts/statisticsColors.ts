import type { CardCategory } from '../../types/statistics'

/** 卡片分类配色：统一取自应用蓝色主题（--learning-primary: #287df5）的冷色阶。 */
export const CATEGORY_COLORS: Record<CardCategory, string> = {
  new: '#1d4ed8',
  learning: '#5b9cf8',
  review: '#287df5',
  relearning: '#0f7fb8',
}

/** 复习次数条形图的主色：主题主蓝。 */
export const REVIEW_BAR_COLOR = '#287df5'

/** 新增卡片条形图的主色：比主蓝浅一档，与复习图区分。 */
export const ADDED_BAR_COLOR = '#5b9cf8'