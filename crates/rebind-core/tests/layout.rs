//! Paragraph-grouping tested against the real fixture (see
//! `tests/ingest.rs` for the page-by-page shape). Headings are
//! deliberately *not* split from the paragraph that follows them here
//! — font-size-based heading detection is a separate pass; grouping
//! only reasons about line-to-line gaps.

use rebind_core::paragraphs_for_page;

fn skip_without_pdfium() -> Option<rebind_core::PdfSource> {
    match rebind_core::PdfSource::bind() {
        Ok(src) => Some(src),
        Err(_) => {
            eprintln!("skipping: no PDFium library (run scripts/fetch-pdfium.sh)");
            None
        }
    }
}

fn fixture_pages() -> Vec<rebind_core::RawPage> {
    let src = rebind_core::PdfSource::bind().expect("bind pdfium");
    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    src.load(&path).expect("load fixture")
}

#[test]
fn page_one_groups_into_four_paragraphs() {
    let Some(_src) = skip_without_pdfium() else {
        return;
    };
    let pages = fixture_pages();

    let paragraphs = paragraphs_for_page(&pages[0]);
    assert_eq!(
        paragraphs.len(),
        4,
        "{:#?}",
        paragraphs.iter().map(|p| p.text()).collect::<Vec<_>>()
    );

    // The heading's gap to the paragraph below it isn't wide enough to
    // register as its own break on this fixture — it rides along with
    // paragraph 0. That's expected (see module docs), not a bug.
    assert!(paragraphs[0].text().contains("Chapter One"));
    assert!(paragraphs[0].text().contains("first paragraph"));

    assert!(paragraphs[1].text().contains("second paragraph"));

    assert!(paragraphs[2].text().contains("italic"));
    assert!(
        paragraphs[2].lines.iter().all(|l| l.is_italic()),
        "every line of the emphasized paragraph should read as italic"
    );

    let last = paragraphs[3].last_line().expect("paragraph has lines");
    assert!(
        last.text.trim_end().ends_with("deliberate"),
        "expected the long paragraph to end mid-sentence, got {:?}",
        last.text
    );
}

#[test]
fn page_two_has_no_heading_and_two_paragraphs() {
    let Some(_src) = skip_without_pdfium() else {
        return;
    };
    let pages = fixture_pages();

    let paragraphs = paragraphs_for_page(&pages[1]);
    assert_eq!(paragraphs.len(), 2);
    assert!(paragraphs[0].text().starts_with("happened here"));
    assert!(paragraphs[1].text().contains("short paragraph"));
}

#[test]
fn page_three_groups_the_heading_the_poem_title_and_the_verse_separately() {
    let Some(_src) = skip_without_pdfium() else {
        return;
    };
    let pages = fixture_pages();

    let paragraphs = paragraphs_for_page(&pages[2]);
    assert_eq!(
        paragraphs.len(),
        3,
        "{:#?}",
        paragraphs.iter().map(|p| p.text()).collect::<Vec<_>>()
    );

    assert!(paragraphs[0].text().contains("Chapter Two"));
    assert!(paragraphs[0].text().contains("deliberate new chapter"));

    assert_eq!(paragraphs[1].text(), "A Short Poem");
    assert!(paragraphs[1].lines[0].is_bold());

    assert!(paragraphs[2].text().contains("Line one of the poem"));
    assert!(paragraphs[2].lines.iter().all(|l| l.is_italic()));
}
