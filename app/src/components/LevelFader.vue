<script setup lang="ts">
// A fader, 0 to 1, in its instrument's colour. Sends while dragged; shows
// what the session reports.
defineProps<{ value: number; color: string; label: string }>();
const emit = defineEmits<{ change: [position: number] }>();

function input(event: Event) {
  emit("change", Number((event.target as HTMLInputElement).value));
}
</script>

<template>
  <input
    class="fader"
    type="range"
    min="0"
    max="1"
    step="0.01"
    :value="value"
    :aria-label="label"
    :style="{ accentColor: color }"
    @input="input"
  />
</template>

<style scoped>
.fader {
  flex: 1;
  min-width: 64px;
  height: 40px;
  margin: 0;
}
</style>
