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
use crate::app::shaders::Preset;
#[cfg(target_os = "macos")]
use crate::app::tape::TapeSource;
use crate::app::{App, AppConfig, MachineConfig, MidiConn};
use kc87::core::debug::{ReplayMetadata, ReplayModule, ReplayPlayer, ReplayRecorder};
use kc87::core::machine::{
    GraphicsModule, Hardware, LoadFormat, MachineType, ModulePreload, RamSize,
};
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

#[cfg(not(target_os = "macos"))]
fn list_midi_outputs(client_name: &str) {
    if let Ok(midi_out) = midir::MidiOutput::new(client_name) {
        for (i, port) in midi_out.ports().iter().enumerate() {
            if let Ok(name) = midi_out.port_name(port) {
                println!("{}: {}", i, name);
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn open_midi_output(client_name: &str, midi_arg: &str) -> Option<MidiConn> {
    let midi_out = midir::MidiOutput::new(client_name).ok()?;
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
            .unwrap_or_else(|_| format!("{} MIDI Out", client_name));
        midi_out.connect(&port, &conn_name).ok().map(MidiConn::new)
    } else {
        #[cfg(unix)]
        {
            use midir::os::unix::VirtualOutput;
            midi_out.create_virtual(midi_arg).ok().map(MidiConn::new)
        }
        #[cfg(not(unix))]
        {
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn list_midi_outputs(_client_name: &str) {
    for i in 0..coremidi::Destinations::count() {
        if let Some(dest) = coremidi::Destination::from_index(i) {
            let name = dest
                .display_name()
                .unwrap_or_else(|| String::from("Unknown Device"));
            println!("{}: {}", i, name);
        }
    }
}

#[cfg(target_os = "macos")]
fn open_midi_output(client_name: &str, midi_arg: &str) -> Option<MidiConn> {
    let count = coremidi::Destinations::count();
    if count == 0 {
        return None;
    }

    let index = if midi_arg.is_empty() {
        Some(0)
    } else {
        (0..count)
            .find(|&i| {
                coremidi::Destination::from_index(i)
                    .and_then(|d| d.display_name())
                    .is_some_and(|name| name == *midi_arg)
            })
            .or_else(|| midi_arg.parse::<usize>().ok().filter(|&idx| idx < count))
    };

    let dest = coremidi::Destination::from_index(index?)?;
    let client = coremidi::Client::new(client_name).ok()?;
    let port = client
        .output_port(&format!("{} MIDI Out", client_name))
        .ok()?;
    Some(MidiConn::new(port, dest))
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

#[cfg(target_os = "macos")]
#[derive(Copy, Clone, Debug, ValueEnum)]
enum TapeSourceArg {
    #[value(name = "input")]
    Input,
    #[value(name = "system")]
    System,
}

#[cfg(target_os = "macos")]
impl From<TapeSourceArg> for TapeSource {
    fn from(arg: TapeSourceArg) -> Self {
        match arg {
            TapeSourceArg::Input => TapeSource::Input,
            TapeSourceArg::System => TapeSource::System,
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
        help_heading = "General options",
        verbatim_doc_comment
    )]
    sss: Option<String>,

    /// Path to a raw binary to load at an address, as file@address
    #[arg(
        long,
        value_name = "file@addr",
        help_heading = "General options",
        verbatim_doc_comment
    )]
    bin: Vec<String>,

    /// Path to a ROM module image (.rom) to map at C000h
    #[arg(long, value_name = "file", help_heading = "General options")]
    rom: Option<String>,

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

    /// Enable the CRT shader
    /// Default: built-in profile
    #[arg(
        long,
        value_name = "preset",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "@default",
        help_heading = "Display options",
        verbatim_doc_comment
    )]
    ostalgie: Option<String>,

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

    /// Tape input source
    /// Possible values: input, system
    #[cfg(target_os = "macos")]
    #[arg(
        long,
        value_name = "source",
        value_enum,
        num_args = 0..=1,
        hide_possible_values = true,
        default_missing_value = "system",
        help_heading = "Tape options",
        verbatim_doc_comment
    )]
    tape: Option<TapeSourceArg>,

    /// Audio input device
    /// Default: system default input
    #[cfg(target_os = "macos")]
    #[arg(
        long,
        value_name = "name",
        requires = "tape",
        help_heading = "Tape options",
        verbatim_doc_comment
    )]
    tape_device: Option<String>,

    /// List available audio input devices and exit
    #[cfg(target_os = "macos")]
    #[arg(long, help_heading = "Tape options")]
    tape_input_list: bool,

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

    let player = if let Some(path) = &args.play {
        Some(ReplayPlayer::from_file(path)?)
    } else {
        None
    };

    let machine_type: MachineType = player
        .as_ref()
        .map(|p| p.replay.metadata.machine_type)
        .unwrap_or_else(|| args.machine.into());
    let midi_name = midi_client_name(machine_type);

    let hardware = match player.as_ref() {
        Some(p) => {
            let m = &p.replay.metadata;
            Hardware {
                ram: m.ram,
                chargen: m.chargen,
                graphics: m.graphics,
                c80: m.c80,
                rtc: m.rtc,
            }
        }
        None => Hardware {
            ram: args.ram.map(RamSize::from).unwrap_or_default(),
            chargen: args.pzg,
            graphics: args.graphics.map(GraphicsModule::from).unwrap_or_default(),
            c80: args.col80,
            rtc: args.rtc,
        },
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
        list_midi_outputs(midi_name);
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    if args.tape_input_list {
        crate::app::tape::list_devices();
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

    let mut sss_path = args.sss.clone();
    let mut module_path = args.kcc.clone().or_else(|| args.tap.clone());
    let mut rom_path = args.rom.clone();

    if let Some(file) = &args.file {
        let ext = std::path::Path::new(file)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        match ext.as_deref() {
            Some("sss") => {
                if sss_path.is_none() {
                    sss_path = Some(file.clone());
                }
            }
            Some("kcc") | Some("tap") => {
                if module_path.is_none() {
                    module_path = Some(file.clone());
                }
            }
            Some("rom") => {
                if rom_path.is_none() {
                    rom_path = Some(file.clone());
                }
            }
            _ => anyhow::bail!(
                "unsupported file extension for '{}': only .kcc, .tap, .sss and .rom are allowed",
                file
            ),
        }
    }

    let mut bin_modules: Vec<(u16, String)> = Vec::new();
    for spec in &args.bin {
        let (path, addr_str) = spec.rsplit_once('@').with_context(|| {
            format!(
                "invalid --bin '{}': expected PATH@ADDRESS (e.g. data.bin@0x2E00)",
                spec
            )
        })?;
        let addr_str = addr_str.trim();
        let addr = match addr_str
            .strip_prefix("0x")
            .or_else(|| addr_str.strip_prefix("0X"))
        {
            Some(hex) => u16::from_str_radix(hex, 16),
            None => addr_str.parse::<u16>(),
        }
        .with_context(|| format!("invalid address in --bin '{}'", spec))?;
        bin_modules.push((addr, path.to_string()));
    }
    let has_bins = !bin_modules.is_empty();

    let (program_path, program_format, driver_module): (
        Option<String>,
        LoadFormat,
        Option<String>,
    ) = match (sss_path, module_path, has_bins) {
        (Some(sss), module, _) => (Some(sss), LoadFormat::Sss, module),
        (None, Some(module), false) => (Some(module), LoadFormat::Auto, None),
        (None, module, true) => (None, LoadFormat::Auto, module),
        (None, None, false) => (None, LoadFormat::Auto, None),
    };

    let program_format = player
        .as_ref()
        .map(|p| p.replay.metadata.payload_format)
        .unwrap_or(program_format);

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

    let mut modules: Vec<ModulePreload> = Vec::new();
    let mut replay_modules: Vec<ReplayModule> = Vec::new();
    if let Some(path) = &driver_module {
        let data = fs::read(path).with_context(|| format!("could not read module '{}'", path))?;
        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        replay_modules.push(ReplayModule {
            name,
            sha256: hex::encode(Sha256::digest(&data)),
            addr: None,
        });
        modules.push(ModulePreload::headered(data));
    }
    for (addr, path) in &bin_modules {
        let data = fs::read(path).with_context(|| format!("could not read binary '{}'", path))?;
        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        replay_modules.push(ReplayModule {
            name,
            sha256: hex::encode(Sha256::digest(&data)),
            addr: Some(*addr),
        });
        modules.push(ModulePreload::raw(data, *addr));
    }

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

    if let Some(p) = &player {
        p.verify_program_hash(&program_sha256)?;
    }

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

    let midi_conn = args
        .midi
        .as_ref()
        .and_then(|midi_arg| open_midi_output(midi_name, midi_arg));

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
        modules,
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
            modules: replay_modules,
        })
    });

    let ostalgie = match args.ostalgie.as_deref() {
        None => None,
        Some("@default") => Some(Preset::default_for(machine_type)),
        Some(path) => {
            let json = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read ostalgie preset '{path}'"))?;
            let preset = Preset::from_json(&json)
                .with_context(|| format!("Failed to parse ostalgie preset '{path}'"))?;
            Some(preset)
        }
    };

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
            ostalgie,
            #[cfg(target_os = "macos")]
            tape: args.tape.map(Into::into),
            #[cfg(target_os = "macos")]
            tape_device: args.tape_device,
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
