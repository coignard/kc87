# Changelog

## 0.7.4

### Changed

- Bumped android_system_properties from 0.1.5 to 0.1.6.
- Bumped bytemuck_derive from 1.11.0 to 1.12.0.
- Bumped cc from 1.4.0 to 1.4.2.
- Bumped clap from 4.6.5 to 4.6.6.
- Bumped clap_builder from 4.6.5 to 4.6.6.
- Bumped coremidi from 0.9.1 to 0.9.2.
- Bumped find-msvc-tools from 0.1.9 to 0.1.10.
- Bumped futures-core from 0.3.33 to 0.3.34.
- Bumped futures-task from 0.3.33 to 0.3.34.
- Bumped futures-util from 0.3.33 to 0.3.34.
- Bumped hybrid-array from 0.4.13 to 0.4.14.
- Bumped jni-min-helper from 0.3.3 to 0.3.4.
- Bumped js-sys from 0.3.103 to 0.3.104.
- Bumped libredox from 0.1.18 to 0.1.19.
- Bumped portable-atomic from 1.14.0 to 1.15.0.
- Bumped redox_syscall from 0.9.0 to 0.9.1.
- Bumped thiserror from 2.0.19 to 2.0.20.
- Bumped thiserror-impl from 2.0.19 to 2.0.20.
- Bumped wasm-bindgen from 0.2.126 to 0.2.127.
- Bumped wasm-bindgen-futures from 0.4.76 to 0.4.77.
- Bumped wasm-bindgen-macro from 0.2.126 to 0.2.127.
- Bumped wasm-bindgen-macro-support from 0.2.126 to 0.2.127.
- Bumped wasm-bindgen-shared from 0.2.126 to 0.2.127.
- Bumped web-sys from 0.3.103 to 0.3.104.
- Bumped xcursor from 0.3.10 to 0.3.11.
- Bumped xml-rs from 0.8.28 to 0.8.29.
- Bumped zerocopy from 0.8.55 to 0.8.56.
- Bumped zerocopy-derive from 0.8.55 to 0.8.56.

## 0.7.3

### Changed

- Bumped bitflags from 2.13.0 to 2.13.1.
- Bumped bytemuck from 1.25.1 to 1.25.2.
- Bumped cc from 1.2.67 to 1.4.0.
- Bumped cfg_aliases from 0.2.1 to 0.2.2.
- Bumped coremidi-sys from 3.2.0 to 3.2.1.
- Bumped either from 1.16.0 to 1.17.0.
- Bumped foreign-types-macros from 0.2.3 to 0.2.4.
- Bumped futures-core from 0.3.32 to 0.3.33.
- Bumped futures-task from 0.3.32 to 0.3.33.
- Bumped futures-util from 0.3.32 to 0.3.33.
- Bumped glob from 0.3.3 to 0.3.4.
- Bumped jni-min-helper from 0.3.2 to 0.3.3.
- Bumped portable-atomic from 1.13.1 to 1.14.0.
- Bumped proc-macro2 from 1.0.106 to 1.0.107.
- Bumped quick-xml from 0.39.4 to 0.41.0.
- Bumped quote from 1.0.46 to 1.0.47.
- Bumped simd-adler32 from 0.3.9 to 0.3.10.
- Bumped simd_cesu8 from 1.1.1 to 1.2.0.
- Bumped syn from 2.0.118 to 2.0.119.
- Bumped thiserror from 2.0.18 to 2.0.19.
- Bumped thiserror-impl from 2.0.18 to 2.0.19.
- Bumped toml_edit from 0.25.12+spec-1.1.0 to 0.25.13+spec-1.1.0.
- Bumped toml_parser from 1.1.2+spec-1.1.0 to 1.1.3+spec-1.1.0.
- Bumped wayland-backend from 0.3.15 to 0.3.16.
- Bumped wayland-client from 0.31.14 to 0.31.15.
- Bumped wayland-scanner from 0.31.10 to 0.31.11.
- Bumped winnow from 1.0.3 to 1.0.4.
- Bumped zerocopy from 0.8.54 to 0.8.55.
- Bumped zerocopy-derive from 0.8.54 to 0.8.55.
- Bumped zmij from 1.0.21 to 1.0.23.

## 0.7.2

### Changed

- Bumped bytemuck from 1.25.0 to 1.25.1.
- Bumped bytemuck_derive from 1.10.2 to 1.11.0.
- Bumped cc from 1.2.66 to 1.2.67.
- Bumped zerocopy from 0.8.53 to 0.8.54.
- Bumped zerocopy-derive from 0.8.53 to 0.8.54.

## 0.7.1

### Changed

- Bumped crossbeam-channel from 0.5.15 to 0.5.16.
- Bumped crossbeam-epoch from 0.9.18 to 0.9.20.

## 0.7.0

### Addedd

- `--ea-module` emulates the robotron E/A expansion module. When MIDI is enabled, the byte stream is routed to the module's port B (`0xC9`) instead of the system PIO port B (`0x89`), which stays the default.

## 0.6.1

### Added

- `--floppy` may be repeated to mount up to four disks on drives A-D, each with an optional `:rw`/`:ro` suffix, mounts are read-only by default.
- Writable mounts persist guest writes to the host file in place.
- A folder may be mounted instead of an image and is served as a live CP/M disk.
- `mldos-1738k` format (1760/1738K ML-DOS) with DateStamper.

## 0.6.0

### Added

- `--floppy <image>` attaches a disk image to drive A through a U8272 controller (status/data at `0x98-0x9F`, terminal-count and reset control at `0xA0-0xA7`). The container format is auto-detected from its signature.
- `--floppy-format <format>` sets the geometry for raw images.
- `utils/img2dir.py`: unpacks the CP/M (CP/A) file system inside a raw image into a folder and repacks it, preserving file order, user areas and R/S/A attributes through a `manifest.json`.
- `--power-save` sleeps when the audio buffer is full instead of busy-waiting and parks the window loop between frames.

### Changed

- Relicensed utils from Zlib to LGPL.
- Added `flate2` and `lzhuf` dependencies for compressed containers.
- Tape support moved from `core` into the `peripherals` module. The public path is now `core::peripherals::tape`.

### Fixed

- 64K RAM module banking now toggles on `IN` as well as `OUT`.

## 0.5.1

### Fixed

- Fixed Z 9001 border colour.

## 0.5.0

### Added

- `--tape <source>` feeds an external audio signal into the cassette input by strobing port A of the system PIO (`0x88`) on every zero crossing, the way the KC 87 reads tape. macOS only.
- `utils/bin2wav.py`: standalone converter for KC 87/Z 9001 cassette formats.

## 0.4.1

### Fixed

- Video and color RAM are now contended. On a read or write to video RAM (`0xEC00-0xEFFF`), or to color RAM (`0xE800-0xEBFF`) unless the character generator window is active, during the visible part of a scanline, the bus asserts the U880 `WAIT` pin and holds the access until the end of the visible region. Line timing is derived in the bus from `MASTER_CLOCK_HZ`, `FRAME_RATE_HZ` and 312 total / 192 visible lines; the visible region is `(TSTATES_PER_LINE + 1) / 2` = 79 of 157 T-states. This requires `u880` >= 0.1.4, where the memory `WAIT` pin is sampled on the data cycle with the address already on the bus instead of before the address is asserted.
- PIO interrupt daisy-chain priority. The keyboard PIO (port `0x90`) is now ticked before the system PIO (port `0x88`) and the CTC (`0x80`).
- Autoload begin only when the program counter is inside the OS console-wait loop at `0xF924-0xF929` (`LD A,(0x0025); OR A; JR Z`), which the OS reaches only after it has finished initialising and is polling the buffer that the keyboard interrupt handler fills. The remaining keys keep the `0x0025 == 0` handshake.
- The "MEMORY END:" reply is no longer timed by a fixed `CLOCK_HZ / 2` delay. `AwaitMemPrompt` now waits for the screen to settle through the shared `poll_screen_settled` helper before answering, so the answer is typed only after BASIC has drawn the prompt.

### Changed

- Bumped `u880` from 0.1.3 to 0.1.4.

## 0.4.0

### Added

- `--ostalgie[=<preset>]` enables a CRT display effect ported from cool-retro-term, rendered as a wgpu post-processing pass over the framebuffer.
- Fullscreen toggle with `F11` or `Alt`+`Enter`.

## 0.3.2

### Added

- `--bin file@address` loads a raw, headerless binary at an explicit address (decimal or hex), and may be repeated.
- An `.sss` program can be loaded together with the modules it depends on: a driver via `--kcc`/`--tap` and data via `--bin`, for example `kc87 --graphics=robotron --sss eric-basic.sss --tap graf_com.tap --bin eric-data.bin@0x2E00 -a`. Modules are pre-loaded after BASIC comes up and are never auto-executed.
- `-a`/`--autorun` with no program now auto-enters BASIC instead of doing nothing.

### Changed

- `--sss` no longer conflicts with `--kcc`/`--tap`; a lone machine-code module with no `--sss`/`--bin` keeps the previous run-the-program behaviour.
- The `.sss` autoloader answers "MEMORY END:" with the lowest module's load address minus one, so BASIC's RAM sizing reaches none of the modules.
- Replays record the pre-loaded modules (name, SHA-256, load address) via `ReplayMetadata::modules`; the e2e harness reconstructs and hash-checks them.

## 0.3.1

### Fixed

- KCC and KC-TAP loader now reads the execution address from header offset 21 unconditionally, instead of only when `num_addr > 2`. Files that declare two address fields yet store a valid entry point previously jumped to the load address and executed packed image bytes as U880 instructions. Values 0x0000 and 0xFFFF are treated as "no autostart", matching JKCEMU's `FileInfo.getStartAddr` behaviour. The unused constant `KCC_BASE_ADDR_COUNT` has been removed.

## 0.3.0

On macOS the MIDI output now uses CoreMIDI directly with host-timestamped, driver-scheduled delivery instead of midir's immediate send. Each message is handed to the driver stamped with the exact host time derived from the emulated CPU cycle on which the music program emitted it, so notes land on their cycle-accurate beat without the scheduler wake-up jitter of a busy-wait followed by an immediate send. The note-to-note intervals computed by the assembly program are reproduced exactly at the output. Every other platform continues to use midir's immediate-send API and is unchanged.

### Added

- `MidiConn` MIDI-output abstraction with two `cfg`-gated backends: a newtype over `midir::MidiOutputConnection` off macOS, and a `coremidi::OutputPort` + `coremidi::Destination` pair on macOS exposing `send_now()` and a timestamped `send_at()`.
- macOS host-clock helpers `now_host()` / `nanos_to_host()` bound to `AudioGetCurrentHostTime` / `AudioConvertNanosToHostTime` from the CoreAudio framework.
- macOS `run_midi_thread` that anchors the emulated cycle counter to a CoreMIDI host-time grid `(anchor_host, anchor_cycle)` and schedules every message at `anchor_host + delta_cycles / cpu_freq`, letting the driver deliver it on time; it re-anchors and calls `coremidi::flush()` on stalls, pauses, and hard resets.

### Changed

- MIDI backend is now platform-specific in `Cargo.toml`: `coremidi 0.9.1` for `cfg(target_os = "macos")`, `midir 0.11.0` for `cfg(not(target_os = "macos"))`.
- `AppConfig::midi_out` is now `Option<MidiConn>` instead of `Option<midir::MidiOutputConnection>`.
- The inline MIDI thread closure is extracted into a `cfg`-gated `run_midi_thread`; the non-macOS path keeps the existing spin-sleep-then-send timing unchanged.
- `silence_active_notes()` now takes `&mut MidiConn` and sends through `send_now()`.
- MIDI device discovery and `--midi` resolution are factored into `cfg`-gated `list_midi_outputs()` / `open_midi_output()`; on macOS they enumerate `coremidi::Destinations` and select by name or index.

## 0.2.4

### Fixed
- Screen border color is now gated on the color module.

## 0.2.3

### Changes
- Bumped cpal from 0.17.3 to 0.18.1.

## 0.2.2

### Changed
- Keyboard keys are now represented as a typed `Key` enum instead of raw `i32` codes.
- Memory map addresses and I/O port numbers are now defined as named constants in a dedicated `memory_map` module.
- Chip register offsets, masks and control command codes in `rtc7242x`, `u855`, and `u857` are now named constants.
- Machine loader offsets (KCC, KC-TAP, SSS headers), CPU reset vector, and screen hash parameters are now named constants.
- Replay player now reads machine type and hardware configuration directly from replay metadata.

## 0.2.1

### Added
- Runtime integrity check.

## 0.2.0

### Added
- KC-BASIC `.sss` program loading via `--sss` or a positional `.sss` file. The loader drives the BASIC interpreter through the OS keyboard buffer, answers the MEMORY END prompt, waits for the screen to settle, and injects the program, optionally autorunning it with `RUN`.
- 80-column display mode via `--80col`, backed by a second video and color RAM bank with runtime framebuffer resizing.
- EPSON 7242X real-time clock via `--rtc`, mapped at I/O ports 0x60..0x6F.
- `payload_format`, `c80` and `rtc` fields in replay metadata for deterministic playback.

### Fixed
- Cursor blink rate now matches the 200ms hardware timer instead of toggling every 320ms.
- Keyboard sticky-key window now uses the real 50Hz frame time, fixing a duration that expired early.

## 0.1.1

### Added

- Initial commit
