//! Turns a whole document's pages into a [`Document`] of [`Block`]s,
//! by walking `layout::paragraphs_for_page` page by page and deciding,
//! at every page boundary, whether the paragraph before it continues
//! into the paragraph after it or genuinely ends there.
//!
//! This is where [`Conflict`]s actually get created: any boundary the
//! classifier isn't confident about becomes a `Conflict::PageBreak`
//! rather than a silent guess in either direction. Boundaries *within*
//! a page never need this — `layout::paragraphs_for_page` already
//! resolved those from the gap between lines, with no ambiguity left
//! for this pass to weigh in on.

use crate::conflict::{Conflict, ConflictKind, PageBreakGuess};
use crate::document::{Block, BlockId, Document, Span};
use crate::ingest::RawPage;
use crate::layout::{paragraphs_for_page, Line, Paragraph};

/// A boundary is only resolved silently at or above this confidence;
/// below it, the boundary gets a [`Conflict`] instead, carrying
/// whatever lean the classifier has as a pre-selected — but not
/// applied — default for the review UI.
const CONFIDENCE_THRESHOLD: f32 = 0.75;

pub struct Recovered {
    pub document: Document,
    pub conflicts: Vec<Conflict>,
}

pub fn recover(pages: &[RawPage]) -> Recovered {
    let mut all_paragraphs: Vec<(usize, Paragraph)> = Vec::new();
    for page in pages {
        for p in paragraphs_for_page(page) {
            all_paragraphs.push((page.index, p));
        }
    }

    let mut document = Document::new();
    let mut conflicts = Vec::new();

    let Some((first_page, first_paragraph)) = all_paragraphs.first() else {
        return Recovered {
            document,
            conflicts,
        };
    };

    let mut next_id: BlockId = 1;
    let mut current_lines = first_paragraph.lines.clone();
    let mut current_page = *first_page;
    let mut current_id = next_id;
    next_id += 1;

    for (page_index, paragraph) in &all_paragraphs[1..] {
        let crosses_a_page = *page_index != current_page;

        if crosses_a_page {
            let prev = Paragraph {
                lines: current_lines.clone(),
            };
            let (guess, confidence) = classify_page_break(&prev, paragraph);

            if guess == PageBreakGuess::Continues && confidence >= CONFIDENCE_THRESHOLD {
                current_lines.extend(paragraph.lines.clone());
                current_page = *page_index;
                continue;
            }

            let finished_id = current_id;
            push_paragraph_block(&mut document, finished_id, current_lines);

            current_id = next_id;
            next_id += 1;

            if confidence < CONFIDENCE_THRESHOLD {
                conflicts.push(Conflict {
                    id: conflicts.len() as u64 + 1,
                    kind: ConflictKind::PageBreak {
                        before: finished_id,
                        after: current_id,
                        guess,
                        confidence,
                    },
                });
            }

            current_lines = paragraph.lines.clone();
            current_page = *page_index;
        } else {
            // A new paragraph on the same page: layout.rs already
            // decided this from the line gap. Nothing to classify.
            push_paragraph_block(&mut document, current_id, current_lines);
            current_id = next_id;
            next_id += 1;
            current_lines = paragraph.lines.clone();
        }
    }
    push_paragraph_block(&mut document, current_id, current_lines);

    Recovered {
        document,
        conflicts,
    }
}

fn push_paragraph_block(document: &mut Document, id: BlockId, lines: Vec<Line>) {
    let italic = !lines.is_empty() && lines.iter().all(|l| l.is_italic());
    let bold = !lines.is_empty() && lines.iter().all(|l| l.is_bold());
    let text = Paragraph { lines }.text();
    document.push(Block::Paragraph {
        id,
        spans: vec![Span { text, italic, bold }],
    });
}

/// The actual judgment call. Two signals are strong enough to decide
/// silently; anything else is a weak, genuinely ambiguous case that
/// should become a [`Conflict`], not a guess dressed up as certainty.
fn classify_page_break(prev: &Paragraph, next: &Paragraph) -> (PageBreakGuess, f32) {
    let (Some(prev_line), Some(next_line)) = (prev.last_line(), next.lines.first()) else {
        return (PageBreakGuess::Unknown, 0.0);
    };

    // Strongest continuation signal: a sentence can't start with a
    // lowercase letter in standard prose, so if the next page's first
    // word is lowercase, this has to be a mid-sentence wrap.
    if next_line
        .text
        .trim_start()
        .chars()
        .next()
        .is_some_and(char::is_lowercase)
    {
        return (PageBreakGuess::Continues, 0.95);
    }

    // Strongest new-section signal: a clearly larger font opening the
    // next page reads as a heading, not a continuation.
    if prev_line.font_size > 0.0 && next_line.font_size / prev_line.font_size >= 1.25 {
        return (PageBreakGuess::NewSection, 0.9);
    }

    // Everything else is a weak lean, not a decision: the previous
    // page ending mid-thought with no terminal punctuation nudges
    // toward "continues"; ending with terminal punctuation and the
    // next page starting uppercase is consistent with either a new
    // paragraph in the same section *or* a new section with no
    // heading — genuinely can't tell those apart from geometry alone.
    let ends_with_terminal_punctuation = prev_line
        .text
        .trim_end()
        .chars()
        .next_back()
        .is_some_and(|c| matches!(c, '.' | '!' | '?' | '”' | '"'));
    let starts_uppercase = next_line
        .text
        .trim_start()
        .chars()
        .next()
        .is_some_and(char::is_uppercase);

    match (ends_with_terminal_punctuation, starts_uppercase) {
        (false, _) => (PageBreakGuess::Continues, 0.55),
        (true, true) => (PageBreakGuess::Unknown, 0.5),
        (true, false) => (PageBreakGuess::Unknown, 0.3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::RawChar;

    fn line(text: &str, font_size: f32) -> Line {
        Line {
            text: text.to_string(),
            left: 0.0,
            right: 100.0,
            top: 100.0,
            bottom: 90.0,
            font_name: "Test".into(),
            font_size,
        }
    }

    fn paragraph(text: &str, font_size: f32) -> Paragraph {
        Paragraph {
            lines: vec![line(text, font_size)],
        }
    }

    #[test]
    fn a_lowercase_continuation_is_a_confident_continues() {
        let prev = paragraph("the sentence trails off with no punctuation", 12.0);
        let next = paragraph("and picks back up here, still lowercase", 12.0);

        let (guess, confidence) = classify_page_break(&prev, &next);
        assert_eq!(guess, PageBreakGuess::Continues);
        assert!(confidence >= CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn a_much_larger_font_reads_as_a_confident_new_section() {
        let prev = paragraph("The chapter ends here.", 12.0);
        let next = paragraph("Chapter Two", 18.0);

        let (guess, confidence) = classify_page_break(&prev, &next);
        assert_eq!(guess, PageBreakGuess::NewSection);
        assert!(confidence >= CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn a_clean_sentence_end_into_a_same_size_capital_is_a_low_confidence_unknown() {
        let prev = paragraph("The chapter ends here.", 12.0);
        let next = paragraph("Something else starts, same size, same font.", 12.0);

        let (guess, confidence) = classify_page_break(&prev, &next);
        assert_eq!(guess, PageBreakGuess::Unknown);
        assert!(confidence < CONFIDENCE_THRESHOLD);
    }

    fn char_at(
        text: char,
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
        font_size: f32,
    ) -> RawChar {
        RawChar {
            text,
            left,
            right,
            top,
            bottom,
            font_name: "Test".into(),
            font_size,
        }
    }

    fn page_of_text(index: usize, text: &str, font_size: f32) -> RawPage {
        let mut chars = Vec::new();
        let mut x = 0.0;
        for c in text.chars() {
            chars.push(char_at(c, 100.0, 90.0, x, x + 10.0, font_size));
            x += 10.0;
        }
        RawPage {
            index,
            width: 400.0,
            height: 600.0,
            chars,
        }
    }

    #[test]
    fn merges_a_paragraph_that_continues_across_a_page_with_no_conflict() {
        let pages = vec![
            page_of_text(0, "this trails off with no ending", 12.0),
            page_of_text(1, "and continues, lowercase, right here", 12.0),
        ];

        let recovered = recover(&pages);
        assert_eq!(recovered.conflicts.len(), 0);
        assert_eq!(recovered.document.blocks.len(), 1);
        match &recovered.document.blocks[0] {
            Block::Paragraph { spans, .. } => {
                assert!(spans[0].text.contains("trails off"));
                assert!(spans[0].text.contains("continues, lowercase"));
            }
            other => panic!("expected a Paragraph block, got {other:?}"),
        }
    }

    #[test]
    fn flags_a_genuinely_ambiguous_boundary_as_a_conflict_instead_of_guessing() {
        let pages = vec![
            page_of_text(0, "The chapter ends here.", 12.0),
            page_of_text(1, "Something else starts here.", 12.0),
        ];

        let recovered = recover(&pages);
        assert_eq!(recovered.conflicts.len(), 1);
        assert_eq!(recovered.document.blocks.len(), 2);
        match &recovered.conflicts[0].kind {
            ConflictKind::PageBreak { confidence, .. } => {
                assert!(*confidence < CONFIDENCE_THRESHOLD);
            }
            other => panic!("expected a PageBreak conflict, got {other:?}"),
        }
    }
}
