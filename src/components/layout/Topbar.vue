<template>
  <div class="topbar">
    <el-input
      v-model="searchTerm"
      class="global-search"
      clearable
      placeholder="搜索学习资产、卡片、文件..."
      size="small"
    >
      <template #prefix>
        <el-icon><Search /></el-icon>
      </template>
    </el-input>

    <div class="topbar-actions">
      <el-button circle plain size="small" title="通知">
        <el-icon><Bell /></el-icon>
      </el-button>
      <el-button circle plain size="small" title="帮助">
        <el-icon><QuestionFilled /></el-icon>
      </el-button>
      <el-avatar :size="30">W</el-avatar>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { Bell, QuestionFilled, Search } from '@element-plus/icons-vue'

const searchTerm = ref('')

function focusSearch() {
  const input = document.querySelector<HTMLInputElement>('.global-search input')
  input?.focus()
}

function handleShortcut(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault()
    focusSearch()
  }
}

onMounted(() => window.addEventListener('keydown', handleShortcut))
onBeforeUnmount(() => window.removeEventListener('keydown', handleShortcut))
</script>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  gap: 18px;
  width: 100%;
  height: 100%;
  padding: 0 24px;
}

.global-search {
  width: min(390px, 48vw);
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-left: auto;
}

@media (max-width: 620px) {
  .topbar {
    padding: 0 14px;
  }

  .global-search {
    width: min(230px, 58vw);
  }

  .topbar-actions :deep(.el-button:nth-child(2)) {
    display: none;
  }
}
</style>
