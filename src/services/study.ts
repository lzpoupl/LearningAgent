import { invoke } from '@tauri-apps/api/core'

import type { StudyTask, TodayOverview } from '../types/study'

export function getTodayOverview(): Promise<TodayOverview> {
  return invoke<TodayOverview>('study_get_today_overview')
}

export function updateStudyTask(taskId: string, done: boolean): Promise<StudyTask> {
  return invoke<StudyTask>('study_update_task', { taskId, done })
}

