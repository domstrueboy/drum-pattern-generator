<script setup lang="ts">
// A count with - and +, kept between `min` and `max`.
import { useLabels } from "../labels";

const props = defineProps<{ value: number; min: number; max: number; label: string }>();
const emit = defineEmits<{ change: [count: number] }>();
const t = useLabels();

function by(step: number) {
  const next = props.value + step;
  if (next >= props.min && next <= props.max) emit("change", next);
}
</script>

<template>
  <div class="group stepper" role="group" :aria-label="label">
    <button class="square" :aria-label="t.fewer" :disabled="value <= min" @click="by(-1)">
      –
    </button>
    <span class="count">{{ value }}</span>
    <button class="square" :aria-label="t.plenty" :disabled="value >= max" @click="by(1)">
      +
    </button>
  </div>
</template>

<style scoped>
.count {
  min-width: 32px;
  text-align: center;
  font-family: ui-monospace, monospace;
  font-size: 18px;
}

button:disabled {
  opacity: 0.35;
}
</style>
