<script setup lang="ts">
// Bars: the pattern's length, stacked or scrolled, and Bar skip.
import { computed } from "vue";
import { useLabels } from "../labels";
import type { SessionState } from "../session";
import { useSession } from "../useSession";

const props = defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();

// The button names the chosen variant once Bar skip is on (its menu is
// step 5f).
const barSkip = computed(() =>
  props.s.barSkip ? `${props.s.barSkip.play}-${props.s.barSkip.skip}` : t.barSkip,
);
</script>

<template>
  <div class="row">
    <span class="caption">{{ t.bars }}</span>
    <div class="group">
      <button
        v-for="n in 4"
        :key="n"
        class="square"
        :class="{ selected: s.bars === n }"
        @click="send({ type: 'bars', count: n })"
      >
        {{ n }}
      </button>
    </div>
    <div class="group">
      <button
        class="square"
        :class="{ selected: s.view === 'stacked' }"
        :aria-label="t.stacked"
        :title="t.stacked"
        @click="send({ type: 'view', view: 'stacked' })"
      >
        ≡
      </button>
      <button
        class="square"
        :class="{ selected: s.view === 'scrolled' }"
        :aria-label="t.scrolled"
        :title="t.scrolled"
        @click="send({ type: 'view', view: 'scrolled' })"
      >
        ↔
      </button>
    </div>
    <button class="wide" :class="{ on: s.barSkip }">{{ barSkip }}</button>
  </div>
</template>
