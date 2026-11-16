# Drum Pattern Generator - AI Coding Assistant Guide

Project-specific conventions that can't be inferred from code alone.
Adapted from looper-pedal's; where they agree, keep them agreeing.

## Architecture

```
app             Tauri 2 shell (src-tauri) + Vue 3 / TypeScript UI
dpg-engine      voices, scheduler, mixer, metronome, bar skip (no cpal)
dpg-samples     decode, resample, trim, kits (no cpal)
dpg-hal         audio output devices and streams, plus a mock (no patterns)
drum-patterns   pattern model + generator rules (no audio, no UI, no I/O)
```

Dependency direction: app -> engine/samples -> hal; anything may use
`drum-patterns`, and it uses nothing of ours.

**Hard boundaries** - never cross:
- `cpal` types only in `dpg-hal`.
- `tauri` types only in `app/src-tauri`.
- `drum-patterns` has no audio, UI, file or platform dependency: the
  looper depends on it as it is.
- The UI is a view: it renders state it is given and sends commands. No
  timing in TypeScript - what sounds when is decided in Rust.

## Conventions

### Rust

| Item | Pattern | Example |
|---|---|---|
| Crate | kebab-case, `dpg-*` for app crates | `dpg-engine` |
| Module file | `snake_case.rs` | `ghost_notes.rs` |
| Type | PascalCase | `Pattern`, `HiHatPulse` |
| Function / method | snake_case | `generate()`, `set_bpm()` |
| Constant | UPPER_SNAKE_CASE | `TICKS_PER_BAR` |
| Test file | `{name}_tests.rs` | `snare_tests.rs` |
| Test function | snake_case, scenario | `half_time_puts_snare_on_three` |

**Tests** - sibling files only, never inline:
```rust
#[cfg(test)]
#[path = "snare_tests.rs"]
mod tests;
```
Test files use `use super::*;`.

**Randomness is seeded.** Every generator takes its RNG or seed from the
caller, so a test can pin a result and a groove can be reproduced.

### UI

Vue 3 `<script setup>` + TypeScript, Vite, pnpm. Components PascalCase
(`TransportBar.vue`), composables `useThing.ts`. Every label starts with
a capital letter (the spec asks for it). Sizes for touch: nothing
tappable under 40 px.

### Doc comments

Prose, explaining *what* and *why*, not *how*. `//!` for modules, `///`
for items. Module docs 2-4 lines. Inline comments only when the "why"
isn't obvious.

## Style

- **Be short.** If a sentence can be cut without losing meaning, cut it.
- **Prefer tables over prose** for structured info.
- **Code speaks for itself.** Explain *why*, not *what*.
- **No hedging.** State what's correct.

## Rules (Non-Negotiable)

1. **Audio thread**: no allocation, no locks, no blocking, no I/O. Bounded work only.
2. **Thread boundary**: UI -> audio via atomics or a lock-free ring buffer. No mutex in the audio path.
3. **Samples are `f32`** above the device layer, full scale at +/-1.0. Clipping happens once, on the way out.
4. **Sounds load off the audio thread** - decoded, resampled and trimmed on a worker, then handed over whole.
5. **Licensed audio never enters the repo, the app bundle or a shared build.** Local kits live in `samples/` (git-ignored). Tests use generated audio.
6. **Tests in sibling files**, never inline.

## Working Process

The agent follows it, the user controls it.

1. **Plan first.** The user defines the feature. The agent splits it into steps before writing code.
2. **One step at a time.** Implement one step, then pause.
3. **Describe what changed** - what was done, not what is planned. Build and test before reporting.
4. **Commit message.** Suggest one: a summary line focused on *why*, then a short paragraph if needed.
5. **The user commits.** The agent never runs `git add`, `git commit` or `git push`.
6. **End with "Your turn":** what the user does next - review, commit, test on the phone, decide, ask the author.
7. **Fixes first.** Requested changes go into the same step.

## Plan and spec

- `PLAN.md` is the single source of truth for what's left to build: ordered steps, open decisions. What's done goes to README and the docs.
- `docs/spec.md` is what the app does. When the prototype and the spec disagree, the spec wins; when the spec is unclear, the question goes to its *Open questions*, not into a guess in code.
- Session plans go in the agent's own ignored dir (`.claude/plans/` etc.).
