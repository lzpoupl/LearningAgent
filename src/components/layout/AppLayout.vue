<template>
  <el-container class="app-layout">
    <Sidebar
      :active-view="activeView"
      :collapsed="collapsed"
      :current-session="currentSession"
      :sessions="sessions"
      :show-history="showHistory"
      @navigate="emit('navigate', $event)"
      @new-session="emit('new-session')"
      @select-session="emit('select-session', $event)"
      @rename="handleRename"
      @delete="emit('delete', $event)"
      @update:collapsed="emit('update:collapsed', $event)"
    />

    <el-container class="main-layout">
      <el-header class="layout-header" height="58px">
        <Topbar />
      </el-header>

      <el-main class="layout-main">
        <slot />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import Sidebar from './Sidebar.vue'
import Topbar from './Topbar.vue'
import type { ChatSession } from '../../types/chat'
import type { AppView } from '../../types/navigation'

defineProps<{
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

function handleRename(sessionId: string, title: string) {
  emit('rename', sessionId, title)
}
</script>

<style scoped>
.app-layout,
.main-layout {
  width: 100%;
  height: 100%;
  min-height: 0;
}

.main-layout {
  min-width: 0;
  background: var(--learning-bg);
}

.layout-header {
  padding: 0;
  border-bottom: 1px solid var(--learning-border);
  background: rgba(255, 255, 255, 0.88);
}

.layout-main {
  min-width: 0;
  min-height: 0;
  padding: 0;
  overflow: hidden;
  background: var(--learning-bg);
}
</style>
