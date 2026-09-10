<template>
  <main class="page-frame agents-page">
    <PageHeader
      description="创建、配置和管理不同学科的 AI 学习助手"
      eyebrow="学习助手 / AGENTS"
      title="Agent 管理"
    >
      <el-button type="primary" @click="openCreateDialog">
        <el-icon><Plus /></el-icon>
        创建 Agent
      </el-button>
    </PageHeader>

    <el-row :gutter="14">
      <el-col v-for="agent in agents" :key="agent.id" :lg="12" :md="12" :sm="24" :xl="12" :xs="24">
        <el-card class="agent-card" shadow="hover">
          <div class="agent-card-heading">
            <el-avatar :size="46" :style="{ background: agent.color }">{{ agent.icon }}</el-avatar>
            <div class="agent-card-title">
              <strong>{{ agent.name }}</strong>
              <span>{{ agent.builtin ? '系统 Agent' : '自定义 Agent' }}</span>
            </div>
            <el-switch v-model="agent.enabled" :aria-label="`${agent.name}启用状态`" />
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
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-card class="permission-card" shadow="never">
      <div>
        <strong>Agent 权限管理</strong>
        <p>按学习资产配置读取、编辑和写入权限。</p>
      </div>
      <el-button plain @click="permissionDialogVisible = true">权限设置</el-button>
    </el-card>

    <el-dialog v-model="dialogVisible" :title="editingAgentId ? '配置 Agent' : '创建 Agent'" width="min(520px, 92vw)">
      <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
        <el-form-item label="Agent 名称" prop="name">
          <el-input v-model="form.name" placeholder="例如：操作系统 Agent" />
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
        <el-button type="primary" @click="permissionDialogVisible = false">完成</el-button>
      </template>
    </el-dialog>
  </main>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'

import PageHeader from '../components/common/PageHeader.vue'
import { agentCatalog } from '../data/agents'
import type { AgentType } from '../types/chat'

interface AgentRecord {
  id: string
  name: string
  subject: string
  description: string
  icon: string
  color: string
  capabilities: string[]
  enabled: boolean
  builtin: boolean
  agentType?: AgentType
}

interface AgentForm {
  name: string
  subject: string
  description: string
  capabilities: string
}

const emit = defineEmits<{
  'start-chat': [agent: AgentType]
}>()

const agents = ref<AgentRecord[]>([
  ...agentCatalog.map(agent => ({
    ...agent,
    subject: agent.id === 'math' ? '数学' : '英语',
    enabled: true,
    builtin: true,
    agentType: agent.id,
  })),
  {
    id: 'custom-os',
    name: '操作系统 Agent',
    subject: '计算机',
    description: '课程知识、知识点总结和习题解析。',
    icon: '</>',
    color: 'linear-gradient(135deg, #397bd9, #2654bd)',
    capabilities: ['课程知识', '知识点总结'],
    enabled: true,
    builtin: false,
  },
])

const permissions = ref([
  { key: 'read-assets', label: '读取学习资料', description: '允许 Agent 检索 PDF、笔记和课件。', enabled: true },
  { key: 'read-notes', label: '读取错题与笔记', description: '允许 Agent 参考你的结构化学习记录。', enabled: true },
  { key: 'write-anki', label: '创建 Anki 卡片', description: '允许 Agent 将结论整理为待复习卡片。', enabled: true },
])

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
}

function openCreateDialog() {
  editingAgentId.value = null
  resetForm()
  dialogVisible.value = true
}

function editAgent(agent: AgentRecord) {
  editingAgentId.value = agent.id
  form.name = agent.name
  form.subject = agent.subject
  form.description = agent.description
  form.capabilities = agent.capabilities.join(', ')
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
      const agent = agents.value.find(item => item.id === editingAgentId.value)
      if (agent) {
        agent.name = form.name.trim()
        agent.subject = form.subject.trim()
        agent.description = form.description.trim()
        agent.capabilities = capabilities
      }
      ElMessage.success('Agent 配置已更新')
    } else {
      agents.value.push({
        id: `custom-${Date.now()}`,
        name: form.name.trim(),
        subject: form.subject.trim(),
        description: form.description.trim(),
        icon: 'AI',
        color: 'linear-gradient(135deg, #9a7bea, #6c5ce7)',
        capabilities,
        enabled: true,
        builtin: false,
      })
      ElMessage.success('自定义 Agent 已创建')
    }
    dialogVisible.value = false
  } finally {
    saving.value = false
  }
}

function startAgent(agent: AgentRecord) {
  if (agent.agentType && agent.enabled) {
    emit('start-chat', agent.agentType)
    return
  }

  ElMessage.info('自定义 Agent 的会话接口将在配置完成后启用')
}
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

.permission-row > div {
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
