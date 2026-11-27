import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Command, Snapshot } from "./session";

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
