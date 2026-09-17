<template>
  <main class="page-frame settings-page">
    <PageHeader
      description="模型服务与复习调度，对应后端 config.toml"
      eyebrow="系统 / SETTINGS"
      title="配置中心"
    />

    <el-card v-loading="loading" class="settings-card" shadow="never">
      <div v-if="errorMessage" class="settings-error" role="alert">{{ errorMessage }}</div>

      <el-tabs v-model="activeTab" tab-position="left" class="settings-tabs">
        <el-tab-pane label="模型服务" name="llm">
          <section class="settings-section">
            <div class="section-heading">
              <h2>默认模型服务</h2>
              <span>未在会话中单独指定 provider 时使用该服务。</span>
            </div>
            <el-select
              :model-value="defaultProvider"
              :disabled="!llmProviders.length || savingDefault"
              class="default-select"
              placeholder="尚未配置 provider"
              @change="changeDefault"
            >
              <el-option
                v-for="provider in llmProviders"
                :key="provider.name"
                :label="provider.name"
                :value="provider.name"
              />
            </el-select>
          </section>

          <section class="settings-section">
            <div class="section-heading">
              <h2>Provider</h2>
              <div class="section-actions">
                <el-button type="primary" @click="openCreateProvider">
                  <el-icon><Plus /></el-icon>
                  新增 Provider
                </el-button>
              </div>
            </div>

            <div v-if="!llmProviders.length" class="empty-hint">
              还没有 provider，点击「新增 Provider」接入 OpenAI 兼容端点。
            </div>

            <div v-for="provider in llmProviders" :key="provider.name" class="provider-row">
              <div class="provider-info">
                <div class="provider-title">
                  <strong>{{ provider.name }}</strong>
                  <el-tag v-if="provider.name === defaultProvider" effect="light" size="small">默认</el-tag>
                  <el-tag v-if="provider.apiKeyConfigured" effect="plain" size="small" type="success">密钥已配置</el-tag>
                  <el-tag v-else effect="plain" size="small" type="warning">缺少密钥</el-tag>
                </div>
                <div class="provider-meta">
                  <span>模型：{{ provider.model }}</span>
                  <span>地址：{{ provider.baseUrl }}</span>
                  <span v-if="provider.apiKeyMasked">密钥：{{ provider.apiKeyMasked }}</span>
                  <span v-if="provider.compressionModel">压缩模型：{{ provider.compressionModel }}</span>
                </div>
              </div>
              <div class="provider-actions">
                <el-button
                  link
                  type="primary"
                  :disabled="provider.name === defaultProvider"
                  @click="changeDefaultTo(provider.name)"
                >
                  设为默认
                </el-button>
                <el-button link :loading="testingName === provider.name" @click="testProvider(provider.name)">
                  测试
                </el-button>
                <el-button link @click="openEditProvider(provider)">编辑</el-button>
              </div>
            </div>
          </section>

          <section class="settings-section">
            <div class="section-heading">
              <h2>运行参数</h2>
              <span>以下参数直接读取自 config.toml，修改文件并重启后生效。</span>
            </div>
            <div class="setting-row">
              <div>
                <strong>单轮最大步数</strong>
                <span>含工具调用轮次</span>
              </div>
              <span class="setting-value">{{ llmConfig?.maxSteps ?? '-' }}</span>
            </div>
            <div class="setting-row">
              <div>
                <strong>流式输出</strong>
                <span>无工具的请求允许边生成边显示</span>
              </div>
              <span class="setting-value">{{ llmConfig?.allowStreaming ? '已启用' : '已关闭' }}</span>
            </div>
          </section>
        </el-tab-pane>

        <el-tab-pane label="复习调度" name="scheduler">
          <section class="settings-section">
            <div class="section-heading">
              <h2>调度算法</h2>
              <span>决定新卡进入复习阶段后间隔的计算方式。</span>
            </div>
            <el-radio-group v-model="scheduler.algorithm" class="algorithm-grid">
              <el-radio-button value="sm2">SM-2</el-radio-button>
              <el-radio-button value="fsrs">FSRS</el-radio-button>
            </el-radio-group>
            <p class="section-hint">{{ algorithmHint }}</p>
          </section>

          <section class="settings-section">
            <div class="section-heading">
              <h2>学习阶段步长</h2>
              <span>新卡与重学卡按「重来 / 困难 / 良好」作答后的复习延时。</span>
            </div>
            <el-form label-position="top">
              <div class="settings-grid">
                <el-form-item label="重来（分钟）">
                  <el-input-number v-model="scheduler.learning_again_minutes" :max="1440" :min="1" />
                </el-form-item>
                <el-form-item label="困难（分钟）">
                  <el-input-number v-model="scheduler.learning_hard_minutes" :max="1440" :min="1" />
                </el-form-item>
                <el-form-item label="良好（分钟）">
                  <el-input-number v-model="scheduler.learning_good_minutes" :max="1440" :min="1" />
                </el-form-item>
              </div>
            </el-form>
          </section>

          <div class="settings-footer">
            <el-button :loading="savingScheduler" type="primary" @click="saveScheduler">
              保存调度配置
            </el-button>
          </div>
        </el-tab-pane>

        <el-tab-pane label="外观" name="appearance">
          <section class="settings-section">
            <div class="section-heading">
              <h2>主题风格</h2>
              <span>选择「跟随系统」时皮肤会随操作系统的浅色/深色偏好自动切换，选择结果在下次启动时恢复。</span>
            </div>
            <el-radio-group v-model="theme" class="theme-grid">
              <el-radio-button value="light">浅色</el-radio-button>
              <el-radio-button value="dark">深色</el-radio-button>
              <el-radio-button value="system">跟随系统</el-radio-button>
            </el-radio-group>
          </section>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <el-dialog
      v-model="providerDialogVisible"
      :title="editingProviderName !== null ? '编辑 Provider' : '新增 Provider'"
      width="min(520px, 92vw)"
    >
      <el-form ref="providerFormRef" :model="providerForm" :rules="providerRules" label-position="top">
        <el-form-item label="名称" prop="name">
          <el-input
            v-model="providerForm.name"
            :disabled="editingProviderName !== null"
            placeholder="例如：deepseek"
          />
        </el-form-item>
        <el-form-item label="Base URL" prop="baseUrl">
          <el-input v-model="providerForm.baseUrl" placeholder="https://api.deepseek.com" />
        </el-form-item>
        <el-form-item label="模型名" prop="model">
          <el-input v-model="providerForm.model" placeholder="deepseek-chat" />
        </el-form-item>
        <el-form-item label="API 密钥">
          <el-input
            v-model="providerForm.apiKey"
            :placeholder="apiKeyPlaceholder"
            show-password
            type="password"
          />
        </el-form-item>
        <el-form-item label="压缩模型（可选）">
          <el-input v-model="providerForm.compressionModel" placeholder="留空表示不启用压缩" />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="providerDialogVisible = false">取消</el-button>
        <el-button :loading="savingProvider" type="primary" @click="saveProvider">保存</el-button>
      </template>
    </el-dialog>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'

import PageHeader from '../components/common/PageHeader.vue'
import { useTheme } from '../composables/useTheme'
import { getSchedulerConfig, updateSchedulerConfig } from '../services/anki'
import {
  getLlmConfig,
  setDefaultLlmProvider,
  testLlmProvider,
  upsertLlmProvider,
} from '../services/llm'
import type { LlmConfigView, LlmProviderInput, LlmProviderView } from '../types/chat'
import type { SchedulerConfig } from '../types/anki'

interface ProviderForm {
  name: string
  baseUrl: string
  model: string
  apiKey: string
  compressionModel: string
}

const activeTab = ref('llm')
const loading = ref(false)
const errorMessage = ref('')

const llmConfig = ref<LlmConfigView | null>(null)
const savingDefault = ref(false)
const testingName = ref<string | null>(null)

const scheduler = reactive<SchedulerConfig>({
  algorithm: 'sm2',
  learning_again_minutes: 1,
  learning_hard_minutes: 6,
  learning_good_minutes: 10,
})
const savingScheduler = ref(false)

const { theme, setTheme } = useTheme()

const providerDialogVisible = ref(false)
const editingProviderName = ref<string | null>(null)
const savingProvider = ref(false)
const providerFormRef = ref<FormInstance>()
const providerForm = reactive<ProviderForm>({
  name: '',
  baseUrl: '',
  model: '',
  apiKey: '',
  compressionModel: '',
})

const providerRules: FormRules<ProviderForm> = {
  name: [{ required: true, message: '请输入 provider 名称', trigger: 'blur' }],
  baseUrl: [{ required: true, message: '请输入 base_url', trigger: 'blur' }],
  model: [{ required: true, message: '请输入模型名', trigger: 'blur' }],
}

const llmProviders = computed<LlmProviderView[]>(() => llmConfig.value?.providers ?? [])
const defaultProvider = computed(() => llmConfig.value?.defaultProvider ?? '')

const algorithmHint = computed(() =>
  scheduler.algorithm === 'fsrs'
    ? 'FSRS 依据记忆稳定性与难度动态计算间隔，适合长期复习。'
    : 'SM-2 按固定的难度系数与间隔倍率递推，行为可预期。',
)

const apiKeyPlaceholder = computed(() => {
  const current = llmProviders.value.find(provider => provider.name === editingProviderName.value)

  if (current?.apiKeyConfigured) {
    return `留空保留原密钥（${current.apiKeyMasked}）`
  }

  return '请输入 API 密钥'
})

function errorText(error: unknown, fallback: string): string {
  if (typeof error === 'string' && error) {
    return error
  }
  if (error && typeof error === 'object' && 'message' in error) {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message) {
      return message
    }
  }
  if (error instanceof Error && error.message) {
    return error.message
  }
  return fallback
}

async function loadConfig() {
  loading.value = true
  errorMessage.value = ''
  try {
    const [llm, schedulerConfig] = await Promise.all([getLlmConfig(), getSchedulerConfig()])
    llmConfig.value = llm
    Object.assign(scheduler, schedulerConfig)
  } catch (error) {
    console.error(error)
    errorMessage.value = '配置加载失败，请稍后重试。'
  } finally {
    loading.value = false
  }
}

function resetProviderForm() {
  providerForm.name = ''
  providerForm.baseUrl = ''
  providerForm.model = ''
  providerForm.apiKey = ''
  providerForm.compressionModel = ''
}

function openCreateProvider() {
  editingProviderName.value = null
  resetProviderForm()
  providerDialogVisible.value = true
}

function openEditProvider(provider: LlmProviderView) {
  editingProviderName.value = provider.name
  providerForm.name = provider.name
  providerForm.baseUrl = provider.baseUrl
  providerForm.model = provider.model
  providerForm.apiKey = ''
  providerForm.compressionModel = provider.compressionModel ?? ''
  providerDialogVisible.value = true
}

async function saveProvider() {
  if (!providerFormRef.value || savingProvider.value) {
    return
  }

  try {
    await providerFormRef.value.validate()
  } catch {
    return
  }

  const input: LlmProviderInput = {
    name: providerForm.name.trim(),
    baseUrl: providerForm.baseUrl.trim(),
    model: providerForm.model.trim(),
  }
  const compressionModel = providerForm.compressionModel.trim()
  const apiKey = providerForm.apiKey.trim()
  if (compressionModel) {
    input.compressionModel = compressionModel
  }
  if (apiKey) {
    input.apiKey = apiKey
  }

  savingProvider.value = true
  try {
    llmConfig.value = await upsertLlmProvider(input)
    providerDialogVisible.value = false
    ElMessage.success(editingProviderName.value ? 'Provider 已更新' : 'Provider 已新增')
  } catch (error) {
    console.error(error)
    ElMessage.error(errorText(error, 'Provider 保存失败，请重试。'))
  } finally {
    savingProvider.value = false
  }
}

async function changeDefault(value: unknown) {
  const name = String(value ?? '')
  if (!name || name === defaultProvider.value || savingDefault.value) {
    return
  }

  savingDefault.value = true
  try {
    llmConfig.value = await setDefaultLlmProvider(name)
    ElMessage.success(`默认 provider 已切换为 ${name}`)
  } catch (error) {
    console.error(error)
    ElMessage.error(errorText(error, '默认 provider 切换失败。'))
  } finally {
    savingDefault.value = false
  }
}

function changeDefaultTo(name: string) {
  void changeDefault(name)
}

async function testProvider(name: string) {
  if (testingName.value) {
    return
  }

  testingName.value = name
  try {
    const result = await testLlmProvider(name)
    const reply = result.reply.length > 60 ? `${result.reply.slice(0, 60)}…` : result.reply
    ElMessage.success(`${result.provider} · ${result.model} · ${result.latencyMs}ms：${reply || '连通'}`)
  } catch (error) {
    console.error(error)
    ElMessage.error(errorText(error, '连通性测试失败，请检查配置。'))
  } finally {
    testingName.value = null
  }
}

async function saveScheduler() {
  if (savingScheduler.value) {
    return
  }

  savingScheduler.value = true
  try {
    Object.assign(scheduler, await updateSchedulerConfig({ ...scheduler }))
    ElMessage.success('调度配置已保存')
  } catch (error) {
    console.error(error)
    ElMessage.error(errorText(error, '调度配置保存失败，请重试。'))
  } finally {
    savingScheduler.value = false
  }
}

watch(theme, nextTheme => setTheme(nextTheme))

onMounted(loadConfig)
</script>

<style scoped>
.page-frame {
  width: min(1100px, 100%);
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  padding: 30px 28px 48px;
  overflow-y: auto;
}

.settings-card {
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
}

.settings-error {
  margin-bottom: 14px;
  padding: 9px 11px;
  border-radius: 6px;
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
  font-size: 12px;
}

.settings-tabs {
  min-height: 430px;
}

.settings-tabs :deep(.el-tabs__header) {
  width: 150px;
}

.settings-tabs :deep(.el-tabs__item) {
  justify-content: flex-start;
  color: var(--learning-text-secondary);
  font-size: 12px;
}

.settings-tabs :deep(.el-tabs__item.is-active) {
  color: var(--el-color-primary);
}

.settings-tabs :deep(.el-tabs__content) {
  padding: 2px 10px 10px 26px;
}

.settings-section + .settings-section {
  margin-top: 30px;
  padding-top: 24px;
  border-top: 1px solid var(--learning-border);
}

.section-heading {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 18px;
}

.section-heading h2 {
  margin: 0;
}

.section-heading > span {
  color: var(--learning-text-muted);
  font-size: 11px;
}

.section-actions {
  flex: 0 0 auto;
}

h2 {
  color: var(--learning-text);
  font-size: 15px;
}

.default-select {
  width: 260px;
}

.empty-hint {
  padding: 18px 0;
  color: var(--learning-text-muted);
  font-size: 11px;
  text-align: center;
}

.provider-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 13px 0;
  border-bottom: 1px solid var(--learning-border);
}

.provider-row:last-child {
  border-bottom: 0;
}

.provider-info {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 6px;
}

.provider-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.provider-title strong {
  color: var(--learning-text);
  font-size: 13px;
}

.provider-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 14px;
  color: var(--learning-text-secondary);
  font-size: 11px;
}

.provider-actions {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 4px;
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}

.settings-grid :deep(.el-input-number) {
  width: 100%;
}

.algorithm-grid {
  display: flex;
}

.section-hint {
  margin-top: 14px;
  color: var(--learning-text-muted);
  font-size: 11px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-height: 62px;
  border-bottom: 1px solid var(--learning-border);
}

.setting-row:last-child {
  border-bottom: 0;
}

.setting-row > div {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-row strong {
  color: var(--learning-text);
  font-size: 12px;
}

.setting-row span {
  color: var(--learning-text-muted);
  font-size: 10px;
}

.setting-value {
  color: var(--learning-text);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.theme-grid {
  display: flex;
}

.settings-footer {
  display: flex;
  justify-content: flex-end;
  margin-top: 18px;
  padding-top: 16px;
  border-top: 1px solid var(--learning-border);
}

@media (max-width: 700px) {
  .page-frame {
    padding: 22px 14px 36px;
  }

  .settings-tabs :deep(.el-tabs__header) {
    width: 112px;
  }

  .settings-tabs :deep(.el-tabs__content) {
    padding-left: 14px;
  }

  .settings-grid {
    grid-template-columns: 1fr;
    gap: 2px;
  }

  .section-heading {
    flex-direction: column;
    gap: 4px;
  }

  .provider-row {
    align-items: flex-start;
    flex-direction: column;
  }

  .default-select {
    width: 100%;
  }
}

@media (max-width: 480px) {
  .settings-tabs {
    min-height: 500px;
  }

  .settings-tabs :deep(.el-tabs__header) {
    float: none;
    width: auto;
  }

  .settings-tabs :deep(.el-tabs__nav-wrap) {
    margin-bottom: 14px;
  }

  .settings-tabs :deep(.el-tabs__nav) {
    display: flex;
    overflow-x: auto;
  }

  .settings-tabs :deep(.el-tabs__content) {
    padding: 0;
  }
}
</style>
