# Drum Pattern Generator

A phone app for drummers and anyone learning the kit: generate a groove,
read it, hear it, practise it. One screen, as simple as a metronome.
Android first, iOS later.

**Status:** the shell is decided - a throwaway spike (branch
`spike/tauri-android`) played a sample-exact loop on the test phone.
The app itself isn't built yet - see [PLAN.md](PLAN.md).

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
app/              Tauri shell + Vue UI
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
`target/aarch64-linux-android/debug/libapp_lib.so` to
`gen/android/app/src/main/jniLibs/arm64-v8a/` and run
`gradlew assembleArm64Debug -x rustBuildArm64Debug` in `gen/android`.
