import type { Card } from '../types/anki'

/** Mock 内部用的牌组记录（cardCount/subdeckCount 在读取时动态计算）。 */
export interface MockDeck {
  path: string
  name: string
  createdAt: string
}

const now = new Date().toISOString()
const hoursLater = (hours: number) => new Date(Date.now() + hours * 3_600_000).toISOString()
const hoursBefore = (hours: number) => new Date(Date.now() - hours * 3_600_000).toISOString()

export const seedDecks: MockDeck[] = [
  { path: '/数学', name: '数学', createdAt: now },
  { path: '/数学/微积分', name: '微积分', createdAt: now },
  { path: '/数学/线性代数', name: '线性代数', createdAt: now },
  { path: '/英语', name: '英语', createdAt: now },
  { path: '/英语/考研词汇', name: '考研词汇', createdAt: now },
  { path: '/物理', name: '物理', createdAt: now }
]

export const seedCards: Card[] = [
  {
    id: '1',
    deckPath: '/数学/微积分',
    front: '求不定积分 $\\int x^2\\,dx$',
    back: '$\\dfrac{x^3}{3}+C$',
    state: 'review',
    dueAt: hoursLater(2),
    createdAt: now,
    updatedAt: now,
  },
  {
    id: '2',
    deckPath: '/数学/微积分',
    front: '求导数 $\\dfrac{d}{dx}\\sin x$',
    back: '$\\cos x$',
    state: 'learning',
    dueAt: hoursLater(1),
    createdAt: now,
    updatedAt: now,
  },
  {
    id: '3',
    deckPath: '/数学/线性代数',
    front: '什么是矩阵的秩？',
    back: '矩阵中线性无关的行（或列）的最大数目。',
    state: 'new',
    dueAt: null,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: '4',
    deckPath: '/英语/考研词汇',
    front: '**abandon**',
    back: 'v. 放弃；抛弃',
    state: 'review',
    dueAt: hoursLater(24),
    createdAt: now,
    updatedAt: now,
  },
  {
    id: '5',
    deckPath: '/物理',
    front: '$E=m c^2$',
    back: '能量与质量的关系',
    state: 'review',
    dueAt: hoursBefore(24),
    createdAt: now,
    updatedAt: now,
  },
]
