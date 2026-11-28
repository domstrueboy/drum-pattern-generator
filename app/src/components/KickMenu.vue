<script setup lang="ts">
// Kick: base kicks with Generate, extra kicks with Generate (harder each
// press).
import { useLabels } from "../labels";
import type { SessionState } from "../session";
import { useSession } from "../useSession";
import CountStepper from "./CountStepper.vue";
import MenuPanel from "./MenuPanel.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();
</script>

<template>
  <MenuPanel :title="t.instruments.kick" color="var(--kick)">
    <div class="field">
      <span class="caption">{{ t.baseKicks }}</span>
      <div class="options">
        <CountStepper
          :value="s.kick.base"
          :min="0"
          :max="s.limits.count"
          :label="t.baseKicks"
          @change="(count) => send({ type: 'baseKicks', count })"
        />
        <button :disabled="s.kick.base === 0" @click="send({ type: 'generateBaseKicks' })">
          {{ t.generate }}
        </button>
      </div>
    </div>
    <div class="field">
      <span class="caption">{{ t.extraKicks }}</span>
      <div class="options">
        <CountStepper
          :value="s.kick.extra"
          :min="0"
          :max="s.limits.count"
          :label="t.extraKicks"
          @change="(count) => send({ type: 'extraKicks', count })"
        />
        <button :disabled="s.kick.extra === 0" @click="send({ type: 'generateExtraKicks' })">
          {{ t.generate }}
        </button>
        <span v-if="s.kick.extra > 0" class="level">
          {{ t.level }} {{ s.kick.extraLevel }}/{{ s.limits.extraKickLevels }}
        </span>
      </div>
    </div>
  </MenuPanel>
</template>

<style scoped>
button:disabled {
  opacity: 0.35;
}
</style>
