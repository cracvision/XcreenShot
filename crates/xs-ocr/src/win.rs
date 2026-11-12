//! Windows: `Windows.Media.Ocr`, which comes with the system's languages. Nothing is
//! downloaded; a language that is missing is added in the system's settings (SMT-004).

use image::GrayImage;
use windows::core::HSTRING;
use windows::Globalization::Language as WinLanguage;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;

use crate::{Language, OcrError, RawLine};

fn engine_error(e: windows::core::Error) -> OcrError {
    OcrError::Engine(e.message())
}

/// The longest side the engine reads at once.
pub fn max_dimension() -> u32 {
    OcrEngine::MaxImageDimension().unwrap_or(4_096).max(1_000)
}

pub fn languages() -> Vec<Language> {
    let Ok(list) = OcrEngine::AvailableRecognizerLanguages() else {
        return Vec::new();
    };
    let n = list.Size().unwrap_or(0);
    (0..n)
        .filter_map(|i| list.GetAt(i).ok())
        .filter_map(|l| {
            Some(Language {
                tag: l.LanguageTag().ok()?.to_string(),
                name: l.DisplayName().ok()?.to_string(),
            })
        })
        .collect()
}

/// Reads a gray image; returns its lines (in its pixels) and the language used.
pub fn recognize(image: &GrayImage, tag: Option<&str>) -> Result<(Vec<RawLine>, String), OcrError> {
    let engine = match tag {
        Some(t) => {
            let lang = WinLanguage::CreateLanguage(&HSTRING::from(t)).map_err(engine_error)?;
            if !OcrEngine::IsLanguageSupported(&lang).unwrap_or(false) {
                return Err(OcrError::NoLanguage);
            }
            OcrEngine::TryCreateFromLanguage(&lang).map_err(|_| OcrError::NoLanguage)?
        }
        None => OcrEngine::TryCreateFromUserProfileLanguages().map_err(|_| OcrError::NoLanguage)?,
    };
    let used = engine
        .RecognizerLanguage()
        .and_then(|l| l.LanguageTag())
        .map(|t| t.to_string())
        .unwrap_or_default();
    let writer = DataWriter::new().map_err(engine_error)?;
    writer.WriteBytes(image.as_raw()).map_err(engine_error)?;
    let buffer = writer.DetachBuffer().map_err(engine_error)?;
    let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &buffer,
        BitmapPixelFormat::Gray8,
        image.width() as i32,
        image.height() as i32,
    )
    .map_err(engine_error)?;
    let result = engine
        .RecognizeAsync(&bitmap)
        .map_err(engine_error)?
        .join()
        .map_err(engine_error)?;
    let lines = result.Lines().map_err(engine_error)?;
    let mut out = Vec::new();
    for i in 0..lines.Size().map_err(engine_error)? {
        let line = lines.GetAt(i).map_err(engine_error)?;
        let words = line.Words().map_err(engine_error)?;
        let mut raw = RawLine { words: Vec::new() };
        for j in 0..words.Size().map_err(engine_error)? {
            let word = words.GetAt(j).map_err(engine_error)?;
            let r = word.BoundingRect().map_err(engine_error)?;
            raw.words.push((
                word.Text().map_err(engine_error)?.to_string(),
                [r.X as f64, r.Y as f64, r.Width as f64, r.Height as f64],
            ));
        }
        out.push(raw);
    }
    Ok((out, used))
}
