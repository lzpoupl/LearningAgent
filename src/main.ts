import { createApp } from "vue";
import App from "./App.vue";

async function bootstrap(): Promise<void> {
  if (import.meta.env.DEV && import.meta.env.VITE_USE_MOCK === "true") {
    const { setupMocks } = await import("./mocks");
    setupMocks();
  }
  createApp(App).mount("#app");
}

void bootstrap();
