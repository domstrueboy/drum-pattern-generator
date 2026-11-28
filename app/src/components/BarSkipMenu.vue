<script setup lang="ts">
// Bar skip's variants: play N bars, then M silent. Invert turns the first
// column round (1-1, 1-2, 1-3). A choice closes the menu.
import { computed, ref } from "vue";
import { useLabels } from "../labels";
import type { SessionState } from "../session";
import { useMenu } from "../useMenu";
import { useSession } from "../useSession";
import MenuPanel from "./MenuPanel.vue";

const props = defineProps<{ s: SessionState }>();
const { send } = useSession();
const { open } = useMenu();
const t = useLabels();

const inverted = ref(props.s.barSkip !== null && props.s.barSkip.play < props.s.barSkip.skip);
const first = computed(() =>
  [1, 2, 3].map((n) => (inverted.value ? [1, n] : [n, 1])),
);
const second = [2, 4, 8].map((n) => [n, n]);

function choose([play, skip]: number[]) {
  send({ type: "barSkip", play, skip });
  open(null);
}

function chosen([play, skip]: number[]) {
  return props.s.barSkip?.play === play && props.s.barSkip?.skip === skip;
}
</script>

<template>
  <MenuPanel :title="t.barSkip">
    <div class="columns">
      <div class="column">
        <button
          v-for="v in first"
          :key="v.join()"
          class="wide"
          :class="{ on: chosen(v) }"
          @click="choose(v)"
        >
          {{ v[0] }}-{{ v[1] }}
        </button>
        <button
          class="wide"
          :class="{ selected: inverted }"
          :aria-label="t.invert"
          :title="t.invert"
          @click="inverted = !inverted"
        >
          ⇄
        </button>
      </div>
      <div class="column">
        <button
          v-for="v in second"
          :key="v.join()"
          class="wide"
          :class="{ on: chosen(v) }"
          @click="choose(v)"
        >
          {{ v[0] }}-{{ v[1] }}
        </button>
      </div>
    </div>
  </MenuPanel>
</template>

<style scoped>
.columns {
  display: flex;
  gap: 8px;
}

.column {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.column .wide {
  min-width: 88px;
}
</style>
