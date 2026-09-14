<template>
  <main class="page-frame agents-page">
    <PageHeader description="创建、配置和管理不同学科的 AI 学习助手" eyebrow="学习助手 / AGENTS" title="Agent 管理">
      <el-button type="primary" @click="openCreateDialog">
        <el-icon>
          <Plus />
        </el-icon>
        创建 Agent
      </el-button>
    </PageHeader>

    <div v-if="loading" class="state-message">正在加载 Agent...</div>
    <div v-else-if="errorMessage" class="state-message error-state" role="alert">{{ errorMessage }}</div>

    <el-row v-else :gutter="14">
      <el-col v-for="agent in agents" :key="agent.id" :lg="12" :md="12" :sm="24" :xl="12" :xs="24">
        <el-card class="agent-card" shadow="hover">
          <div class="agent-card-heading">
            <AgentIcon :color="agent.color" :icon="agent.icon" :size="46" />
            <div class="agent-card-title">
              <strong>{{ agent.name }}</strong>
              <span>{{ agent.builtin ? '系统 Agent' : '自定义 Agent' }}</span>
            </div>
            <el-switch
              :aria-label="`${agent.name}启用状态`"
              :model-value="agent.enabled"
              @change="toggleAgent(agent, $event)"
            />
          </div>

          <p class="agent-description">{{ agent.description }}</p>

          <div class="agent-tags">
            <el-tag v-for="tag in agent.capabilities" :key="tag" effect="plain" size="small">
              {{ tag }}
            </el-tag>
            <el-tag effect="plain" size="small" type="info">{{ agent.subject }}</el-tag>
          </div>

          <div class="agent-card-footer">
            <el-button link type="primary" @click="startAgent(agent)">开始对话</el-button>
            <el-button link @click="editAgent(agent)">配置 Agent</el-button>
            <el-button v-if="!agent.builtin" link type="danger" @click="removeAgent(agent)">删除</el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-card class="permission-card" shadow="never">
      <div>
        <strong>Agent 权限管理</strong>
        <p>按学习资产配置读取、编辑和写入权限。</p>
      </div>
      <el-button plain :loading="permissionLoading" @click="openPermissionDialog">权限设置</el-button>
    </el-card>

    <el-dialog v-model="dialogVisible" :title="editingAgentId ? '配置 Agent' : '创建 Agent'" width="min(520px, 92vw)">
      <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
        <el-form-item label="Agent 名称" prop="name">
          <el-input v-model="form.name" placeholder="例如：操作系统 Agent" />
        </el-form-item>
        <el-form-item label="Agent 图标">
          <div class="icon-picker">
            <button v-for="option in agentIconOptions" :key="option.key" class="icon-choice"
              :class="{ active: form.icon === option.key }" :title="option.label" type="button"
              @click="form.icon = option.key">
              <AgentIcon :color="form.color" :icon="option.key" :size="38" />
            </button>
          </div>
        </el-form-item>
        <el-form-item label="Agent 配色">
          <div class="color-picker">
            <button v-for="preset in agentColorPresets" :key="preset.key" class="color-choice"
              :class="{ active: form.color === preset.color }" :style="{ background: preset.color }"
              :title="preset.label" type="button" @click="form.color = preset.color" />
          </div>
        </el-form-item>
        <el-form-item label="所属学科" prop="subject">
          <el-input v-model="form.subject" placeholder="例如：计算机" />
        </el-form-item>
        <el-form-item label="职责描述" prop="description">
          <el-input v-model="form.description" :rows="3" type="textarea" />
        </el-form-item>
        <el-form-item label="能力标签">
          <el-input v-model="form.capabilities" placeholder="用逗号分隔，例如：课程知识, 习题解析" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button :loading="saving" type="primary" @click="saveAgent">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="permissionDialogVisible" title="Agent 权限管理" width="min(520px, 92vw)">
      <div class="permission-list">
        <div v-for="permission in permissions" :key="permission.key" class="permission-row">
          <div>
            <strong>{{ permission.label }}</strong>
            <span>{{ permission.description }}</span>
          </div>
           <el-switch v-model="permission.enabled" />
        </div>
      </div>
      <template #footer>
        <el-button :loading="permissionLoading" type="primary" @click="savePermissions">完成</el-button>
      </template>
    </el-dialog>
  </main>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'

import AgentIcon from '../components/agent/AgentIcon.vue'
import { agentIconOptions, defaultAgentIcon } from '../components/agent/agentIcons'
import { agentColorPresets, defaultAgentColor } from '../components/agent/agentColors'
import PageHeader from '../components/common/PageHeader.vue'
import {
  createAgent,
  deleteAgent,
  getAgentPermissions,
  listAgents,
  setAgentEnabled,
  updateAgent,
  updateAgentPermissions,
} from '../services/agent'
import type { AgentConfigInput, AgentInfo, AgentPermission, AgentType } from '../types/chat'

interface AgentForm extends Omit<AgentConfigInput, 'capabilities' | 'icon' | 'color'> {
  capabilities: string
  icon: string
  color: string
}

const emit = defineEmits<{
  'start-chat': [agent: AgentType]
}>()

const agents = ref<AgentInfo[]>([])
const permissions = ref<AgentPermission[]>([])
const loading = ref(false)
const errorMessage = ref('')
const permissionLoading = ref(false)

const dialogVisible = ref(false)
const permissionDialogVisible = ref(false)
const editingAgentId = ref<string | null>(null)
const saving = ref(false)
const formRef = ref<FormInstance>()
const form = reactive<AgentForm>({
  name: '',
  subject: '',
  description: '',
  capabilities: '',
  icon: defaultAgentIcon,
  color: defaultAgentColor,
})

const rules: FormRules<AgentForm> = {
  name: [{ required: true, message: '请输入 Agent 名称', trigger: 'blur' }],
  subject: [{ required: true, message: '请输入所属学科', trigger: 'blur' }],
  description: [{ required: true, message: '请输入职责描述', trigger: 'blur' }],
}

function resetForm() {
  form.name = ''
  form.subject = ''
  form.description = ''
  form.capabilities = ''
  form.icon = defaultAgentIcon
  form.color = defaultAgentColor
}

function openCreateDialog() {
  editingAgentId.value = null
  resetForm()
  dialogVisible.value = true
}

function editAgent(agent: AgentInfo) {
  editingAgentId.value = agent.id
  form.name = agent.name
  form.subject = agent.subject
  form.description = agent.description
  form.capabilities = agent.capabilities.join(', ')
  form.icon = agent.icon || defaultAgentIcon
  form.color = agent.color || defaultAgentColor
  dialogVisible.value = true
}

async function saveAgent() {
  if (!formRef.value || saving.value) {
    return
  }

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  saving.value = true
  const capabilities = form.capabilities
    .split(',')
    .map(item => item.trim())
    .filter(Boolean)

  try {
    if (editingAgentId.value) {
      const updatedAgent = await updateAgent(editingAgentId.value, {
        name: form.name.trim(),
        subject: form.subject.trim(),
        description: form.description.trim(),
        capabilities,
        icon: form.icon,
        color: form.color,
      })
      const index = agents.value.findIndex(item => item.id === editingAgentId.value)
      if (index !== -1) {
        agents.value[index] = updatedAgent
      }
      ElMessage.success('Agent 配置已更新')
    } else {
      const createdAgent = await createAgent({
        name: form.name.trim(),
        subject: form.subject.trim(),
        description: form.description.trim(),
        capabilities,
        icon: form.icon,
        color: form.color,
      })
      agents.value.push(createdAgent)
      ElMessage.success('自定义 Agent 已创建')
    }
    dialogVisible.value = false
  } catch (error) {
    console.error(error)
    ElMessage.error('Agent 保存失败，请重试。')
  } finally {
    saving.value = false
  }
}

async function toggleAgent(agent: AgentInfo, value: string | number | boolean) {
  const enabled = Boolean(value)
  if (enabled === agent.enabled) {
    return
  }

  try {
    const updatedAgent = await setAgentEnabled(agent.id, enabled)
    Object.assign(agent, updatedAgent)
  } catch (error) {
    console.error(error)
    ElMessage.error('Agent 状态更新失败，请重试。')
  }
}

function startAgent(agent: AgentInfo) {
  if (agent.enabled) {
    emit('start-chat', agent.id)
    return
  }

  ElMessage.info('请先启用这个 Agent。')
}

async function removeAgent(agent: AgentInfo) {
  if (agent.builtin || !window.confirm(`确定删除“${agent.name}”吗？`)) {
    return
  }

  try {
    await deleteAgent(agent.id)
    agents.value = agents.value.filter(item => item.id !== agent.id)
    ElMessage.success('Agent 已删除')
  } catch (error) {
    console.error(error)
    ElMessage.error('Agent 删除失败，请重试。')
  }
}

async function openPermissionDialog() {
  permissionDialogVisible.value = true
  permissionLoading.value = true

  try {
    permissions.value = await getAgentPermissions()
  } catch (error) {
    console.error(error)
    ElMessage.error('权限加载失败，请重试。')
  } finally {
    permissionLoading.value = false
  }
}

async function savePermissions() {
  permissionLoading.value = true
  try {
    permissions.value = await updateAgentPermissions(permissions.value)
    permissionDialogVisible.value = false
    ElMessage.success('Agent 权限已更新')
  } catch (error) {
    console.error(error)
    ElMessage.error('权限保存失败，请重试。')
  } finally {
    permissionLoading.value = false
  }
}

async function loadAgents() {
  loading.value = true
  errorMessage.value = ''
  try {
    agents.value = await listAgents()
  } catch (error) {
    console.error(error)
    errorMessage.value = 'Agent 加载失败，请稍后重试。'
  } finally {
    loading.value = false
  }
}

onMounted(loadAgents)
</script>

<style scoped>
.page-frame {
  width: min(1180px, 100%);
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  padding: 30px 28px 48px;
  overflow-y: auto;
}

.agent-card {
  height: 100%;
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
}

.agent-card-heading,
.agent-card-footer,
.permission-card,
.permission-row {
  display: flex;
  align-items: center;
}

.agent-card-heading {
  gap: 12px;
}

.icon-picker {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 8px;
  padding: 4px 2px;
  overflow-x: auto;
  scrollbar-width: thin;
}

.icon-choice {
  flex: 0 0 auto;
  padding: 3px;
  border: 1px solid transparent;
  border-radius: 50%;
  background: transparent;
  cursor: pointer;
  line-height: 0;
  transition: border-color 140ms ease, box-shadow 140ms ease;
}

.icon-choice:hover {
  border-color: #b7d0f8;
}

.icon-choice.active {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 0 2px rgba(40, 125, 245, 0.16);
}

.color-picker {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 10px;
  padding: 4px 2px;
  overflow-x: auto;
  scrollbar-width: thin;
}

.color-choice {
  flex: 0 0 auto;
  width: 30px;
  height: 30px;
  padding: 0;
  border: 2px solid transparent;
  border-radius: 50%;
  cursor: pointer;
  transition: border-color 140ms ease, box-shadow 140ms ease, transform 140ms ease;
}

.color-choice:hover {
  transform: scale(1.06);
}

.color-choice.active {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 0 2px rgba(40, 125, 245, 0.16);
}

.agent-card-title {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 3px;
}

.agent-card-title strong {
  color: var(--learning-text);
  font-size: 14px;
}

.agent-card-title span {
  color: var(--learning-text-muted);
  font-size: 10px;
}

.agent-description {
  min-height: 38px;
  margin: 18px 0 14px;
  color: var(--learning-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.agent-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.agent-card-footer {
  justify-content: flex-end;
  gap: 4px;
  margin-top: 16px;
  padding-top: 11px;
  border-top: 1px solid var(--learning-border);
}

.permission-card {
  justify-content: space-between;
  gap: 16px;
  margin-top: 15px;
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
}

.permission-card strong {
  color: var(--learning-text);
  font-size: 13px;
}

.permission-card p {
  margin: 5px 0 0;
  color: var(--learning-text-muted);
  font-size: 10px;
}

.permission-list {
  display: flex;
  flex-direction: column;
}

.permission-row {
  justify-content: space-between;
  gap: 16px;
  padding: 14px 0;
  border-bottom: 1px solid var(--learning-border);
}

.permission-row:last-child {
  border-bottom: 0;
}

.permission-row>div {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.permission-row strong {
  color: var(--learning-text);
  font-size: 13px;
}

.permission-row span {
  color: var(--learning-text-secondary);
  font-size: 11px;
}

@media (max-width: 620px) {
  .page-frame {
    padding: 22px 14px 36px;
  }

  .permission-card {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
