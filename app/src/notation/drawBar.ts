// Draws one bar with VexFlow at the width it is given, and reports where
// each tick sits so the count, the konnakol and the playhead line up with
// the notes. What to draw - values, dots, tuplets, rests - comes spelled
// from Rust (crates/dpg-session/src/notation.rs).

import {
  Articulation,
  Beam,
  Dot,
  Formatter,
  Modifier,
  Parenthesis,
  Renderer,
  Stave,
  StaveNote,
  Stem,
  Tuplet,
  Voice,
} from "vexflow/bravura";
import type { Instrument, NoteEvent, NoteGroup } from "../session";

/**
 * Notes are drawn this much larger than VexFlow's size, to read from a
 * stand - less when a bar is too busy for its width.
 */
const SCALE = 1.25;

/** Room left and right of the notes, in VexFlow units. */
const MARGIN = 16;

/** The drawing's height in VexFlow units: beams and brackets above, the kick below. */
const HEIGHT = 112;

/** Where the invisible staff sits: its top line is 40 below this. */
const STAVE_Y = 26;

/** Each instrument's place, lowest first, and its notehead. */
const KEYS: [Instrument, string][] = [
  ["kick", "f/4"],
  ["snare", "c/5"],
  ["hiHat", "g/5/x2"],
  ["crash", "a/5/d2"],
];

/** Where one bar's notes landed, in CSS pixels from the bar's left edge. */
export interface BarLayout {
  /** Where `tick` (from the pattern's start) falls in this bar. */
  xAt(tick: number): number;
  /** Each note's centre, by its tick. */
  notes: { tick: number; x: number; event: NoteEvent }[];
  /** The drawing's height in CSS pixels. */
  height: number;
}

function color(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function staveNote(
  event: NoteEvent,
  colors: Record<Instrument, string>,
  ink: string,
): StaveNote {
  // VexFlow draws stems and flags black whatever the context says: black
  // on this background hides them and cuts into the noteheads.
  const style = { fillStyle: ink, strokeStyle: ink };
  if (event.rest) {
    const rest = new StaveNote({
      keys: ["b/4"],
      duration: `${event.value}r`,
      dots: event.dots,
    });
    rest.setStyle(style);
    if (event.dots > 0) Dot.buildAndAttach([rest], { all: true });
    return rest;
  }
  const keys = KEYS.filter(([instrument]) =>
    event.hits.some((h) => h.instrument === instrument),
  );
  const note = new StaveNote({
    keys: keys.map(([, key]) => key),
    duration: `${event.value}`,
    dots: event.dots,
    stemDirection: Stem.UP,
  });
  note.setStyle(style);
  keys.forEach(([instrument], i) => {
    const c = colors[instrument];
    note.setKeyStyle(i, { fillStyle: c, strokeStyle: c });
    const hit = event.hits.find((h) => h.instrument === instrument);
    if (hit?.kind === "ghost") {
      note.addModifier(new Parenthesis(Modifier.Position.LEFT), i);
      note.addModifier(new Parenthesis(Modifier.Position.RIGHT), i);
    }
  });
  // The accent mark in the accented instrument's colour, not the lowest
  // notehead's: a hi-hat accent over a kick stays cyan.
  const accent = keys.findIndex(([instrument]) =>
    event.hits.some((h) => h.instrument === instrument && h.kind === "accent"),
  );
  if (accent >= 0) {
    const c = colors[keys[accent][0]];
    const mark = new Articulation("a>").setPosition(Modifier.Position.ABOVE);
    mark.setStyle({ fillStyle: c, strokeStyle: c });
    note.addModifier(mark, accent);
  }
  if (event.dots > 0) Dot.buildAndAttach([note], { all: true });
  return note;
}

/** Beams each run of notes shorter than a quarter; a rest ends a run. */
function beams(notes: StaveNote[], events: NoteEvent[]): Beam[] {
  const out: Beam[] = [];
  let run: StaveNote[] = [];
  const close = () => {
    if (run.length > 1) out.push(new Beam(run));
    run = [];
  };
  events.forEach((event, i) => {
    if (event.rest || event.value < 8) close();
    else run.push(notes[i]);
  });
  close();
  return out;
}

/**
 * Draws `groups` - one bar, starting at `start` - into `host`, `width`
 * CSS pixels wide and at most `height` tall, replacing what was there.
 */
export function drawBar(
  host: HTMLElement,
  groups: NoteGroup[],
  start: number,
  ticksPerBar: number,
  ticksPerBeat: number,
  width: number,
  height = Infinity,
): BarLayout {
  host.replaceChildren();
  const colors: Record<Instrument, string> = {
    crash: color("--crash"),
    hiHat: color("--hi-hat"),
    snare: color("--snare"),
    kick: color("--kick"),
  };
  const ink = color("--text");

  const notes: StaveNote[] = [];
  const events: NoteEvent[] = [];
  const allBeams: Beam[] = [];
  const tuplets: Tuplet[] = [];
  for (const group of groups) {
    const own = group.events.map((e) => staveNote(e, colors, ink));
    notes.push(...own);
    events.push(...group.events);
    allBeams.push(...beams(own, group.events));
    if (group.tuplet) {
      tuplets.push(
        new Tuplet(own, {
          numNotes: group.tuplet.notes,
          notesOccupied: group.tuplet.occupied,
          ratioed: false,
          bracketed: true,
        }),
      );
    }
  }
  const voice = new Voice({
    numBeats: ticksPerBar / ticksPerBeat,
    beatValue: 4,
  }).setMode(Voice.Mode.SOFT);
  voice.addTickables(notes);
  const formatter = new Formatter().joinVoices([voice]);

  // Smaller rather than cut off: 24 notes a bar need the room, and in
  // landscape a bar must fit the height it is given.
  const least = formatter.preCalculateMinTotalWidth([voice]) + 3 * MARGIN;
  const scale = Math.min(SCALE, width / least, height / HEIGHT);
  const drawn = HEIGHT * scale;

  const renderer = new Renderer(host as HTMLDivElement, Renderer.Backends.SVG);
  renderer.resize(width, drawn);
  const ctx = renderer.getContext();
  ctx.scale(scale, scale);
  ctx.setFillStyle(ink);
  ctx.setStrokeStyle(ink);

  const stave = new Stave(0, STAVE_Y, width / scale, { leftBar: false, rightBar: false });
  stave.setContext(ctx);
  formatter.format([voice], stave.getNoteEndX() - stave.getNoteStartX() - MARGIN);
  voice.draw(ctx, stave);
  allBeams.forEach((b) => b.setContext(ctx).draw());
  tuplets.forEach((t) => t.setContext(ctx).draw());

  const placed = notes.map((note, i) => ({
    tick: events[i].tick,
    x: ((note.getNoteHeadBeginX() + note.getNoteHeadEndX()) / 2) * scale,
    event: events[i],
  }));
  // The bar's end sits as far past its last note as its first sits in.
  const end = { tick: start + ticksPerBar, x: width - (placed[0]?.x ?? 0) / 2 };
  const points = [...placed, end];

  return {
    xAt(tick: number) {
      if (points.length === 1 || tick <= points[0].tick) return points[0].x;
      for (let i = 1; i < points.length; i++) {
        const [a, b] = [points[i - 1], points[i]];
        if (tick <= b.tick) {
          return a.x + ((tick - a.tick) / (b.tick - a.tick)) * (b.x - a.x);
        }
      }
      return end.x;
    },
    notes: placed,
    height: drawn,
  };
}
