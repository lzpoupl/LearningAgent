<template>
  <aside class="sidebar" :class="{ 'is-collapsed': collapsed }">
    <div class="sidebar-brand">
      <div class="brand-mark">AI</div>

      <div v-if="!collapsed" class="brand-copy">
        <strong>AI 学习助手</strong>
        <span>Personal Learning OS</span>
      </div>

      <el-button
        class="collapse-button"
        circle
        text
        :title="collapsed ? '展开侧边栏' : '收起侧边栏'"
        @click="emit('update:collapsed', !collapsed)"
      >
        <el-icon><Expand v-if="collapsed" /><Fold v-else /></el-icon>
      </el-button>
    </div>

    <el-menu
      ref="menuRef"
      class="navigation-menu"
      :collapse="collapsed"
      :default-active="activeView"
      :collapse-transition="false"
      @select="handleNavigation"
    >
      <el-menu-item v-for="item in navigationItems" :key="item.id" :index="item.id">
        <el-icon><component :is="item.icon" /></el-icon>
        <template #title>{{ item.label }}</template>
      </el-menu-item>
    </el-menu>

    <section v-if="showHistory && !collapsed" class="history-panel">
      <div class="history-heading">
        <span>最近会话</span>
        <el-button link type="primary" @click="emit('new-session')">新建</el-button>
      </div>

      <el-scrollbar class="session-scroll">
        <el-empty v-if="sessions.length === 0" :image-size="52" description="暂无会话" />

        <div v-for="session in sessions" :key="session.id" class="session-row">
          <el-button
            class="session-button"
            :class="{ active: currentSession === session.id }"
            text
            @click="emit('select-session', session.id)"
          >
            <span>{{ session.title }}</span>
          </el-button>

          <el-dropdown trigger="click" @command="handleSessionCommand(session.id, $event)">
            <el-button class="session-more" text circle title="会话操作">
              <el-icon><MoreFilled /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="rename">
                  <el-icon><Edit /></el-icon>
                  重命名
                </el-dropdown-item>
                <el-dropdown-item command="delete" divided>
                  <el-icon><Delete /></el-icon>
                  删除会话
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-scrollbar>
    </section>

    <div class="sidebar-user">
      <el-avatar :size="32">W</el-avatar>
      <div v-if="!collapsed" class="sidebar-user-copy">
        <strong>Wannamai</strong>
        <span>考研学习者</span>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import {
  ChatDotRound,
  Collection,
  DataAnalysis,
  Delete,
  Document,
  Edit,
  Expand,
  Fold,
  HomeFilled,
  MoreFilled,
  Reading,
  Setting,
  User,
} from '@element-plus/icons-vue'

import type { ChatSession } from '../../types/chat'
import type { AppView } from '../../types/navigation'

const props = defineProps<{
  activeView: AppView
  collapsed: boolean
  currentSession: string
  sessions: ChatSession[]
  showHistory: boolean
}>()

const emit = defineEmits<{
  navigate: [view: AppView]
  'new-session': []
  'select-session': [sessionId: string]
  rename: [sessionId: string, title: string]
  delete: [sessionId: string]
  'update:collapsed': [collapsed: boolean]
}>()

const menuRef = ref<{ updateActiveIndex: (index: string) => void }>()

const navigationItems: Array<{ id: AppView; label: string; icon: Component }> = [
  { id: 'home', label: '首页', icon: HomeFilled },
  { id: 'agents', label: 'Agent 管理', icon: User },
  { id: 'chat', label: 'Agent 会话', icon: ChatDotRound },
  { id: 'anki', label: 'Anki 管理', icon: Collection },
  { id: 'review', label: 'Anki 复习', icon: Reading },
  { id: 'assets', label: '学习资料', icon: Document },
  { id: 'stats', label: '学习统计', icon: DataAnalysis },
  { id: 'settings', label: '设置', icon: Setting },
]

watch(
  () => props.activeView,
  view => menuRef.value?.updateActiveIndex(view),
  { immediate: true },
)

function handleNavigation(index: string) {
  emit('navigate', index as AppView)
}

async function handleSessionCommand(sessionId: string, command: string | number | object) {
  if (command === 'rename') {
    await renameSession(sessionId)
  }

  if (command === 'delete') {
    await deleteSession(sessionId)
  }
}

async function renameSession(sessionId: string) {
  const session = props.sessions.find(item => item.id === sessionId)

  if (!session) {
    return
  }

  try {
    const result = await ElMessageBox.prompt('请输入新的会话名称', '重命名会话', {
      inputValue: session.title,
      confirmButtonText: '保存',
      cancelButtonText: '取消',
      inputValidator: value => Boolean(value?.trim()) || '会话名称不能为空',
    })

    emit('rename', sessionId, result.value)
  } catch {
    // 用户取消操作时不需要反馈。
  }
}

async function deleteSession(sessionId: string) {
  try {
    await ElMessageBox.confirm('删除后无法恢复该会话记录。', '删除会话', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消',
    })
    emit('delete', sessionId)
  } catch {
    // 用户取消操作时不需要反馈。
  }
}
</script>

<style scoped>
.sidebar {
  position: relative;
  display: flex;
  flex: 0 0 232px;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  border-right: 1px solid var(--learning-border);
  background: rgba(255, 255, 255, 0.94);
  transition: flex-basis 180ms ease;
}

.sidebar.is-collapsed {
  flex-basis: 72px;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 72px;
  padding: 14px 12px 12px;
}

.brand-mark {
  display: grid;
  flex: 0 0 34px;
  place-items: center;
  width: 34px;
  height: 34px;
  border-radius: 10px;
  background: linear-gradient(135deg, #438fff, #5c6df5);
  box-shadow: 0 5px 15px rgba(50, 120, 240, 0.25);
  color: #fff;
  font-size: 12px;
  font-weight: 700;
}

.brand-copy {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 2px;
}

.brand-copy strong {
  color: var(--learning-text);
  font-size: 13px;
}

.brand-copy span {
  overflow: hidden;
  color: var(--learning-text-muted);
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.collapse-button {
  flex: 0 0 auto;
  color: var(--learning-text-muted);
}

.sidebar.is-collapsed .sidebar-brand {
  justify-content: center;
  padding-right: 8px;
  padding-left: 8px;
}

.sidebar.is-collapsed .collapse-button {
  position: absolute;
  top: 54px;
  left: 51px;
  z-index: 2;
  width: 22px;
  height: 22px;
  border: 1px solid var(--learning-border);
  background: #fff;
}

.navigation-menu {
  width: 100%;
  flex: 0 0 auto;
  border-right: 0;
  background: transparent;
}

.navigation-menu:not(.el-menu--collapse) {
  padding: 0 9px;
}

.navigation-menu :deep(.el-menu-item) {
  height: 42px;
  margin: 3px 0;
  border-radius: 9px;
  color: var(--learning-text-secondary);
  font-size: 12px;
}

.navigation-menu :deep(.el-menu-item:hover) {
  background: var(--learning-surface-muted);
  color: var(--learning-text);
}

.navigation-menu :deep(.el-menu-item.is-active) {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
  font-weight: 600;
}

.navigation-menu :deep(.el-menu-item .el-icon) {
  color: var(--learning-text-muted);
  font-size: 17px;
}

.navigation-menu :deep(.el-menu-item.is-active .el-icon) {
  color: var(--el-color-primary);
}

.history-panel {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  margin: 16px 12px 0;
  padding-top: 14px;
  border-top: 1px solid var(--learning-border);
}

.history-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px 8px;
  color: var(--learning-text-muted);
  font-size: 11px;
}

.history-heading :deep(.el-button) {
  font-size: 11px;
}

.session-scroll {
  min-height: 0;
  flex: 1;
}

.session-scroll :deep(.el-empty) {
  padding: 18px 0;
}

.session-scroll :deep(.el-empty__description p) {
  color: var(--learning-text-muted);
  font-size: 11px;
}

.session-row {
  display: flex;
  align-items: center;
  gap: 2px;
  margin: 2px 0;
}

.session-button {
  min-width: 0;
  flex: 1;
  justify-content: flex-start;
  overflow: hidden;
  padding: 8px 9px;
  border-radius: 8px;
  color: var(--learning-text-secondary);
  font-size: 11px;
  text-align: left;
}

.session-button span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.session-button:hover,
.session-button.active {
  background: var(--learning-surface-muted);
  color: var(--learning-text);
}

.session-more {
  width: 28px;
  height: 28px;
  color: var(--learning-text-muted);
}

.sidebar-user {
  display: flex;
  align-items: center;
  gap: 9px;
  min-height: 61px;
  margin-top: auto;
  padding: 12px;
  border-top: 1px solid var(--learning-border);
}

.sidebar-user-copy {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
}

.sidebar-user-copy strong {
  color: var(--learning-text);
  font-size: 11px;
}

.sidebar-user-copy span {
  color: var(--learning-text-muted);
  font-size: 9px;
}

.sidebar.is-collapsed .sidebar-user {
  justify-content: center;
  padding-right: 8px;
  padding-left: 8px;
}

@media (max-width: 700px) {
  .sidebar {
    flex-basis: 64px;
  }

  .sidebar:not(.is-collapsed) {
    flex-basis: 64px;
  }

  .sidebar:not(.is-collapsed) .brand-copy,
  .sidebar:not(.is-collapsed) .sidebar-user-copy,
  .sidebar:not(.is-collapsed) .history-panel,
  .sidebar:not(.is-collapsed) .collapse-button {
    display: none;
  }

  .sidebar:not(.is-collapsed) .sidebar-brand,
  .sidebar:not(.is-collapsed) .sidebar-user {
    justify-content: center;
  }

  .navigation-menu:not(.el-menu--collapse) {
    padding: 0;
  }
}
</style>
