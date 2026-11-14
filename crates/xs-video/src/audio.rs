//! The system's sound (VID-040): what the default output plays, read back with WASAPI
//! loopback (no microphone, no extra component). Stereo, 16-bit, at the output's rate
//! (44.1 or 48 kHz, which AAC takes). When nothing plays, Windows sends nothing: the
//! recording fills that time with silence so sound and picture stay together.

use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL};

/// The default output, listened to.
pub struct Loopback {
    client: IAudioClient,
    capture: IAudioCaptureClient,
    channels: usize,
    float: bool,
    pub rate: u32,
}

fn err(context: &str, e: windows::core::Error) -> String {
    format!("{context}: {} (0x{:08X})", e.message(), e.code().0 as u32)
}

impl Loopback {
    /// Opens the default output's loopback; `None` with the reason if it cannot.
    pub fn open() -> Result<Loopback, String> {
        // SAFETY: COM calls on this thread (initialized by the recording); the mix
        // format is freed with CoTaskMemFree after it is read.
        unsafe {
            let devices: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| err("audio devices", e))?;
            let device = devices
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| err("audio output", e))?;
            let client: IAudioClient = device
                .Activate(CLSCTX_ALL, None)
                .map_err(|e| err("audio output", e))?;
            let format = client.GetMixFormat().map_err(|e| err("audio format", e))?;
            let f = std::ptr::read_unaligned(format);
            let (channels, rate, bits) = (f.nChannels as usize, f.nSamplesPerSec, f.wBitsPerSample);
            let float = match (f.wFormatTag, bits) {
                // IEEE float, or the extensible format the shared mix always uses.
                (3, 32) | (0xFFFE, 32) => true,
                (1, 16) | (0xFFFE, 16) => false,
                _ => {
                    CoTaskMemFree(Some(format as *const _));
                    return Err(format!("audio format not supported ({bits} bits)"));
                }
            };
            if rate != 44_100 && rate != 48_000 {
                CoTaskMemFree(Some(format as *const _));
                return Err(format!("audio at {rate} Hz is not supported yet"));
            }
            // One second of buffer: the recording reads it at least every 100 ms.
            let init = client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK,
                10_000_000,
                0,
                format,
                None,
            );
            CoTaskMemFree(Some(format as *const _));
            init.map_err(|e| err("audio loopback", e))?;
            let capture: IAudioCaptureClient = client.GetService().map_err(|e| err("audio loopback", e))?;
            client.Start().map_err(|e| err("audio loopback", e))?;
            Ok(Loopback {
                client,
                capture,
                channels,
                float,
                rate,
            })
        }
    }

    /// What arrived since the last call, as interleaved stereo 16-bit samples. Err
    /// when the output went away (another device was chosen).
    pub fn read(&self) -> Result<Vec<i16>, windows::core::Error> {
        let mut out = Vec::new();
        // SAFETY: each buffer is read within the frames it reports and released.
        unsafe {
            loop {
                let next = self.capture.GetNextPacketSize()?;
                if next == 0 {
                    break;
                }
                let mut data = std::ptr::null_mut();
                let mut frames = 0u32;
                let mut flags = 0u32;
                self.capture
                    .GetBuffer(&mut data, &mut frames, &mut flags, None, None)?;
                let n = frames as usize;
                let silent = flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
                out.reserve(n * 2);
                for i in 0..n {
                    for ch in 0..2 {
                        // Left and right are the first two channels (VID-040).
                        let c = ch.min(self.channels - 1);
                        let v = if silent {
                            0
                        } else if self.float {
                            let p = (data as *const f32).add(i * self.channels + c);
                            (p.read_unaligned().clamp(-1.0, 1.0) * 32767.0) as i16
                        } else {
                            (data as *const i16).add(i * self.channels + c).read_unaligned()
                        };
                        out.push(v);
                    }
                }
                self.capture.ReleaseBuffer(frames)?;
            }
        }
        Ok(out)
    }
}

impl Drop for Loopback {
    fn drop(&mut self) {
        // SAFETY: stops the stream this struct started.
        unsafe {
            let _ = self.client.Stop();
        }
    }
}

/// A microphone (VID-039): `device` by its id, or the system's default input. Windows
/// converts whatever it gives (mono or stereo, 16 or 24 bits, 44.1 or 48 kHz) to
/// stereo float at `rate`, so it adds to the system's sound sample for sample.
pub fn microphone(device: Option<&str>, rate: u32) -> Result<Loopback, String> {
    // SAFETY: COM calls on this thread (initialized by the caller); the format lives
    // through Initialize.
    unsafe {
        let devices: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| err("audio devices", e))?;
        let device = match device {
            Some(id) => devices
                .GetDevice(&windows::core::HSTRING::from(id))
                .map_err(|e| err("microphone", e))?,
            None => devices
                .GetDefaultAudioEndpoint(eCapture, eConsole)
                .map_err(|e| err("microphone", e))?,
        };
        let client: IAudioClient = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| err("microphone", e))?;
        let format = windows::Win32::Media::Audio::WAVEFORMATEX {
            wFormatTag: 3, // IEEE float
            nChannels: 2,
            nSamplesPerSec: rate,
            nAvgBytesPerSec: rate * 8,
            nBlockAlign: 8,
            wBitsPerSample: 32,
            cbSize: 0,
        };
        client
            .Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                10_000_000,
                0,
                &format,
                None,
            )
            .map_err(|e| err("microphone", e))?;
        let capture: IAudioCaptureClient = client.GetService().map_err(|e| err("microphone", e))?;
        client.Start().map_err(|e| err("microphone", e))?;
        Ok(Loopback {
            client,
            capture,
            channels: 2,
            float: true,
            rate,
        })
    }
}

/// A sound input: its id, the name Windows shows, and whether it is wireless
/// (Bluetooth: the voice may sound worse, VID-039).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Input {
    pub id: String,
    pub name: String,
    pub wireless: bool,
}

/// COM on this thread, once (the command threads have no COM of their own).
fn com() {
    thread_local!(static DONE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
    DONE.with(|d| {
        if !d.get() {
            // SAFETY: initializes COM on this thread; if it already has it in another
            // mode, that one is kept, which serves as well.
            unsafe {
                let _ = windows::Win32::System::Com::CoInitializeEx(
                    None,
                    windows::Win32::System::Com::COINIT_MULTITHREADED,
                );
            }
            d.set(true);
        }
    });
}

/// Whether an input (`device`, or the default) is muted in the system or has its
/// volume at zero (VID-019); `None` when it cannot be told.
pub fn input_muted(device: Option<&str>) -> Option<bool> {
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    com();
    // SAFETY: COM reads of the input's mute and volume.
    unsafe {
        let devices: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let d = match device {
            Some(id) => devices.GetDevice(&windows::core::HSTRING::from(id)).ok()?,
            None => devices.GetDefaultAudioEndpoint(eCapture, eConsole).ok()?,
        };
        let v: IAudioEndpointVolume = d.Activate(CLSCTX_ALL, None).ok()?;
        let muted = v.GetMute().ok()?.as_bool();
        let volume = v.GetMasterVolumeLevelScalar().ok()?;
        Some(muted || volume < 0.005)
    }
}

/// The microphones and other inputs that are on (VID-039), and the default one's id.
pub fn inputs() -> Result<(Vec<Input>, Option<String>), String> {
    use windows::Win32::Devices::FunctionDiscovery::{PKEY_Device_EnumeratorName, PKEY_Device_FriendlyName};
    use windows::Win32::System::Com::STGM_READ;
    com();
    // SAFETY: COM calls; the ids are freed after they are copied, the property values
    // are read as the strings they are.
    unsafe {
        let devices: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| err("audio devices", e))?;
        let id_of = |d: &IMMDevice| -> Option<String> {
            let p = d.GetId().ok()?;
            let s = p.to_string().ok();
            CoTaskMemFree(Some(p.0 as *const _));
            s
        };
        let default = devices
            .GetDefaultAudioEndpoint(eCapture, eConsole)
            .ok()
            .and_then(|d| id_of(&d));
        let list = devices
            .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
            .map_err(|e| err("audio devices", e))?;
        let mut out = Vec::new();
        for i in 0..list.GetCount().unwrap_or(0) {
            let Ok(d) = list.Item(i) else { continue };
            let Some(id) = id_of(&d) else { continue };
            let store = d.OpenPropertyStore(STGM_READ).ok();
            let read = |key| {
                store
                    .as_ref()
                    .and_then(|s| s.GetValue(key).ok())
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            };
            let name = Some(read(&PKEY_Device_FriendlyName))
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| id.clone());
            // Bluetooth inputs come from its enumerators (BTHENUM, BTHHFENUM).
            let wireless = read(&PKEY_Device_EnumeratorName)
                .to_ascii_uppercase()
                .starts_with("BTH");
            out.push(Input { id, name, wireless });
        }
        Ok((out, default))
    }
}

/// Whether an error means the output device changed or went away.
pub fn device_gone(e: &windows::core::Error) -> bool {
    e.code() == AUDCLNT_E_DEVICE_INVALIDATED
}

#[cfg(test)]
mod tests {
    /// The inputs this computer has (by hand: it lists devices, nothing is recorded).
    #[test]
    #[ignore]
    fn lists_the_inputs() {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }
        println!("{:?}", super::inputs());
        println!("muted: {:?}", super::input_muted(None));
    }

    /// Reads the default microphone for 3 s and prints its loudest sample every half
    /// second (by hand; nothing is kept).
    #[test]
    #[ignore]
    fn listens_to_the_microphone() {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }
        // SAFETY: COM reads of the default input's volume.
        unsafe {
            use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
            use windows::Win32::Media::Audio::*;
            use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
            let devices: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
            let d = devices.GetDefaultAudioEndpoint(eCapture, eConsole).unwrap();
            let v: IAudioEndpointVolume = d.Activate(CLSCTX_ALL, None).unwrap();
            println!(
                "muted {:?}, volume {:?}",
                v.GetMute().map(|b| b.as_bool()),
                v.GetMasterVolumeLevelScalar()
            );
        }
        let m = super::microphone(None, 48_000).unwrap();
        // Windows's own meter of the input, apart from this code's samples.
        // SAFETY: COM reads of the default input's mix format and meter.
        let meter = unsafe {
            use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
            use windows::Win32::Media::Audio::*;
            use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
            let devices: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
            let d = devices.GetDefaultAudioEndpoint(eCapture, eConsole).unwrap();
            let c: IAudioClient = d.Activate(CLSCTX_ALL, None).unwrap();
            let f = std::ptr::read_unaligned(c.GetMixFormat().unwrap());
            println!(
                "input format: tag {:#x}, {} channels, {} Hz, {} bits",
                { f.wFormatTag },
                { f.nChannels },
                { f.nSamplesPerSec },
                { f.wBitsPerSample }
            );
            d.Activate::<IAudioMeterInformation>(CLSCTX_ALL, None).unwrap()
        };
        let seconds = std::env::var("XS_MIC_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);
        for _ in 0..seconds * 2 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let s = m.read().unwrap();
            let peak = s.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
            // SAFETY: a read of the meter's current peak.
            let windows_peak = unsafe { meter.GetPeakValue() };
            println!("{} samples, peak {peak}, windows meter {windows_peak:?}", s.len());
        }
    }

    #[test]
    #[ignore]
    fn opens_the_loopback() {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }
        match super::Loopback::open() {
            Ok(l) => println!(
                "loopback at {} Hz, {} channels, float {}",
                l.rate, l.channels, l.float
            ),
            Err(e) => panic!("{e}"),
        }
    }
}
