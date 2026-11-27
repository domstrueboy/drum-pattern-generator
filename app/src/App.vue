<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

/** What Rust reports; the screen only renders it. */
interface Snapshot {
  error: string | null;
  playing: boolean;
  bpm: number;
  device: string;
  rate: number;
  latencyMs: number;
  xruns: number;
}

const state = ref<Snapshot | null>(null);

async function send(command: string, args?: Record<string, unknown>) {
  state.value = await invoke<Snapshot>(command, args);
}

function nudgeBpm(by: number) {
  if (state.value) send("set_bpm", { bpm: state.value.bpm + by });
}

// Refreshes the latency and xrun readout only; nothing here keeps time.
let refresh: number | undefined;
onMounted(() => {
  send("snapshot");
  refresh = window.setInterval(() => send("snapshot"), 1000);
});
onUnmounted(() => window.clearInterval(refresh));
</script>

<template>
  <main>
    <p v-if="state?.error" class="error">{{ state.error }}</p>

    <div class="bpm">
      <button @click="nudgeBpm(-1)">–</button>
      <span>{{ state ? Math.round(state.bpm) : "" }}</span>
      <button @click="nudgeBpm(1)">+</button>
      <small>BPM</small>
    </div>

    <button
      class="start"
      :class="{ playing: state?.playing }"
      @click="send('start_stop')"
    >
      {{ state?.playing ? "Stop" : "Start" }}
    </button>

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
  --on: rgb(245, 158, 11);
  font-family: system-ui, sans-serif;
  color: var(--text);
  background: var(--bg);
}

body {
  margin: 0;
  background: var(--bg);
  user-select: none;
}

main {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
}

button {
  min-width: 40px;
  min-height: 40px;
  border: none;
  border-radius: 6px;
  background: var(--button);
  color: var(--text);
  font-size: 18px;
}

.bpm {
  display: flex;
  align-items: center;
  gap: 8px;
}

.bpm span {
  width: 56px;
  text-align: center;
  font-family: ui-monospace, monospace;
  font-size: 24px;
}

.bpm small,
.status {
  color: var(--muted);
}

.start {
  width: 160px;
  height: 64px;
  font-size: 22px;
  font-weight: 600;
}

.start.playing {
  background: var(--on);
  color: var(--bg);
}

.status {
  font-size: 12px;
}

.error {
  color: rgb(248, 113, 113);
}
</style>
