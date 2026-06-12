// This file is part of kc87.
//
// Copyright (c) 2026  René Coignard <contact@renecoignard.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use assert_json_diff::assert_json_eq;
use kc87::core::debug::ReplayPlayer;
use kc87::core::machine::{Hardware, Machine, MachineType};
use kc87::core::video::VideoRenderer;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const KC87_OS_ROM: &[u8] = include_bytes!("../firmware/kc87/os.rom");
const KC87_BASIC_ROM: &[u8] = include_bytes!("../firmware/kc87/basic.rom");
const KC87_CHARGEN_ROM: &[u8] = include_bytes!("../firmware/kc87/chargen.rom");

const Z9001_OS_1: &[u8] = include_bytes!("../firmware/z9001/os_1.rom");
const Z9001_OS_2: &[u8] = include_bytes!("../firmware/z9001/os_2.rom");
const Z9001_BASIC_ROM: &[u8] = include_bytes!("../firmware/z9001/basic.rom");
const Z9001_CHARGEN_ROM: &[u8] = include_bytes!("../firmware/z9001/chargen.rom");

const BOOT_SENTINEL: &str = "os";

#[test]
fn replays_match_snapshots() {
    let entries = match fs::read_dir("tests/replays") {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            run_replay(&path.to_string_lossy());
        }
    }
}

fn run_replay(replay_path_str: &str) {
    let mut player = ReplayPlayer::from_file(replay_path_str)
        .unwrap_or_else(|_| panic!("Failed to parse replay JSON: {:?}", replay_path_str));

    let program = player.replay.metadata.program.clone();

    let base_name = if program.is_empty() {
        BOOT_SENTINEL.to_string()
    } else {
        Path::new(&program)
            .file_stem()
            .expect("program should have a valid stem")
            .to_string_lossy()
            .into_owned()
    };

    let sample_rate = player.replay.metadata.sample_rate;
    let autorun = player.replay.metadata.autorun;
    let machine_type = player.replay.metadata.machine_type;
    let hardware = Hardware {
        ram: player.replay.metadata.ram,
        chargen: player.replay.metadata.chargen,
        graphics: player.replay.metadata.graphics,
        c80: player.replay.metadata.c80,
        rtc: player.replay.metadata.rtc,
    };

    let (basic_rom, os_rom_1, os_rom_2, font_rom, os_rom_hash) = match machine_type {
        MachineType::KC87 => (
            KC87_BASIC_ROM.to_vec(),
            KC87_OS_ROM.to_vec(),
            None,
            KC87_CHARGEN_ROM.to_vec(),
            hex::encode(Sha256::digest(KC87_OS_ROM)),
        ),
        MachineType::Z9001 => {
            let mut combined = Z9001_OS_1.to_vec();
            combined.extend_from_slice(Z9001_OS_2);
            (
                Z9001_BASIC_ROM.to_vec(),
                Z9001_OS_1.to_vec(),
                Some(Z9001_OS_2.to_vec()),
                Z9001_CHARGEN_ROM.to_vec(),
                hex::encode(Sha256::digest(&combined)),
            )
        }
    };

    let rom_module = player.replay.metadata.rom_module.as_ref().map(|name| {
        let rom_path = PathBuf::from("tests/assets").join(name);
        let data = fs::read(&rom_path)
            .unwrap_or_else(|_| panic!("Failed to read ROM module at {:?}", rom_path));
        if let Some(expected) = &player.replay.metadata.rom_module_sha256 {
            let actual = hex::encode(Sha256::digest(&data));
            assert_eq!(
                expected, &actual,
                "ROM module hash mismatch for '{}' ({:?})",
                name, machine_type
            );
        }
        data
    });

    let mut machine = Machine::new(
        machine_type,
        hardware,
        rom_module,
        basic_rom,
        os_rom_1,
        os_rom_2,
        sample_rate,
    );
    let mut video = VideoRenderer::new(font_rom, hardware.c80);

    if base_name == BOOT_SENTINEL {
        assert_eq!(
            os_rom_hash, player.replay.metadata.program_sha256,
            "OS ROM hash mismatch for '{}' ({:?})",
            base_name, machine_type
        );
    } else {
        let program_path = PathBuf::from("tests/assets").join(&program);

        let program_data = fs::read(&program_path)
            .unwrap_or_else(|_| panic!("Failed to read program at {:?}", program_path));

        let loaded_hash = hex::encode(Sha256::digest(&program_data));

        assert_eq!(
            loaded_hash, player.replay.metadata.program_sha256,
            "program hash mismatch for '{}'",
            base_name
        );

        machine.schedule_load(program_data, player.replay.metadata.payload_format, autorun);
    }

    let update_snapshots = std::env::var("UPDATE_SNAPSHOTS").is_ok();
    let update_screenshots = std::env::var("UPDATE_SCREENSHOTS").is_ok();

    while !player.is_finished() {
        let snapshots = player.apply_pending_events(&mut machine);

        for snap in snapshots {
            let json_path = format!("tests/dumps/{}/{}.json", base_name, snap);
            let state = machine.state();

            if update_snapshots {
                let file = fs::File::create(&json_path)
                    .unwrap_or_else(|_| panic!("Failed to overwrite JSON: {}", json_path));
                serde_json::to_writer_pretty(file, &state).expect("Failed to write updated JSON");
            } else {
                let expected_json_file = fs::File::open(&json_path)
                    .unwrap_or_else(|_| panic!("Failed to open expected JSON: {}", json_path));

                let expected_state: serde_json::Value = serde_json::from_reader(expected_json_file)
                    .expect("Failed to parse expected JSON");

                let actual_state =
                    serde_json::to_value(&state).expect("Failed to serialize machine state");

                assert_json_eq!(expected_state, actual_state);
            }

            let png_path = format!("tests/dumps/{}/{}.png", base_name, snap);
            let actual_pixels = video.frame_buffer();
            let width = video.width();
            let height = video.height();

            if update_screenshots {
                if let Ok(old_img) = image::open(&png_path) {
                    let old_rgba = old_img.into_rgba8();
                    let expected_pixels = old_rgba.as_raw();

                    if actual_pixels != expected_pixels {
                        let before_path = format!("{}_before.png", snap);
                        let after_path = format!("{}_after.png", snap);

                        let _ = image::save_buffer(
                            &before_path,
                            expected_pixels,
                            old_rgba.width(),
                            old_rgba.height(),
                            image::ExtendedColorType::Rgba8,
                        );

                        let _ = image::save_buffer(
                            &after_path,
                            actual_pixels,
                            width,
                            height,
                            image::ExtendedColorType::Rgba8,
                        );

                        image::save_buffer(
                            &png_path,
                            actual_pixels,
                            width,
                            height,
                            image::ExtendedColorType::Rgba8,
                        )
                        .unwrap_or_else(|_| panic!("Failed to overwrite PNG: {}", png_path));
                    }
                } else {
                    image::save_buffer(
                        &png_path,
                        actual_pixels,
                        width,
                        height,
                        image::ExtendedColorType::Rgba8,
                    )
                    .unwrap_or_else(|_| panic!("Failed to create new PNG: {}", png_path));
                }
            } else {
                let expected_img = image::open(&png_path)
                    .unwrap_or_else(|_| panic!("Failed to open expected PNG: {}", png_path))
                    .into_rgba8();

                let expected_pixels = expected_img.as_raw();

                assert_eq!(
                    actual_pixels.len(),
                    expected_pixels.len(),
                    "framebuffer size mismatch at snapshot '{}'",
                    snap
                );

                if actual_pixels != expected_pixels {
                    let mut diffs = 0;
                    let mut first_diff = None;

                    for (i, (&a, &e)) in
                        actual_pixels.iter().zip(expected_pixels.iter()).enumerate()
                    {
                        if a != e {
                            diffs += 1;
                            if first_diff.is_none() {
                                first_diff = Some((i, a, e));
                            }
                        }
                    }

                    if let Some((idx, act, exp)) = first_diff {
                        panic!(
                            "pixel mismatch at snapshot '{}' (program: {}): {} bytes differ, first at [{}]: actual={}, expected={}",
                            snap, base_name, diffs, idx, act, exp
                        );
                    }
                }
            }
        }

        let vblank_occurred = machine.tick(|_| {});

        if vblank_occurred {
            video.render_frame(&machine.bus, machine.machine_type);
        }
    }
}
