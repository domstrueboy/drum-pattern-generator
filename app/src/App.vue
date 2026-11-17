<script setup lang="ts">
// Step 1 spike: renders what Rust reports and sends commands. No timing
// here - the playhead is Rust's play position, drawn as it comes.
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface Info {
  ok: boolean;
  error: string;
  device: string;
  sample_rate: number;
  channels: number;
  lanes: number[];
  bpm: number;
}
interface Status {
  playing: boolean;
  phase: number;
  latency_ms: number;
  xruns: number;
}

const LANES = ["Kick", "Snare", "Hi-hat", "Sample"];
const STEPS = 16;

const info = ref<Info | null>(null);
const status = ref<Status | null>(null);
const lanes = ref<number[]>([0, 0, 0, 0]);
const bpm = ref(120);
const sample = ref("");
const message = ref("");
let raf = 0;
let alive = true;

async function poll() {
  if (!alive) return;
  try {
    status.value = await invoke<Status | null>("status");
  } catch (e) {
    message.value = String(e);
  }
  raf = requestAnimationFrame(poll);
}

onMounted(async () => {
  info.value = await invoke<Info>("info");
  if (info.value.ok) {
    lanes.value = info.value.lanes;
    bpm.value = info.value.bpm;
  }
  poll();
});
onUnmounted(() => {
  alive = false;
  cancelAnimationFrame(raf);
});

async function run(cmd: string, args: Record<string, unknown>) {
  try {
    await invoke(cmd, args);
  } catch (e) {
    message.value = String(e);
  }
}

function toggle() {
  run("set_playing", { playing: !status.value?.playing });
}

function changeBpm(delta: number) {
  bpm.value = Math.min(300, Math.max(30, bpm.value + delta));
  run("set_bpm", { bpm: bpm.value });
}

function toggleStep(lane: number, step: number) {
  lanes.value[lane] ^= 1 << step;
  run("set_lane", { lane, steps: lanes.value[lane] });
}

function isOn(lane: number, step: number) {
  return (lanes.value[lane] & (1 << step)) !== 0;
}

async function pickSample() {
  try {
    const name = await invoke<string>("pick_sample");
    if (name) sample.value = name;
  } catch (e) {
    message.value = String(e);
  }
}

function currentStep() {
  const s = status.value;
  return s?.playing ? Math.floor(s.phase * STEPS) % STEPS : -1;
}
</script>

<template>
  <main>
    <header>
      <h1>Spike</h1>
      <p v-if="info && !info.ok" class="error">Audio failed: {{ info.error }}</p>
      <p v-else-if="info" class="meta">
        {{ info.device }} · {{ info.sample_rate }} Hz · {{ info.channels }} ch ·
        Latency {{ status?.latency_ms.toFixed(1) ?? "-" }} ms · Xruns {{ status?.xruns ?? 0 }}
      </p>
    </header>

    <section class="transport">
      <button class="small" @click="changeBpm(-1)">−</button>
      <div class="bpm">
        <span>{{ bpm }}</span>
        <small>BPM</small>
      </div>
      <button class="small" @click="changeBpm(1)">+</button>
      <button class="start" :class="{ on: status?.playing }" @click="toggle">
        {{ status?.playing ? "Stop" : "Start" }}
      </button>
    </section>

    <section class="grid">
      <div
        v-if="status?.playing"
        class="playhead"
        :style="{ left: `calc(var(--label) + (100% - var(--label)) * ${status.phase})` }"
      />
      <div v-for="(name, lane) in LANES" :key="name" class="lane">
        <span class="label">{{ name }}</span>
        <button
          v-for="step in STEPS"
          :key="step"
          class="cell"
          :class="{ on: isOn(lane, step - 1), beat: (step - 1) % 4 === 0, now: currentStep() === step - 1 }"
          @click="toggleStep(lane, step - 1)"
        />
      </div>
    </section>

    <section class="pads">
      <button v-for="(name, lane) in LANES" :key="name" class="pad" @pointerdown="run('hit', { lane })">
        {{ name }}
      </button>
    </section>

    <section class="file">
      <button @click="pickSample">Load WAV</button>
      <span>{{ sample || "Sample: built-in blip" }}</span>
    </section>

    <p v-if="message" class="error" @click="message = ''">{{ message }}</p>
  </main>
</template>

<style>
:root {
  font-family: system-ui, sans-serif;
  color: #111;
  background: #f4f1ea;
  -webkit-tap-highlight-color: transparent;
  user-select: none;
}
body {
  margin: 0;
  background: #f4f1ea;
}
main {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 720px;
  margin: 0 auto;
}
h1 {
  margin: 0;
  font-size: 22px;
}
.meta {
  margin: 4px 0 0;
  font-size: 13px;
  color: #555;
}
.error {
  color: #b00020;
  font-size: 14px;
}
button {
  font: inherit;
  font-weight: 600;
  min-height: 44px;
  border: 1px solid #bbb;
  border-radius: 10px;
  background: #fff;
  touch-action: manipulation;
}
.transport {
  display: flex;
  align-items: center;
  gap: 10px;
}
.small {
  width: 48px;
  font-size: 22px;
}
.bpm {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 56px;
}
.bpm span {
  font-size: 28px;
  font-weight: 700;
}
.bpm small {
  color: #666;
}
.start {
  margin-left: auto;
  min-width: 120px;
  min-height: 60px;
  font-size: 20px;
}
.start.on {
  background: #f5c400;
}
.grid {
  --label: 64px;
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.lane {
  display: flex;
  align-items: center;
  gap: 2px;
}
.label {
  width: var(--label);
  flex: none;
  font-size: 14px;
  font-weight: 600;
}
.cell {
  flex: 1;
  min-width: 0;
  min-height: 40px;
  padding: 0;
  border-radius: 4px;
  border-color: #ddd;
  background: #fff;
}
.cell.beat {
  border-color: #999;
}
.cell.on {
  background: #333;
}
.cell.now {
  outline: 2px solid #f5c400;
}
.playhead {
  position: absolute;
  top: -4px;
  bottom: -4px;
  width: 2px;
  background: #e03000;
  pointer-events: none;
  z-index: 1;
}
.pads {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}
.pad {
  min-height: 72px;
  font-size: 16px;
}
.pad:active {
  background: #f5c400;
}
.file {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 14px;
}
.file button {
  padding: 0 16px;
  flex: none;
}
</style>
