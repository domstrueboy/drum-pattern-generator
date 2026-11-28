<script setup lang="ts">
// Metronome: on/off, BPM by 1, clicks a beat, and its sound - each press
// of the sound button steps to the next.
import { useLabels } from "../labels";
import type { SessionState, Subdivision } from "../session";
import { useSession } from "../useSession";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();
const subdivisions: Subdivision[] = ["4", "8", "8t", "16"];
</script>

<template>
  <div class="row">
    <button
      class="wide"
      :class="{ on: s.metronome }"
      @click="send({ type: 'metronome' })"
    >
      {{ t.metronome }}
    </button>
    <div class="group bpm">
      <button
        class="square"
        :aria-label="t.less"
        @click="send({ type: 'nudgeBpm', by: -1 })"
      >
        –
      </button>
      <span class="value">{{ Math.round(s.bpm) }}</span>
      <button
        class="square"
        :aria-label="t.more"
        @click="send({ type: 'nudgeBpm', by: 1 })"
      >
        +
      </button>
      <span class="caption">{{ t.bpm }}</span>
    </div>
    <div class="group">
      <button
        v-for="value in subdivisions"
        :key="value"
        class="square"
        :class="{ on: s.subdivision === value }"
        @click="send({ type: 'subdivision', value })"
      >
        {{ t.subdivisions[value] }}
      </button>
    </div>
    <button class="wide" @click="send({ type: 'metronomeSound' })">
      🔊 {{ t.clickSounds[s.metronomeSound] }}
    </button>
  </div>
</template>

<style scoped>
.value {
  width: 48px;
  text-align: center;
  font-family: ui-monospace, monospace;
  font-size: 20px;
}
</style>
