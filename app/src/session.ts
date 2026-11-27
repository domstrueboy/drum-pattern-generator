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

export interface ChannelState {
  level: number;
  solo: boolean;
  mute: boolean;
}

export interface Snapshot {
  // The output.
  error: string | null;
  device: string;
  rate: number;
  latencyMs: number;
  xruns: number;
  // The session - missing when the output couldn't open.
  playing?: boolean;
  bpm?: number;
  metronome?: boolean;
  subdivision?: Subdivision;
  bars?: number;
  view?: View;
  barSkip?: { play: number; skip: number } | null;
  crashEvery?: number | null;
  hiHat?: {
    pulse: Pulse;
    random: boolean;
    randomCount: number;
    randomIndex: number;
    density: number;
  };
  snare?: {
    feel: Feel;
    accents: number;
    ghosts: number;
    ghostLevel: number;
    ghostRandomCount: number;
    ghostRandomIndex: number;
  };
  kick?: { base: number; extra: number; extraLevel: number };
  mixer?: {
    drums: number;
    metronome: number;
    hiHat: ChannelState;
    snare: ChannelState;
    kick: ChannelState;
  };
  ticksPerBeat?: number;
  ticksPerBar?: number;
  laps?: Hit[][];
}
