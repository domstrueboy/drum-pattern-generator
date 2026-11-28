<script setup lang="ts">
// Transport: Color, Sound and Settings on the left (Sound opens its menu;
// the others wait for steps 7-8), Tap tempo
// over Start/Stop in the centre, the Drums and Metronome faders right.
import { useLabels } from "../labels";
import type { SessionState } from "../session";
import { useMenu } from "../useMenu";
import { useSession } from "../useSession";
import LevelFader from "./LevelFader.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();
const { menu, open } = useMenu();
</script>

<template>
  <div class="row transport">
    <div class="menus">
      <button>{{ t.color }}</button>
      <button
        :class="{ selected: menu === 'sound' }"
        @click="open(menu === 'sound' ? null : 'sound')"
      >
        {{ t.sound }}
      </button>
      <button>{{ t.settings }}</button>
    </div>
    <div class="centre">
      <button class="tap" @click="send({ type: 'tap' })">{{ t.tapTempo }}</button>
      <button
        class="start"
        :class="{ playing: s.playing }"
        @click="send({ type: 'startStop' })"
      >
        {{ s.playing ? `■ ${t.stop}` : `▶ ${t.start}` }}
      </button>
    </div>
    <div class="faders">
      <label>
        <span class="caption">{{ t.drums }}</span>
        <LevelFader
          :value="s.mixer.drums"
          color="var(--drums)"
          :label="t.drums"
          @change="(position) => send({ type: 'fader', fader: 'drums', position })"
        />
      </label>
      <label>
        <span class="caption">{{ t.metronome }}</span>
        <LevelFader
          :value="s.mixer.metronome"
          color="var(--on)"
          :label="t.metronome"
          @change="(position) => send({ type: 'fader', fader: 'metronome', position })"
        />
      </label>
    </div>
  </div>
</template>

<style scoped>
.transport {
  flex-wrap: nowrap;
  align-items: stretch;
}

.menus,
.centre,
.faders {
  display: flex;
  flex-direction: column;
  gap: 4px;
  justify-content: center;
}

.menus button {
  font-size: 13px;
  padding: 0 8px;
}

.centre {
  flex: 1;
  align-items: center;
}

.tap {
  width: 100%;
  max-width: 140px;
  font-size: 14px;
}

.start {
  width: 100%;
  max-width: 140px;
  height: 48px;
  font-size: 18px;
  font-weight: 600;
  background: var(--start);
  color: white;
}

.start.playing {
  background: var(--stop);
}

.faders {
  width: 34%;
  max-width: 180px;
}

.faders label {
  display: flex;
  flex-direction: column;
}

.faders .caption {
  font-size: 11px;
  line-height: 1;
}
</style>
