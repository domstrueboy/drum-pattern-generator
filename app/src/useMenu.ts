import { ref } from "vue";
import type { DrumInstrument } from "./session";

/** A submenu: an instrument's, Bar skip's, the crash's or Sound's. */
export type Menu = DrumInstrument | "barSkip" | "crash" | "sound";

// One submenu open at a time, for the whole screen: the Bars row opens
// two of them, the kit three and the transport one.
const menu = ref<Menu | null>(null);

/** The open submenu, and a way to open one or close it (`null`). */
export function useMenu() {
  return {
    menu,
    open(which: Menu | null) {
      menu.value = which;
    },
  };
}
