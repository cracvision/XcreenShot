//! The real engine on images with text drawn by the app's own text engine (SMT-003,
//! SMT-004). They need a recognition language installed; without one they say so and
//! pass (a computer without any is not this crate's fault).

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use image::RgbaImage;
use serde_json::{json, Map, Value};
use xs_core::document::{Document, Object};
use xs_image::render::{render, Base};

/// An image of `width` × `height` with each text at its place, in black or its color.
fn page(width: u32, height: u32, texts: &[(f64, f64, &str, f64, &str)]) -> RgbaImage {
    let mut doc = Document::new(width, height, 96.0);
    for (i, (x, y, text, size, color)) in texts.iter().enumerate() {
        let props: Map<String, Value> = json!({
            "at": [x, y],
            "width": null,
            "text": text,
            "rotation": 0,
            "style": {"font": "Segoe UI", "size": size, "bold": false, "italic": false,
                      "color": color, "align": "left", "opacity": 100}
        })
        .as_object()
        .unwrap()
        .clone();
        doc.objects.push(Object {
            id: format!("o{i}"),
            kind: "text".into(),
            props,
        });
    }
    let base = Base {
        width,
        height,
        rgba: Arc::new(vec![255; (width * height * 4) as usize]),
        images: Arc::new(Default::default()),
    };
    render(&doc, &base)
}

fn available() -> bool {
    let langs = xs_ocr::languages();
    if langs.is_empty() {
        eprintln!("no recognition language on this computer: skipped");
    }
    !langs.is_empty()
}

#[test]
fn reads_spanish_text_with_its_signs() {
    if !available() {
        return;
    }
    let img = page(
        900,
        200,
        &[
            (20.0, 20.0, "¿Dónde está el señor Muñoz?", 18.0, "#000000"),
            (20.0, 80.0, "La canción del año: ¡qué éxito!", 18.0, "#C00000"),
        ],
    );
    let langs: Vec<String> = xs_ocr::languages()
        .into_iter()
        .filter(|l| l.tag.starts_with("es"))
        .map(|l| l.tag)
        .collect();
    let r = xs_ocr::recognize(&img, &langs, &AtomicBool::new(false)).unwrap();
    let text = xs_ocr::plain(&r.blocks, false);
    assert!(text.contains("Dónde"), "{text}");
    assert!(text.contains("Muñoz"), "{text}");
    assert!(text.contains("canción"), "{text}");
    // The red line comes back red.
    let red = r
        .blocks
        .iter()
        .flat_map(|b| &b.lines)
        .find(|l| l.text.contains("canción"))
        .unwrap();
    let rgb = u32::from_str_radix(red.color.trim_start_matches('#'), 16).unwrap();
    assert!((rgb >> 16) > 150 && (rgb >> 8 & 0xFF) < 90, "{}", red.color);
}

#[test]
fn reads_light_text_on_dark_and_small_interface_text() {
    if !available() {
        return;
    }
    let mut img = page(
        700,
        120,
        &[(10.0, 10.0, "Archivo  Edición  Ver  Ayuda", 9.0, "#FFFFFF")],
    );
    // A dark interface: the background goes dark, the text stays light.
    for p in img.pixels_mut() {
        if p.0[0] > 200 && p.0[1] > 200 && p.0[2] > 200 && p.0 == [255, 255, 255, 255] {
            *p = image::Rgba([30, 30, 36, 255]);
        }
    }
    let img = {
        // Text drawn white on white earlier: draw it again white on the dark ground.
        let mut doc_img = img;
        let white = page(
            700,
            120,
            &[(10.0, 10.0, "Archivo  Edición  Ver  Ayuda", 9.0, "#000000")],
        );
        for (d, w) in doc_img.pixels_mut().zip(white.pixels()) {
            if w.0[0] < 128 {
                let v = 255 - w.0[0];
                *d = image::Rgba([v.max(30), v.max(30), v.max(36), 255]);
            }
        }
        doc_img
    };
    let spanish: Vec<String> = xs_ocr::languages()
        .into_iter()
        .filter(|l| l.tag.starts_with("es"))
        .map(|l| l.tag)
        .collect();
    let r = xs_ocr::recognize(&img, &spanish, &AtomicBool::new(false)).unwrap();
    let text = xs_ocr::plain(&r.blocks, true).to_lowercase();
    assert!(text.contains("archivo"), "{text}");
    assert!(text.contains("ayuda"), "{text}");
}

/// SMT-003 criterion: a tall capture (20,000 px) with 400 numbered lines gives the 400
/// lines, in order, without repeating any at the joins.
#[test]
#[ignore = "slow: about 10 s; run with --ignored"]
fn a_tall_capture_gives_every_line_once_and_in_order() {
    if !available() {
        return;
    }
    let lines: Vec<String> = (1..=400).map(|i| format!("Linea numero {i:03}")).collect();
    let texts: Vec<(f64, f64, &str, f64, &str)> = lines
        .iter()
        .enumerate()
        .map(|(i, t)| (20.0, 10.0 + i as f64 * 50.0, t.as_str(), 14.0, "#000000"))
        .collect();
    let img = page(500, 20_000, &texts);
    let r = xs_ocr::recognize(&img, &[], &AtomicBool::new(false)).unwrap();
    let found: Vec<u32> = xs_ocr::plain(&r.blocks, false)
        .lines()
        .filter_map(|l| l.split_whitespace().last()?.parse().ok())
        .collect();
    assert_eq!(found, (1..=400).collect::<Vec<u32>>());
}

/// SMT-003 criterion: a 1920 x 1080 page of text in under 1 s, from the second use.
#[test]
#[ignore = "timing: run with --ignored on the reference computer"]
fn a_full_hd_page_reads_in_under_a_second() {
    if !available() {
        return;
    }
    let lines: Vec<String> = (1..=40)
        .map(|i| format!("Esta es la línea {i} de una página de prueba con texto normal de interfaz"))
        .collect();
    let texts: Vec<(f64, f64, &str, f64, &str)> = lines
        .iter()
        .enumerate()
        .map(|(i, t)| (40.0, 20.0 + i as f64 * 26.0, t.as_str(), 11.0, "#000000"))
        .collect();
    let img = page(1920, 1080, &texts);
    let cancel = AtomicBool::new(false);
    xs_ocr::recognize(&img, &[], &cancel).unwrap();
    let started = std::time::Instant::now();
    let r = xs_ocr::recognize(&img, &[], &cancel).unwrap();
    let took = started.elapsed();
    eprintln!("1920x1080: {} words in {took:?}", r.words());
    assert!(r.words() > 400);
    assert!(took < std::time::Duration::from_secs(1), "{took:?}");
}
