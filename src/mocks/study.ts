import type {
  StudyStatistics,
  StudyTask,
  TodayOverview,
} from '../types/study'

const seedTasks: StudyTask[] = [
  {
    id: 'math-calculus-practice',
    title: '数学：高数习题练习',
    detail: '极限与导数 · 2/3',
    minutes: 90,
    done: true,
  },
  {
    id: 'english-vocabulary-review',
    title: '英语：背单词 + 例句翻译',
    detail: '考研英语词汇',
    minutes: 60,
    done: true,
  },
  {
    id: 'operating-system-reading',
    title: '操作系统：阅读课件',
    detail: '进程管理',
    minutes: 90,
    done: false,
  },
  {
    id: 'anki-review',
    title: 'Anki 复习',
    detail: '新卡与到期复习卡片',
    minutes: 60,
    done: false,
  },
]

const seedStatistics: StudyStatistics = {
  today: {
    minutes: 155,
    changePercent: 18,
  },
  week: {
    minutes: 680,
    targetMinutes: 960,
    progressPercent: 71,
  },
  streakDays: 12,
  masteredKnowledgePoints: 128,
  masteredThisWeek: 9,
  weeklyStudy: [
    { day: '周三', minutes: 80, today: false },
    { day: '周四', minutes: 95, today: false },
    { day: '周五', minutes: 60, today: false },
    { day: '周六', minutes: 120, today: false },
    { day: '周日', minutes: 80, today: false },
    { day: '周一', minutes: 90, today: false },
    { day: '今天', minutes: 155, today: true },
  ],
  subjects: [
    { id: 'math', name: '数学', icon: '∑', theme: 'math', minutes: 370, percentage: 54 },
    { id: 'english', name: '英语', icon: 'A', theme: 'english', minutes: 215, percentage: 32 },
    { id: 'anki', name: 'Anki 复习', icon: '✦', theme: 'anki', minutes: 95, percentage: 14 },
  ],
  insight: {
    title: '保持这个节奏',
    description: '你已经连续学习 5 天。平均每天投入 1 小时 37 分，距离本周目标还差 4 小时 40 分。',
    progressPercent: 71,
  },
}

function cloneTask(task: StudyTask): StudyTask {
  return { ...task }
}

function cloneStatistics(): StudyStatistics {
  return {
    ...seedStatistics,
    today: { ...seedStatistics.today },
    week: { ...seedStatistics.week },
    weeklyStudy: seedStatistics.weeklyStudy.map(day => ({ ...day })),
    subjects: seedStatistics.subjects.map(subject => ({ ...subject })),
    insight: { ...seedStatistics.insight },
  }
}

/** 内存版学习计划与统计后端。 */
export class StudyMock {
  private tasks = seedTasks.map(cloneTask)

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'study_get_today_overview':
        return this.getTodayOverview()
      case 'study_update_task':
        return this.updateTask(String(payload.taskId ?? ''), Boolean(payload.done))
      case 'study_get_statistics':
        return cloneStatistics()
      default:
        return undefined
    }
  }

  private getTodayOverview(): TodayOverview {
    return {
      date: new Date().toISOString(),
      tasks: this.tasks.map(cloneTask),
      studyMinutes: seedStatistics.today.minutes,
      targetMinutes: 240,
      dueCardCount: 2,
      totalCardCount: 4,
    }
  }

  private updateTask(taskId: string, done: boolean): StudyTask {
    const task = this.tasks.find(item => item.id === taskId)
    if (!task) {
      throw new Error(`学习任务不存在: ${taskId}`)
    }
    task.done = done
    return cloneTask(task)
  }
}
