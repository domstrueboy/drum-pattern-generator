import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Command, Snapshot } from "./session";

/** The session's latest snapshot, and a way to send it commands. */
export function useSession() {
  const state = ref<Snapshot | null>(null);

  async function send(command: Command) {
    state.value = await invoke<Snapshot>("send", { command });
  }

  async function refresh() {
    state.value = await invoke<Snapshot>("snapshot");
  }

  // Refreshes the latency and xrun readout only; nothing here keeps time.
  let timer: number | undefined;
  onMounted(() => {
    refresh();
    timer = window.setInterval(refresh, 1000);
  });
  onUnmounted(() => window.clearInterval(timer));

  return { state, send };
}
