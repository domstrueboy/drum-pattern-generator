<script setup lang="ts">
// The notation: the lap playing, a bar a row (stacked) or all bars in one
// row (scrolled), with the count above, the note values over the notes,
// the konnakol under them, and the playhead. The lap follows the
// playhead, so a dotted hi-hat is redrawn as each lap comes round.
import { computed, nextTick, onMounted, onUnmounted, ref, shallowRef, watch } from "vue";
import { useLabels } from "../labels";
import { drawBar, type BarLayout } from "../notation/drawBar";
import type { SessionState } from "../session";
import { usePlayhead } from "../useSession";

const props = defineProps<{ s: SessionState }>();
const t = useLabels();
const playhead = usePlayhead(() => props.s.playing);

const root = ref<HTMLElement | null>(null);
const staffs = ref<HTMLElement[]>([]);
const width = ref(0);
const layouts = shallowRef<BarLayout[]>([]);

const lap = computed(() => {
  const n = playhead.value?.lap ?? 0;
  return props.s.notation[n < props.s.notation.length ? n : 0];
});
const barWidth = computed(() => width.value);

// Bravura is bundled; VexFlow measures with it, so it must load first.
const fonts = document.fonts.load("30px Bravura");

let observer: ResizeObserver | undefined;
onMounted(() => {
  observer = new ResizeObserver(([entry]) => {
    width.value = Math.floor(entry.contentRect.width);
  });
  if (root.value) observer.observe(root.value);
});
onUnmounted(() => observer?.disconnect());

// A new snapshot every second carries the same notation; draw again only
// when what's drawn would change.
const drawn = computed(() =>
  JSON.stringify([lap.value, barWidth.value, props.s.view]),
);
watch(
  drawn,
  async () => {
    await fonts;
    await nextTick();
    const notation = lap.value;
    if (!notation || barWidth.value === 0) return;
    const { ticksPerBar, ticksPerBeat } = props.s;
    layouts.value = notation.bars.map((groups, i) =>
      drawBar(
        staffs.value[i],
        groups,
        i * ticksPerBar,
        ticksPerBar,
        ticksPerBeat,
        barWidth.value,
      ),
    );
  },
  { immediate: true },
);

function inBar(tick: number, bar: number) {
  return Math.floor(tick / props.s.ticksPerBar) === bar;
}

const countRows = computed(() =>
  layouts.value.map((layout, bar) =>
    lap.value.count
      .filter((c) => inBar(c.tick, bar))
      .map((c) => ({
        x: layout.xAt(c.tick),
        label: t.letters[c.label] ?? c.label,
        beat: c.label !== "и",
      })),
  ),
);

const playheadAt = computed(() => {
  const p = playhead.value;
  if (!p) return null;
  const bar = Math.floor(p.tick / props.s.ticksPerBar);
  const layout = layouts.value[bar];
  return layout ? { bar, x: layout.xAt(p.tick) } : null;
});

// Scrolled: keep the bar being played in view.
watch(
  () => playheadAt.value?.bar,
  (bar) => {
    if (bar !== undefined && props.s.view === "scrolled") {
      root.value?.scrollTo({ left: bar * barWidth.value, behavior: "smooth" });
    }
  },
);

function value(label: string) {
  return label.replace("t", t.triplet);
}
</script>

<template>
  <section ref="root" class="notation" :class="s.view" :aria-label="t.notation">
    <div class="bars">
      <div
        v-for="(_, bar) in lap?.bars ?? []"
        :key="bar"
        class="bar"
        :style="{ width: `${barWidth}px` }"
      >
        <div class="line count">
          <span
            v-for="c in countRows[bar] ?? []"
            :key="c.x"
            :style="{ left: `${c.x}px` }"
            :class="{ beat: c.beat }"
          >
            {{ c.label }}
          </span>
        </div>
        <div class="line values">
          <template v-for="n in layouts[bar]?.notes ?? []" :key="n.tick">
            <span v-if="n.event.label" :style="{ left: `${n.x}px` }">
              {{ value(n.event.label) }}
            </span>
          </template>
        </div>
        <div
          :ref="(el) => (staffs[bar] = el as HTMLElement)"
          class="staff"
        ></div>
        <div class="line konnakol">
          <template v-for="n in layouts[bar]?.notes ?? []" :key="n.tick">
            <span
              v-if="n.event.konnakol"
              :class="{ ta: n.event.konnakol === 'т' }"
              :style="{ left: `${n.x}px` }"
            >
              {{ t.letters[n.event.konnakol] ?? n.event.konnakol }}
            </span>
          </template>
        </div>
        <div
          v-if="playheadAt?.bar === bar"
          class="playhead"
          :style="{ transform: `translateX(${playheadAt.x}px)` }"
        ></div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.notation {
  background: rgba(0, 0, 0, 0.35);
  overflow-x: hidden;
}

.notation.scrolled {
  overflow-x: auto;
  scroll-snap-type: x mandatory;
}

.bars {
  display: flex;
  flex-direction: column;
}

.scrolled .bars {
  flex-direction: row;
  width: max-content;
}

.bar {
  position: relative;
  flex: none;
  scroll-snap-align: start;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.scrolled .bar {
  border-bottom: none;
  border-right: 1px solid rgba(255, 255, 255, 0.06);
}

.line {
  position: relative;
  font-weight: 700;
}

.line span {
  position: absolute;
  transform: translateX(-50%);
  white-space: nowrap;
}

.count {
  height: 22px;
  font-size: 20px;
  line-height: 22px;
  color: var(--muted);
}

.count .beat {
  color: var(--snare);
}

.values {
  height: 18px;
  font-size: 13px;
  line-height: 18px;
  color: var(--muted);
}

.staff {
  line-height: 0;
}

.konnakol {
  height: 22px;
  font-size: 20px;
  line-height: 20px;
  color: var(--muted);
}

.konnakol .ta {
  color: var(--snare);
}

.playhead {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 2px;
  margin-left: -1px;
  background: var(--on);
  opacity: 0.8;
  pointer-events: none;
  will-change: transform;
}
</style>
