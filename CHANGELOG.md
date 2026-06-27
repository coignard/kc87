# Changelog

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
