<script setup lang="ts">
// Snare: the time feel, accents a bar, ghost notes with Generate (harder
// each press) and Random.
import { useLabels } from "../labels";
import type { Feel, SessionState } from "../session";
import { useSession } from "../useSession";
import CountStepper from "./CountStepper.vue";
import MenuPanel from "./MenuPanel.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();
const feels: Feel[] = ["basic", "half", "double"];
const accents = [0, 1, 2, 3, 4];
</script>

<template>
  <MenuPanel :title="t.instruments.snare" color="var(--snare)">
    <div class="field">
      <span class="caption">{{ t.feel }}</span>
      <div class="options">
        <button
          v-for="f in feels"
          :key="f"
          :class="{ selected: s.snare.feel === f }"
          @click="send({ type: 'feel', feel: f })"
        >
          {{ t.feels[f] }}
        </button>
      </div>
    </div>
    <div class="field">
      <span class="caption">{{ t.accents }}</span>
      <div class="options">
        <button
          v-for="n in accents"
          :key="n"
          :class="{ selected: s.snare.accents === n, square: n > 0 }"
          @click="send({ type: 'accents', count: n })"
        >
          {{ n === 0 ? t.none : n }}
        </button>
      </div>
    </div>
    <div class="field">
      <span class="caption">{{ t.ghosts }}</span>
      <div class="options">
        <CountStepper
          :value="s.snare.ghosts"
          :min="0"
          :max="s.limits.count"
          :label="t.ghosts"
          @change="(count) => send({ type: 'ghosts', count })"
        />
        <button :disabled="s.snare.ghosts === 0" @click="send({ type: 'generateGhosts' })">
          {{ t.generate }}
        </button>
        <span v-if="s.snare.ghosts > 0" class="level">
          {{ t.level }} {{ s.snare.ghostLevel }}/{{ s.limits.ghostLevels }}
        </span>
      </div>
      <div class="options">
        <button
          class="wide"
          :disabled="s.snare.ghosts === 0"
          @click="send({ type: 'ghostRandom' })"
        >
          {{ t.random }} {{ s.snare.ghostRandomIndex + 1 }}/{{ s.snare.ghostRandomCount }}
        </button>
        <CountStepper
          :value="s.snare.ghostRandomCount"
          :min="1"
          :max="s.limits.variants"
          :label="t.variants"
          @change="(count) => send({ type: 'ghostRandomCount', count })"
        />
      </div>
    </div>
  </MenuPanel>
</template>

<style scoped>
button:disabled {
  opacity: 0.35;
}
</style>
