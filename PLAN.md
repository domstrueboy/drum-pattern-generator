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
  only. Confirmed by the step 1 spike (2026-11-17): all six checks
  passed on the test phone - see *Android notes*.
- **Two layouts, portrait first.** The author wants portrait and
  landscape; portrait is how a phone sits on a stand, so it comes first
  and the layout stays responsive. The mockup is wide only because Figma
  Make previews in a desktop browser.
- **Notation with VexFlow**, not hand-drawn SVG: beams, tuplets, x and
  diamond noteheads, parentheses, hidden staff lines - and drawn at the
  real width, never stretched.
- **12 ticks a quarter note** (48 a bar in 4/4): the smallest grid
  holding 16ths, every triplet and both dotted values (see the spec's
  table). The model carries a time signature: 4/4 first, others later.
- **A kit is data:** each instrument holds 1 or 2 samples (one played
  louder and quieter, or separate normal and ghost - see the spec's
  Sound menu); the format leaves room for more layers. Built-in
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
- **Remote.** A GitHub repo on the personal account - private until the
  licence is chosen, then public if the author agrees.

## Android notes

From the step 1 spike (branch `spike/tauri-android`, kept, never
merged). They constrain steps 4, 5 and 9.

| Finding | Consequence |
|---|---|
| Loop steady while scrolling and tapping; WAV picked from storage and played; playback survives a locked screen; playhead in step with the sound | Tauri passes the gate |
| AAudio needs API 26; Tauri's default minSdk 24 fails to link `-laaudio` | `bundle.android.minSdkVersion: 26` |
| cpal asks for AAudio's low-latency mode only with its `realtime` feature; without it output lagged 0.3-0.5 s | `dpg-hal` enables `realtime` |
| AAudio's default buffer is its whole capacity (2048 frames, 43 ms); a fixed 256-frame request keeps 512 | `dpg-hal` asks for a fixed buffer on Android; total output latency 43 ms, 0 xruns |
| The remaining 32 ms is the phone's fast-mixer path: its MediaTek driver refuses MMAP for the speaker | 43 ms is the floor on this phone's speaker; it only matters for live pads, which the spec doesn't have |
| Output latency is reported per callback (`playback - callback` timestamps) | The playhead and kit highlights subtract it, so they match what's heard on any output, Bluetooth included |
| Picking a file: the dialog plugin returns a `content://` URI; the fs plugin reads it from Rust | No storage permission needed; no JS file access |
| Screen off worked without a foreground service on the test phone | The spec wants it on by default, with a setting to turn it off. Other vendors' battery savers may still stop it - revisit in step 9 |
| `tauri android build/dev` symlinks the library into `jniLibs`, which Windows refuses without Developer Mode | Turn on Developer Mode, or copy the `.so` and run `gradlew assembleArm64Debug -x rustBuildArm64Debug` |

## Roadmap

Ordered. One step at a time, each a commit.

### Next session - start here

1. **Step 3** (`dpg-samples`), once 2h is reviewed and committed. Plan
   it first: decoders, the kit format, where kits live.

### 2. `drum-patterns`

Pure, tested, one commit per sub-step. Randomness through `rand`: every
generator takes `&mut impl Rng` from the caller. The crash isn't here -
it follows bars of playback, so it's the engine's (step 4).

Done: 2a (the model), 2b (count and konnakol), 2c (hi-hat pulse),
2d (snare feel and accents), 2e (base kicks), 2f (ghost notes and extra
kicks, on one difficulty ladder - the app keeps the level), 2g (Random:
a seeded set of N variants; the app keeps seed, N and index), 2h
(`Groove`: settings plus one seed build a whole pattern - the looper's
way in, and the app's first pattern; after that the app keeps the
pattern and regenerates parts by hand).

### 3. `dpg-samples`

Decode WAV / FLAC / MP3 / M4A (phones record M4A), resample to the
device rate, trim leading silence (a home recording's 30 ms makes a drum
sound late), normalise, mono. The kit format and where kits are stored.

### 4. `dpg-hal` and `dpg-engine`

Output backend + mock. Voice pool, per-instrument level, solo, mute,
drums and metronome buses, metronome subdivisions, bar skip, the crash
counted in bars of playback (once, every 4 bars, every 8...), a dotted
hi-hat pulse drifting on across loops (`hi_hat_pulse` gains a starting
offset; the next loop's hi-hat is prepared off the audio thread),
swapping a kit while playing, the play position published for the UI.

### 5. First real UI

The whole screen on the phone, plain visuals: bars, metronome,
transport, VexFlow notation, faders, the three instrument submenus.
The notation redraws the hi-hat on each loop while a dotted pulse
drifts. The author's prototype ([docs/mockup.html](docs/mockup.html)) is the
reference for order and colour (dark; hi-hat cyan, snare white, kick
orange), reflowed for portrait; where it and the spec differ, the spec
wins. Landscape and the orientation button (four orientations, no
auto-rotate) follow once portrait works; the button needs a small
Kotlin call (`setRequestedOrientation`).

### 6. Sounds

Five presets per instrument and five metronome sounds - synthesized
where the character allows (Electro, Hip-hop, clicks), samples we may
ship otherwise (the author recording a kit is the best source). User
sample import.

### 7. Look

The kit photo with per-drum highlights (an image of our own, each
drum's position known), background colour and texture, tap tempo
blinking in time.

### 8. Saved grooves and settings

Save a groove and open it again; the Settings menu with "play with
the screen off". A saved groove stores its hits, not just its `Groove`:
parts regenerated by hand can't be rebuilt from one seed, and `rand`
may change its generator.

### 9. Android release

Signing, an install the author can test, then a store listing.

### 10. The looper uses `drum-patterns`

In looper-pedal's step 14.

### 11. iOS

Needs a Mac and an Apple developer account.

## Later (unordered)

- Recording samples inside the app (needs the mic and an input stream -
  the looper knows how).
- Time signatures other than 4/4 - the model allows them from step 2;
  the rules and the UI come later.
- Desktop builds - free with Tauri, useful for the looper's users.
- MIDI export.
