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

/// Whether an error means the output device changed or went away.
pub fn device_gone(e: &windows::core::Error) -> bool {
    e.code() == AUDCLNT_E_DEVICE_INVALIDATED
}

#[cfg(test)]
mod tests {
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
