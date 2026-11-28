<script setup lang="ts">
// The one screen, portrait, top to bottom as the spec lists it.
import { computed } from "vue";
import BarsRow from "./components/BarsRow.vue";
import KitPanel from "./components/KitPanel.vue";
import MetronomeRow from "./components/MetronomeRow.vue";
import NotationView from "./components/NotationView.vue";
import TransportRow from "./components/TransportRow.vue";
import type { SessionState } from "./session";
import { useRefresh, useSession } from "./useSession";

const { state } = useSession();
useRefresh();

// The session's part, once an output has opened.
const s = computed(() =>
  state.value?.bpm !== undefined ? (state.value as SessionState) : null,
);
</script>

<template>
  <main>
    <p v-if="state?.error" class="error">{{ state.error }}</p>
    <template v-if="s">
      <BarsRow :s="s" />
      <MetronomeRow :s="s" />
      <TransportRow :s="s" />
      <NotationView :s="s" />
      <KitPanel :s="s" />
    </template>
    <p v-if="state && !state.error" class="status">
      {{ state.device }} · {{ state.rate }} Hz ·
      {{ Math.round(state.latencyMs) }} ms · {{ state.xruns }} xruns
    </p>
  </main>
</template>

<style>
:root {
  --bg: rgb(4, 5, 11);
  --text: rgb(203, 213, 225);
  --muted: rgb(100, 116, 139);
  --button: rgba(30, 41, 59, 0.6);
  --selected: rgb(203, 213, 225);
  --on: rgb(245, 158, 11);
  --start: rgb(22, 163, 74);
  --stop: rgb(185, 28, 28);
  --drums: rgb(148, 163, 184);
  --hi-hat: rgb(34, 211, 238);
  --snare: rgb(226, 232, 240);
  --kick: rgb(251, 146, 60);
  --crash: rgb(250, 204, 21);
  font-family: system-ui, sans-serif;
  color: var(--text);
  background: var(--bg);
}

body {
  margin: 0;
  background: var(--bg);
  user-select: none;
  -webkit-tap-highlight-color: transparent;
}

main {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  padding: env(safe-area-inset-top) env(safe-area-inset-right)
    env(safe-area-inset-bottom) env(safe-area-inset-left);
  box-sizing: border-box;
}

button {
  min-width: 40px;
  min-height: 40px;
  padding: 0 12px;
  border: none;
  border-radius: 6px;
  background: var(--button);
  color: var(--text);
  font: inherit;
  font-size: 15px;
}

button.selected {
  background: var(--selected);
  color: var(--bg);
}

button.on {
  background: var(--on);
  color: var(--bg);
}

.row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  background: rgba(255, 255, 255, 0.03);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.square {
  width: 40px;
  padding: 0;
}

.wide {
  min-width: 104px;
}

.caption {
  font-size: 12px;
  color: var(--muted);
}

.status {
  margin: 2px 8px;
  font-size: 11px;
  color: var(--muted);
}

.error {
  margin: 0;
  padding: 8px;
  background: rgb(127, 29, 29);
  color: white;
}
</style>
