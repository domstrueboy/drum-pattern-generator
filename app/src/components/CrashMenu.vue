<script setup lang="ts">
// Where the crash plays, in bars of playback: once at the start, or every
// квадрат (4 bars), 2 or 4 квадрата. A choice closes the menu.
import { useLabels } from "../labels";
import type { SessionState } from "../session";
import { useMenu } from "../useMenu";
import { useSession } from "../useSession";
import MenuPanel from "./MenuPanel.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const { open } = useMenu();
const t = useLabels();
const choices: (number | null)[] = [null, 4, 8, 16];

function choose(every: number | null) {
  send({ type: "crash", every });
  open(null);
}
</script>

<template>
  <MenuPanel :title="t.crash" color="var(--crash)">
    <div class="choices">
      <button
        v-for="every in choices"
        :key="every ?? 0"
        :class="{ selected: s.crashEvery === every }"
        @click="choose(every)"
      >
        {{ every === null ? t.crashOnce : t.crashEvery(every) }}
      </button>
    </div>
  </MenuPanel>
</template>

<style scoped>
.choices {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
}
</style>
