import { createApp } from "vue";
import ElementPlus from 'element-plus'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import App from "./App.vue";
import { initTheme } from "./composables/useTheme";
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './styles/app.css'

async function bootstrap(): Promise<void> {
    // 真实 Tauri WebView 会在页面加载前注入只读的 __TAURI_INTERNALS__，
    // 此时 mockIPC 对其赋值会抛 TypeError，因此 mock 仅在纯浏览器（无 Tauri 运行时）生效；
    // 在 WebView 中运行（如 tauri dev）时走真实后端 IPC，测试数据由后端内存 SQLite 承载。
    const inTauriRuntime = "__TAURI_INTERNALS__" in window;
    if (
        import.meta.env.DEV &&
        import.meta.env.VITE_USE_MOCK === "true" &&
        !inTauriRuntime
    ) {
        const { setupMocks } = await import("./mocks");
        setupMocks();
    }

    // 主题需要读取配置，因此放在 mock 就位之后，保证首次绘制就是上次选择的皮肤。
    await initTheme();

    createApp(App).use(ElementPlus, { locale: zhCn }).mount("#app");
}

void bootstrap();
