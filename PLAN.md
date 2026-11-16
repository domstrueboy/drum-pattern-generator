# Drum Pattern Generator - Plan

Forward-looking: what's left to build, in what order, and the decisions
that shouldn't drift. What the app does is [docs/spec.md](docs/spec.md);
what exists is in README.

## Product

- **For drummers and learners:** generate a groove, read it, hear it,
  practise it. One screen, metronome-simple.
- **The author is a musician**, not a developer: the spec is his, the
  code is ours. Questions about behaviour go to him through the spec's
  *Open questions*.
- **Android first** (the test phone: Honor LNA-NX1, Android 16, arm64).
  iOS later - nothing Android-only goes into the crates.

## Decisions

Settled; worth not re-litigating without a new reason.

- **Rust core, Tauri 2 shell, Vue 3 + TypeScript UI.** The app only
  plays a pattern - no live input to monitor - so what matters is that
  hits land exactly, which the Rust audio thread does on its own; latency
  only shifts everything equally. The web view is strong at what the
  screen needs (notation, faders, a photo with highlights), and one UI
  covers Android, iOS and desktop. Flutter + flutter_rust_bridge is the
  fallback: the crates don't know the UI, so a switch replaces `app/`
  only. Step 1 decides.
- **Notation with VexFlow**, not hand-drawn SVG: beams, tuplets, x and
  diamond noteheads, parentheses, hidden staff lines - and drawn at the
  real width, never stretched.
- **48 ticks a bar** (12 a beat): the smallest grid holding 16ths,
  every triplet and both dotted values (see the spec's table).
- **A kit is data:** each instrument holds one or more samples. Built-in
  presets are kits shipped with the app; user kits are the same thing
  loaded from files. Synthesized presets (Electro, Hip-hop) are rendered
  into buffers at load, so the engine only ever plays buffers.
- **The generator is shared.** `drum-patterns` is pure so that
  looper-pedal's drum machine (its step 14: a grid of preset grooves,
  variation x strength) can use it as a git dependency.
- **Ideas from the looper, not its code.** Its `Scheduler`/`Voice`
  (sample-exact steps, nothing allocated in the callback), its HAL's
  two-phase open and mock backend, its handover of a loaded buffer to
  the audio thread - rewritten here for a voice pool, a 48-tick grid and
  output only. Factored into a crate both use only once both have
  settled.
- **Build on Windows**, where the phone's adb and the looper's toolchain
  already are.

## Ground rules

- No locks, no allocation, no I/O on the audio thread (AGENTS.md).
- Licensed audio stays local: `samples/` is git-ignored, the bundle
  ships only sounds we may ship, and no shared build carries a licensed
  kit. The dev test kit is the user's own SSD5 render
  (`C:\Users\pbobylyov\Music\DP-2000 SSD5 Kit\WAVIMPORT`), loaded on the
  phone through the user-sample feature.
- One small commit per step, reviewed before it lands.

## Open decisions

- **Licence.** Not chosen. The looper is MIT; this one is shared with
  the author, so it's his call too. Needed before the repo goes public.
- **The spec's open questions** (docs/spec.md) - needed by step 2's
  rules, not before.
- **Remote.** A GitHub repo on the personal account - public or
  private, created when the first commit is ready.

## Roadmap

Ordered. One step at a time, each a commit.

### Next session - start here

Planned 2026-11-16 in a session outside this repo. In order:

1. **Step 0 is written but not committed.** Review it with the user;
   the user commits.
2. **The author's answers** to the spec's open questions may have come
   in - fold them into docs/spec.md.
3. **Step 1a:** the Android toolchain. Android Studio was uninstalled
   from this laptop on purpose (2026-08), so prefer the command-line
   tools and a standalone JDK; ask before installing.

### 0. Repo and docs - this step

README, AGENTS.md, PLAN.md, the spec in English and the original in
Russian.

### 1. Test app - throwaway, decides Tauri

On a branch `spike/tauri-android`, never merged; findings land here.

- **1a. Toolchain on Windows:** a JDK (17+), the Android SDK
  (command-line tools, a platform, build-tools, the NDK), Rust's
  `aarch64-linux-android` target, the Tauri CLI. Ask before installing:
  several GB.
- **1b.** Tauri 2 + Vue scaffold; runs as a desktop window, then on the
  phone through `tauri android dev`.
- **1c. Sound:** Rust plays a WAV through cpal (AAudio) on a tap, then a
  16th loop at 120 BPM. Listen for wobble while scrolling and tapping.
- **1d. Files:** pick a WAV from phone storage and play it.
- **1e. Screen off:** does playback survive a locked screen, or does it
  need a foreground service (a small Kotlin plugin)?
- **1f. Sync:** a playhead driven by Rust's play position stays in step
  with what's heard.

**Gate:** all six pass -> Tauri. Otherwise the failing point decides
between fixing it and Flutter.

### 2. `drum-patterns`

Pure, tested, in parallel with step 1. The model (1-4 bars, 48 ticks,
instruments, hit kinds accent / normal / ghost), then each submenu's
rules: hi-hat pulses, snare accents and feel, ghost notes simple to
complex, base and extra kicks, Random. Seeded RNG throughout. Count and
konnakol derived from a pattern.

### 3. `dpg-samples`

Decode WAV / FLAC / MP3 / M4A (phones record M4A), resample to the
device rate, trim leading silence (a home recording's 30 ms makes a drum
sound late), normalise, mono. The kit format and where kits are stored.

### 4. `dpg-hal` and `dpg-engine`

Output backend + mock. Voice pool, per-instrument level, solo, mute,
drums and metronome buses, metronome subdivisions, bar skip, swapping a
kit while playing, the play position published for the UI.

### 5. First real UI

The whole screen on the phone, plain visuals: bars, metronome,
transport, VexFlow notation, faders, the three instrument submenus.

### 6. Sounds

Five presets per instrument and five metronome sounds - synthesized
where the character allows (Electro, Hip-hop, clicks), samples we may
ship otherwise (the author recording a kit is the best source). User
sample import.

### 7. Look

The kit photo with per-drum highlights (an image of our own, each
drum's position known), background colour and texture, tap tempo
blinking in time.

### 8. Android release

Signing, an install the author can test, then a store listing.

### 9. The looper uses `drum-patterns`

In looper-pedal's step 14.

### 10. iOS

Needs a Mac and an Apple developer account.

## Later (unordered)

- Recording samples inside the app (needs the mic and an input stream -
  the looper knows how).
- Saving grooves, if the author wants it (spec Q7).
- Desktop builds - free with Tauri, useful for the looper's users.
- MIDI export.
