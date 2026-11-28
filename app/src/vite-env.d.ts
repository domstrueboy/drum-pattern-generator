/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

/** Android's orientation call (MainActivity.kt); absent on the desktop. */
interface Window {
  DpgOrientation?: { next(): void };
}
