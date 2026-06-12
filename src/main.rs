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

mod app;

use std::fs;

use anyhow::{Context, Result, ensure};
use clap::{CommandFactory, FromArgMatches, Parser, ValueEnum};
use sha2::{Digest, Sha256};
use winit::event_loop::EventLoop;

use crate::app::audio::AudioSystem;
use crate::app::keyboard::KeyboardLayout;
use crate::app::{App, AppConfig, MachineConfig};
use kc87::core::debug::{ReplayMetadata, ReplayPlayer, ReplayRecorder};
use kc87::core::machine::{GraphicsModule, Hardware, LoadFormat, MachineType, RamSize};
use kc87::core::video::VideoRenderer;

const KC87_OS_ROM: &[u8] = include_bytes!("../firmware/kc87/os.rom");
const KC87_BASIC_ROM: &[u8] = include_bytes!("../firmware/kc87/basic.rom");
const KC87_CHARGEN_ROM: &[u8] = include_bytes!("../firmware/kc87/chargen.rom");

const Z9001_OS_1: &[u8] = include_bytes!("../firmware/z9001/os_1.rom");
const Z9001_OS_2: &[u8] = include_bytes!("../firmware/z9001/os_2.rom");
const Z9001_BASIC_ROM: &[u8] = include_bytes!("../firmware/z9001/basic.rom");
const Z9001_CHARGEN_ROM: &[u8] = include_bytes!("../firmware/z9001/chargen.rom");

const KC87_OS_ROM_HASH: &str = include_str!("../firmware/kc87/os.rom.sha256").trim_ascii();
const KC87_BASIC_ROM_HASH: &str = include_str!("../firmware/kc87/basic.rom.sha256").trim_ascii();
const KC87_CHARGEN_ROM_HASH: &str =
    include_str!("../firmware/kc87/chargen.rom.sha256").trim_ascii();

const Z9001_OS_1_HASH: &str = include_str!("../firmware/z9001/os_1.rom.sha256").trim_ascii();
const Z9001_OS_2_HASH: &str = include_str!("../firmware/z9001/os_2.rom.sha256").trim_ascii();
const Z9001_BASIC_ROM_HASH: &str = include_str!("../firmware/z9001/basic.rom.sha256").trim_ascii();
const Z9001_CHARGEN_ROM_HASH: &str =
    include_str!("../firmware/z9001/chargen.rom.sha256").trim_ascii();

fn check_integrity() -> Result<()> {
    let verify = |name: &str, data: &[u8], expected: &str| -> Result<()> {
        let hash = Sha256::digest(data);
        let actual = hex::encode(hash);
        ensure!(
            actual == expected,
            "integrity check failed for asset '{}'",
            name
        );
        Ok(())
    };

    verify("kc87/os.rom", KC87_OS_ROM, KC87_OS_ROM_HASH)?;
    verify("kc87/basic.rom", KC87_BASIC_ROM, KC87_BASIC_ROM_HASH)?;
    verify("kc87/chargen.rom", KC87_CHARGEN_ROM, KC87_CHARGEN_ROM_HASH)?;

    verify("z9001/os_1.rom", Z9001_OS_1, Z9001_OS_1_HASH)?;
    verify("z9001/os_2.rom", Z9001_OS_2, Z9001_OS_2_HASH)?;
    verify("z9001/basic.rom", Z9001_BASIC_ROM, Z9001_BASIC_ROM_HASH)?;
    verify(
        "z9001/chargen.rom",
        Z9001_CHARGEN_ROM,
        Z9001_CHARGEN_ROM_HASH,
    )?;

    Ok(())
}

fn midi_client_name(machine_type: MachineType) -> &'static str {
    match machine_type {
        MachineType::KC87 => "Robotron KC 87",
        MachineType::Z9001 => "Robotron Z 9001",
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum MachineModel {
    #[value(name = "kc87")]
    KC87,
    #[value(name = "z9001")]
    Z9001,
}

impl From<MachineModel> for MachineType {
    fn from(model: MachineModel) -> Self {
        match model {
            MachineModel::KC87 => MachineType::KC87,
            MachineModel::Z9001 => MachineType::Z9001,
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum RamArg {
    #[value(name = "16k")]
    K16,
    #[value(name = "32k")]
    K32,
    #[value(name = "48k")]
    K48,
    #[value(name = "64k")]
    K64,
}

impl From<RamArg> for RamSize {
    fn from(arg: RamArg) -> Self {
        match arg {
            RamArg::K16 => RamSize::K16,
            RamArg::K32 => RamSize::K32,
            RamArg::K48 => RamSize::K48,
            RamArg::K64 => RamSize::K64,
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum GraphicsArg {
    #[value(name = "robotron")]
    Robotron,
    #[value(name = "krt")]
    Krt,
}

impl From<GraphicsArg> for GraphicsModule {
    fn from(arg: GraphicsArg) -> Self {
        match arg {
            GraphicsArg::Robotron => GraphicsModule::Robotron,
            GraphicsArg::Krt => GraphicsModule::Krt,
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum KeyboardLayoutArg {
    #[value(name = "smart")]
    Smart,
    #[value(name = "qwertz")]
    Qwertz,
}

impl From<KeyboardLayoutArg> for KeyboardLayout {
    fn from(arg: KeyboardLayoutArg) -> Self {
        match arg {
            KeyboardLayoutArg::Smart => KeyboardLayout::Smart,
            KeyboardLayoutArg::Qwertz => KeyboardLayout::Qwertz,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "kc87",
    version,
    override_usage = "kc87 [options] [file]",
    disable_help_flag = true,
    disable_version_flag = true,
    next_line_help = true,
    help_template = "Usage: {usage}\n\n{all-args}"
)]
struct Args {
    #[arg(value_name = "file", hide = true)]
    file: Option<String>,

    /// Select machine model
    /// Default: kc87
    /// Possible values: kc87, z9001
    #[arg(
        long,
        value_name = "model",
        value_enum,
        default_value_t = MachineModel::KC87,
        hide_default_value = true,
        hide_possible_values = true,
        help_heading = "General options",
        verbatim_doc_comment
    )]
    machine: MachineModel,

    /// Path to a KCC program image (.kcc) to load
    #[arg(
        long,
        value_name = "file",
        conflicts_with = "tap",
        help_heading = "General options"
    )]
    kcc: Option<String>,

    /// Path to a KC-TAP program image (.tap) to load
    #[arg(long, value_name = "file", help_heading = "General options")]
    tap: Option<String>,

    /// Path to a KC-BASIC program (.sss) to load
    #[arg(
        long,
        value_name = "file",
        conflicts_with_all = ["kcc", "tap"],
        help_heading = "General options"
    )]
    sss: Option<String>,

    /// Path to a ROM module image (.rom) to map at C000h
    #[arg(long, value_name = "file", help_heading = "General options")]
    rom: Option<String>,

    /// RAM expansion
    /// Default: 48k
    /// Possible values: 16k, 32k, 48k, 64k
    #[arg(
        long,
        value_name = "size",
        value_enum,
        hide_possible_values = true,
        help_heading = "Extension options",
        verbatim_doc_comment
    )]
    ram: Option<RamArg>,

    /// Enable the programmable character generator
    #[arg(long, help_heading = "Extension options")]
    pzg: bool,

    /// Enable the 80-character display mode
    #[arg(long = "80col", help_heading = "Extension options")]
    col80: bool,

    /// Enable the real-time clock
    #[arg(long, help_heading = "Extension options")]
    rtc: bool,

    /// Full-graphics expansion
    /// Possible values: robotron, krt
    #[arg(
        long,
        value_name = "type",
        value_enum,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "robotron",
        hide_possible_values = true,
        help_heading = "Extension options",
        verbatim_doc_comment
    )]
    graphics: Option<GraphicsArg>,

    /// Select keyboard layout
    /// Default: smart
    /// Possible values: smart, qwertz
    #[arg(
        long,
        value_name = "layout",
        value_enum,
        default_value_t = KeyboardLayoutArg::Smart,
        hide_default_value = true,
        hide_possible_values = true,
        help_heading = "Keyboard options",
        verbatim_doc_comment
    )]
    keyboard_layout: KeyboardLayoutArg,

    /// Run the loaded program immediately after startup
    #[arg(short = 'a', long = "autorun", help_heading = "General options")]
    autorun: bool,

    /// Print this message and exit
    #[arg(
        short = 'h',
        long = "help",
        action = clap::ArgAction::Help,
        help_heading = "General options"
    )]
    help: Option<bool>,

    /// Print version information and exit
    #[arg(
        short = 'V',
        long = "version",
        action = clap::ArgAction::Version,
        help_heading = "General options"
    )]
    version: Option<bool>,

    /// Connect to a MIDI output port by name or index
    /// Default: first available port
    #[arg(
        long,
        value_name = "port",
        num_args = 0..=1,
        default_missing_value = "",
        hide_default_value = true,
        help_heading = "MIDI options",
        verbatim_doc_comment
    )]
    midi: Option<String>,

    /// List available MIDI output ports and exit
    #[arg(long, help_heading = "MIDI options")]
    midi_list: bool,

    /// Enable debug hotkeys
    #[arg(long, help_heading = "Debug options")]
    debug: bool,

    /// Enable replay recording mode
    #[arg(long, requires = "debug", help_heading = "Debug options")]
    record: bool,

    /// Play a recorded replay from a file
    #[arg(
        long,
        value_name = "file",
        conflicts_with = "record",
        help_heading = "Debug options"
    )]
    play: Option<String>,
}

fn main() -> Result<()> {
    check_integrity()?;

    let mut cmd = Args::command();

    if !std::env::args_os().any(|arg| arg == "--debug") {
        cmd = cmd
            .mut_arg("debug", |a| a.hide(true))
            .mut_arg("record", |a| a.hide(true))
            .mut_arg("play", |a| a.hide(true));
    }

    let matches = cmd.get_matches();
    let args = Args::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    let machine_type: MachineType = args.machine.into();
    let midi_name = midi_client_name(machine_type);

    let hardware = Hardware {
        ram: args.ram.map(RamSize::from).unwrap_or_default(),
        chargen: args.pzg,
        graphics: args.graphics.map(GraphicsModule::from).unwrap_or_default(),
        c80: args.col80,
        rtc: args.rtc,
    };

    if matches!(hardware.graphics, GraphicsModule::Robotron)
        && matches!(machine_type, MachineType::Z9001)
    {
        use clap::CommandFactory;
        Args::command()
            .error(
                clap::error::ErrorKind::ArgumentConflict,
                "the Robotron graphics extension requires the colour module (only available with --machine kc87)",
            )
            .exit();
    }

    if args.midi_list {
        if let Ok(midi_out) = midir::MidiOutput::new(midi_name) {
            for (i, port) in midi_out.ports().iter().enumerate() {
                if let Ok(name) = midi_out.port_name(port) {
                    println!("{}: {}", i, name);
                }
            }
        }
        return Ok(());
    }

    let (os_rom_1, os_rom_2, basic_rom, font_rom, os_rom_hash) = match machine_type {
        MachineType::KC87 => (
            KC87_OS_ROM.to_vec(),
            None,
            KC87_BASIC_ROM.to_vec(),
            KC87_CHARGEN_ROM.to_vec(),
            hex::encode(Sha256::digest(KC87_OS_ROM)),
        ),
        MachineType::Z9001 => {
            let mut combined = Z9001_OS_1.to_vec();
            combined.extend_from_slice(Z9001_OS_2);
            (
                Z9001_OS_1.to_vec(),
                Some(Z9001_OS_2.to_vec()),
                Z9001_BASIC_ROM.to_vec(),
                Z9001_CHARGEN_ROM.to_vec(),
                hex::encode(Sha256::digest(&combined)),
            )
        }
    };

    let explicit_program: Option<(String, LoadFormat)> = args
        .kcc
        .clone()
        .map(|p| (p, LoadFormat::Auto))
        .or_else(|| args.tap.clone().map(|p| (p, LoadFormat::Auto)))
        .or_else(|| args.sss.clone().map(|p| (p, LoadFormat::Sss)));
    let (program_path, program_format, rom_path) = match &args.file {
        Some(file) => {
            let ext = std::path::Path::new(file)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            match ext.as_deref() {
                Some("kcc") | Some("tap") => match explicit_program {
                    Some((p, fmt)) => (Some(p), fmt, args.rom.clone()),
                    None => (Some(file.clone()), LoadFormat::Auto, args.rom.clone()),
                },
                Some("sss") => match explicit_program {
                    Some((p, fmt)) => (Some(p), fmt, args.rom.clone()),
                    None => (Some(file.clone()), LoadFormat::Sss, args.rom.clone()),
                },
                Some("rom") => match explicit_program {
                    Some((p, fmt)) => (
                        Some(p),
                        fmt,
                        args.rom.clone().or_else(|| Some(file.clone())),
                    ),
                    None => (
                        None,
                        LoadFormat::Auto,
                        args.rom.clone().or_else(|| Some(file.clone())),
                    ),
                },
                _ => anyhow::bail!(
                    "unsupported file extension for '{}': only .kcc, .tap, .sss and .rom are allowed",
                    file
                ),
            }
        }
        None => match explicit_program {
            Some((p, fmt)) => (Some(p), fmt, args.rom.clone()),
            None => (None, LoadFormat::Auto, args.rom.clone()),
        },
    };

    let (payload, program, program_sha256, program_name) = if let Some(path) = &program_path {
        let data = fs::read(path).with_context(|| format!("could not read '{}'", path))?;
        let sha256 = hex::encode(Sha256::digest(&data));
        let p = std::path::Path::new(path);
        let program = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = p
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("program"));

        (Some(data), program, sha256, name)
    } else {
        (None, String::from("os"), os_rom_hash, String::from("os"))
    };

    let (rom_module, rom_module_name, rom_module_sha256) = if let Some(path) = &rom_path {
        let data =
            fs::read(path).with_context(|| format!("could not read ROM module '{}'", path))?;
        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let sha256 = hex::encode(Sha256::digest(&data));
        (Some(data), Some(name), Some(sha256))
    } else {
        (None, None, None)
    };

    let player = if let Some(path) = &args.play {
        let player = ReplayPlayer::from_file(path)?;
        player.verify_program_hash(&program_sha256)?;
        Some(player)
    } else {
        None
    };

    let autorun = player
        .as_ref()
        .map(|p| p.replay.metadata.autorun)
        .unwrap_or(args.autorun);

    let event_loop = EventLoop::new().context("Failed to create winit event loop")?;

    let audio = AudioSystem::new().context("Failed to initialize audio system")?;
    let video = VideoRenderer::new(font_rom, hardware.c80);

    let sample_rate = player
        .as_ref()
        .map(|p| p.replay.metadata.sample_rate)
        .unwrap_or(audio.sample_rate);

    let midi_conn = if let Some(midi_arg) = &args.midi {
        midir::MidiOutput::new(midi_name).ok().and_then(|midi_out| {
            let ports = midi_out.ports();
            let target_port = if midi_arg.is_empty() {
                ports.first().cloned()
            } else {
                ports
                    .iter()
                    .find(|p| midi_out.port_name(p).is_ok_and(|name| name == *midi_arg))
                    .or_else(|| {
                        midi_arg
                            .parse::<usize>()
                            .ok()
                            .and_then(|idx| ports.get(idx))
                    })
                    .cloned()
            };

            if let Some(port) = target_port {
                let conn_name = midi_out
                    .port_name(&port)
                    .unwrap_or_else(|_| format!("{} MIDI Out", midi_name));
                midi_out.connect(&port, &conn_name).ok()
            } else {
                #[cfg(unix)]
                {
                    use midir::os::unix::VirtualOutput;
                    midi_out.create_virtual(midi_arg).ok()
                }
                #[cfg(not(unix))]
                {
                    None
                }
            }
        })
    } else {
        None
    };

    let midi_enabled = midi_conn.is_some() || args.midi.is_some();

    let machine_config = MachineConfig {
        machine_type,
        hardware,
        rom_module,
        basic_rom,
        os_rom_1,
        os_rom_2,
        sample_rate,
        payload,
        payload_format: program_format,
        autorun,
        program_name,
        midi_enabled,
    };

    let recorder = args.record.then(|| {
        ReplayRecorder::new(ReplayMetadata {
            program,
            program_sha256,
            autorun,
            sample_rate,
            machine_type,
            ram: hardware.ram,
            chargen: hardware.chargen,
            graphics: hardware.graphics,
            c80: hardware.c80,
            rtc: hardware.rtc,
            payload_format: program_format,
            rom_module: rom_module_name,
            rom_module_sha256,
        })
    });

    let mut app = App::new(
        machine_config,
        video,
        audio,
        AppConfig {
            debug_mode: args.debug,
            recorder,
            player,
            midi_out: midi_conn,
            keyboard_layout: args.keyboard_layout.into(),
        },
    );

    event_loop
        .run_app(&mut app)
        .context("Application execution failed")?;

    if let Some(err) = app.fatal_error.take() {
        return Err(err);
    }

    Ok(())
}
