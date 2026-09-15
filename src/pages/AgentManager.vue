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
          </div>

          <p class="agent-description">{{ agent.description }}</p>

          <div class="agent-card-footer">
            <el-button link type="primary" @click="startAgent(agent)">开始对话</el-button>
            <el-button link @click="editAgent(agent)">配置 Agent</el-button>
            <el-button v-if="!agent.builtin" link type="danger" @click="removeAgent(agent)">删除</el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-dialog v-model="dialogVisible" :title="editingAgentId !== null ? '配置 Agent' : '创建 Agent'"
      width="min(920px, 94vw)">
      <div class="agent-config">
        <el-form ref="formRef" class="agent-config-form" :model="form" :rules="rules" label-position="top">
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
          <el-form-item label="职责描述" prop="description">
            <el-input v-model="form.description" :rows="4" type="textarea" />
          </el-form-item>
        </el-form>

        <section class="permission-section">
          <div class="permission-section-heading">
            <strong>工具权限</strong>
            <span>为「{{ form.name || '该 Agent' }}」配置「允许 / 询问 / 拒绝」三级权限。</span>
          </div>

          <div v-loading="permissionsLoading" class="permission-groups">
            <div v-for="group in groupedPermissions" :key="group.name" class="permission-group">
              <div class="permission-group-title">
                <strong>{{ groupLabel(group.name) }}</strong>
                <span>{{ group.entries.length }} 个工具</span>
              </div>
              <div v-for="entry in group.entries" :key="toolKey(entry)" class="permission-row">
                <div>
                  <strong>{{ entry.tool.name }}</strong>
                  <span>{{ entry.tool.description }}</span>
                </div>
                <el-select v-model="entry.permission" class="permission-level" aria-label="工具权限" size="small">
                  <el-option label="允许" value="allow" />
                  <el-option label="询问" value="ask" />
                  <el-option label="拒绝" value="deny" />
                </el-select>
              </div>
            </div>
            <div v-if="!permissionsLoading && !permissionEntries.length" class="permission-empty">暂无可配置的工具。</div>
          </div>
        </section>
      </div>

      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button :loading="saving" type="primary" @click="saveAgent">保存</el-button>
      </template>
    </el-dialog>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
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
  getAgentToolPermissions,
  listAgents,
  listTools,
  updateAgent,
} from '../services/agent'
import type {
  AgentConfigInput,
  AgentInfo,
  AgentToolPermission,
  AgentToolPermissionInput,
  AgentType,
  ToolPermission,
} from '../types/chat'

interface AgentForm {
  name: string
  description: string
  icon: string
  color: string
}

const emit = defineEmits<{
  'start-chat': [agent: AgentType]
}>()

/** 工具组的中文名，未知组回退为原始组名。 */
const toolGroupLabels: Record<string, string> = {
  anki: 'Anki 卡片',
  asset: '学习资产',
  user: '用户交互',
}

const agents = ref<AgentInfo[]>([])
const loading = ref(false)
const errorMessage = ref('')

const dialogVisible = ref(false)
const editingAgentId = ref<AgentType | null>(null)
const saving = ref(false)
const formRef = ref<FormInstance>()
const form = reactive<AgentForm>({
  name: '',
  description: '',
  icon: defaultAgentIcon,
  color: defaultAgentColor,
})

const rules: FormRules<AgentForm> = {
  name: [{ required: true, message: '请输入 Agent 名称', trigger: 'blur' }],
  description: [{ required: true, message: '请输入职责描述', trigger: 'blur' }],
}

const permissionsLoading = ref(false)
const permissionEntries = ref<AgentToolPermission[]>([])

const groupedPermissions = computed(() => {
  const groups: { name: string; entries: AgentToolPermission[] }[] = []

  for (const entry of permissionEntries.value) {
    let group = groups.find(item => item.name === entry.tool.group)
    if (!group) {
      group = { name: entry.tool.group, entries: [] }
      groups.push(group)
    }
    group.entries.push(entry)
  }

  return groups
})

function groupLabel(name: string) {
  return toolGroupLabels[name] ?? name
}

function toolKey(entry: AgentToolPermission) {
  return `${entry.tool.group}.${entry.tool.id}`
}

function resetForm() {
  form.name = ''
  form.description = ''
  form.icon = defaultAgentIcon
  form.color = defaultAgentColor
}

function openCreateDialog() {
  editingAgentId.value = null
  resetForm()
  dialogVisible.value = true
  void loadToolPermissions(null)
}

function editAgent(agent: AgentInfo) {
  editingAgentId.value = agent.id
  form.name = agent.name
  form.description = agent.description
  form.icon = agent.icon || defaultAgentIcon
  form.color = agent.color || defaultAgentColor
  dialogVisible.value = true
  void loadToolPermissions(agent.id)
}

/** 编辑时读取该 Agent 的逐项生效级别；新建时按「未配置等价于拒绝」填充工具目录。 */
async function loadToolPermissions(agentId: AgentType | null) {
  permissionsLoading.value = true
  permissionEntries.value = []

  try {
    permissionEntries.value = agentId === null
      ? (await listTools()).map(tool => ({ tool, permission: 'deny' as ToolPermission }))
      : await getAgentToolPermissions(agentId)
  } catch (error) {
    console.error(error)
    ElMessage.error('工具权限加载失败，请重试。')
  } finally {
    permissionsLoading.value = false
  }
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
  const toolPermissions: AgentToolPermissionInput[] = permissionEntries.value.map(entry => ({
    toolId: toolKey(entry),
    permission: entry.permission,
  }))

  const input: AgentConfigInput = {
    name: form.name.trim(),
    description: form.description.trim(),
    icon: form.icon,
    color: form.color,
    ...(toolPermissions.length ? { toolPermissions } : {}),
  }

  try {
    if (editingAgentId.value !== null) {
      const updatedAgent = await updateAgent(editingAgentId.value, input)
      const index = agents.value.findIndex(item => item.id === editingAgentId.value)
      if (index !== -1) {
        agents.value[index] = updatedAgent
      }
      ElMessage.success('Agent 配置已更新')
    } else {
      agents.value.push(await createAgent(input))
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

function startAgent(agent: AgentInfo) {
  emit('start-chat', agent.id)
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

.agent-card-footer {
  justify-content: flex-end;
  gap: 4px;
  margin-top: 16px;
  padding-top: 11px;
  border-top: 1px solid var(--learning-border);
}

.agent-config {
  display: flex;
  align-items: flex-start;
  gap: 24px;
}

.agent-config-form {
  min-width: 0;
  flex: 0 0 300px;
}

.permission-section {
  min-width: 0;
  flex: 1 1 auto;
  padding-left: 24px;
  border-left: 1px solid var(--learning-border);
}

.permission-section-heading {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-bottom: 12px;
}

.permission-section-heading strong {
  color: var(--learning-text);
  font-size: 13px;
}

.permission-section-heading span {
  color: var(--learning-text-muted);
  font-size: 10px;
}

.permission-groups {
  max-height: 360px;
  overflow-y: auto;
}

.permission-group+.permission-group {
  margin-top: 14px;
}

.permission-group-title {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--learning-border);
}

.permission-group-title strong {
  color: var(--learning-text);
  font-size: 12px;
}

.permission-group-title span {
  color: var(--learning-text-muted);
  font-size: 10px;
}

.permission-row {
  justify-content: space-between;
  gap: 16px;
  padding: 11px 0;
  border-bottom: 1px solid var(--learning-border);
}

.permission-row:last-child {
  border-bottom: 0;
}

.permission-row>div {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 3px;
}

.permission-row strong {
  color: var(--learning-text);
  font-size: 12px;
}

.permission-row span {
  color: var(--learning-text-secondary);
  font-size: 11px;
}

.permission-level {
  flex: 0 0 auto;
  width: 96px;
}

.permission-empty {
  padding: 18px 0;
  color: var(--learning-text-muted);
  font-size: 11px;
  text-align: center;
}

@media (max-width: 720px) {
  .agent-config {
    flex-direction: column;
    gap: 16px;
  }

  .agent-config-form,
  .permission-section {
    width: 100%;
    flex: 1 1 auto;
  }

  .permission-section {
    padding-top: 16px;
    padding-left: 0;
    border-top: 1px solid var(--learning-border);
    border-left: 0;
  }
}

@media (max-width: 620px) {
  .page-frame {
    padding: 22px 14px 36px;
  }
}
</style>
