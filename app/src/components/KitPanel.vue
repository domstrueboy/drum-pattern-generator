<script setup lang="ts">
// The kit: Hi-hat, Snare and Kick, each level with its drum, with a
// fader, S and M; the photo beside them (step 7). The instrument buttons
// open their submenus in step 5f and never move.
import { useLabels } from "../labels";
import type { DrumInstrument, SessionState } from "../session";
import { useSession } from "../useSession";
import LevelFader from "./LevelFader.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();
const instruments: DrumInstrument[] = ["hiHat", "snare", "kick"];
const colors: Record<DrumInstrument, string> = {
  hiHat: "var(--hi-hat)",
  snare: "var(--snare)",
  kick: "var(--kick)",
};
</script>

<template>
  <div class="kit">
    <div class="strips">
      <div v-for="i in instruments" :key="i" class="strip">
        <button class="name" :style="{ color: colors[i], borderColor: colors[i] }">
          {{ t.instruments[i] }}
        </button>
        <LevelFader
          :value="s.mixer[i].level"
          :color="colors[i]"
          :label="t.instruments[i]"
          @change="(position) => send({ type: 'fader', fader: i, position })"
        />
        <div class="group">
          <button
            class="square"
            :class="{ on: s.mixer[i].solo }"
            :aria-label="t.solo"
            :title="t.solo"
            @click="send({ type: 'solo', instrument: i })"
          >
            S
          </button>
          <button
            class="square"
            :class="{ muted: s.mixer[i].mute }"
            :aria-label="t.mute"
            :title="t.mute"
            @click="send({ type: 'mute', instrument: i })"
          >
            M
          </button>
        </div>
      </div>
    </div>
    <div class="photo" :aria-label="t.kit"></div>
  </div>
</template>

<style scoped>
.kit {
  flex: 1;
  display: flex;
  min-height: 180px;
  background: rgba(0, 0, 0, 0.3);
}

.strips {
  width: 62%;
  display: flex;
  flex-direction: column;
  justify-content: space-around;
  gap: 8px;
  padding: 8px;
}

.strip {
  display: flex;
  align-items: center;
  gap: 6px;
}

.name {
  width: 76px;
  padding: 0 6px;
  text-align: left;
  font-size: 13px;
  font-weight: 600;
  border: 1px solid;
  border-color: currentColor;
  background: var(--button);
}

.muted {
  background: var(--stop);
  color: white;
}

.photo {
  flex: 1;
  margin: 8px 8px 8px 0;
  border-radius: 8px;
  background:
    radial-gradient(transparent 40%, rgba(0, 0, 0, 0.5) 100%),
    linear-gradient(160deg, rgb(30, 34, 48), rgb(12, 14, 22));
}
</style>
