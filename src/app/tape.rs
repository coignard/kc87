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

use std::ffi::{CString, c_void};
use std::ptr::{self, NonNull};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use block2::RcBlock;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, bounded};
use dispatch2::{DispatchQueue, DispatchRetained};
use objc2::AnyThread;
use objc2::runtime::Bool;
use objc2_core_audio::{
    AudioDeviceCreateIOProcIDWithBlock, AudioDeviceDestroyIOProcID, AudioDeviceIOProcID,
    AudioDeviceStart, AudioDeviceStop, AudioHardwareCreateAggregateDevice,
    AudioHardwareCreateProcessTap, AudioHardwareDestroyAggregateDevice,
    AudioHardwareDestroyProcessTap, AudioObjectGetPropertyData, AudioObjectID,
    AudioObjectPropertyAddress, CATapDescription, CATapMuteBehavior,
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceIsStackedKey,
    kAudioAggregateDeviceMainSubDeviceKey, kAudioAggregateDeviceSubDeviceListKey,
    kAudioAggregateDeviceTapAutoStartKey, kAudioAggregateDeviceTapListKey,
    kAudioAggregateDeviceUIDKey, kAudioDevicePropertyDeviceUID,
    kAudioHardwarePropertyDefaultSystemOutputDevice, kAudioObjectPropertyElementMain,
    kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, kAudioSubDeviceUIDKey,
    kAudioSubTapDriftCompensationKey, kAudioSubTapUIDKey, kAudioTapPropertyFormat,
};
use objc2_core_audio_types::{AudioBufferList, AudioStreamBasicDescription, AudioTimeStamp};
use objc2_core_foundation::{
    CFArray, CFBoolean, CFDictionary, CFRetained, CFRunLoop, CFString, CFType,
    kCFRunLoopDefaultMode,
};
use objc2_foundation::{NSArray, NSNumber, NSString, NSUUID};

const DISCLAIM_GUARD: &str = "KC87_TAPE_DISCLAIMED";
const PERMISSION_TIMEOUT: Duration = Duration::from_secs(120);

type TapeIoBlock = RcBlock<
    dyn Fn(
        NonNull<AudioTimeStamp>,
        NonNull<AudioBufferList>,
        NonNull<AudioTimeStamp>,
        NonNull<AudioBufferList>,
        NonNull<AudioTimeStamp>,
    ),
>;

type DisclaimFn = unsafe extern "C" fn(*mut libc::posix_spawnattr_t, libc::c_int) -> libc::c_int;
type PreflightFn = unsafe extern "C" fn(*const c_void, *const c_void) -> libc::c_int;
type RequestFn = unsafe extern "C" fn(*const c_void, *const c_void, *const c_void);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TapeSource {
    Input,
    System,
}

pub trait TapeIn {
    fn poll_error(&mut self) -> Option<anyhow::Error>;
}

pub fn start(
    source: TapeSource,
    device: Option<&str>,
) -> Result<(Box<dyn TapeIn>, Receiver<f32>, u32)> {
    match source {
        TapeSource::Input => CpalTapeIn::start(device),
        TapeSource::System => CoreAudioTapIn::start(),
    }
}

pub fn list_devices() {
    let host = cpal::default_host();
    let default = host
        .default_input_device()
        .and_then(|device| device.id().ok())
        .map(|id| id.to_string());

    let Ok(devices) = host.input_devices() else {
        return;
    };
    for device in devices {
        let Ok(id) = device.id() else {
            continue;
        };
        let id = id.to_string();
        if Some(&id) == default.as_ref() {
            println!("{id} (default)");
        } else {
            println!("{id}");
        }
    }
}

struct CpalTapeIn {
    _stream: cpal::Stream,
    err_rx: Receiver<anyhow::Error>,
}

impl CpalTapeIn {
    fn start(device_name: Option<&str>) -> Result<(Box<dyn TapeIn>, Receiver<f32>, u32)> {
        let host = cpal::default_host();
        let device = match device_name {
            Some(name) => host
                .input_devices()
                .context("Failed to enumerate audio input devices")?
                .find(|device| {
                    device
                        .id()
                        .ok()
                        .is_some_and(|id| id.id() == name || id.to_string() == name)
                })
                .ok_or_else(|| anyhow!("Audio input device not found: {name}"))?,
            None => host
                .default_input_device()
                .context("No default audio input device available")?,
        };

        let config = device
            .default_input_config()
            .context("Failed to get default audio input config")?;
        if config.sample_format() != cpal::SampleFormat::F32 {
            bail!(
                "Unsupported tape input sample format: {:?}",
                config.sample_format()
            );
        }

        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let capacity = (sample_rate / 2).max(1) as usize;

        let (tx, rx) = bounded::<f32>(capacity);
        let (err_tx, err_rx) = bounded::<anyhow::Error>(1);

        let stream = device
            .build_input_stream(
                config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    for frame in data.chunks(channels) {
                        let mixed = frame.iter().sum::<f32>() / channels as f32;
                        let _ = tx.try_send(mixed);
                    }
                },
                move |err| {
                    let _ =
                        err_tx.try_send(anyhow::Error::new(err).context("Tape input stream error"));
                },
                None,
            )
            .context("Failed to build tape input stream")?;

        stream.play().context("Failed to start tape input stream")?;

        Ok((
            Box::new(Self {
                _stream: stream,
                err_rx,
            }),
            rx,
            sample_rate,
        ))
    }
}

impl TapeIn for CpalTapeIn {
    fn poll_error(&mut self) -> Option<anyhow::Error> {
        self.err_rx.try_recv().ok()
    }
}

struct CoreAudioTapIn {
    tap_id: AudioObjectID,
    aggregate_id: AudioObjectID,
    proc_id: AudioDeviceIOProcID,
    _block: TapeIoBlock,
    _queue: DispatchRetained<DispatchQueue>,
}

impl CoreAudioTapIn {
    fn start() -> Result<(Box<dyn TapeIn>, Receiver<f32>, u32)> {
        disclaim_responsibility();
        ensure_audio_access()?;

        let (tap_id, tap_uid) = create_global_tap()?;

        let (sample_rate, channels) = match tap_stream_format(tap_id) {
            Ok(format) => format,
            Err(err) => {
                destroy_tap(tap_id);
                return Err(err);
            }
        };

        let aggregate_id = match build_aggregate(&tap_uid) {
            Ok(id) => id,
            Err(err) => {
                destroy_tap(tap_id);
                return Err(err);
            }
        };

        let capacity = (sample_rate / 2).max(1) as usize;
        let (tx, rx) = bounded::<f32>(capacity);
        let block = build_capture_block(channels, tx);
        let queue = DispatchQueue::new("org.coignard.kc87.tape", None);

        let proc_id = match start_io_proc(aggregate_id, &block, &queue) {
            Ok(id) => id,
            Err(err) => {
                destroy_aggregate(aggregate_id);
                destroy_tap(tap_id);
                return Err(err);
            }
        };

        Ok((
            Box::new(Self {
                tap_id,
                aggregate_id,
                proc_id,
                _block: block,
                _queue: queue,
            }),
            rx,
            sample_rate,
        ))
    }
}

impl TapeIn for CoreAudioTapIn {
    fn poll_error(&mut self) -> Option<anyhow::Error> {
        None
    }
}

impl Drop for CoreAudioTapIn {
    fn drop(&mut self) {
        unsafe {
            AudioDeviceStop(self.aggregate_id, self.proc_id);
            AudioDeviceDestroyIOProcID(self.aggregate_id, self.proc_id);
        }
        destroy_aggregate(self.aggregate_id);
        destroy_tap(self.tap_id);
    }
}

fn destroy_tap(tap_id: AudioObjectID) {
    unsafe {
        AudioHardwareDestroyProcessTap(tap_id);
    }
}

fn destroy_aggregate(aggregate_id: AudioObjectID) {
    unsafe {
        AudioHardwareDestroyAggregateDevice(aggregate_id);
    }
}

fn cf_type<T: AsRef<CFType> + ?Sized>(value: &T) -> &CFType {
    value.as_ref()
}

fn read_property<T>(object: AudioObjectID, selector: u32) -> Result<T> {
    let address = AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };
    let mut value: T = unsafe { core::mem::zeroed() };
    let mut size = core::mem::size_of::<T>() as u32;
    let status = unsafe {
        AudioObjectGetPropertyData(
            object,
            NonNull::from(&address),
            0,
            ptr::null(),
            NonNull::from(&mut size),
            NonNull::new(&mut value as *mut _ as *mut c_void).unwrap(),
        )
    };
    if status != 0 {
        bail!("CoreAudio property {selector:#x} query failed (status {status})");
    }
    Ok(value)
}

fn create_global_tap() -> Result<(AudioObjectID, String)> {
    let exclude = NSArray::<NSNumber>::new();
    let description = unsafe {
        CATapDescription::initMonoGlobalTapButExcludeProcesses(CATapDescription::alloc(), &exclude)
    };
    let uuid = NSUUID::UUID();
    unsafe {
        description.setUUID(&uuid);
        description.setName(&NSString::from_str("KC87TapeTap"));
        description.setMuteBehavior(CATapMuteBehavior::Unmuted);
        description.setPrivate(true);
    }

    let mut tap_id: AudioObjectID = 0;
    let status = unsafe { AudioHardwareCreateProcessTap(Some(&description), &mut tap_id) };
    if status != 0 || tap_id == 0 {
        bail!("AudioHardwareCreateProcessTap failed (status {status})");
    }

    Ok((tap_id, uuid.UUIDString().to_string()))
}

fn tap_stream_format(tap_id: AudioObjectID) -> Result<(u32, usize)> {
    let asbd: AudioStreamBasicDescription = read_property(tap_id, kAudioTapPropertyFormat)?;
    let sample_rate = asbd.mSampleRate as u32;
    if sample_rate == 0 {
        bail!("Tap reported a zero sample rate");
    }
    let channels = asbd.mChannelsPerFrame.max(1) as usize;
    Ok((sample_rate, channels))
}

fn default_output_uid() -> Result<CFRetained<CFString>> {
    let device: AudioObjectID = read_property(
        kAudioObjectSystemObject as AudioObjectID,
        kAudioHardwarePropertyDefaultSystemOutputDevice,
    )?;
    if device == 0 {
        bail!("No default system output device");
    }

    let uid_ptr: *const CFString = read_property(device, kAudioDevicePropertyDeviceUID)?;
    let uid = NonNull::new(uid_ptr as *mut CFString)
        .context("Default output device returned a null UID")?;
    Ok(unsafe { CFRetained::from_raw(uid) })
}

fn build_aggregate(tap_uid: &str) -> Result<AudioObjectID> {
    let output_uid = default_output_uid()?;

    let key_uid = CFString::from_str(kAudioAggregateDeviceUIDKey.to_str().unwrap());
    let key_private = CFString::from_str(kAudioAggregateDeviceIsPrivateKey.to_str().unwrap());
    let key_stacked = CFString::from_str(kAudioAggregateDeviceIsStackedKey.to_str().unwrap());
    let key_main = CFString::from_str(kAudioAggregateDeviceMainSubDeviceKey.to_str().unwrap());
    let key_autostart = CFString::from_str(kAudioAggregateDeviceTapAutoStartKey.to_str().unwrap());
    let key_subdevices =
        CFString::from_str(kAudioAggregateDeviceSubDeviceListKey.to_str().unwrap());
    let key_subdevice_uid = CFString::from_str(kAudioSubDeviceUIDKey.to_str().unwrap());
    let key_taps = CFString::from_str(kAudioAggregateDeviceTapListKey.to_str().unwrap());
    let key_tap_uid = CFString::from_str(kAudioSubTapUIDKey.to_str().unwrap());
    let key_drift = CFString::from_str(kAudioSubTapDriftCompensationKey.to_str().unwrap());

    let aggregate_uid = CFString::from_str("KC87TapeAggregateDevice");
    let tap_uid = CFString::from_str(tap_uid);
    let yes = CFBoolean::new(true);
    let no = CFBoolean::new(false);

    let subdevice = CFDictionary::from_slices(&[&*key_subdevice_uid], &[cf_type(&*output_uid)]);
    let subdevices = CFArray::from_objects(&[cf_type(&*subdevice)]);

    let subtap = CFDictionary::from_slices(
        &[&*key_tap_uid, &*key_drift],
        &[cf_type(&*tap_uid), cf_type(yes)],
    );
    let taps = CFArray::from_objects(&[cf_type(&*subtap)]);

    let description = CFDictionary::from_slices(
        &[
            &*key_uid,
            &*key_private,
            &*key_stacked,
            &*key_main,
            &*key_autostart,
            &*key_subdevices,
            &*key_taps,
        ],
        &[
            cf_type(&*aggregate_uid),
            cf_type(yes),
            cf_type(no),
            cf_type(&*output_uid),
            cf_type(yes),
            cf_type(&*subdevices),
            cf_type(&*taps),
        ],
    );

    let mut aggregate_id: AudioObjectID = 0;
    let status = unsafe {
        AudioHardwareCreateAggregateDevice(
            (*description).as_ref(),
            NonNull::from(&mut aggregate_id),
        )
    };
    if status != 0 || aggregate_id == 0 {
        bail!("AudioHardwareCreateAggregateDevice failed (status {status})");
    }

    Ok(aggregate_id)
}

fn build_capture_block(channels: usize, tx: crossbeam_channel::Sender<f32>) -> TapeIoBlock {
    RcBlock::new(
        move |_in_now: NonNull<AudioTimeStamp>,
              in_data: NonNull<AudioBufferList>,
              _in_time: NonNull<AudioTimeStamp>,
              _out_data: NonNull<AudioBufferList>,
              _out_time: NonNull<AudioTimeStamp>| {
            let buffers = unsafe { in_data.as_ref() };
            if buffers.mNumberBuffers == 0 {
                return;
            }
            let buffer = buffers.mBuffers[0];
            if buffer.mData.is_null() {
                return;
            }

            let count = buffer.mDataByteSize as usize / core::mem::size_of::<f32>();
            let samples = unsafe { core::slice::from_raw_parts(buffer.mData as *const f32, count) };
            for frame in samples.chunks(channels) {
                let mixed = frame.iter().sum::<f32>() / channels as f32;
                let _ = tx.try_send(mixed);
            }
        },
    )
}

fn start_io_proc(
    aggregate_id: AudioObjectID,
    block: &TapeIoBlock,
    queue: &DispatchQueue,
) -> Result<AudioDeviceIOProcID> {
    let mut proc_id: AudioDeviceIOProcID = None;
    let status = unsafe {
        AudioDeviceCreateIOProcIDWithBlock(
            NonNull::from(&mut proc_id),
            aggregate_id,
            Some(queue),
            RcBlock::as_ptr(block),
        )
    };
    if status != 0 || proc_id.is_none() {
        bail!("AudioDeviceCreateIOProcIDWithBlock failed (status {status})");
    }

    let status = unsafe { AudioDeviceStart(aggregate_id, proc_id) };
    if status != 0 {
        unsafe {
            AudioDeviceDestroyIOProcID(aggregate_id, proc_id);
        }
        bail!("AudioDeviceStart failed (status {status})");
    }

    Ok(proc_id)
}

fn disclaim_responsibility() {
    if std::env::var_os(DISCLAIM_GUARD).is_some() {
        return;
    }

    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Ok(exe) = CString::new(exe.as_os_str().as_encoded_bytes()) else {
        return;
    };

    let args: Vec<CString> = std::env::args_os()
        .filter_map(|arg| CString::new(arg.as_encoded_bytes()).ok())
        .collect();
    let mut argv: Vec<*mut libc::c_char> = args
        .iter()
        .map(|arg| arg.as_ptr() as *mut libc::c_char)
        .collect();
    argv.push(ptr::null_mut());

    let mut env: Vec<CString> = std::env::vars_os()
        .filter_map(|(key, value)| {
            let mut entry = key.into_encoded_bytes();
            entry.push(b'=');
            entry.extend_from_slice(value.as_encoded_bytes());
            CString::new(entry).ok()
        })
        .collect();
    if let Ok(guard) = CString::new(format!("{DISCLAIM_GUARD}=1")) {
        env.push(guard);
    }
    let mut envp: Vec<*mut libc::c_char> = env
        .iter()
        .map(|entry| entry.as_ptr() as *mut libc::c_char)
        .collect();
    envp.push(ptr::null_mut());

    unsafe {
        let handle = libc::dlopen(ptr::null(), libc::RTLD_NOW);
        if handle.is_null() {
            return;
        }
        let symbol = libc::dlsym(handle, c"responsibility_spawnattrs_setdisclaim".as_ptr());
        if symbol.is_null() {
            return;
        }
        let set_disclaim: DisclaimFn = core::mem::transmute(symbol);

        let mut attr: libc::posix_spawnattr_t = ptr::null_mut();
        if libc::posix_spawnattr_init(&mut attr) != 0 {
            return;
        }
        set_disclaim(&mut attr, 1);
        libc::posix_spawnattr_setflags(&mut attr, libc::POSIX_SPAWN_SETEXEC as libc::c_short);

        let mut pid: libc::pid_t = 0;
        libc::posix_spawn(
            &mut pid,
            exe.as_ptr(),
            ptr::null(),
            &attr,
            argv.as_ptr(),
            envp.as_ptr(),
        );
        libc::posix_spawnattr_destroy(&mut attr);
    }
}

fn ensure_audio_access() -> Result<()> {
    unsafe {
        let path = c"/System/Library/PrivateFrameworks/TCC.framework/Versions/A/TCC".as_ptr();
        let handle = libc::dlopen(path, libc::RTLD_NOW);
        if handle.is_null() {
            bail!("failed to open TCC.framework for audio-capture permission");
        }

        let preflight_sym = libc::dlsym(handle, c"TCCAccessPreflight".as_ptr());
        let request_sym = libc::dlsym(handle, c"TCCAccessRequest".as_ptr());
        if preflight_sym.is_null() || request_sym.is_null() {
            bail!("TCC permission SPI symbols not found");
        }
        let preflight: PreflightFn = core::mem::transmute(preflight_sym);
        let request: RequestFn = core::mem::transmute(request_sym);

        let service = CFString::from_str("kTCCServiceAudioCapture");
        let service_ptr = CFRetained::as_ptr(&service).as_ptr() as *const c_void;

        match preflight(service_ptr, ptr::null()) {
            0 => return Ok(()),
            1 => bail!("system audio recording is denied"),
            _ => {}
        }

        let (tx, rx) = bounded::<bool>(1);
        let handler = RcBlock::new(move |granted: Bool| {
            let _ = tx.try_send(granted.as_bool());
        });
        request(
            service_ptr,
            ptr::null(),
            RcBlock::as_ptr(&handler) as *const c_void,
        );

        let deadline = Instant::now() + PERMISSION_TIMEOUT;
        let mut granted = false;
        while Instant::now() < deadline {
            if let Ok(result) = rx.try_recv() {
                granted = result;
                break;
            }
            CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, 0.1, true);
        }
        drop(handler);

        if granted {
            Ok(())
        } else {
            bail!("system audio recording permission was not granted")
        }
    }
}
