<script setup lang="ts">
// Sound: the kit's style, every instrument at once, switchable while
// playing. Kits made from your own samples will join the list (step 6e).
import { useLabels } from "../labels";
import type { Preset, SessionState } from "../session";
import { useSession } from "../useSession";
import MenuPanel from "./MenuPanel.vue";

defineProps<{ s: SessionState }>();
const { send } = useSession();
const t = useLabels();
const presets: Preset[] = ["rock", "hipHop", "electro", "jazz", "metal"];
</script>

<template>
  <MenuPanel :title="t.sound">
    <div class="field">
      <span class="caption">{{ t.kits }}</span>
      <div class="options">
        <button
          v-for="p in presets"
          :key="p"
          :class="{ selected: s.kit === p }"
          @click="send({ type: 'soundPreset', preset: p })"
        >
          {{ t.presets[p] }}
        </button>
      </div>
    </div>
  </MenuPanel>
</template>
