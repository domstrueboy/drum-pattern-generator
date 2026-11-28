// Every label on the screen, in each language. Until Settings has a
// language choice (step 8), the phone's language picks.

import type {
  ClickSound,
  DrumInstrument,
  Feel,
  HiHatAccents,
  Preset,
  Pulse,
  Subdivision,
} from "./session";

export interface Labels {
  bars: string;
  stacked: string;
  scrolled: string;
  barSkip: string;
  metronome: string;
  bpm: string;
  less: string;
  more: string;
  clickSounds: Record<ClickSound, string>;
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
  back: string;
  presets: Record<Preset, string>;
  crashFollows: string;
  rotate: string;
  fewer: string;
  plenty: string;
  crash: string;
  crashOnce: string;
  /** "Every 4 bars" and on, from the bar count. */
  crashEvery: (bars: number) => string;
  invert: string;
  off: string;
  pulse: string;
  pulses: Record<Pulse, string>;
  random: string;
  variants: string;
  density: string;
  feel: string;
  feels: Record<Feel, string>;
  accents: string;
  hiHatAccents: Record<HiHatAccents, string>;
  none: string;
  ghosts: string;
  generate: string;
  level: string;
  baseKicks: string;
  extraKicks: string;
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
  clickSounds: {
    click: "Click",
    ableton: "Ableton",
    cowbell: "Cowbell",
    clave: "Clave",
    bongo: "Bongo",
    electronic: "Electronic",
  },
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
  back: "Back",
  presets: { rock: "Rock", hipHop: "Hip-hop", electro: "Electro", jazz: "Jazz", metal: "Metal" },
  crashFollows: "The crash follows the hi-hat.",
  rotate: "Turn the screen",
  fewer: "Fewer",
  plenty: "More",
  crash: "Crash",
  crashOnce: "Once at the start",
  crashEvery: (bars) => `Every ${bars} bars`,
  invert: "Invert",
  off: "Off",
  pulse: "Pulse",
  pulses: { "4": "4", "8": "8", "16": "16", "4t": "4t", "8t": "8t", "16t": "16t", "4.": "4.", "8.": "8." },
  random: "Random",
  variants: "Variants",
  density: "Density",
  feel: "Time feel",
  feels: { basic: "Basic", half: "Half time", double: "Double time" },
  accents: "Accents",
  hiHatAccents: { off: "Off", beats: "On the beats", offbeats: "On the &" },
  none: "None",
  ghosts: "Ghost notes",
  generate: "Generate",
  level: "Level",
  baseKicks: "Base kicks",
  extraKicks: "Extra kicks",
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
  clickSounds: {
    click: "Клик",
    ableton: "Ableton",
    cowbell: "Ковбелл",
    clave: "Клава",
    bongo: "Бонго",
    electronic: "Электро",
  },
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
  back: "Назад",
  presets: { rock: "Рок", hipHop: "Хип-хоп", electro: "Электро", jazz: "Джаз", metal: "Метал" },
  crashFollows: "Крэш звучит в стиле хай-хета.",
  rotate: "Повернуть экран",
  fewer: "Меньше",
  plenty: "Больше",
  crash: "Крэш",
  crashOnce: "Один раз в начале",
  crashEvery: (bars) =>
    bars === 4 ? "Каждый квадрат" : `Каждые ${bars / 4} квадрата`,
  invert: "Наоборот",
  off: "Выкл.",
  pulse: "Пульсация",
  pulses: { "4": "4", "8": "8", "16": "16", "4t": "4т", "8t": "8т", "16t": "16т", "4.": "4.", "8.": "8." },
  random: "Случайно",
  variants: "Вариантов",
  density: "Плотность",
  feel: "Основной ритм",
  feels: { basic: "Базовый", half: "Халф-тайм", double: "Дабл-тайм" },
  accents: "Акценты",
  hiHatAccents: { off: "Выкл.", beats: "На доли", offbeats: "На «и»" },
  none: "Нет",
  ghosts: "Гоуст-ноты",
  generate: "Сгенерировать",
  level: "Уровень",
  baseKicks: "Основные бочки",
  extraKicks: "Доп. бочки",
};

const labels: Labels = navigator.language.toLowerCase().startsWith("ru")
  ? ru
  : en;

/** The labels in the screen's language. */
export function useLabels(): Labels {
  return labels;
}
