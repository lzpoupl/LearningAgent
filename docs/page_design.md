# 环境说明

- 已经安装 Element Plus。页面组件优先使用 Element Plus 的布局、导航、表单、卡片、弹窗、上传、表格和反馈组件。
- 已经安装 Element Icons。页面组件优先使用 `@element-plus/icons-vue` 提供的图标。
- 已经配置 unplugin-vue-components和unplugin-auto-import。页面组件可以直接使用 Element Plus 组件和 Vue 组合式 API，无需手动导入。
- 已经安装 KaTeX。页面组件使用 `katex` 渲染公式，使用 `marked` 渲染 Markdown 内容。
- mocks/ 目录下使用了Tauri提供的mock功能，在开发环境，前端必须使用 `src/services/` 里面接口进行开发，不得在代码中硬编码假数据，而应该放到 mocks/ 目录下。

# 前端页面结构

## 应用壳层

- `src/App.vue`
    - 维护当前页面、临时页面和 Anki 编辑卡片状态。
    - 注册页面组件映射，负责页面之间的导航和事件转发。
    - 不承载页面布局、页面样式或具体业务请求。
- `src/components/layout/AppLayout.vue`
    - 使用 `el-container` 组织侧边栏、顶部栏和页面内容区域。
- `src/components/layout/Sidebar.vue`
    - 使用 `el-menu` 展示主导航。
    - 支持展开、收起侧边栏。
    - 在 Agent 会话页面展示历史会话，并负责重命名和删除操作。
- `src/components/layout/Topbar.vue`
    - 提供全局搜索入口、通知、帮助和用户头像。

## 页面组件

- `src/pages/Home.vue`
    - 首页欢迎区域、快捷入口、今日学习计划和 Anki 汇总。
    - 通过事件通知应用壳层打开 Agent 会话、Agent 管理、资料或复习页面。
- `src/pages/AgentManager.vue`
    - 展示内置和自定义 Agent。
    - 负责 Agent 创建、配置、启停和权限设置弹窗。
- `src/pages/AgentSession.vue`
    - 组织 Agent 会话头部、消息流、输入框和学习上下文面板。
    - 不直接管理会话列表，消息和会话生命周期由 `useChatSessions` 提供。
- `src/pages/NewSession.vue`
    - 新会话准备页，选择 Agent 并提交第一个问题。
- `src/pages/AnkiManager.vue`
    - 牌组树、卡片检索、卡片预览、移动、删除、重置和复习评分。
    - 通过 `useDeckTree` 读取牌组层级，通过 `services/anki.ts` 调用后端。
- `src/pages/AnkiCreator.vue`
    - 新建和编辑 Anki 卡片。
    - 负责表单内容、图片插入、牌组选择和实时预览。
- `src/pages/AnkiReview.vue`
    - 选择牌组、显示待复习卡片和记录四种复习结果。
- `src/pages/AssetManager.vue`
    - 学习资料上传、学科筛选、排序、打开和移除。
    - 当前文件选择使用浏览器本地对象 URL；接入文件后端时只替换该页面的数据层。
- `src/pages/Statistics.vue`
    - 今日、本周学习摘要、最近七天学习时间和学科构成。
- `src/pages/Settings.vue`
    - 学习目标、学习科目、主题和 Agent 高级设置。

## 通用组件

- `src/components/common/PageHeader.vue`
    - 统一页面标题、说明和右侧操作区。
- `src/components/chat/UserMessage.vue`
    - 用户消息气泡。
- `src/components/chat/AIMessage.vue`
    - AI 消息容器，按内容块组合文字、公式和工具结果。
- `src/components/chat/AITextMessage.vue`
    - AI 普通文本内容。
- `src/components/chat/LatexMessage.vue`
    - 使用 KaTeX 渲染公式。
- `src/components/chat/AIResultMessage.vue`
    - 展示 Agent 工具调用结果。
- `src/components/chat/ChatMessage.vue`
    - 根据消息角色选择用户或 AI 消息组件。
- `src/components/chat/ChatContextPanel.vue`
    - 展示当前 Agent 可访问的学习资产和权限。
- `src/components/anki/CardContent.vue`
    - 统一处理 Anki 卡片的 Markdown、LaTeX 和安全 HTML 渲染。
- `src/components/anki/CardShow.vue`
    - 统一的卡片展示组件，按 `manager` / `preview` / `review` / `creator` 四种形态展示卡片正反面。

## Composable 与数据边界

- `src/composables/useChatSessions.ts`
    - 管理会话列表、当前会话、发送消息、创建、选择、重命名和删除。
- `src/composables/useDeckTree.ts`
    - 加载牌组树、计算牌组卡片数、展开视图和完整牌组列表。
- `src/services/agent.ts`、`src/services/api.ts`
    - 提供 Agent 配置、权限与会话的后端请求边界。
- `src/services/study.ts`、`src/services/settings.ts`、`src/services/assets.ts`、`src/services/user.ts`
    - 提供首页计划、统计、设置、学习资料和用户信息的后端请求边界。
- `src/mocks/`
    - 仅在开发环境拦截服务层的 Tauri 命令并返回内存假数据，页面和 composable 不直接引用 mock 数据。
- `src/services/api.ts`
    - Agent 会话接口边界。
- `src/services/anki.ts`
    - Anki 牌组、卡片和复习接口边界。
- `src/types/chat.ts`、`src/types/anki.ts`、`src/types/navigation.ts`
    - 领域数据和页面导航类型，不在页面内重复声明。

## 页面流转

1. 侧边栏和首页快捷入口只发出导航事件。
2. `App.vue` 更新当前页面，必要时创建临时页面状态。
3. 页面组件通过 `services` 或 composable 读取和修改数据。
4. Anki 卡片编辑和复习完成后通过事件回到 `AnkiManager` 或对应页面。
5. Agent 会话统一通过 `useChatSessions` 更新，`AgentSession` 只负责呈现和输入。
