# Ragdoll Cricket

A physics batting game written in Rust with [macroquad](https://macroquad.rs) and [rapier2d](https://rapier.rs).

Move the mouse to swing a floppy ragdoll batsman's bat. The batsman stands in an enclosed field.
Hit the far wall to score: the higher the ball strikes it, the more runs (1, 2, 4 or 6).
Hitting the roof, the low wall behind you, or your own stumps gets you out.

## Run

```sh
cargo run --release
```

Esc quits.

## Developer aids

- `RC_PROFILE=1`: print fps and per-frame physics/draw costs.
- `RC_SNAPSHOT=<dir>`: auto-play a short innings and save screenshots to `<dir>`.
- `cargo test --release`: headless physics checks (ragdoll stability, bat tracking, joint limits, bowling).

## Layout

- `src/main.rs`: physics world, rules, game flow
- `src/art.rs`: all drawing (procedural scene; the batsman is painted parts from `assets/batsman/`)
- `src/audio.rs`: procedurally synthesised sound effects
- `SPEC.md`: the gameplay spec the game is built from
- `assets/`: Lilita One font (SIL Open Font License, see `assets/OFL.txt`)

## Download (macOS)

Grab `RagdollCricket-<version>-macos-universal.zip` from [Releases](../../releases), unzip, and open
**Ragdoll Cricket.app** (runs natively on Apple Silicon and Intel, macOS 11+).

The app is ad-hoc signed, not notarized, so the first launch is blocked by Gatekeeper.
Either right-click the app → **Open** → **Open**, or run:

```sh
xattr -dr com.apple.quarantine "Ragdoll Cricket.app"
```

To build the release zip yourself: `scripts/package-macos.sh` (outputs to `dist/`).

## Download (Android)

Grab `RagdollCricket-<version>-android-arm64.apk` from [Releases](../../releases) and open it on the
phone (allow installing from your browser or file manager when asked), or from a computer:

```sh
adb install RagdollCricket-<version>-android-arm64.apk
```

Drag anywhere on the screen to swing the bat: it follows your finger's movement, like a trackpad.
The APK is debug-signed (not from the Play Store), so Play Protect may ask you to confirm the install.

To build it yourself (needs Docker): `scripts/build-android.sh`. It builds in macroquad's
`cargo-quad-apk` image and installs to a connected device; with several adb devices, set
`ANDROID_SERIAL`.
