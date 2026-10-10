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
- `src/art.rs`: all drawing (procedural; no image assets)
- `src/audio.rs`: procedurally synthesised sound effects
- `SPEC.md`: the gameplay spec the game is built from
- `assets/`: Lilita One font (SIL Open Font License, see `assets/OFL.txt`)
