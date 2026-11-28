<script setup lang="ts">
// Hi-hat: the pulse in three columns (straight, triplet, dotted), Random
// with its count of variants, and Density for Random.
import { useLabels } from "../labels";
import type { Pulse, SessionState } from "../session";
import { useSession } from "../useSession";
import CountStepper from "./CountStepper.vue";
import MenuPanel from "./MenuPanel.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();

// Row by row, so the columns read 4 8 16 | 4т 8т 16т | 4. 8.
const pulses: (Pulse | null)[] = ["4", "4t", "4.", "8", "8t", "8.", "16", "16t", null];

function density(event: Event) {
  send({ type: "density", percent: Number((event.target as HTMLInputElement).value) });
}
</script>

<template>
  <MenuPanel :title="t.instruments.hiHat" color="var(--hi-hat)">
    <div class="field">
      <span class="caption">{{ t.pulse }}</span>
      <div class="pulses">
        <template v-for="(p, i) in pulses" :key="i">
          <button
            v-if="p"
            class="square"
            :class="{ selected: s.hiHat.pulse === p }"
            @click="send({ type: 'pulse', value: p })"
          >
            {{ t.pulses[p] }}
          </button>
          <span v-else></span>
        </template>
      </div>
    </div>
    <div class="field">
      <span class="caption">{{ t.random }}</span>
      <div class="options">
        <button
          class="wide"
          :class="{ on: s.hiHat.random }"
          @click="send({ type: 'hiHatRandom' })"
        >
          {{ t.random }}
          <template v-if="s.hiHat.random">
            {{ s.hiHat.randomIndex + 1 }}/{{ s.hiHat.randomCount }}
          </template>
        </button>
        <CountStepper
          :value="s.hiHat.randomCount"
          :min="1"
          :max="s.limits.variants"
          :label="t.variants"
          @change="(count) => send({ type: 'hiHatRandomCount', count })"
        />
      </div>
    </div>
    <label class="field">
      <span class="caption">{{ t.density }} {{ s.hiHat.density }}%</span>
      <input
        class="density"
        type="range"
        min="0"
        max="100"
        step="5"
        :value="s.hiHat.density"
        @change="density"
      />
    </label>
  </MenuPanel>
</template>

<style scoped>
.pulses {
  display: grid;
  grid-template-columns: repeat(3, 48px);
  gap: 6px;
}

.pulses button {
  width: 48px;
}

.density {
  height: 40px;
  margin: 0;
  accent-color: var(--hi-hat);
}
</style>
