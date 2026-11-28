import { onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Command, Playhead, Snapshot } from "./session";

// One snapshot for the whole screen: every component reads the same one.
const state = ref<Snapshot | null>(null);

async function send(command: Command) {
  state.value = await invoke<Snapshot>("send", { command });
}

async function refresh() {
  state.value = await invoke<Snapshot>("snapshot");
}

/** The session's latest snapshot, and a way to send it commands. */
export function useSession() {
  return { state, send };
}

/**
 * Fetches a snapshot every second while mounted: it brings the latency
 * and xruns up to date and lets the shell reopen a lost output. Nothing
 * here keeps time. Used once, by the app.
 */
export function useRefresh() {
  let timer: number | undefined;
  onMounted(() => {
    refresh();
    timer = window.setInterval(refresh, 1000);
  });
  onUnmounted(() => window.clearInterval(timer));
}

/**
 * The playhead, asked of Rust once a screen frame while `playing` is
 * true; `null` while stopped. Rust places it, latency and all; this only
 * fetches it.
 */
export function usePlayhead(playing: () => boolean) {
  const playhead = ref<Playhead | null>(null);
  let running = false;
  async function frame() {
    if (!running) return;
    const p = await invoke<Playhead | null>("playhead");
    if (!running) return;
    playhead.value = p;
    requestAnimationFrame(frame);
  }
  watch(
    playing,
    (on) => {
      running = on;
      if (on) requestAnimationFrame(frame);
      else playhead.value = null;
    },
    { immediate: true },
  );
  onUnmounted(() => (running = false));
  return playhead;
}
