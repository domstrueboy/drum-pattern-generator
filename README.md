# Drum Pattern Generator

A phone app for drummers and anyone learning the kit: generate a groove,
read it, hear it, practise it. One screen, as simple as a metronome.
Android first, iOS later.

**Status:** the shell is decided - a throwaway spike (branch
`spike/tauri-android`) played a sample-exact loop on the test phone.
Done: `drum-patterns` (every generator rule), `dpg-samples` (audio
files to kit sounds), `dpg-hal` (the output) and `dpg-engine` (playing a
groove: metronome, tap tempo, bar skip, crash, mixer) and the first
real UI - `app/` shows the screen in portrait and landscape with the
groove in notation (VexFlow) and a playhead, through `dpg-session`,
which holds every command the screen sends. Now: sounds - six metronome
sounds and five synthesized kit presets are in; the Sound menu and
your own samples are next - see [PLAN.md](PLAN.md).

To run the app on a desktop: `pnpm install`, then `pnpm tauri dev`, in
`app/` from a Windows terminal.

To listen on a desktop: `cargo run -p dpg-play -- <kit folder>`, then
`?` for the commands. The kit's file names are in
`crates/dpg-play/src/kit.rs`.

- [docs/spec.md](docs/spec.md) - what the app does, rule by rule
- [docs/spec-original-ru.md](docs/spec-original-ru.md) - the original
  description, as written
- [docs/mockup.html](docs/mockup.html) - the author's prototype, open
  in a browser
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
  dpg-session     the screen's state and every command - the app's logic
  dpg-play        a terminal player for listening on a desktop - dev only
app/              Tauri shell (src-tauri, crate dpg-app) + Vue UI
```

The generator is a crate of its own so that
[looper-pedal](https://github.com/domstrueboy/looper-pedal)'s drum
machine can use it too.

## Building for Android (Windows)

Build from a Windows terminal, not WSL: `node_modules` holds the Windows
build of the Tauri CLI.

| Tool | Version | Where |
|---|---|---|
| JDK | Temurin 21 | `JAVA_HOME` |
| Android SDK | command-line tools, platform 36, build-tools 36.1 | `ANDROID_HOME` |
| NDK | 29 | `NDK_HOME` |
| Rust target | `aarch64-linux-android` | rustup |

`pnpm tauri android dev` / `build` link the Rust library into the
Android project with a symlink, which needs Windows Developer Mode
(Settings > System > For developers). Without it, copy
`target/aarch64-linux-android/debug/libdpg_app_lib.so` to
`gen/android/app/src/main/jniLibs/arm64-v8a/` and run
`gradlew assembleArm64Debug -x rustBuildArm64Debug` in `gen/android`.
