# Changelog

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
