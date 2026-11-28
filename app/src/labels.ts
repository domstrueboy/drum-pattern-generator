// Every label on the screen, in each language. Until Settings has a
// language choice (step 8), the phone's language picks.

import type { DrumInstrument, Subdivision } from "./session";

export interface Labels {
  bars: string;
  stacked: string;
  scrolled: string;
  barSkip: string;
  metronome: string;
  bpm: string;
  less: string;
  more: string;
  click: string;
  subdivisions: Record<Subdivision, string>;
  color: string;
  sound: string;
  settings: string;
  tapTempo: string;
  start: string;
  stop: string;
  drums: string;
  instruments: Record<DrumInstrument, string>;
  solo: string;
  mute: string;
  notation: string;
  kit: string;
  /** Rust writes the count's "и" and konnakol's "т" and "к"; how this language writes them. */
  letters: Record<string, string>;
  /** The triplet mark in the note-value row: 8t. */
  triplet: string;
}

const en: Labels = {
  bars: "Bars",
  stacked: "Stacked",
  scrolled: "Scrolled",
  barSkip: "Bar skip",
  metronome: "Metronome",
  bpm: "BPM",
  less: "Slower",
  more: "Faster",
  click: "Click",
  subdivisions: { "4": "4", "8": "8", "8t": "8t", "16": "16" },
  color: "Color",
  sound: "Sound",
  settings: "Settings",
  tapTempo: "Tap tempo",
  start: "Start",
  stop: "Stop",
  drums: "Drums",
  instruments: { hiHat: "Hi-hat", snare: "Snare", kick: "Kick" },
  solo: "Solo",
  mute: "Mute",
  notation: "Notation",
  kit: "Kit",
  letters: { "и": "&", "т": "t", "к": "k" },
  triplet: "t",
};

const ru: Labels = {
  bars: "Такты",
  stacked: "Друг под другом",
  scrolled: "В ряд",
  barSkip: "Пропуск такта",
  metronome: "Метроном",
  bpm: "BPM",
  less: "Медленнее",
  more: "Быстрее",
  click: "Клик",
  subdivisions: { "4": "4", "8": "8", "8t": "8т", "16": "16" },
  color: "Цвет",
  sound: "Звук",
  settings: "Настройки",
  tapTempo: "Tap tempo",
  start: "Старт",
  stop: "Стоп",
  drums: "Ударные",
  instruments: { hiHat: "Хай-хет", snare: "Малый", kick: "Бочка" },
  solo: "Соло",
  mute: "Заглушить",
  notation: "Нотация",
  kit: "Установка",
  letters: {},
  triplet: "т",
};

const labels: Labels = navigator.language.toLowerCase().startsWith("ru")
  ? ru
  : en;

/** The labels in the screen's language. */
export function useLabels(): Labels {
  return labels;
}
