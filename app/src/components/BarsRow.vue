<script setup lang="ts">
// Bars: the pattern's length, stacked or scrolled, Bar skip and the crash.
// Bar skip is a toggle: off, it opens its menu; on, it turns off.
import { computed } from "vue";
import { useLabels } from "../labels";
import type { SessionState } from "../session";
import { useMenu } from "../useMenu";
import { useSession } from "../useSession";

const props = defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();
const { menu, open } = useMenu();

// The button names the chosen variant once Bar skip is on.
const barSkip = computed(() =>
  props.s.barSkip ? `${props.s.barSkip.play}-${props.s.barSkip.skip}` : t.barSkip,
);

function barSkipPressed() {
  if (props.s.barSkip) send({ type: "barSkipOff" });
  else open(menu.value === "barSkip" ? null : "barSkip");
}

function crashPressed() {
  open(menu.value === "crash" ? null : "crash");
}
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
    <button
      class="wide"
      :class="{ on: s.barSkip, selected: menu === 'barSkip' }"
      @click="barSkipPressed"
    >
      {{ barSkip }}
    </button>
    <button :class="{ selected: menu === 'crash' }" @click="crashPressed">
      {{ t.crash }}
    </button>
  </div>
</template>
