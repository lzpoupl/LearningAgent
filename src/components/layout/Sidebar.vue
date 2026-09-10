<template>
  <aside class="sidebar">
    <div class="brand-mark">AI</div>

    <el-menu class="navigation-menu" collapse :default-active="activeView" :collapse-transition="false"
      @select="handleNavigation">
      <el-menu-item v-for="item in navigationItems" :key="item.id" :index="item.id">
        <el-icon>
          <component :is="item.icon" />
        </el-icon>
        <template #title>{{ item.label }}</template>
      </el-menu-item>
    </el-menu>
  </aside>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import {
  ChatDotRound,
  Collection,
  DataAnalysis,
  Document,
  HomeFilled,
  Reading,
  Setting,
  User,
} from '@element-plus/icons-vue'

import type { AppView } from '../../types/navigation'

defineProps<{
  activeView: AppView
}>()

const emit = defineEmits<{
  navigate: [view: AppView]
}>()

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

function handleNavigation(index: string) {
  emit('navigate', index as AppView)
}
</script>

<style scoped>
.sidebar {
  position: relative;
  display: flex;
  width: 100%;
  height: 100%;
  flex-direction: column;
  align-items: center;
  min-height: 0;
  overflow: hidden;
  border-right: 1px solid var(--learning-border);
  background: rgba(255, 255, 255, 0.94);
}

.brand-mark {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  margin: 14px 0 12px;
  border-radius: 10px;
  background: linear-gradient(135deg, #438fff, #5c6df5);
  box-shadow: 0 5px 15px rgba(50, 120, 240, 0.25);
  color: #fff;
  font-size: 12px;
  font-weight: 700;
}

.navigation-menu {
  width: 100%;
  flex: 0 0 auto;
  border-right: 0;
  background: transparent;
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
</style>
