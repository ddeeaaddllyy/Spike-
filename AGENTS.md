# Repository Guidelines

## Project structure

Spike is a `no_std` Rust firmware application for an ESP32-S3 with a round touch AMOLED display. Start in `src/main.rs` for board initialization, button handling, and screen switching. Keep display transfers in `src/display.rs`, touch gesture recognition in `src/gestures.rs`, topic navigation in `src/topics.rs`, and pet animation in `src/pet.rs`.

Edit the pet sprite atlas and background in `assets/`. The host-side `build.rs` converts the PNGs into flash-resident data during compilation. Preserve the atlas frame order when editing poses. Keep display rendering row-based to limit RAM use.

## Build and run

Use the `esp` Rust toolchain selected by `rust-toolchain.toml`. CI uses Xtensa toolchain version `1.97.0.0`. Run these commands from the repository root.

- `cargo fmt --all -- --check` checks formatting. Use `cargo fmt --all` to apply fixes.
- `cargo clippy --release --locked -- -D warnings` checks the release configuration and treats lint warnings as errors.
- `cargo build --release --locked` builds the firmware for `xtensa-esp32s3-none-elf` using the committed lockfile.
- `cargo run --release --locked` flashes a connected board over USB and opens the serial monitor through `espflash`.

## Coding style

Follow rustfmt with four-space indentation. Use `snake_case` for functions and modules, `UpperCamelCase` for types and enum variants, and `SCREAMING_SNAKE_CASE` for constants. Keep PNG decoding and other asset preparation on the build host.

## Testing

The repository has no automated test suite or coverage threshold. Run the formatting, lint, and build checks locally. For screen or input changes, verify the greeting after reset, tap BOOT for settings, hold BOOT for the map, drag it, open a topic and tap the return button, then open the pet with PWR and try a tap, hold, and four directional swipes.

## Commits, pull requests, and releases

Commit completed changes in logical groups. Use `git cm -m "Imperative subject" -m "Explain what changed and why."` with the local `cm` alias for `commit`. In a pull request, describe the observable change and report which checks ran on hardware. A board photo or short clip helps reviewers assess display changes.

Pushing a `v*` tag triggers the release workflow. The tagged commit must be reachable from `origin/main`. A successful run publishes the merged flash image and its SHA-256 checksum.
