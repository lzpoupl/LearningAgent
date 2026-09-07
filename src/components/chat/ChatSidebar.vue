<template>
  <aside
    class="sidebar"
    :style="{ width: sidebarWidth ? `${sidebarWidth}px` : undefined }"
  >

    <!-- Logo -->
    <div class="logo">
      <div class="logo-icon">
        L
      </div>

      <span>
        LearningAgent
      </span>
    </div>

    <!-- 新建会话 -->
    <button
      class="new-chat"
      @click="$emit('new-chat')"
    >
      <span class="plus">＋</span>
      新建会话
    </button>

    <!-- 最近会话 -->
    <div class="section-title">
      最近会话
    </div>

    <div class="session-list">

      <div
        v-for="session in sessions"
        :key="session.id"
        class="session-wrapper"
      >

        <div
          class="session-item"
          :class="{
            active: currentSession === session.id
          }"
          @click="$emit('select-session', session.id)"
        >

          <span class="session-title">
            {{ session.title }}
          </span>

          <!-- 操作按钮 -->
          <button
            class="more-button"
            @click.stop="toggleMenu(session.id)"
          >
            ⋯
          </button>

        </div>

        <!-- 操作菜单 -->
        <div
          v-if="openMenu === session.id"
          class="session-menu"
        >
          <button
            @click="renameSession(session.id)"
          >
            ✎ 重命名
          </button>

          <button
            class="delete-button"
            @click="deleteSession(session.id)"
          >
            🗑 删除
          </button>
        </div>

      </div>

      <!-- 没有会话 -->
      <div
        v-if="sessions.length === 0"
        class="empty-session"
      >
        暂无会话
      </div>

    </div>

    <div
      class="sidebar-resizer"
      title="拖动调整侧栏宽度"
      @pointerdown="startResize"
    />

  </aside>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { ChatSession } from '../../types/chat'

const props = defineProps<{
  currentSession: string
  sessions: ChatSession[]
}>()

const emit = defineEmits<{
  'new-chat': []
  'select-session': [sessionId: string]
  rename: [sessionId: string, title: string]
  delete: [sessionId: string]
}>()

const openMenu = ref<string | null>(null)
const sidebarWidth = ref<number | null>(null)

function startResize(event: PointerEvent) {
  event.preventDefault()

  window.addEventListener('pointermove', resizeSidebar)
  window.addEventListener('pointerup', stopResize, { once: true })
}

function resizeSidebar(event: PointerEvent) {
  sidebarWidth.value = Math.min(360, Math.max(180, event.clientX))
}

function stopResize() {
  window.removeEventListener('pointermove', resizeSidebar)
}

function toggleMenu(sessionId: string) {
  if (openMenu.value === sessionId) {
    openMenu.value = null
  } else {
    openMenu.value = sessionId
  }
}

function renameSession(sessionId: string) {
  openMenu.value = null

  const session = props.sessions.find(
    item => item.id === sessionId
  )

  if (!session) {
    return
  }

  const newTitle = window.prompt(
    '请输入新的会话名称',
    session.title
  )

  if (!newTitle?.trim()) {
    return
  }

  emit(
    'rename',
    sessionId,
    newTitle.trim()
  )
}

function deleteSession(sessionId: string) {
  openMenu.value = null

  const confirmed = window.confirm(
    '确定要删除这个会话吗？'
  )

  if (!confirmed) {
    return
  }

  emit('delete', sessionId)
}
</script>

<style scoped>
.sidebar {
  position: relative;

  width: clamp(200px, 22vw, 300px);
  height: 100vh;

  flex-shrink: 0;

  padding: 20px 14px;

  background: #f7f7f8;

  border-right: 1px solid #e5e5e5;
}

.sidebar-resizer {
  position: absolute;
  z-index: 5;
  top: 0;
  right: -4px;
  bottom: 0;
  width: 8px;
  cursor: col-resize;
}

.sidebar-resizer:hover {
  background: rgba(17, 17, 17, 0.08);
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;

  padding: 4px 8px 22px;

  font-size: 17px;
  font-weight: 600;
}

.logo-icon {
  width: 32px;
  height: 32px;

  display: flex;
  align-items: center;
  justify-content: center;

  border-radius: 8px;

  background: #111111;
  color: #ffffff;
}

.new-chat {
  width: 100%;
  height: 42px;

  display: flex;
  align-items: center;
  justify-content: center;

  gap: 6px;

  border: 1px solid #dddddd;
  border-radius: 9px;

  background: #ffffff;

  cursor: pointer;

  font-size: 14px;
}

.new-chat:hover {
  background: #eeeeee;
}

.plus {
  font-size: 18px;
}

.section-title {
  margin: 22px 8px 8px;

  font-size: 12px;
  color: #999999;
}

.session-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.session-wrapper {
  position: relative;
}

.session-item {
  min-height: 40px;

  display: flex;
  align-items: center;

  padding: 0 8px 0 11px;

  border-radius: 8px;

  cursor: pointer;
}

.session-item:hover,
.session-item.active {
  background: #e9e9eb;
}

.session-title {
  flex: 1;

  min-width: 0;

  overflow: hidden;

  white-space: nowrap;

  text-overflow: ellipsis;

  font-size: 13px;
}

.more-button {
  width: 28px;
  height: 28px;

  flex-shrink: 0;

  display: flex;
  align-items: center;
  justify-content: center;

  border: none;
  border-radius: 6px;

  background: transparent;

  color: #777777;

  cursor: pointer;

  font-size: 17px;

  opacity: 0;
}

.session-item:hover .more-button,
.more-button:focus {
  opacity: 1;
}

.more-button:hover {
  background: #dcdcdc;
}

.session-menu {
  position: absolute;

  top: 38px;
  right: 4px;

  z-index: 10;

  width: 130px;

  padding: 5px;

  border: 1px solid #e5e5e5;
  border-radius: 9px;

  background: #ffffff;

  box-shadow:
    0 5px 20px rgba(0, 0, 0, 0.12);
}

.session-menu button {
  width: 100%;
  height: 34px;

  display: flex;
  align-items: center;

  padding: 0 10px;

  border: none;
  border-radius: 6px;

  background: transparent;

  cursor: pointer;

  font-size: 13px;

  text-align: left;
}

.session-menu button:hover {
  background: #f3f3f3;
}

.session-menu .delete-button {
  color: #d33;
}

.empty-session {
  padding: 20px 10px;

  text-align: center;

  color: #aaa;

  font-size: 13px;
}
</style>