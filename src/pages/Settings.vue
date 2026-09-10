<template>
  <main class="page-frame settings-page">
    <PageHeader
      description="学习目标、主题、隐私和 Agent 偏好"
      eyebrow="系统 / SETTINGS"
      title="用户设置"
    />

    <el-card class="settings-card" shadow="never">
      <el-tabs v-model="activeTab" tab-position="left" class="settings-tabs">
        <el-tab-pane label="学习偏好" name="preference">
          <section class="settings-section">
            <h2>学习目标</h2>
            <el-form :model="settings" label-position="top">
              <div class="settings-grid">
                <el-form-item label="学习目标">
                  <el-select v-model="settings.goal">
                    <el-option label="考研" value="考研" />
                    <el-option label="课程学习" value="课程学习" />
                    <el-option label="长期学习" value="长期学习" />
                  </el-select>
                </el-form-item>
                <el-form-item label="每天学习时长">
                  <el-input-number v-model="settings.dailyHours" :max="12" :min="1" />
                  <span class="input-suffix">小时</span>
                </el-form-item>
                <el-form-item label="每日提醒时间">
                  <el-time-select v-model="settings.reminderTime" end="22:00" start="07:00" step="00:30" />
                </el-form-item>
              </div>
            </el-form>
          </section>

          <section class="settings-section">
            <h2>学习科目</h2>
            <div class="subject-list">
              <el-tag v-for="subject in subjects" :key="subject" closable effect="light" @close="removeSubject(subject)">
                {{ subject }}
              </el-tag>
              <el-input
                v-if="addingSubject"
                ref="subjectInput"
                v-model="newSubject"
                class="subject-input"
                size="small"
                @blur="finishAddingSubject"
                @keydown.enter.prevent="finishAddingSubject"
              />
              <el-button v-else link type="primary" @click="addingSubject = true">
                <el-icon><Plus /></el-icon>
                添加学科
              </el-button>
            </div>
          </section>
        </el-tab-pane>

        <el-tab-pane label="外观" name="appearance">
          <section class="settings-section">
            <h2>主题风格</h2>
            <el-radio-group v-model="settings.theme" class="theme-grid">
              <el-radio-button label="light">浅色</el-radio-button>
              <el-radio-button label="dark">深色</el-radio-button>
              <el-radio-button label="mountain">自定义</el-radio-button>
            </el-radio-group>
            <p class="section-hint">主题切换入口已预留，当前界面使用浅色学习工作台。</p>
          </section>
        </el-tab-pane>

        <el-tab-pane label="高级设置" name="advanced">
          <section class="settings-section">
            <h2>学习行为</h2>
            <div class="setting-row">
              <div><strong>启用 Anki 间隔重复</strong><span>根据记忆强度自动安排复习</span></div>
              <el-switch v-model="settings.enableSpacedRepetition" />
            </div>
            <div class="setting-row">
              <div><strong>首页显示学习统计</strong><span>在首页显示每日学习数据</span></div>
              <el-switch v-model="settings.showStatistics" />
            </div>
            <div class="setting-row">
              <div><strong>对话中自动引用学习资料</strong><span>Agent 根据上下文自动检索相关资产</span></div>
              <el-switch v-model="settings.autoReference" />
            </div>
          </section>
        </el-tab-pane>
      </el-tabs>

      <div class="settings-footer">
        <el-button type="primary" @click="saveSettings">保存设置</el-button>
      </div>
    </el-card>
  </main>
</template>

<script setup lang="ts">
import { nextTick, reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'

import PageHeader from '../components/common/PageHeader.vue'

const activeTab = ref('preference')
const subjects = ref(['数学', '英语', '计算机'])
const addingSubject = ref(false)
const newSubject = ref('')
const subjectInput = ref<{ focus: () => void } | null>(null)

const settings = reactive({
  goal: '考研',
  dailyHours: 4,
  reminderTime: '09:00',
  theme: 'light',
  enableSpacedRepetition: true,
  showStatistics: true,
  autoReference: true,
})

function removeSubject(subject: string) {
  subjects.value = subjects.value.filter(item => item !== subject)
}

function finishAddingSubject() {
  const subject = newSubject.value.trim()

  if (subject && !subjects.value.includes(subject)) {
    subjects.value.push(subject)
  }

  newSubject.value = ''
  addingSubject.value = false
}

function saveSettings() {
  ElMessage.success('设置已保存')
}

watch(addingSubject, visible => {
  if (visible) {
    void nextTick(() => subjectInput.value?.focus())
  }
})
</script>

<style scoped>
.page-frame {
  width: min(1100px, 100%);
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  padding: 30px 28px 48px;
  overflow-y: auto;
}

.settings-card {
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
}

.settings-tabs {
  min-height: 430px;
}

.settings-tabs :deep(.el-tabs__header) {
  width: 150px;
}

.settings-tabs :deep(.el-tabs__item) {
  justify-content: flex-start;
  color: var(--learning-text-secondary);
  font-size: 12px;
}

.settings-tabs :deep(.el-tabs__item.is-active) {
  color: var(--el-color-primary);
}

.settings-tabs :deep(.el-tabs__content) {
  padding: 2px 10px 10px 26px;
}

.settings-section + .settings-section {
  margin-top: 30px;
  padding-top: 24px;
  border-top: 1px solid var(--learning-border);
}

h2 {
  margin: 0 0 18px;
  color: var(--learning-text);
  font-size: 15px;
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}

.settings-grid :deep(.el-select),
.settings-grid :deep(.el-time-editor),
.settings-grid :deep(.el-input-number) {
  width: 100%;
}

.input-suffix {
  margin-left: 8px;
  color: var(--learning-text-secondary);
  font-size: 12px;
}

.subject-list {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.subject-input {
  width: 120px;
}

.theme-grid {
  display: flex;
}

.section-hint {
  margin-top: 14px;
  color: var(--learning-text-muted);
  font-size: 11px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-height: 62px;
  border-bottom: 1px solid var(--learning-border);
}

.setting-row:last-child {
  border-bottom: 0;
}

.setting-row > div {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-row strong {
  color: var(--learning-text);
  font-size: 12px;
}

.setting-row span {
  color: var(--learning-text-muted);
  font-size: 10px;
}

.settings-footer {
  display: flex;
  justify-content: flex-end;
  margin-top: 18px;
  padding-top: 16px;
  border-top: 1px solid var(--learning-border);
}

@media (max-width: 700px) {
  .page-frame {
    padding: 22px 14px 36px;
  }

  .settings-tabs :deep(.el-tabs__header) {
    width: 112px;
  }

  .settings-tabs :deep(.el-tabs__content) {
    padding-left: 14px;
  }

  .settings-grid {
    grid-template-columns: 1fr;
    gap: 2px;
  }
}

@media (max-width: 480px) {
  .settings-tabs {
    min-height: 500px;
  }

  .settings-tabs :deep(.el-tabs__header) {
    float: none;
    width: auto;
  }

  .settings-tabs :deep(.el-tabs__nav-wrap) {
    margin-bottom: 14px;
  }

  .settings-tabs :deep(.el-tabs__nav) {
    display: flex;
    overflow-x: auto;
  }

  .settings-tabs :deep(.el-tabs__content) {
    padding: 0;
  }
}
</style>
