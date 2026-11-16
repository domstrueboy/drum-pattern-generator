# Drum Pattern Generator

A phone app for drummers and anyone learning the kit: generate a groove,
read it, hear it, practise it. One screen, as simple as a metronome.
Android first, iOS later.

**Status:** planning. Nothing runs yet - see [PLAN.md](PLAN.md).

- [docs/spec.md](docs/spec.md) - what the app does, rule by rule
- [docs/spec-original-ru.md](docs/spec-original-ru.md) - the original
  description, as written
- [AGENTS.md](AGENTS.md) - conventions for anyone (or any agent) writing code here

## Shape

A Rust core under a Tauri 2 shell with a Vue UI. All audio is Rust, on
its own real-time thread; the web view draws and sends commands.

```
crates/
  drum-patterns   pattern model + generator rules - pure, shared with the looper
  dpg-samples     decoding, resampling, trimming, kits
  dpg-engine      voices, scheduler, mixer, metronome, bar skip
  dpg-hal         the cpal output backend + a mock for tests
app/              Tauri shell + Vue UI
```

The generator is a crate of its own so that
[looper-pedal](https://github.com/domstrueboy/looper-pedal)'s drum
machine can use it too.
