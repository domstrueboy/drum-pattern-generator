import { ref } from "vue";
import type { DrumInstrument } from "./session";

/** A submenu: an instrument's, Bar skip's or the crash's. */
export type Menu = DrumInstrument | "barSkip" | "crash";

// One submenu open at a time, for the whole screen: the Bars row opens
// two of them and the kit three.
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
