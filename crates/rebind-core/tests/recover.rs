//! The whole point of the project, tested against the real fixture:
//! a paragraph that overflows a page mid-sentence should come back as
//! one continuous block, indistinguishable from a paragraph that
//! never crossed a page at all, with the page break itself invisible.
//! A paragraph that opens a deliberate new chapter should not.
//!
//! See `tests/ingest.rs` for the fixture's exact page-by-page shape.

use rebind_core::{recover, Block, ConflictKind};
use std::path::PathBuf;

fn fixture_pages() -> Vec<rebind_core::RawPage> {
    let src = rebind_core::PdfSource::bind().expect("bind pdfium");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    src.load(&path).expect("load fixture")
}

fn skip_without_pdfium() -> bool {
    if rebind_core::PdfSource::bind().is_err() {
        eprintln!("skipping: no PDFium library (run scripts/fetch-pdfium.sh)");
        return true;
    }
    false
}

#[test]
fn the_paragraph_that_overflows_a_page_merges_into_one_seamless_block() {
    if skip_without_pdfium() {
        return;
    }
    let pages = fixture_pages();
    let recovered = recover(&pages);

    let merged = recovered
        .document
        .blocks
        .iter()
        .find_map(|b| match b {
            Block::Paragraph { spans, .. } if spans[0].text.contains("deliberately long") => {
                Some(spans[0].text.clone())
            }
            _ => None,
        })
        .expect("the long paragraph should exist as a single block");

    // Both halves present, and nothing marking where the page broke —
    // reads as one sentence, not two concatenated fragments.
    assert!(merged.contains("recognizing that nothing deliberate"));
    assert!(merged.contains("happened here, that the words before"));
    assert!(merged.ends_with("on purpose."));

    // No stray double space or other seam at the join point.
    assert!(!merged.contains("  "));
}

#[test]
fn the_deliberate_new_chapter_does_not_merge_with_what_precedes_it() {
    if skip_without_pdfium() {
        return;
    }
    let pages = fixture_pages();
    let recovered = recover(&pages);

    let chapter_two_text = recovered.document.blocks.iter().find_map(|b| match b {
        Block::Paragraph { spans, .. } if spans[0].text.contains("Chapter Two") => {
            Some(spans[0].text.clone())
        }
        _ => None,
    });

    let chapter_two_text = chapter_two_text.expect("Chapter Two should exist as its own block");
    assert!(
        !chapter_two_text.contains("short paragraph"),
        "should not have merged with page 2's last paragraph"
    );
}

#[test]
fn both_ground_truth_boundaries_resolve_with_no_conflicts() {
    if skip_without_pdfium() {
        return;
    }
    let pages = fixture_pages();
    let recovered = recover(&pages);

    // This fixture was built specifically so both page boundaries have
    // a strong, unambiguous signal (lowercase continuation; a clearly
    // larger heading font) — neither should need a human's input.
    let page_break_conflicts: Vec<_> = recovered
        .conflicts
        .iter()
        .filter(|c| matches!(c.kind, ConflictKind::PageBreak { .. }))
        .collect();
    assert_eq!(
        page_break_conflicts.len(),
        0,
        "expected both fixture boundaries to be confidently classified, got {page_break_conflicts:#?}"
    );
}
