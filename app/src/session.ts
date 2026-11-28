// The commands and snapshot of `dpg-session`, as they cross to the UI.
// Kept by hand in step with crates/dpg-session (command.rs, snapshot.rs).

export type Pulse = "4" | "8" | "16" | "4t" | "8t" | "16t" | "4." | "8.";
export type Subdivision = "4" | "8" | "8t" | "16";
export type Feel = "basic" | "half" | "double";
export type Instrument = "crash" | "hiHat" | "snare" | "kick";
export type Kind = "accent" | "normal" | "ghost";
export type View = "stacked" | "scrolled";
export type Fader = "drums" | "metronome" | Instrument;

export type Command =
  | { type: "startStop" }
  | { type: "bpm"; bpm: number }
  | { type: "nudgeBpm"; by: number }
  | { type: "tap" }
  | { type: "metronome" }
  | { type: "subdivision"; value: Subdivision }
  | { type: "bars"; count: number }
  | { type: "view"; view: View }
  | { type: "barSkip"; play: number; skip: number }
  | { type: "barSkipOff" }
  | { type: "crash"; every: number | null }
  | { type: "pulse"; value: Pulse }
  | { type: "hiHatRandom" }
  | { type: "hiHatRandomCount"; count: number }
  | { type: "density"; percent: number }
  | { type: "feel"; feel: Feel }
  | { type: "accents"; count: number }
  | { type: "ghosts"; count: number }
  | { type: "generateGhosts" }
  | { type: "ghostRandom" }
  | { type: "ghostRandomCount"; count: number }
  | { type: "baseKicks"; count: number }
  | { type: "generateBaseKicks" }
  | { type: "extraKicks"; count: number }
  | { type: "generateExtraKicks" }
  | { type: "fader"; fader: Fader; position: number }
  | { type: "solo"; instrument: Instrument }
  | { type: "mute"; instrument: Instrument };

export interface Hit {
  tick: number;
  instrument: Instrument;
  kind: Kind;
}

/** A note or a rest, spelled in Rust (crates/dpg-session/src/notation.rs). */
export interface NoteEvent {
  tick: number;
  ticks: number;
  /** 4 a quarter, 8 an 8th, 16 a 16th... */
  value: number;
  dots: number;
  rest: boolean;
  /** The note-value row: "16", "8.", "8t" - notes only. */
  label: string | null;
  hits: { instrument: Instrument; kind: Kind }[];
  /** "т" or "к" - notes only. */
  konnakol: string | null;
}

/** A beat, or two under a quarter-triplet bracket, beamed together. */
export interface NoteGroup {
  tick: number;
  ticks: number;
  tuplet: { notes: number; occupied: number } | null;
  events: NoteEvent[];
}

export interface Notation {
  count: { tick: number; label: string }[];
  bars: NoteGroup[][];
}

/** Where the sound is: the lap playing and the tick in it. */
export interface Playhead {
  lap: number;
  tick: number;
}

export interface ChannelState {
  level: number;
  solo: boolean;
  mute: boolean;
}

/** The output, always reported. */
export interface OutputState {
  error: string | null;
  device: string;
  rate: number;
  latencyMs: number;
  xruns: number;
}

/** The session, reported once an output has opened. */
export interface SessionState {
  playing: boolean;
  bpm: number;
  metronome: boolean;
  subdivision: Subdivision;
  bars: number;
  view: View;
  barSkip: { play: number; skip: number } | null;
  crashEvery: number | null;
  hiHat: {
    pulse: Pulse;
    random: boolean;
    randomCount: number;
    randomIndex: number;
    density: number;
  };
  snare: {
    feel: Feel;
    accents: number;
    ghosts: number;
    ghostLevel: number;
    ghostRandomCount: number;
    ghostRandomIndex: number;
  };
  kick: { base: number; extra: number; extraLevel: number };
  mixer: {
    drums: number;
    metronome: number;
    hiHat: ChannelState;
    snare: ChannelState;
    kick: ChannelState;
  };
  ticksPerBeat: number;
  ticksPerBar: number;
  laps: Hit[][];
  notation: Notation[];
  limits: { count: number; variants: number; ghostLevels: number; extraKickLevels: number };
}

export type Snapshot = OutputState & Partial<SessionState>;

/** The instruments with their own buttons, top to bottom. */
export type DrumInstrument = "hiHat" | "snare" | "kick";
