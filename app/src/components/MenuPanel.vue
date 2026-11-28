<script setup lang="ts">
// A submenu's frame: Back and the title at the top, the menu under them.
import { useLabels } from "../labels";
import { useMenu } from "../useMenu";

defineProps<{ title: string; color?: string }>();
const { open } = useMenu();
const t = useLabels();
</script>

<template>
  <section class="panel" :aria-label="title">
    <header>
      <button class="back" @click="open(null)">← {{ t.back }}</button>
      <h2 :style="{ color }">{{ title }}</h2>
    </header>
    <slot />
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 8px;
  min-width: 0;
}

header {
  display: flex;
  align-items: center;
  gap: 12px;
}

h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

/* Shared by the menus inside. */
.panel :deep(.field) {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.panel :deep(.field > .caption) {
  font-size: 13px;
}

.panel :deep(.options) {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.panel :deep(.level) {
  font-size: 13px;
  color: var(--muted);
}
</style>
