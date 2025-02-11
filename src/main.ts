import { createApp } from "vue";
import App from "./App.vue";
import {
  provideFluentDesignSystem,
  fluentButton,
  fluentCard,
  fluentTextField,
  fluentDivider,
} from "@fluentui/web-components";

// 注册 Fluent UI 组件
provideFluentDesignSystem().register(
  fluentButton(),
  fluentCard(),
  fluentTextField(),
  fluentDivider()
);

createApp(App).mount("#app");
