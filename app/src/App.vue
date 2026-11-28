<script setup lang="ts">
// The one screen. Portrait: top to bottom as the spec lists it.
// Landscape: the notation and the kit on the left, the other controls on
// the right.
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
    <div v-if="s" class="screen">
      <div class="area-controls">
        <BarsRow :s="s" />
        <MetronomeRow :s="s" />
        <TransportRow :s="s" />
      </div>
      <NotationView class="area-notation" :s="s" />
      <KitPanel class="area-kit" :s="s" />
    </div>
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

.screen {
  flex: 1;
  display: flex;
  flex-direction: column;
}

/*
 * On its side a phone is too short for the rows one under another: the
 * notation and the kit share the left half, the other rows the right.
 * The notation takes what height is left, but never less than one bar's
 * worth: a tall menu scrolls inside the kit instead. The page itself never
 * scrolls, or it would slide under the status bar.
 */
@media (orientation: landscape) and (min-width: 600px) {
  main {
    height: 100vh;
    overflow: hidden;
  }

  .screen {
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    grid-template-rows: minmax(170px, 1fr) minmax(0, auto);
    grid-template-areas:
      "notation controls"
      "kit controls";
  }

  .area-controls {
    grid-area: controls;
    min-height: 0;
    overflow-y: auto;
  }

  .area-controls .row {
    padding: 4px 8px;
  }

  .area-notation {
    grid-area: notation;
    min-height: 0;
    overflow-y: auto;
  }

  .area-notation,
  .area-kit {
    border-right: 1px solid rgba(255, 255, 255, 0.08);
  }

  .area-kit {
    grid-area: kit;
    min-height: 0;
    overflow-y: auto;
  }

  /* The developer's readout: no room for it on its side. */
  .status {
    display: none;
  }
}

/* The developer's readout: on a phone it costs a row the screen lacks. */
@media (pointer: coarse) {
  .status {
    display: none;
  }
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
