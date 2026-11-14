//! Editing videos with Media Foundation (ADR 0008): the frame at a moment (VID-061,
//! VID-062) and a new MP4 with only some parts of a file, with or without its sound
//! (VID-059, VID-060, VID-063). It decodes and encodes again with the system's codecs.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::{Interface, GUID, HSTRING};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::System::Variant::VT_I8;

use crate::timeline::Span;
use crate::win::{encoder_settings, err, media_type, HNS};

/// Half a millisecond: a frame "at" a time may be this much after it (rounding).
const SLACK: i64 = 5_000;

fn start() {
    // SAFETY: COM and Media Foundation start-up; repeated calls are harmless.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
    }
}

/// A reader of the file; with `processing`, it also converts the pictures to the
/// format asked for (RGB, other sizes).
fn reader(path: &Path, processing: bool) -> Result<IMFSourceReader, String> {
    // SAFETY: Media Foundation calls with arguments that live through each call.
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 2).map_err(|e| err("attributes", e))?;
        let attrs: IMFAttributes = attrs.ok_or("no attributes")?;
        if processing {
            let _ = attrs.SetUINT32(&MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, 1);
        }
        MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), &attrs).map_err(|e| unreadable(path, e))
    }
}

/// The message for a file the system cannot read, with its codec when known (VID-055).
fn unreadable(path: &Path, e: windows::core::Error) -> String {
    err(&format!("reading {}", path.display()), e)
}

/// The stream numbers of the first picture and the first sound.
fn streams(reader: &IMFSourceReader) -> (Option<u32>, Option<u32>) {
    let (mut video, mut audio) = (None, None);
    for i in 0..32u32 {
        // SAFETY: asks for a stream's type; an error ends the list.
        let Ok(t) = (unsafe { reader.GetNativeMediaType(i, 0) }) else {
            break;
        };
        // SAFETY: attribute read.
        match unsafe { t.GetGUID(&MF_MT_MAJOR_TYPE) } {
            Ok(g) if g == MFMediaType_Video && video.is_none() => video = Some(i),
            Ok(g) if g == MFMediaType_Audio && audio.is_none() => audio = Some(i),
            _ => {}
        }
    }
    (video, audio)
}

/// Copies the picture and the sound of `src` into a regular MP4 at `dst` without
/// coding them again (VID-022): a recording's fragmented file into the one kept. A
/// file cut short by a crash gives what can be read of it. Returns the seconds of
/// picture copied.
pub fn remux(src: &Path, dst: &Path) -> Result<f64, String> {
    remux_until(src, dst, None)
}

/// A recording cut short (VID-022) as a regular MP4 at `dst`, as long as it was
/// recorded: its last picture stays until the time noted next to it. Returns its
/// length in seconds.
pub fn recover(fragmented: &Path, dst: &Path) -> Result<f64, String> {
    remux_until(fragmented, dst, crate::noted_time(fragmented))
}

/// How long a recording cut short is (VID-022): what can be read of it, or the time
/// noted next to it if later.
pub fn recovered_length(fragmented: &Path) -> Option<f64> {
    let read = crate::probe(fragmented).ok().map(|p| p.3);
    match (read, crate::noted_time(fragmented)) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    }
}

fn remux_until(src: &Path, dst: &Path, until: Option<f64>) -> Result<f64, String> {
    start();
    // SAFETY: Media Foundation calls with arguments that live through each call;
    // samples go from the reader to the writer as they are.
    unsafe {
        let reader = MFCreateSourceReaderFromURL(&HSTRING::from(src.as_os_str()), None)
            .map_err(|e| unreadable(src, e))?;
        let (video, audio) = streams(&reader);
        let video = video.ok_or("the recording has no picture")?;
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 1).map_err(|e| err("attributes", e))?;
        let attrs: IMFAttributes = attrs.ok_or("no attributes")?;
        attrs
            .SetGUID(&MF_TRANSCODE_CONTAINERTYPE, &MFTranscodeContainerType_MPEG4)
            .map_err(|e| err("attributes", e))?;
        let _ = std::fs::remove_file(dst);
        let sink = MFCreateSinkWriterFromURL(&HSTRING::from(dst.as_os_str()), None, &attrs)
            .map_err(|e| err("the video file", e))?;
        let _ = reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false);
        let mut map = Vec::new();
        for stream in [Some(video), audio].into_iter().flatten() {
            let t = reader
                .GetNativeMediaType(stream, 0)
                .map_err(|e| err("stream type", e))?;
            reader
                .SetCurrentMediaType(stream, None, &t)
                .map_err(|e| err("stream type", e))?;
            reader
                .SetStreamSelection(stream, true)
                .map_err(|e| err("stream", e))?;
            let out = sink.AddStream(&t).map_err(|e| err("stream", e))?;
            sink.SetInputMediaType(out, &t, None)
                .map_err(|e| err("stream type", e))?;
            map.push((stream, out, false));
        }
        sink.BeginWriting().map_err(|e| err("video writing", e))?;
        let mut end = 0.0f64;
        // The last picture waits: it may have to last until `until`.
        let mut held: Option<(IMFSample, i64, u32)> = None;
        while map.iter().any(|m| !m.2) {
            let (mut actual, mut flags, mut time, mut sample) = (0u32, 0u32, 0i64, None);
            let read = reader.ReadSample(
                MF_SOURCE_READER_ANY_STREAM.0 as u32,
                0,
                Some(&mut actual),
                Some(&mut flags),
                Some(&mut time),
                Some(&mut sample),
            );
            if read.is_err() || flags & MF_SOURCE_READERF_ERROR.0 as u32 != 0 {
                // The rest of a file cut short: what was read is kept.
                break;
            }
            let Some(m) = map.iter_mut().find(|m| m.0 == actual) else {
                continue;
            };
            if let Some(s) = sample {
                if actual == video {
                    let length = s.GetSampleDuration().unwrap_or(0);
                    end = end.max((time + length) as f64 / HNS);
                    if let Some((prev, _, out)) = held.replace((s, time, m.1)) {
                        sink.WriteSample(out, &prev).map_err(|e| err("writing", e))?;
                    }
                } else {
                    sink.WriteSample(m.1, &s).map_err(|e| err("writing", e))?;
                }
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                m.2 = true;
            }
        }
        if let Some((s, time, out)) = held {
            if let Some(u) = until.filter(|u| *u > end) {
                let _ = s.SetSampleDuration((u * HNS) as i64 - time);
                end = u;
            }
            sink.WriteSample(out, &s).map_err(|e| err("writing", e))?;
        }
        sink.Finalize().map_err(|e| err("finishing the video", e))?;
        Ok(end)
    }
}

fn seek(reader: &IMFSourceReader, seconds: f64) -> Result<(), String> {
    let mut v = PROPVARIANT::default();
    // SAFETY: a VT_I8 PROPVARIANT, filled as such and read by the reader.
    unsafe {
        let inner = &mut *v.Anonymous.Anonymous;
        inner.vt = VT_I8;
        inner.Anonymous.hVal = (seconds.max(0.0) * HNS) as i64;
        reader
            .SetCurrentPosition(&GUID::zeroed(), &v)
            .map_err(|e| err("going to a time", e))
    }
}

/// The decoded pictures: the size of their buffers and the part that is the video.
#[derive(Clone, Copy)]
struct Geometry {
    buffer: (u32, u32),
    shown: (u32, u32),
}

fn geometry(t: &IMFMediaType) -> Result<Geometry, String> {
    // SAFETY: attribute reads; the blob is read into a struct of its size.
    unsafe {
        let size = t.GetUINT64(&MF_MT_FRAME_SIZE).map_err(|e| err("video size", e))?;
        let buffer = ((size >> 32) as u32, size as u32);
        let mut shown = buffer;
        for key in [&MF_MT_MINIMUM_DISPLAY_APERTURE, &MF_MT_GEOMETRIC_APERTURE] {
            let mut area = MFVideoArea::default();
            let bytes = std::slice::from_raw_parts_mut(
                &mut area as *mut MFVideoArea as *mut u8,
                std::mem::size_of::<MFVideoArea>(),
            );
            if t.GetBlob(key, bytes, None).is_ok() && area.Area.cx > 0 && area.Area.cy > 0 {
                shown = (
                    (area.Area.cx as u32).min(buffer.0),
                    (area.Area.cy as u32).min(buffer.1),
                );
                break;
            }
        }
        Ok(Geometry { buffer, shown })
    }
}

/// Reads a sample's picture as rows: calls `row(index, bytes)` for `rows` rows of
/// `width` bytes, the buffer being `height` rows tall (for the planes below).
fn read_rows(
    sample: &IMFSample,
    stride_hint: usize,
    rows: impl Iterator<Item = usize>,
    width: usize,
    mut row: impl FnMut(&[u8]),
) -> Result<(), String> {
    // SAFETY: the buffer is locked while read and unlocked on every path; each row
    // read lies within the locked length (checked).
    unsafe {
        let buffer = sample.GetBufferByIndex(0).map_err(|e| err("frame", e))?;
        if let Ok(b2) = buffer.cast::<IMF2DBuffer>() {
            let mut scan0 = std::ptr::null_mut();
            let mut pitch = 0i32;
            if b2.Lock2D(&mut scan0, &mut pitch).is_ok() {
                for r in rows {
                    let p = scan0.offset(r as isize * pitch as isize);
                    row(std::slice::from_raw_parts(p, width));
                }
                let _ = b2.Unlock2D();
                return Ok(());
            }
        }
        let buffer = sample.ConvertToContiguousBuffer().map_err(|e| err("frame", e))?;
        let mut data = std::ptr::null_mut();
        let mut len = 0u32;
        buffer
            .Lock(&mut data, None, Some(&mut len))
            .map_err(|e| err("frame", e))?;
        let mut result = Ok(());
        for r in rows {
            let at = r * stride_hint;
            if at + width > len as usize {
                result = Err("the frame is smaller than its size".to_string());
                break;
            }
            row(std::slice::from_raw_parts(data.add(at), width));
        }
        let _ = buffer.Unlock();
        result
    }
}

/// The picture shown at `seconds` (the last frame that starts at or before it), as
/// BGRA pixels: width, height, pixels (VID-061, VID-062, thumbnails).
pub fn frame_at(path: &Path, seconds: f64) -> Result<(u32, u32, Vec<u8>), String> {
    start();
    let reader = reader(path, true)?;
    let (Some(video), _) = streams(&reader) else {
        return Err("the file has no picture".into());
    };
    // SAFETY: Media Foundation reading calls; samples are kept as COM references.
    unsafe {
        let _ = reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false);
        let _ = reader.SetStreamSelection(video, true);
        let want = MFCreateMediaType().map_err(|e| err("media type", e))?;
        let _ = want.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video);
        let _ = want.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32);
        reader
            .SetCurrentMediaType(video, None, &want)
            .map_err(|e| err("video type", e))?;
        let t = reader
            .GetCurrentMediaType(video)
            .map_err(|e| err("video type", e))?;
        let g = geometry(&t)?;
        if seconds > 0.0 {
            seek(&reader, seconds)?;
        }
        let target = (seconds.max(0.0) * HNS) as i64 + SLACK;
        let mut best: Option<IMFSample> = None;
        loop {
            let mut flags = 0u32;
            let mut time = 0i64;
            let mut sample = None;
            reader
                .ReadSample(
                    video,
                    0,
                    None,
                    Some(&mut flags),
                    Some(&mut time),
                    Some(&mut sample),
                )
                .map_err(|e| err("reading a frame", e))?;
            if let Some(s) = sample {
                if time > target && best.is_some() {
                    break;
                }
                best = Some(s);
                if time >= target - SLACK {
                    break;
                }
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
        }
        let sample = best.ok_or("the video has no frames")?;
        let (w, h) = g.shown;
        let mut pixels = Vec::with_capacity((w * h * 4) as usize);
        read_rows(
            &sample,
            g.buffer.0 as usize * 4,
            0..h as usize,
            w as usize * 4,
            |r| pixels.extend_from_slice(r),
        )?;
        // The video processor leaves the fourth byte at zero: the frame is opaque.
        for px in pixels.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }
        Ok((w, h, pixels))
    }
}

/// A reader that gives NV12 pictures: the reader, its picture and sound streams and
/// the pictures' geometry.
fn nv12_reader(
    path: &Path,
    processing: bool,
) -> Result<(IMFSourceReader, u32, Option<u32>, Geometry), String> {
    let reader = reader(path, processing)?;
    let (Some(video), audio) = streams(&reader) else {
        return Err("the file has no picture".into());
    };
    // SAFETY: Media Foundation calls with arguments that live through each call.
    unsafe {
        let want = MFCreateMediaType().map_err(|e| err("media type", e))?;
        let _ = want.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video);
        let _ = want.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12);
        reader
            .SetCurrentMediaType(video, None, &want)
            .map_err(|e| err("video type", e))?;
        let g = geometry(
            &reader
                .GetCurrentMediaType(video)
                .map_err(|e| err("video type", e))?,
        )?;
        Ok((reader, video, audio, g))
    }
}

/// The frames of `spans` of a video at `fps` (an animated GIF, VID-068): calls `each`
/// with the time in the result, the size and the BGRA pixels of the picture shown then;
/// `each` returns false to stop.
pub fn frames(path: &Path, spans: &[Span], fps: f64, each: &mut crate::FrameSink) -> Result<(), String> {
    start();
    let reader = reader(path, true)?;
    let (Some(video), _) = streams(&reader) else {
        return Err("the file has no picture".into());
    };
    let step = 1.0 / fps.max(0.1);
    // SAFETY: Media Foundation reading calls; samples are copied as they are read.
    let g = unsafe {
        let _ = reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false);
        let _ = reader.SetStreamSelection(video, true);
        let want = MFCreateMediaType().map_err(|e| err("media type", e))?;
        let _ = want.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video);
        let _ = want.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32);
        reader
            .SetCurrentMediaType(video, None, &want)
            .map_err(|e| err("video type", e))?;
        geometry(
            &reader
                .GetCurrentMediaType(video)
                .map_err(|e| err("video type", e))?,
        )?
    };
    let (w, h) = g.shown;
    let picture = |s: &IMFSample| -> Result<Vec<u8>, String> {
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        read_rows(s, g.buffer.0 as usize * 4, 0..h as usize, w as usize * 4, |r| {
            px.extend_from_slice(r)
        })?;
        Ok(px)
    };
    let mut out = 0.0;
    for span in spans.iter().filter(|s| !s.is_empty()) {
        seek(&reader, span.start)?;
        let mut tick = span.start;
        let mut held: Option<Vec<u8>> = None;
        loop {
            let mut flags = 0u32;
            let mut time = 0i64;
            let mut sample = None;
            // SAFETY: a read from the selected stream.
            unsafe {
                reader
                    .ReadSample(
                        video,
                        0,
                        None,
                        Some(&mut flags),
                        Some(&mut time),
                        Some(&mut sample),
                    )
                    .map_err(|e| err("reading a frame", e))?;
            }
            if let Some(s) = sample {
                let t = time as f64 / HNS;
                // The ticks before this frame show the one held.
                while tick < span.end - 1e-9 && t > tick + 1e-4 {
                    if let Some(p) = &held {
                        if !each(out, w, h, p) {
                            return Ok(());
                        }
                        out += step;
                    }
                    tick += step;
                }
                if t >= span.end - 1e-9 || tick >= span.end - 1e-9 {
                    break;
                }
                held = Some(picture(&s)?);
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
        }
        // A still end: the last picture until the part ends.
        if let Some(p) = &held {
            while tick < span.end - 1e-9 {
                if !each(out, w, h, p) {
                    return Ok(());
                }
                out += step;
                tick += step;
            }
        }
    }
    Ok(())
}

/// What the editor needs to open a video at once (ADR 0008), without reading all of
/// it: its size, length, frame rate, codec and whether it has sound.
pub fn facts(path: &Path) -> Result<crate::Facts, String> {
    start();
    let reader = reader(path, false)?;
    let (Some(video), audio) = streams(&reader) else {
        return Err("the file has no picture".into());
    };
    // SAFETY: Media Foundation reading calls; the PROPVARIANT is read as the VT_UI8
    // the source gives for its duration.
    unsafe {
        let native = reader
            .GetNativeMediaType(video, 0)
            .map_err(|e| err("video type", e))?;
        let codec = match native.GetGUID(&MF_MT_SUBTYPE) {
            Ok(g) if g == MFVideoFormat_H264 => "H.264".to_string(),
            Ok(g) if g == MFVideoFormat_HEVC => "H.265".to_string(),
            Ok(g) if g == MFVideoFormat_AV1 => "AV1".to_string(),
            Ok(g) if g == MFVideoFormat_VP90 => "VP9".to_string(),
            Ok(g) if g == MFVideoFormat_MP4V => "MPEG-4".to_string(),
            Ok(g) if g == MFVideoFormat_MJPG => "Motion JPEG".to_string(),
            Ok(g) if g == MFVideoFormat_WMV3 => "WMV".to_string(),
            Ok(g) => format!("{g:?}"),
            Err(_) => String::new(),
        };
        let g = geometry(&native)?;
        let duration = reader
            .GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION)
            .map(|v| v.Anonymous.Anonymous.Anonymous.uhVal as f64 / HNS)
            .unwrap_or(0.0);
        Ok(crate::Facts {
            width: g.shown.0,
            height: g.shown.1,
            duration: (duration * 1000.0).round() / 1000.0,
            fps: nominal_fps(&reader, video),
            codec,
            sound: audio.is_some(),
        })
    }
}

/// The frame rate a file says it has, or 30.
fn nominal_fps(reader: &IMFSourceReader, video: u32) -> u32 {
    // SAFETY: attribute read of the native type.
    unsafe {
        reader
            .GetNativeMediaType(video, 0)
            .and_then(|t| t.GetUINT64(&MF_MT_FRAME_RATE))
            .ok()
            .map(|r| ((r >> 32) as f64 / (r as u32).max(1) as f64).round() as u32)
            .filter(|f| (1..=240).contains(f))
            .unwrap_or(30)
    }
}

/// The new file being written.
struct Out {
    sink: IMFSinkWriter,
    video: u32,
    audio: Option<(u32, u32)>,
}

fn out(
    path: &Path,
    size: (u32, u32),
    fps: u32,
    audio_rate: Option<u32>,
    hardware: bool,
) -> Result<Out, String> {
    // SAFETY: Media Foundation calls with arguments that live through each call.
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 3).map_err(|e| err("attributes", e))?;
        let attrs: IMFAttributes = attrs.ok_or("no attributes")?;
        let _ = attrs.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1);
        let _ = attrs.SetGUID(&MF_TRANSCODE_CONTAINERTYPE, &MFTranscodeContainerType_MPEG4);
        let _ = attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, hardware as u32);
        let sink = MFCreateSinkWriterFromURL(&HSTRING::from(path.as_os_str()), None, &attrs)
            .map_err(|e| err("video file", e))?;
        // More bits than a recording (0.08 per pixel and frame in High), so a video
        // edited again and again keeps its look.
        let bitrate =
            ((size.0 as f64 * size.1 as f64 * fps as f64 * 0.12) as u32).clamp(1_000_000, 80_000_000);
        let t = media_type(&MFVideoFormat_H264, size, fps, Some(bitrate))?;
        let _ = t.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_High.0 as u32);
        let video = match sink.AddStream(&t) {
            Ok(s) => s,
            Err(_) => {
                let _ = t.DeleteItem(&MF_MT_MPEG2_PROFILE);
                sink.AddStream(&t).map_err(|e| err("video stream", e))?
            }
        };
        let input = media_type(&MFVideoFormat_NV12, size, fps, None)?;
        sink.SetInputMediaType(video, &input, encoder_settings().as_ref())
            .map_err(|e| err("video input type", e))?;
        let audio = match audio_rate {
            Some(rate) => {
                let index = sink
                    .AddStream(&pcm(&MFAudioFormat_AAC, rate).map_err(|e| err("audio type", e))?)
                    .map_err(|e| err("audio stream", e))?;
                sink.SetInputMediaType(
                    index,
                    &pcm(&MFAudioFormat_PCM, rate).map_err(|e| err("audio type", e))?,
                    None,
                )
                .map_err(|e| err("audio input type", e))?;
                Some((index, rate))
            }
            None => None,
        };
        sink.BeginWriting().map_err(|e| err("video writing", e))?;
        Ok(Out { sink, video, audio })
    }
}

/// 16-bit stereo PCM at `rate`, or AAC at 128 kbps from it.
fn pcm(subtype: &GUID, rate: u32) -> windows::core::Result<IMFMediaType> {
    // SAFETY: attribute setters on a new media type.
    unsafe {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        t.SetGUID(&MF_MT_SUBTYPE, subtype)?;
        t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
        t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, rate)?;
        t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, 2)?;
        if *subtype == MFAudioFormat_AAC {
            t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 16_000)?;
        } else {
            t.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, 4)?;
            t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, rate * 4)?;
        }
        Ok(t)
    }
}

impl Out {
    fn write(&self, stream: u32, bytes: &[u8], time: f64, length: f64) -> Result<(), String> {
        // SAFETY: the buffer is locked, filled within its size and unlocked.
        unsafe {
            let b = MFCreateMemoryBuffer(bytes.len() as u32).map_err(|e| err("buffer", e))?;
            let mut dst = std::ptr::null_mut();
            b.Lock(&mut dst, None, None).map_err(|e| err("buffer", e))?;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
            let _ = b.Unlock();
            let _ = b.SetCurrentLength(bytes.len() as u32);
            let sample = MFCreateSample().map_err(|e| err("sample", e))?;
            sample.AddBuffer(&b).map_err(|e| err("sample", e))?;
            let _ = sample.SetSampleTime((time * HNS).round() as i64);
            let _ = sample.SetSampleDuration((length * HNS).round().max(1.0) as i64);
            self.sink
                .WriteSample(stream, &sample)
                .map_err(|e| err("encoding", e))
        }
    }

    fn finish(&self) -> Result<(), String> {
        // SAFETY: closes the file.
        unsafe { self.sink.Finalize() }.map_err(|e| err("closing the video", e))
    }
}

/// A decoded NV12 picture, packed to the new video's size.
fn nv12(sample: &IMFSample, g: Geometry, size: (u32, u32)) -> Result<Vec<u8>, String> {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let (bw, bh) = (g.buffer.0 as usize, g.buffer.1 as usize);
    let mut out = Vec::with_capacity(w * h * 3 / 2);
    // Luma rows, then the chroma rows, which start below all the buffer's luma rows.
    let rows = (0..h).chain((0..h / 2).map(|r| bh + r));
    read_rows(sample, bw, rows, w, |r| out.extend_from_slice(r))?;
    Ok(out)
}

/// Writes a new MP4 at `dst` with the parts `keep` of `src`, one after the other, with
/// its sound when `sound` (VID-059, VID-060, VID-063). `progress` gets 0..1; `cancel`
/// stops it, leaving no file.
pub fn export(
    src: &Path,
    dst: &Path,
    keep: &[Span],
    sound: bool,
    progress: &mut dyn FnMut(f64),
    cancel: &AtomicBool,
) -> Result<(), String> {
    let result = write_parts(src, dst, keep, sound, progress, cancel);
    if result.is_err() {
        let _ = std::fs::remove_file(dst);
    }
    result
}

fn write_parts(
    src: &Path,
    dst: &Path,
    keep: &[Span],
    sound: bool,
    progress: &mut dyn FnMut(f64),
    cancel: &AtomicBool,
) -> Result<(), String> {
    start();
    let keep: Vec<Span> = keep.iter().copied().filter(|s| !s.is_empty()).collect();
    if keep.is_empty() {
        return Err("nothing to keep".into());
    }
    // NV12 straight from the decoder keeps the colors as they are; the video processor
    // only for a decoder that gives something else (10-bit H.265).
    let (reader, video, audio, g) = match nv12_reader(src, false) {
        Ok(r) => r,
        Err(_) => nv12_reader(src, true)?,
    };
    let audio = audio.filter(|_| sound);
    let fps = nominal_fps(&reader, video);
    // SAFETY: Media Foundation calls with arguments that live through each call.
    let rate = unsafe {
        match audio {
            Some(a) => {
                let native = reader
                    .GetNativeMediaType(a, 0)
                    .and_then(|t| t.GetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND))
                    .unwrap_or(48_000);
                let rate = if native == 44_100 { 44_100 } else { 48_000 };
                reader
                    .SetCurrentMediaType(
                        a,
                        None,
                        &pcm(&MFAudioFormat_PCM, rate).map_err(|e| err("audio type", e))?,
                    )
                    .map_err(|e| err("audio type", e))?;
                Some(rate)
            }
            None => None,
        }
    };
    let size = (g.shown.0 & !1, g.shown.1 & !1);
    // The software encoder: fed from memory, the hardware ones mark SD pictures as
    // BT.709 and the colors shift (ADR 0008). Feeding them GPU surfaces, as the
    // recorder does, is a later improvement.
    let out = out(dst, size, fps, rate, false)?;
    let total: f64 = keep.iter().map(Span::len).sum();
    let same = |p: Vec<u8>| p;
    let input = Input {
        reader: &reader,
        g,
        video,
        audio,
        size,
        fit: &same,
    };
    let mut offset = 0.0;
    for span in &keep {
        part(&input, &out, *span, offset, total, progress, cancel)?;
        offset += span.len();
    }
    out.finish()?;
    progress(1.0);
    Ok(())
}

/// A file being read into a new video: its reader, streams and picture size, and how
/// a picture goes onto the new video's frame.
struct Input<'a> {
    reader: &'a IMFSourceReader,
    g: Geometry,
    video: u32,
    audio: Option<u32>,
    /// The picture's size, even.
    size: (u32, u32),
    fit: &'a dyn Fn(Vec<u8>) -> Vec<u8>,
}

/// One video of a combination (VID-067): its file and the parts kept.
#[derive(Debug, Clone)]
pub struct Piece {
    pub path: std::path::PathBuf,
    pub keep: Vec<Span>,
}

/// NV12 black: luma 16, chroma 128.
fn black(size: (u32, u32)) -> Vec<u8> {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let mut v = vec![16u8; w * h];
    v.resize(w * h * 3 / 2, 128);
    v
}

/// Puts an NV12 picture of `size` in the middle of a black frame of `frame` (both even).
fn centered(picture: &[u8], size: (u32, u32), frame: (u32, u32)) -> Vec<u8> {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let (fw, fh) = (frame.0 as usize, frame.1 as usize);
    let (x, y) = (((fw - w) / 2) & !1, ((fh - h) / 2) & !1);
    let mut out = black(frame);
    for row in 0..h {
        out[(y + row) * fw + x..(y + row) * fw + x + w].copy_from_slice(&picture[row * w..row * w + w]);
    }
    let (luma, frame_luma) = (w * h, fw * fh);
    for row in 0..h / 2 {
        let to = frame_luma + (y / 2 + row) * fw + x;
        out[to..to + w].copy_from_slice(&picture[luma + row * w..luma + row * w + w]);
    }
    out
}

/// Re-codes an NV12 picture from one color matrix to the other (BT.601 and BT.709,
/// limited range), so a small video keeps its colors in a large one (VID-067).
pub(crate) fn rematrix(picture: &mut [u8], size: (u32, u32), from_709: bool) {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let coef = |hd: bool| if hd { (0.2126, 0.0722) } else { (0.299, 0.114) };
    let ((kr0, kb0), (kr1, kb1)) = (coef(from_709), coef(!from_709));
    let (kg0, kg1) = (1.0 - kr0 - kb0, 1.0 - kr1 - kb1);
    let luma = w * h;
    for by in 0..h / 2 {
        for bx in 0..w / 2 {
            let c = luma + by * w + bx * 2;
            let u = (picture[c] as f64 - 128.0) / 224.0;
            let v = (picture[c + 1] as f64 - 128.0) / 224.0;
            let (mut su, mut sv) = (0.0, 0.0);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let i = (by * 2 + dy) * w + bx * 2 + dx;
                let y = (picture[i] as f64 - 16.0) / 219.0;
                let r = y + 2.0 * (1.0 - kr0) * v;
                let b = y + 2.0 * (1.0 - kb0) * u;
                let g = (y - kr0 * r - kb0 * b) / kg0;
                let y1 = kr1 * r + kg1 * g + kb1 * b;
                su += (b - y1) / (2.0 * (1.0 - kb1));
                sv += (r - y1) / (2.0 * (1.0 - kr1));
                picture[i] = (16.0 + 219.0 * y1).round().clamp(0.0, 255.0) as u8;
            }
            picture[c] = (128.0 + 224.0 * su / 4.0).round().clamp(0.0, 255.0) as u8;
            picture[c + 1] = (128.0 + 224.0 * sv / 4.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// Writes the videos of `pieces`, one after the other, as a new MP4 at `dst` (VID-067):
/// as wide and as tall as the largest, the smaller ones centered on black; the
/// highest frame rate; sound when any has it (silence for the others). Returns its
/// size. `cancel` stops it, leaving no file.
pub fn combine(
    pieces: &[Piece],
    dst: &Path,
    progress: &mut dyn FnMut(f64),
    cancel: &AtomicBool,
) -> Result<(u32, u32), String> {
    let result = combine_into(pieces, dst, progress, cancel);
    if result.is_err() {
        let _ = std::fs::remove_file(dst);
    }
    result
}

fn combine_into(
    pieces: &[Piece],
    dst: &Path,
    progress: &mut dyn FnMut(f64),
    cancel: &AtomicBool,
) -> Result<(u32, u32), String> {
    start();
    if pieces.len() < 2 {
        return Err("at least two videos".into());
    }
    let facts: Vec<crate::Facts> = pieces.iter().map(|p| facts(&p.path)).collect::<Result<_, _>>()?;
    let frame = (
        facts.iter().map(|f| f.width).max().unwrap_or(2) & !1,
        facts.iter().map(|f| f.height).max().unwrap_or(2) & !1,
    );
    let fps = facts.iter().map(|f| f.fps).max().unwrap_or(30);
    let rate = facts.iter().any(|f| f.sound).then_some(48_000);
    let out = out(dst, frame, fps, rate, false)?;
    let total: f64 = pieces.iter().flat_map(|p| p.keep.iter()).map(Span::len).sum();
    let hd = crate::win::is_hd(frame);
    let mut offset = 0.0;
    for piece in pieces {
        let (reader, video, audio, g) = match nv12_reader(&piece.path, false) {
            Ok(r) => r,
            Err(_) => nv12_reader(&piece.path, true)?,
        };
        if let Some(a) = audio {
            // SAFETY: Media Foundation call with a new media type.
            unsafe {
                reader
                    .SetCurrentMediaType(
                        a,
                        None,
                        &pcm(&MFAudioFormat_PCM, 48_000).map_err(|e| err("audio type", e))?,
                    )
                    .map_err(|e| err("audio type", e))?;
            }
        }
        let size = (g.shown.0 & !1, g.shown.1 & !1);
        let convert = crate::win::is_hd(size) != hd;
        let place = move |mut p: Vec<u8>| {
            if convert {
                rematrix(&mut p, size, !hd);
            }
            if size == frame {
                p
            } else {
                centered(&p, size, frame)
            }
        };
        let input = Input {
            reader: &reader,
            g,
            video,
            audio,
            size,
            fit: &place,
        };
        for span in piece.keep.iter().filter(|s| !s.is_empty()) {
            part(&input, &out, *span, offset, total, progress, cancel)?;
            // A video without sound: silence for its time, so the rest stays in step.
            if let (Some((stream, rate)), None) = (out.audio, audio) {
                let frames = (span.len() * rate as f64).round() as usize;
                let mut done = 0;
                while done < frames {
                    let n = (frames - done).min(4800);
                    out.write(
                        stream,
                        &vec![0u8; n * 4],
                        offset + done as f64 / rate as f64,
                        n as f64 / rate as f64,
                    )?;
                    done += n;
                }
            }
            offset += span.len();
        }
    }
    out.finish()?;
    progress(1.0);
    Ok(frame)
}

/// Writes one part of a file at `offset` of the new video.
fn part(
    input: &Input,
    out: &Out,
    span: Span,
    offset: f64,
    total: f64,
    progress: &mut dyn FnMut(f64),
    cancel: &AtomicBool,
) -> Result<(), String> {
    let Input {
        reader,
        g,
        video,
        audio,
        size,
        fit,
    } = *input;
    let (a, b) = ((span.start * HNS) as i64, (span.end * HNS) as i64);
    let at = |t: i64| offset + (t - a) as f64 / HNS;
    // SAFETY: Media Foundation reading calls.
    unsafe {
        let _ = reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false);
        let _ = reader.SetStreamSelection(video, true);
        if let Some(s) = audio {
            let _ = reader.SetStreamSelection(s, true);
        }
    }
    seek(reader, span.start)?;
    // The last picture read, waiting to know how long it lasts: a picture from before
    // the part shows from its start.
    let mut held: Option<(Vec<u8>, i64)> = None;
    let (mut video_done, mut audio_done) = (false, audio.is_none());
    while !(video_done && audio_done) {
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        let mut stream = 0u32;
        let mut flags = 0u32;
        let mut time = 0i64;
        let mut sample = None;
        // SAFETY: a read from the selected streams.
        unsafe {
            reader
                .ReadSample(
                    MF_SOURCE_READER_ANY_STREAM.0 as u32,
                    0,
                    Some(&mut stream),
                    Some(&mut flags),
                    Some(&mut time),
                    Some(&mut sample),
                )
                .map_err(|e| err("reading the video", e))?;
        }
        if flags & MF_SOURCE_READERF_ERROR.0 as u32 != 0 {
            return Err("the video could not be read to the end".into());
        }
        let ended = flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0;
        if stream == video {
            if let Some(s) = sample.filter(|_| !video_done) {
                if time >= b {
                    video_done = true;
                } else {
                    let picture = fit(nv12(&s, g, size)?);
                    let start = time.max(a);
                    if let Some((p, t)) = held.take() {
                        if start > t {
                            out.write(out.video, &p, at(t), (start - t) as f64 / HNS)?;
                        }
                    }
                    held = Some((picture, start));
                    progress(at(start) / total.max(1e-9));
                }
            }
            if ended || video_done {
                video_done = true;
                // SAFETY: stops reading this stream.
                let _ = unsafe { reader.SetStreamSelection(video, false) };
            }
        } else if Some(stream) == audio {
            if let Some(s) = sample.filter(|_| !audio_done) {
                if time >= b {
                    audio_done = true;
                } else {
                    sound(out, &s, time, a, b, offset)?;
                }
            }
            if ended || audio_done {
                audio_done = true;
                // SAFETY: stops reading this stream.
                let _ = unsafe { reader.SetStreamSelection(stream, false) };
            }
        }
    }
    if let Some((p, t)) = held {
        out.write(out.video, &p, at(t), ((b - t) as f64 / HNS).max(1.0 / 1000.0))?;
    }
    Ok(())
}

/// Writes the part of a sound sample that falls in `a..b`, at its place.
fn sound(out: &Out, s: &IMFSample, time: i64, a: i64, b: i64, offset: f64) -> Result<(), String> {
    let Some((stream, rate)) = out.audio else {
        return Ok(());
    };
    // SAFETY: the buffer is locked while copied and unlocked.
    let bytes = unsafe {
        let buffer = s.ConvertToContiguousBuffer().map_err(|e| err("sound", e))?;
        let mut data = std::ptr::null_mut();
        let mut len = 0u32;
        buffer
            .Lock(&mut data, None, Some(&mut len))
            .map_err(|e| err("sound", e))?;
        let v = std::slice::from_raw_parts(data, len as usize).to_vec();
        let _ = buffer.Unlock();
        v
    };
    let frames = bytes.len() / 4;
    let per = HNS / rate as f64;
    let first = (((a - time) as f64 / per).ceil().max(0.0) as usize).min(frames);
    let last = (((b - time) as f64 / per).ceil().max(0.0) as usize).min(frames);
    if last <= first {
        return Ok(());
    }
    let start = time as f64 + first as f64 * per;
    out.write(
        stream,
        &bytes[first * 4..last * 4],
        offset + (start - a as f64) / HNS,
        (last - first) as f64 / rate as f64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::Span;

    const W: u32 = 320;
    const H: u32 = 240;

    /// NV12 of one color (BT.601, limited range, as the writer says below 720p).
    fn solid(rgb: [f64; 3]) -> Vec<u8> {
        let [r, g, b] = rgb;
        let y = (0.257 * r + 0.504 * g + 0.098 * b + 16.0).round() as u8;
        let u = (-0.148 * r - 0.291 * g + 0.439 * b + 128.0).round() as u8;
        let v = (0.439 * r - 0.368 * g - 0.071 * b + 128.0).round() as u8;
        let mut out = vec![y; (W * H) as usize];
        for _ in 0..(W * H / 4) {
            out.extend_from_slice(&[u, v]);
        }
        out
    }

    /// A test video: `frames` of (time, color), lasting until `end`, with a tone.
    fn make(path: &Path, frames: &[(f64, [f64; 3])], end: f64, tone: bool) {
        start();
        let _ = std::fs::remove_file(path);
        let o = out(path, (W, H), 10, tone.then_some(48_000), false).unwrap();
        for (i, (t, c)) in frames.iter().enumerate() {
            let next = frames.get(i + 1).map(|f| f.0).unwrap_or(end);
            o.write(o.video, &solid(*c), *t, next - t).unwrap();
        }
        if tone {
            let rate = 48_000.0;
            let mut at = 0.0;
            while at < end - 1e-9 {
                let n = 4800usize.min(((end - at) * rate).round() as usize);
                let mut pcm = Vec::with_capacity(n * 4);
                for i in 0..n {
                    let s = ((at + i as f64 / rate) * 440.0 * std::f64::consts::TAU).sin();
                    let v = ((s * 8000.0) as i16).to_le_bytes();
                    pcm.extend_from_slice(&[v[0], v[1], v[0], v[1]]);
                }
                o.write(o.audio.unwrap().0, &pcm, at, n as f64 / rate).unwrap();
                at += n as f64 / rate;
            }
        }
        o.finish().unwrap();
    }

    const RED: [f64; 3] = [200.0, 30.0, 30.0];
    const GREEN: [f64; 3] = [30.0, 180.0, 40.0];
    const BLUE: [f64; 3] = [30.0, 60.0, 210.0];

    /// Red for the first second, green for the second, blue for the third.
    fn three(path: &Path, tone: bool) {
        let mut frames = Vec::new();
        for i in 0..30 {
            let c = [RED, GREEN, BLUE][i / 10];
            frames.push((i as f64 / 10.0, c));
        }
        make(path, &frames, 3.0, tone);
    }

    fn color_at(path: &Path, t: f64) -> [u8; 3] {
        let (w, h, px) = frame_at(path, t).unwrap();
        let i = (((h / 2) * w + w / 2) * 4) as usize;
        [px[i + 2], px[i + 1], px[i]]
    }

    fn near(c: [u8; 3], want: [f64; 3]) -> bool {
        c.iter().zip(want).all(|(a, b)| (*a as f64 - b).abs() <= 12.0)
    }

    fn dir() -> std::path::PathBuf {
        let d = std::env::temp_dir().join("xs-video-edit-test");
        let _ = std::fs::create_dir_all(&d);
        d
    }

    /// VID-061: the frame at a moment is the one shown then.
    #[test]
    fn frames_at_moments() {
        let src = dir().join("frames.mp4");
        three(&src, false);
        assert!(near(color_at(&src, 0.0), RED), "{:?}", color_at(&src, 0.0));
        assert!(near(color_at(&src, 1.55), GREEN), "{:?}", color_at(&src, 1.55));
        assert!(near(color_at(&src, 2.9), BLUE), "{:?}", color_at(&src, 2.9));
        let (w, h, _) = frame_at(&src, 1.0).unwrap();
        assert_eq!((w, h), (W, H));
    }

    /// VID-059: cutting the green second joins red and blue; the sound stays, as long.
    #[test]
    fn a_cut_joins_both_sides() {
        let src = dir().join("cut-src.mp4");
        let dst = dir().join("cut.mp4");
        three(&src, true);
        let mut seen = 0.0;
        export(
            &src,
            &dst,
            &[Span::new(0.0, 1.0), Span::new(2.0, 3.0)],
            true,
            &mut |p| seen = p,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(seen, 1.0);
        let info = crate::info(&dst).unwrap();
        assert!((info.duration - 2.0).abs() < 0.1, "duration {}", info.duration);
        assert!(near(color_at(&dst, 0.5), RED));
        assert!(near(color_at(&dst, 1.2), BLUE), "{:?}", color_at(&dst, 1.2));
        let (_, _, sound_end) = crate::audio_info(&dst).unwrap().expect("sound");
        assert!((sound_end - 2.0).abs() < 0.1, "sound until {sound_end}");
    }

    /// VID-063: a silent copy has no sound and lasts the same.
    #[test]
    fn a_silent_copy() {
        let src = dir().join("silent-src.mp4");
        let dst = dir().join("silent.mp4");
        three(&src, true);
        export(
            &src,
            &dst,
            &[Span::new(0.0, 3.0)],
            false,
            &mut |_| {},
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(crate::audio_info(&dst).unwrap(), None);
        let info = crate::info(&dst).unwrap();
        assert!((info.duration - 3.0).abs() < 0.15, "duration {}", info.duration);
    }

    /// Screen recordings only have a frame when something changes: a part with no frame
    /// of its own shows the one from before it, for as long as it lasts.
    #[test]
    fn a_still_part_keeps_its_picture() {
        let src = dir().join("still-src.mp4");
        let dst = dir().join("still.mp4");
        make(&src, &[(0.0, GREEN), (2.5, BLUE)], 3.0, false);
        export(
            &src,
            &dst,
            &[Span::new(1.0, 2.0)],
            true,
            &mut |_| {},
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(near(color_at(&dst, 0.5), GREEN));
        let info = crate::info(&dst).unwrap();
        assert!((info.duration - 1.0).abs() < 0.1, "duration {}", info.duration);
    }

    /// VID-068: frames at a steady rate across parts, each the picture shown then.
    #[test]
    fn frames_at_a_steady_rate() {
        let src = dir().join("frames-rate.mp4");
        three(&src, false);
        let mut seen = Vec::new();
        frames(
            &src,
            &[Span::new(0.5, 1.5), Span::new(2.0, 2.5)],
            4.0,
            &mut |t, w, h, px| {
                let i = (((h / 2) * w + w / 2) * 4) as usize;
                seen.push((t, [px[i + 2], px[i + 1], px[i]]));
                true
            },
        )
        .unwrap();
        // 1 s at 4 fps, then 0.5 s: 6 frames, 0.25 s apart.
        assert_eq!(seen.len(), 6, "{seen:?}");
        assert!((seen[5].0 - 1.25).abs() < 1e-9);
        assert!(
            near(seen[0].1, RED) && near(seen[3].1, GREEN) && near(seen[4].1, BLUE),
            "{seen:?}"
        );
    }

    /// Opening a video reads its size, length and sound without reading it all.
    #[test]
    fn facts_of_a_file() {
        let src = dir().join("facts.mp4");
        three(&src, true);
        let f = facts(&src).unwrap();
        assert_eq!((f.width, f.height, f.fps, f.sound), (W, H, 10, true));
        assert_eq!(f.codec, "H.264");
        assert!((f.duration - 3.0).abs() < 0.05, "duration {}", f.duration);
    }

    /// Writes the editor's test video (`tests/fixtures/video/three-colors.mp4`): run with
    /// `XS_VIDEO_FIXTURE=<path>`.
    #[test]
    #[ignore]
    fn write_fixture() {
        let Ok(path) = std::env::var("XS_VIDEO_FIXTURE") else {
            return;
        };
        three(Path::new(&path), true);
    }

    /// VID-067: a 320 × 240 video with sound and a 160 × 120 one without, one after the
    /// other: 320 × 240, the small one centered on black, sound all along.
    #[test]
    fn videos_combined() {
        let big = dir().join("combine-big.mp4");
        let small = dir().join("combine-small.mp4");
        let dst = dir().join("combined.mp4");
        three(&big, true);
        start();
        let _ = std::fs::remove_file(&small);
        let o = out(&small, (160, 120), 10, None, false).unwrap();
        let green = {
            let [r, g, b] = GREEN;
            let y = (0.257 * r + 0.504 * g + 0.098 * b + 16.0).round() as u8;
            let u = (-0.148 * r - 0.291 * g + 0.439 * b + 128.0).round() as u8;
            let v = (0.439 * r - 0.368 * g - 0.071 * b + 128.0).round() as u8;
            let mut p = vec![y; 160 * 120];
            for _ in 0..(160 * 120 / 4) {
                p.extend_from_slice(&[u, v]);
            }
            p
        };
        for i in 0..20 {
            o.write(o.video, &green, i as f64 / 10.0, 0.1).unwrap();
        }
        o.finish().unwrap();
        let pieces = [
            Piece {
                path: big.clone(),
                keep: vec![Span::new(0.0, 1.0)],
            },
            Piece {
                path: small.clone(),
                keep: vec![Span::new(0.0, 2.0)],
            },
        ];
        let size = combine(&pieces, &dst, &mut |_| {}, &AtomicBool::new(false)).unwrap();
        assert_eq!(size, (W, H));
        let info = crate::info(&dst).unwrap();
        assert!((info.duration - 3.0).abs() < 0.15, "duration {}", info.duration);
        assert!(near(color_at(&dst, 0.5), RED));
        let (w, h, px) = frame_at(&dst, 2.0).unwrap();
        let at = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            [px[i + 2], px[i + 1], px[i]]
        };
        assert!(near(at(w / 2, h / 2), GREEN), "{:?}", at(w / 2, h / 2));
        assert!(at(5, 5).iter().all(|c| *c < 30), "black band: {:?}", at(5, 5));
        let (_, _, sound_end) = crate::audio_info(&dst).unwrap().expect("sound");
        assert!((sound_end - 3.0).abs() < 0.15, "sound until {sound_end}");
    }

    /// BT.601 to BT.709 and back leaves a picture as it was (±2).
    #[test]
    fn colors_change_matrix_and_back() {
        let mut p = vec![0u8; 4 * 4 * 3 / 2];
        for (i, v) in p.iter_mut().enumerate() {
            *v = if i < 16 {
                40 + (i as u8) * 10
            } else {
                100 + (i as u8) * 3
            };
        }
        let before = p.clone();
        rematrix(&mut p, (4, 4), false);
        assert_ne!(p, before);
        rematrix(&mut p, (4, 4), true);
        for (a, b) in p.iter().zip(&before) {
            assert!((*a as i32 - *b as i32).abs() <= 3, "{p:?} vs {before:?}");
        }
    }

    /// VID-067: cancelling leaves no file.
    #[test]
    fn cancelling_leaves_nothing() {
        let src = dir().join("cancel-src.mp4");
        let dst = dir().join("cancel.mp4");
        three(&src, false);
        let e = export(
            &src,
            &dst,
            &[Span::new(0.0, 3.0)],
            true,
            &mut |_| {},
            &AtomicBool::new(true),
        )
        .unwrap_err();
        assert_eq!(e, "cancelled");
        assert!(!dst.exists());
    }
}
