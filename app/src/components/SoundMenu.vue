<script setup lang="ts">
// Sound: a kit style for each of the hi-hat, snare and kick, switchable
// while playing. The crash follows the hi-hat.
import { useLabels } from "../labels";
import type { DrumInstrument, Preset, SessionState } from "../session";
import { useSession } from "../useSession";
import MenuPanel from "./MenuPanel.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();
const instruments: DrumInstrument[] = ["hiHat", "snare", "kick"];
const presets: Preset[] = ["rock", "hipHop", "electro", "jazz", "metal"];
const colors: Record<DrumInstrument, string> = {
  hiHat: "var(--hi-hat)",
  snare: "var(--snare)",
  kick: "var(--kick)",
};
</script>

<template>
  <MenuPanel :title="t.sound">
    <div v-for="i in instruments" :key="i" class="field">
      <span class="caption name" :style="{ color: colors[i] }">{{ t.instruments[i] }}</span>
      <div class="options">
        <button
          v-for="p in presets"
          :key="p"
          :class="{ selected: s.sounds[i] === p }"
          @click="send({ type: 'soundPreset', instrument: i, preset: p })"
        >
          {{ t.presets[p] }}
        </button>
      </div>
    </div>
    <p class="caption note">{{ t.crashFollows }}</p>
  </MenuPanel>
</template>

<style scoped>
.name {
  font-weight: 600;
}

.options button {
  padding: 0 10px;
}

.note {
  margin: 0;
}
</style>
