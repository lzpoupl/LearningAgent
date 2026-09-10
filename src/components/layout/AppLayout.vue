<template>
  <el-splitter class="app-layout">
    <el-splitter-panel class="sidebar-panel" :size="SIDEBAR_WIDTH" :resizable="false">
      <Sidebar :active-view="activeView" @navigate="emit('navigate', $event)" />
    </el-splitter-panel>

    <el-splitter-panel class="main-panel" :resizable="false">
      <el-container class="main-layout">
        <el-main class="layout-main">
          <slot />
        </el-main>
      </el-container>
    </el-splitter-panel>
  </el-splitter>
</template>

<script setup lang="ts">
import Sidebar from './Sidebar.vue'
import type { AppView } from '../../types/navigation'

const SIDEBAR_WIDTH = 72

defineProps<{
  activeView: AppView
}>()

const emit = defineEmits<{
  navigate: [view: AppView]
}>()
</script>

<style scoped>
.app-layout,
.main-layout {
  width: 100%;
  height: 100%;
  min-height: 0;
}

.sidebar-panel {
  overflow: hidden;
}

.main-panel {
  min-width: 0;
  overflow: hidden;
}

.main-layout {
  min-width: 0;
  background: var(--learning-bg);
}

.layout-main {
  min-width: 0;
  min-height: 0;
  padding: 0;
  overflow: hidden;
  background: var(--learning-bg);
}
</style>
