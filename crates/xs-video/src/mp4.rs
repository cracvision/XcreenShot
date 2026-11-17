//! What XcreenShot writes in an MP4 itself, after the system's writer made it: the
//! names of its sound tracks (VID-042), in each track's handler box (`trak/mdia/hdlr`),
//! and its creation date and title (VID-026), in the movie header (`mvhd`) and in the
//! `©nam` and `©day` items players and Explorer show. Only the movie box changes; when
//! it comes before the media data, the chunk offsets move with it. Plain file work,
//! the same on every platform.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// A box: where it starts in its parent's bytes, its header and its total size.
struct Bx {
    at: usize,
    header: usize,
    size: usize,
    kind: [u8; 4],
}

/// The boxes inside `data` (a box's contents), in order. Stops at the first that
/// does not fit.
fn boxes(data: &[u8]) -> Vec<Bx> {
    let mut out = Vec::new();
    let mut at = 0;
    while at + 8 <= data.len() {
        let size32 = u32::from_be_bytes(data[at..at + 4].try_into().unwrap()) as usize;
        let kind: [u8; 4] = data[at + 4..at + 8].try_into().unwrap();
        let (header, size) = match size32 {
            0 => (8, data.len() - at),
            1 if at + 16 <= data.len() => (
                16,
                u64::from_be_bytes(data[at + 8..at + 16].try_into().unwrap()) as usize,
            ),
            1 => break,
            n => (8, n),
        };
        if size < header || at + size > data.len() {
            break;
        }
        out.push(Bx {
            at,
            header,
            size,
            kind,
        });
        at += size;
    }
    out
}

fn child<'a>(data: &'a [u8], kind: &[u8; 4]) -> Option<&'a [u8]> {
    boxes(data)
        .into_iter()
        .find(|b| &b.kind == kind)
        .map(|b| &data[b.at + b.header..b.at + b.size])
}

/// A top-level box of the file: where it starts, its header and total size.
struct Top {
    at: u64,
    header: u64,
    size: u64,
    kind: [u8; 4],
}

fn top_boxes(file: &mut File) -> std::io::Result<Vec<Top>> {
    let len = file.metadata()?.len();
    let mut out = Vec::new();
    let mut at = 0u64;
    while at + 8 <= len {
        file.seek(SeekFrom::Start(at))?;
        let mut h = [0u8; 16];
        let n = if at + 16 <= len { 16 } else { 8 };
        file.read_exact(&mut h[..n])?;
        let size32 = u32::from_be_bytes(h[0..4].try_into().unwrap()) as u64;
        let kind: [u8; 4] = h[4..8].try_into().unwrap();
        let (header, size) = match size32 {
            0 => (8, len - at),
            1 if n == 16 => (16, u64::from_be_bytes(h[8..16].try_into().unwrap())),
            1 => break,
            s => (8, s),
        };
        if size < header || at + size > len {
            break;
        }
        out.push(Top {
            at,
            header,
            size,
            kind,
        });
        at += size;
    }
    Ok(out)
}

/// The file's top-level boxes, which of them is the movie box, and its contents.
type Movie = (Vec<Top>, usize, Vec<u8>);

/// The movie box's contents, and where it is among the file's boxes.
fn read_moov(file: &mut File) -> std::io::Result<Option<Movie>> {
    let tops = top_boxes(file)?;
    let Some(i) = tops.iter().position(|t| &t.kind == b"moov") else {
        return Ok(None);
    };
    let m = &tops[i];
    if m.size > 256 << 20 {
        return Ok(None);
    }
    let mut data = vec![0u8; (m.size - m.header) as usize];
    file.seek(SeekFrom::Start(m.at + m.header))?;
    file.read_exact(&mut data)?;
    Ok(Some((tops, i, data)))
}

/// Whether a track (its `trak` contents) holds sound.
fn is_sound(trak: &[u8]) -> bool {
    child(trak, b"mdia")
        .and_then(|m| child(m, b"hdlr"))
        .is_some_and(|h| h.len() >= 12 && &h[8..12] == b"soun")
}

/// The name in a handler box's contents: after version, flags, pre-defined, type and
/// three reserved words, up to its end or a NUL.
fn handler_name(hdlr: &[u8]) -> String {
    let name = hdlr.get(24..).unwrap_or(&[]);
    let end = name.iter().position(|b| *b == 0).unwrap_or(name.len());
    String::from_utf8_lossy(&name[..end]).into_owned()
}

/// The names of the file's sound tracks, in order ("" for one without a name).
pub fn sound_track_names(path: &Path) -> Result<Vec<String>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let Some((_, _, moov)) = read_moov(&mut file).map_err(|e| e.to_string())? else {
        return Ok(Vec::new());
    };
    Ok(boxes(&moov)
        .iter()
        .filter(|b| &b.kind == b"trak")
        .map(|b| &moov[b.at + b.header..b.at + b.size])
        .filter(|t| is_sound(t))
        .filter_map(|t| child(t, b"mdia").and_then(|m| child(m, b"hdlr")))
        .map(handler_name)
        .collect())
}

/// A box of `kind` around `body`.
fn wrap(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(body);
    out
}

/// What [`rebuild`] asks of each box: its new contents, or `None` to keep it.
type Change<'a> = dyn FnMut(&[u8; 4], &[u8]) -> Option<Vec<u8>> + 'a;

/// Rebuilds the boxes of `data`, giving each to `f` (kind, contents) for its new
/// contents, or keeping it as it is when `f` returns `None`.
fn rebuild(data: &[u8], f: &mut Change) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 64);
    let list = boxes(data);
    let end = list.last().map_or(0, |b| b.at + b.size);
    for b in &list {
        let body = &data[b.at + b.header..b.at + b.size];
        match f(&b.kind, body) {
            Some(new) => out.extend_from_slice(&wrap(&b.kind, &new)),
            None => out.extend_from_slice(&data[b.at..b.at + b.size]),
        }
    }
    // Whatever trails the last box stays.
    out.extend_from_slice(&data[end..]);
    out
}

/// The handler box's contents with `name` in place of its own.
fn renamed(hdlr: &[u8], name: &str) -> Vec<u8> {
    let mut out = hdlr[..24.min(hdlr.len())].to_vec();
    out.resize(24, 0);
    out.extend_from_slice(name.as_bytes());
    out.push(0);
    out
}

/// Adds `delta` to every chunk offset (`stco`, `co64`) of the movie box's contents.
fn shift_offsets(moov: &[u8], delta: i64) -> Vec<u8> {
    fn walk(data: &[u8], delta: i64) -> Vec<u8> {
        rebuild(data, &mut |kind, body| match kind {
            b"trak" | b"mdia" | b"minf" | b"stbl" => Some(walk(body, delta)),
            b"stco" if body.len() >= 8 => {
                let mut b = body.to_vec();
                let n = u32::from_be_bytes(b[4..8].try_into().unwrap()) as usize;
                for i in 0..n.min((b.len() - 8) / 4) {
                    let at = 8 + i * 4;
                    let v = u32::from_be_bytes(b[at..at + 4].try_into().unwrap()) as i64 + delta;
                    b[at..at + 4].copy_from_slice(&(v.clamp(0, u32::MAX as i64) as u32).to_be_bytes());
                }
                Some(b)
            }
            b"co64" if body.len() >= 8 => {
                let mut b = body.to_vec();
                let n = u32::from_be_bytes(b[4..8].try_into().unwrap()) as usize;
                for i in 0..n.min((b.len() - 8) / 8) {
                    let at = 8 + i * 8;
                    let v = u64::from_be_bytes(b[at..at + 8].try_into().unwrap()) as i64 + delta;
                    b[at..at + 8].copy_from_slice(&(v.max(0) as u64).to_be_bytes());
                }
                Some(b)
            }
            _ => None,
        })
    }
    walk(moov, delta)
}

/// Gives the file's sound tracks the `names`, in order (VID-042); tracks beyond the
/// names keep theirs. Nothing changes when the file has no movie box or no sound.
pub fn name_sound_tracks(path: &Path, names: &[String]) -> Result<(), String> {
    let io = |e: std::io::Error| e.to_string();
    let mut file = File::options().read(true).write(true).open(path).map_err(io)?;
    let Some((tops, i, moov)) = read_moov(&mut file).map_err(io)? else {
        return Ok(());
    };
    let mut n = 0;
    let new_body = rebuild(&moov, &mut |kind, trak| {
        if kind != b"trak" || !is_sound(trak) {
            return None;
        }
        let name = names.get(n)?;
        n += 1;
        Some(rebuild(trak, &mut |kind, mdia| {
            (kind == b"mdia").then(|| {
                rebuild(mdia, &mut |kind, hdlr| {
                    (kind == b"hdlr").then(|| renamed(hdlr, name))
                })
            })
        }))
    });
    if n == 0 {
        return Ok(());
    }
    write_moov(file, path, &tops, i, new_body)
}

/// Puts `new_body` in place of the movie box `i` of `tops`: in place when it is last,
/// else through a new file; the chunk offsets follow media data that moves.
fn write_moov(mut file: File, path: &Path, tops: &[Top], i: usize, new_body: Vec<u8>) -> Result<(), String> {
    let io = |e: std::io::Error| e.to_string();
    let m = &tops[i];
    let old_size = m.size;
    // The media data after the movie box moves by the size it gains or loses.
    let new_size = new_body.len() as u64 + 8;
    let delta = new_size as i64 - old_size as i64;
    let data_after = tops[i + 1..].iter().any(|t| &t.kind == b"mdat");
    let new_body = if data_after && delta != 0 {
        shift_offsets(&new_body, delta)
    } else {
        new_body
    };
    let moov = wrap(b"moov", &new_body);
    if i + 1 == tops.len() {
        // The usual case: the movie box last, rewritten in place.
        file.seek(SeekFrom::Start(m.at)).map_err(io)?;
        file.write_all(&moov).map_err(io)?;
        file.set_len(m.at + moov.len() as u64).map_err(io)?;
        return file.sync_all().map_err(io);
    }
    // Boxes after it: a new file, then put in its place.
    let tmp = path.with_extension("names.tmp");
    let copy = || -> std::io::Result<()> {
        let mut out = File::create(&tmp)?;
        let mut src = File::open(path)?;
        std::io::copy(&mut (&mut src).take(m.at), &mut out)?;
        out.write_all(&moov)?;
        src.seek(SeekFrom::Start(m.at + old_size))?;
        std::io::copy(&mut src, &mut out)?;
        out.sync_all()
    };
    drop(file);
    if let Err(e) = copy() {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.to_string());
    }
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        e.to_string()
    })
}

/// Seconds from 1904-01-01 (the MP4 epoch) to 1970-01-01.
const MP4_EPOCH: u64 = 2_082_844_800;

/// An iTunes-style item (`©nam`, `©day`): a `data` box with UTF-8 text.
fn text_item(kind: &[u8; 4], text: &str) -> Vec<u8> {
    let mut data = vec![0, 0, 0, 1, 0, 0, 0, 0];
    data.extend_from_slice(text.as_bytes());
    wrap(kind, &wrap(b"data", &data))
}

/// The `udta` with the title and date, keeping whatever else it had.
fn user_data(old: Option<&[u8]>, title: &str, day: &str) -> Vec<u8> {
    let mut hdlr = vec![0u8; 8];
    hdlr.extend_from_slice(b"mdir");
    hdlr.extend_from_slice(b"appl");
    hdlr.extend_from_slice(&[0u8; 9]);
    let mut ilst = text_item(b"\xa9nam", title);
    ilst.extend_from_slice(&text_item(b"\xa9day", day));
    let mut meta = vec![0u8; 4];
    meta.extend_from_slice(&wrap(b"hdlr", &hdlr));
    meta.extend_from_slice(&wrap(b"ilst", &ilst));
    let mut out = Vec::new();
    // Another `meta` (a writer's own) goes: ours says the title and date.
    if let Some(old) = old {
        for b in boxes(old).iter().filter(|b| &b.kind != b"meta") {
            out.extend_from_slice(&old[b.at..b.at + b.size]);
        }
    }
    out.extend_from_slice(&wrap(b"meta", &meta));
    out
}

/// Writes the file's creation date (`created`, seconds since 1970, UTC) and `title`
/// (VID-026): in the movie header, and as the title and date items players show.
pub fn set_title_and_date(path: &Path, title: &str, created: u64) -> Result<(), String> {
    let io = |e: std::io::Error| e.to_string();
    let mut file = File::options().read(true).write(true).open(path).map_err(io)?;
    let Some((tops, i, moov)) = read_moov(&mut file).map_err(io)? else {
        return Err("no movie box".into());
    };
    let when = created + MP4_EPOCH;
    let day = chrono::DateTime::from_timestamp(created as i64, 0)
        .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_default();
    let mut had_udta = false;
    let mut body = rebuild(&moov, &mut |kind, b| match kind {
        b"mvhd" if b.len() >= 20 => {
            let mut b = b.to_vec();
            if b[0] == 1 && b.len() >= 20 {
                b[4..12].copy_from_slice(&when.to_be_bytes());
                b[12..20].copy_from_slice(&when.to_be_bytes());
            } else {
                b[4..8].copy_from_slice(&(when as u32).to_be_bytes());
                b[8..12].copy_from_slice(&(when as u32).to_be_bytes());
            }
            Some(b)
        }
        b"udta" => {
            had_udta = true;
            Some(user_data(Some(b), title, &day))
        }
        _ => None,
    });
    if !had_udta {
        body.extend_from_slice(&wrap(b"udta", &user_data(None, title, &day)));
    }
    write_moov(file, path, &tops, i, body)
}

/// The title and creation date (seconds since 1970) written in the file, if any.
pub fn title_and_date(path: &Path) -> Result<(Option<String>, Option<u64>), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let Some((_, _, moov)) = read_moov(&mut file).map_err(|e| e.to_string())? else {
        return Ok((None, None));
    };
    let created = child(&moov, b"mvhd").and_then(|b| {
        let secs = if b.first() == Some(&1) {
            u64::from_be_bytes(b.get(4..12)?.try_into().ok()?)
        } else {
            u32::from_be_bytes(b.get(4..8)?.try_into().ok()?) as u64
        };
        secs.checked_sub(MP4_EPOCH)
    });
    let title = child(&moov, b"udta")
        .and_then(|u| child(u, b"meta"))
        .and_then(|m| child(m.get(4..)?, b"ilst"))
        .and_then(|l| child(l, b"\xa9nam"))
        .and_then(|n| child(n, b"data"))
        .and_then(|d| d.get(8..))
        .map(|t| String::from_utf8_lossy(t).into_owned());
    Ok((title, created))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hdlr(kind: &[u8; 4], name: &str) -> Vec<u8> {
        let mut b = vec![0u8; 8];
        b.extend_from_slice(kind);
        b.extend_from_slice(&[0u8; 12]);
        b.extend_from_slice(name.as_bytes());
        b.push(0);
        wrap(b"hdlr", &b)
    }

    fn trak(kind: &[u8; 4], name: &str, offset: u32) -> Vec<u8> {
        let mut stco = vec![0u8; 4];
        stco.extend_from_slice(&1u32.to_be_bytes());
        stco.extend_from_slice(&offset.to_be_bytes());
        let stbl = wrap(b"stbl", &wrap(b"stco", &stco));
        let minf = wrap(b"minf", &stbl);
        let mut mdia = hdlr(kind, name);
        mdia.extend_from_slice(&minf);
        wrap(b"trak", &wrap(b"mdia", &mdia))
    }

    fn movie(tracks: &[(&[u8; 4], &str)], offset: u32) -> Vec<u8> {
        let mut body = wrap(b"mvhd", &[0u8; 20]);
        for (k, n) in tracks {
            body.extend_from_slice(&trak(k, n, offset));
        }
        wrap(b"moov", &body)
    }

    fn offsets(path: &Path) -> Vec<u32> {
        let mut f = File::open(path).unwrap();
        let (_, _, moov) = read_moov(&mut f).unwrap().unwrap();
        let mut out = Vec::new();
        for t in boxes(&moov).iter().filter(|b| &b.kind == b"trak") {
            let t = &moov[t.at + t.header..t.at + t.size];
            let stco = child(t, b"mdia")
                .and_then(|m| child(m, b"minf"))
                .and_then(|m| child(m, b"stbl"))
                .and_then(|s| child(s, b"stco"))
                .unwrap();
            out.push(u32::from_be_bytes(stco[8..12].try_into().unwrap()));
        }
        out
    }

    fn dir() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("xs-mp4-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    const TRACKS: [(&[u8; 4], &str); 3] = [
        (b"vide", "VideoHandler"),
        (b"soun", "SoundHandler"),
        (b"soun", "SoundHandler"),
    ];

    #[test]
    fn names_the_sound_tracks_with_the_movie_last() {
        let path = dir().join("last.mp4");
        let mut file = wrap(b"ftyp", b"isom\0\0\0\0");
        let mdat = wrap(b"mdat", &[7u8; 100]);
        let data_at = (file.len() + 8) as u32;
        file.extend_from_slice(&mdat);
        file.extend_from_slice(&movie(&TRACKS, data_at));
        std::fs::write(&path, &file).unwrap();
        assert_eq!(
            sound_track_names(&path).unwrap(),
            ["SoundHandler", "SoundHandler"]
        );
        let names = ["Micrófono".to_string(), "Audio del sistema".to_string()];
        name_sound_tracks(&path, &names).unwrap();
        assert_eq!(sound_track_names(&path).unwrap(), names);
        // The media data did not move.
        assert_eq!(offsets(&path), [data_at; 3]);
        let after = std::fs::read(&path).unwrap();
        assert_eq!(&after[data_at as usize..data_at as usize + 100], &[7u8; 100]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn names_the_sound_tracks_with_the_movie_first() {
        let path = dir().join("first.mp4");
        let ftyp = wrap(b"ftyp", b"isom\0\0\0\0");
        let first = movie(&TRACKS, 0);
        let data_at = (ftyp.len() + first.len() + 8) as u32;
        let mut file = ftyp.clone();
        file.extend_from_slice(&movie(&TRACKS, data_at));
        file.extend_from_slice(&wrap(b"mdat", &[9u8; 50]));
        std::fs::write(&path, &file).unwrap();
        name_sound_tracks(&path, &["A much longer microphone name".to_string()]).unwrap();
        assert_eq!(
            sound_track_names(&path).unwrap(),
            ["A much longer microphone name", "SoundHandler"]
        );
        // The chunk offsets follow the media data, which moved.
        let after = std::fs::read(&path).unwrap();
        let moved = offsets(&path)[0] as usize;
        assert!(moved > data_at as usize);
        assert!(offsets(&path).iter().all(|o| *o as usize == moved));
        assert_eq!(&after[moved..moved + 50], &[9u8; 50]);
        let _ = std::fs::remove_file(&path);
    }

    /// VID-026: the title and date go in the file, read back, the media data in place;
    /// writing them again replaces them.
    #[test]
    fn title_and_date_written() {
        let path = dir().join("meta.mp4");
        let mut file = wrap(b"ftyp", b"isom\0\0\0\0");
        let mdat = wrap(b"mdat", &[5u8; 64]);
        let data_at = (file.len() + 8) as u32;
        file.extend_from_slice(&mdat);
        let mut mvhd = vec![0u8; 100];
        mvhd[12..16].copy_from_slice(&1000u32.to_be_bytes());
        let mut body = wrap(b"mvhd", &mvhd);
        body.extend_from_slice(&trak(b"soun", "SoundHandler", data_at));
        file.extend_from_slice(&wrap(b"moov", &body));
        std::fs::write(&path, &file).unwrap();
        assert_eq!(title_and_date(&path).unwrap(), (None, None));
        set_title_and_date(&path, "Video 2026-11-14 19.39.00", 1_794_700_000).unwrap();
        assert_eq!(
            title_and_date(&path).unwrap(),
            (Some("Video 2026-11-14 19.39.00".into()), Some(1_794_700_000))
        );
        set_title_and_date(&path, "Otro título", 1_794_700_100).unwrap();
        let (title, _) = title_and_date(&path).unwrap();
        assert_eq!(title.as_deref(), Some("Otro título"));
        // The sound track and the media data are where they were.
        assert_eq!(sound_track_names(&path).unwrap(), ["SoundHandler"]);
        assert_eq!(offsets(&path), [data_at]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_file_without_sound_stays_as_it_is() {
        let path = dir().join("silent.mp4");
        let mut file = wrap(b"ftyp", b"isom\0\0\0\0");
        file.extend_from_slice(&movie(&[(b"vide", "VideoHandler")], 0));
        std::fs::write(&path, &file).unwrap();
        name_sound_tracks(&path, &["Microphone".to_string()]).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), file);
        assert!(sound_track_names(&path).unwrap().is_empty());
        let _ = std::fs::remove_file(&path);
    }
}
