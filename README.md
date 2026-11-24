![Spike with his notebook by the moonlit lake](docs/spike-cover.png)

# Spike

[English](README.md) · [Русский](README-ru.md) · [Deutsch](README-de.md)

Press **PWR** and Spike steps onto a stone terrace by the lake. Tap him and he jumps for joy. Hold your finger still and he purrs. The little traveller in the plum cloak is the project's mascot, sharing the round display with a maths map for grades 10 and 11.

## How Spike took shape

- **First pixels.** The first firmware drew `hello everynwan`. Reading it on the board revealed that the image needed a 180° turn. The driver now rotates each strip and clears the panel memory around the visible circle at startup.
- **From portrait to pet.** Spike began as a single sprite. Giving him separate walking, jumping and sleeping poses made gestures visible in his silhouette. The moonlit lake and the ivory lettering under pointed arches gave the two-button interface its character.
- **A map you can use with one finger.** The early maths tree grew into 14 topic cards. On the board, a small finger movement could turn a tap into a drag. Larger targets and a 24-pixel movement allowance let you open a topic near its edge and return to the same spot.
- **Power and releases.** An idle timeout puts the AMOLED to sleep even while Spike is animating. The tagged release workflow builds Russian and English images separately. Checks on the packaged images confirmed that the sampled translations appear only in the matching build.

## On the board

| Control | What happens |
| --- | --- |
| Press PWR | Open Spike’s scene. Swipe left or right to walk, up to leap, down to curl up asleep. Press PWR again to return to the greeting. |
| Press and release BOOT | Open settings. Tap **30**, **60** or **120** seconds to set the display timeout until restart. The default is 60 seconds. |
| Hold BOOT for 0.9 s | Open the study map. Drag sideways for more topics, or swipe up to reach grade 10 below grade 11. |
| Tap a topic | Read its card. **Back to map** restores the same spot. |
| Press either button while the display sleeps | Wake the current screen. The next press works normally. |

The map contains fourteen revision cards drawn from MCKO maths materials for grade 10 (2026) and grade 11 (2023). Start at **Derivative** and slide across to **Functions**. [Grade 10 outline](https://100ballnik.com/wp-content/uploads/2026/04/opisanie_ma_10_profil_mcko2026.pdf) · [Grade 11 outline](https://100ballnik.com/wp-content/uploads/2023/09/mcko_2023_mat_11klass_variant.pdf)

## Build and flash

Use a **Waveshare ESP32-S3-Touch-AMOLED-1.32** with a USB data cable. Install the Xtensa Rust toolchain named `esp` and the `espflash` CLI. To match CI, use toolchain 1.97.0.0 and espflash 4.7.0. Run the commands from the repository root. The second command flashes the board and opens the serial monitor. [espup](https://github.com/esp-rs/espup) · [espflash](https://github.com/esp-rs/espflash)

**Russian firmware (default)**

```sh
cargo build --release --locked
cargo run --release --locked
```

**English firmware**

```sh
cargo build --release --locked --features english
cargo run --release --locked --features english
```

Each image includes one UI catalogue and one label font. The startup greeting `hello everynwan` is shared. To change the interface language, flash the other build. German is available for the documentation.

## Inside the round display

![The eight character poses used by the firmware](assets/spike-poses.png)

The 466 × 466 AMOLED is painted four rows at a time using a 3,840-byte DMA buffer. At build time, the eight poses become 4-bit palette images and the lake becomes runs of RGB565 pixels. Both live in flash. Pet animation updates just its drawing region.

Edit the artwork in [assets](assets), translated labels and cards in [src/lang](src/lang), and gestures in [src/gestures.rs](src/gestures.rs). See the [contributor guide](AGENTS.md) for local checks. A `v*` tag on a commit reachable from `main` builds two release images, `spike-esp32s3.bin` (RU) and `spike-esp32s3-en.bin` (EN), each with a SHA-256 checksum. [Release workflow](.github/workflows/firmware.yml)

## Ideas for the next version

1. **Remember the timeout.** Save it when the chosen value changes so it survives a restart.
2. **Add practice to the cards.** Start with one derivative problem and a tap to reveal the worked solution. The existing cards can still serve as quick references.
3. **Measure battery use.** Compare current on the same board with Spike animating, the map open and the display asleep. Use those readings to decide whether deeper sleep is worth implementing.
