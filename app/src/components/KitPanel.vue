<script setup lang="ts">
// The kit: Hi-hat, Snare and Kick, each level with its drum, with a
// fader, S and M; the photo beside them (step 7). A submenu opens beside
// the instrument buttons, over the faders and the photo; the buttons
// themselves never move.
import { nextTick, ref, watch } from "vue";
import { useLabels } from "../labels";
import type { DrumInstrument, SessionState } from "../session";
import { useMenu } from "../useMenu";
import { useSession } from "../useSession";
import BarSkipMenu from "./BarSkipMenu.vue";
import CrashMenu from "./CrashMenu.vue";
import HiHatMenu from "./HiHatMenu.vue";
import KickMenu from "./KickMenu.vue";
import LevelFader from "./LevelFader.vue";
import SnareMenu from "./SnareMenu.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const { menu, open } = useMenu();
const t = useLabels();
const instruments: DrumInstrument[] = ["hiHat", "snare", "kick"];
const colors: Record<DrumInstrument, string> = {
  hiHat: "var(--hi-hat)",
  snare: "var(--snare)",
  kick: "var(--kick)",
};

// Bar skip and the crash open their menus from the top of the screen:
// bring the menu into view, or the press seems to do nothing.
const panel = ref<HTMLElement | null>(null);
watch(menu, async (which) => {
  if (!which) return;
  await nextTick();
  panel.value?.scrollIntoView({ block: "nearest", behavior: "smooth" });
});

function toggle(i: DrumInstrument) {
  open(menu.value === i ? null : i);
}
</script>

<template>
  <div class="kit">
    <button
      v-for="(i, row) in instruments"
      :key="i"
      class="name"
      :class="{ open: menu === i }"
      :style="{ color: colors[i], borderColor: colors[i], gridRow: row + 1 }"
      @click="toggle(i)"
    >
      {{ t.instruments[i] }}
    </button>

    <template v-if="!menu">
      <div v-for="(i, row) in instruments" :key="i" class="mix" :style="{ gridRow: row + 1 }">
        <LevelFader
          :value="s.mixer[i].level"
          :color="colors[i]"
          :label="t.instruments[i]"
          @change="(position) => send({ type: 'fader', fader: i, position })"
        />
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
      <div class="photo" :aria-label="t.kit"></div>
    </template>

    <div v-else ref="panel" class="menu">
      <HiHatMenu v-if="menu === 'hiHat'" :s="s" />
      <SnareMenu v-else-if="menu === 'snare'" :s="s" />
      <KickMenu v-else-if="menu === 'kick'" :s="s" />
      <BarSkipMenu v-else-if="menu === 'barSkip'" :s="s" />
      <CrashMenu v-else :s="s" />
    </div>
  </div>
</template>

<style scoped>
/*
 * Three fixed rows, one an instrument, and a last one that takes what a
 * tall menu needs - so the buttons stay where they are.
 */
.kit {
  flex: 1;
  display: grid;
  grid-template-columns: 84px minmax(0, 1.4fr) minmax(0, 1fr);
  grid-template-rows: repeat(3, 56px) auto;
  column-gap: 8px;
  align-items: center;
  padding: 8px;
  min-height: 200px;
  background: rgba(0, 0, 0, 0.3);
}

.name {
  grid-column: 1;
  width: 84px;
  padding: 0 6px;
  text-align: left;
  font-size: 13px;
  font-weight: 600;
  border: 1px solid;
  background: var(--button);
}

.name.open {
  background: rgba(255, 255, 255, 0.12);
  box-shadow: inset 4px 0 0 currentColor;
}

.mix {
  grid-column: 2;
  display: flex;
  align-items: center;
  gap: 6px;
}

.muted {
  background: var(--stop);
  color: white;
}

.photo {
  grid-column: 3;
  grid-row: 1 / 5;
  align-self: stretch;
  border-radius: 8px;
  background:
    radial-gradient(transparent 40%, rgba(0, 0, 0, 0.5) 100%),
    linear-gradient(160deg, rgb(30, 34, 48), rgb(12, 14, 22));
}

.menu {
  grid-column: 2 / 4;
  grid-row: 1 / 5;
  align-self: start;
  min-width: 0;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.04);
}
</style>
