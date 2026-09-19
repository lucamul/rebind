//! Integration test against a real (small, committed) PDF fixture —
//! not a mock. See `tests/fixtures/sample.pdf`, generated from
//! `sample.typ` via Typst: two pages, a heading, plain prose, an
//! italic line, a page break with no heading, and a centered poem.
//!
//! Skips (rather than fails) when no PDFium library is available,
//! since that's an environment-setup problem (run
//! `scripts/fetch-pdfium.sh`), not a code problem.

use rebind_core::PdfSource;
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf")
}

fn page_text(chars: &[rebind_core::RawChar]) -> String {
    chars.iter().map(|c| c.text).collect()
}

#[test]
fn loads_both_pages_with_correct_dimensions() {
    let Ok(src) = PdfSource::bind() else {
        eprintln!("skipping: no PDFium library (run scripts/fetch-pdfium.sh)");
        return;
    };
    let pages = src.load(fixture_path()).expect("load fixture");

    assert_eq!(pages.len(), 2);
    // 4in x 6in page, set in sample.typ, in points (72pt/in).
    assert!((pages[0].width - 288.0).abs() < 1.0);
    assert!((pages[0].height - 432.0).abs() < 1.0);
}

#[test]
fn extracts_body_text_in_reading_order() {
    let Ok(src) = PdfSource::bind() else {
        eprintln!("skipping: no PDFium library (run scripts/fetch-pdfium.sh)");
        return;
    };
    let pages = src.load(fixture_path()).expect("load fixture");

    let text = page_text(&pages[0].chars);
    assert!(text.contains("Chapter One"));
    assert!(text.contains("first paragraph"));
    assert!(text.contains("second paragraph"));
    // "first paragraph" must come before "second paragraph" — proves
    // chars come out in reading order, not e.g. z-order or object order.
    assert!(text.find("first paragraph").unwrap() < text.find("second paragraph").unwrap());
}

#[test]
fn distinguishes_italic_from_plain_text_by_font_name() {
    let Ok(src) = PdfSource::bind() else {
        eprintln!("skipping: no PDFium library (run scripts/fetch-pdfium.sh)");
        return;
    };
    let pages = src.load(fixture_path()).expect("load fixture");

    // Find the run of chars spelling "italic" (from the emphasized
    // line) and the run spelling "paragraph" (plain body text), and
    // check their font names actually differ — that's the real signal
    // a later pass uses for style detection, not just "some font
    // exists".
    let text = page_text(&pages[0].chars);
    // `find` returns a byte offset; convert to a char index so it maps
    // onto `chars` correctly regardless of any multi-byte characters
    // earlier in the page (smart quotes, etc.).
    let char_index_of = |byte_offset: usize| text[..byte_offset].chars().count();

    let italic_byte = text.find("italic").expect("fixture has the word 'italic'");
    let plain_byte = text
        .find("paragraph")
        .expect("fixture has the word 'paragraph'");

    let italic_font = &pages[0].chars[char_index_of(italic_byte)].font_name;
    let plain_font = &pages[0].chars[char_index_of(plain_byte)].font_name;

    assert!(!italic_font.is_empty());
    assert!(!plain_font.is_empty());
    assert_ne!(
        italic_font, plain_font,
        "expected the italic line's font to differ from the plain paragraph's font"
    );
}

#[test]
fn second_page_opens_mid_flow_with_no_heading() {
    let Ok(src) = PdfSource::bind() else {
        eprintln!("skipping: no PDFium library (run scripts/fetch-pdfium.sh)");
        return;
    };
    let pages = src.load(fixture_path()).expect("load fixture");

    let text = page_text(&pages[1].chars);
    assert!(text.contains("This paragraph opens the second page"));
    assert!(text.contains("A Short Poem"));
    assert!(text.contains("centered and italic"));
}
