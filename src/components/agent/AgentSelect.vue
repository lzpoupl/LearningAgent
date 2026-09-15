<template>
  <el-select class="agent-select" :disabled="disabled" :model-value="selectValue" :placeholder="placeholder"
    :popper-class="popperClass" :size="size" @update:model-value="handleSelect">
    <template v-if="selectedAgentInfo" #prefix>
      <AgentIcon :color="selectedAgentInfo.color" :icon="selectedAgentInfo.icon" :size="18" />
    </template>
    <el-option v-for="agent in agents" :key="agent.id" :label="agent.name" :value="agent.id">
      <span class="agent-option-label">
        <AgentIcon :color="agent.color" :icon="agent.icon" :size="optionIconSize" />
        {{ agent.name }}
      </span>
      <span v-if="showDescription" class="agent-option-value">{{ agent.description }}</span>
    </el-option>
  </el-select>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import AgentIcon from './AgentIcon.vue'
import type { AgentInfo, AgentType } from '../../types/chat'

const props = withDefaults(defineProps<{
  /** 未选择任何助手时传入 0，此时展示占位文案而不是 0。 */
  modelValue: AgentType | 0
  agents: AgentInfo[]
  placeholder?: string
  size?: 'large' | 'default' | 'small'
  disabled?: boolean
  /** 在选项右侧展示助手描述。 */
  showDescription?: boolean
}>(), {
  placeholder: '请选择学习助手',
  size: 'default',
  disabled: false,
  showDescription: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: AgentType]
}>()

// 0 代表未选择，交给占位文案渲染，避免下拉框直接显示 0
const selectValue = computed(() => props.modelValue || undefined)
const selectedAgentInfo = computed(() => props.agents.find(agent => agent.id === props.modelValue) ?? null)
const optionIconSize = computed(() => (props.size === 'small' ? 18 : 20))
const popperClass = computed(() => (props.showDescription ? 'agent-select-popper is-detailed' : 'agent-select-popper'))

function handleSelect(value: AgentType) {
  if (value) {
    emit('update:modelValue', value)
  }
}
</script>

<style scoped>
.agent-select {
  min-width: 0;
}

.agent-option-label {
  display: inline-flex;
  float: left;
  align-items: center;
  gap: 8px;
}

.agent-option-value {
  float: right;
  color: #8492a6;
  font-size: 13px;
}

/* 下拉选项留出图标与右对齐描述的空间 */
:global(.agent-select-popper.is-detailed .el-select-dropdown__item) {
  height: auto;
  min-height: 40px;
  padding: 8px 16px;
  line-height: 1.5;
}
</style>
