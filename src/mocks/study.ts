import type { StudyTask, TodayOverview } from '../types/study'

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

function cloneTask(task: StudyTask): StudyTask {
  return { ...task }
}

/** 内存版学习计划后端；卡片相关统计由 mocks/statistics.ts 提供。 */
export class StudyMock {
  private tasks = seedTasks.map(cloneTask)

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'study_get_today_overview':
        return this.getTodayOverview()
      case 'study_update_task':
        return this.updateTask(String(payload.taskId ?? ''), Boolean(payload.done))
      default:
        return undefined
    }
  }

  private getTodayOverview(): TodayOverview {
    return {
      date: new Date().toISOString(),
      tasks: this.tasks.map(cloneTask),
      studyMinutes: 155,
      targetMinutes: 240,
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