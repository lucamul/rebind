//! Turns a page's [`RawChar`]s into [`Line`]s, then [`Line`]s into
//! [`Paragraph`]s — the structural inference layer underneath
//! everything else (page-break classification needs to know what the
//! last paragraph before a page boundary and the first paragraph
//! after it actually are).
//!
//! Two separate signals, used for two separate jobs:
//! - **Line breaks** come straight from the PDF's own text stream
//!   (PDFium emits a `\r\n` pair at every visual line wrap, both
//!   paragraph-ending and mid-paragraph). Reliable, no inference
//!   needed.
//! - **Paragraph breaks** are inferred from the *gap* between
//!   consecutive lines: an ordinary wrapped line sits close to its
//!   neighbor (empirically, on a real fixture, a tight band around a
//!   page's typical line spacing); a paragraph break's gap runs
//!   noticeably wider. This is a real heuristic, not a certainty —
//!   it's exactly the kind of judgment call later work should be
//!   willing to flag as a [`crate::Conflict`] when the gap sits in
//!   between, rather than silently picking a side.

use crate::ingest::{RawChar, RawPage};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub font_name: String,
    pub font_size: f32,
}

impl Line {
    pub fn is_italic(&self) -> bool {
        let lower = self.font_name.to_lowercase();
        lower.contains("italic") || lower.contains("oblique")
    }

    pub fn is_bold(&self) -> bool {
        self.font_name.to_lowercase().contains("bold")
    }
}

/// Splits a page's characters into lines on the PDF's own `\r\n`
/// line-break markers. Control characters (line breaks included, plus
/// any other non-printable glyphs PDFium occasionally reports, e.g.
/// ligature-break artifacts) never end up in a line's text.
pub fn lines_for_page(page: &RawPage) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut current: Vec<&RawChar> = Vec::new();

    for c in &page.chars {
        if c.text == '\n' {
            if !current.is_empty() {
                lines.push(build_line(&current));
                current.clear();
            }
            continue;
        }
        if c.text.is_control() {
            continue; // '\r' and any other stray control chars
        }
        current.push(c);
    }
    if !current.is_empty() {
        lines.push(build_line(&current));
    }
    lines
}

fn build_line(chars: &[&RawChar]) -> Line {
    // A wrapped line often carries the trailing space that used to
    // separate it from the next word before the PDF wrapped it;
    // untrimmed, joining lines with a space doubles it up.
    let text: String = chars
        .iter()
        .map(|c| c.text)
        .collect::<String>()
        .trim()
        .to_string();
    let left = chars.iter().map(|c| c.left).fold(f32::INFINITY, f32::min);
    let right = chars
        .iter()
        .map(|c| c.right)
        .fold(f32::NEG_INFINITY, f32::max);
    let top = chars
        .iter()
        .map(|c| c.top)
        .fold(f32::NEG_INFINITY, f32::max);
    let bottom = chars.iter().map(|c| c.bottom).fold(f32::INFINITY, f32::min);

    // The line's dominant font: whichever (name, size) covers the most
    // characters. Spaces often come back with no font info from
    // PDFium, so they don't get a vote.
    let mut votes: HashMap<(String, u32), usize> = HashMap::new();
    for c in chars {
        if c.font_name.is_empty() {
            continue;
        }
        *votes
            .entry((c.font_name.clone(), c.font_size.to_bits()))
            .or_insert(0) += 1;
    }
    let (font_name, font_size) = votes
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|((name, size_bits), _)| (name, f32::from_bits(size_bits)))
        .unwrap_or_default();

    Line {
        text,
        left,
        right,
        top,
        bottom,
        font_name,
        font_size,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub lines: Vec<Line>,
}

impl Paragraph {
    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The last line's own text, trimmed — what a page-break
    /// classifier actually needs to look at when this paragraph is
    /// the last thing on a page.
    pub fn last_line(&self) -> Option<&Line> {
        self.lines.last()
    }
}

/// A paragraph break is inferred wherever the gap before a line is
/// noticeably wider than the page's typical (median) line-to-line
/// gap. `1.6` sits comfortably in the gap this fixture's own numbers
/// leave open — real within-paragraph gaps land near 1.0x the median,
/// real paragraph gaps land above 2x — but it's a tunable heuristic,
/// not a law; expect to revisit it against messier real-world PDFs.
const PARAGRAPH_GAP_MULTIPLIER: f32 = 1.6;

pub fn paragraphs_for_page(page: &RawPage) -> Vec<Paragraph> {
    let lines = lines_for_page(page);
    if lines.is_empty() {
        return Vec::new();
    }

    let gaps: Vec<f32> = lines.windows(2).map(|w| w[0].bottom - w[1].top).collect();
    let threshold = median(&gaps).map(|m| m * PARAGRAPH_GAP_MULTIPLIER);

    let mut paragraphs: Vec<Paragraph> = vec![Paragraph {
        lines: vec![lines[0].clone()],
    }];
    for (i, line) in lines.into_iter().enumerate().skip(1) {
        let gap = gaps[i - 1];
        let is_new_paragraph = match threshold {
            Some(t) => gap > t,
            None => false,
        };
        if is_new_paragraph {
            paragraphs.push(Paragraph { lines: vec![line] });
        } else {
            paragraphs.last_mut().unwrap().lines.push(line);
        }
    }
    paragraphs
}

fn median(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(sorted[sorted.len() / 2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn char_at(text: char, top: f32, bottom: f32, left: f32, right: f32) -> RawChar {
        RawChar {
            text,
            left,
            right,
            top,
            bottom,
            font_name: "Test".into(),
            font_size: 12.0,
        }
    }

    fn line_break() -> [RawChar; 2] {
        [
            char_at('\r', 0.0, 0.0, 0.0, 0.0),
            char_at('\n', 0.0, 0.0, 0.0, 0.0),
        ]
    }

    #[test]
    fn groups_chars_into_lines_on_line_break_markers() {
        let mut chars = vec![
            char_at('h', 100.0, 90.0, 0.0, 10.0),
            char_at('i', 100.0, 90.0, 10.0, 15.0),
        ];
        chars.extend(line_break());
        chars.push(char_at('a', 80.0, 70.0, 0.0, 10.0));

        let page = RawPage {
            index: 0,
            width: 200.0,
            height: 300.0,
            chars,
        };
        let lines = lines_for_page(&page);

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "hi");
        assert_eq!(lines[1].text, "a");
    }

    #[test]
    fn a_wide_gap_starts_a_new_paragraph_a_narrow_one_does_not() {
        // Three lines at a tight ~5pt gap (one paragraph), then a
        // ~14pt gap (a new one) — the same ratio the real fixture
        // showed between wrapped lines and true paragraph breaks.
        let mut chars = Vec::new();
        for (text, top, bottom) in [('a', 100.0, 90.0), ('b', 85.0, 75.0), ('c', 70.0, 60.0)] {
            chars.push(char_at(text, top, bottom, 0.0, 10.0));
            chars.extend(line_break());
        }
        chars.push(char_at('d', 46.0, 36.0, 0.0, 10.0)); // gap of 14 from 'c's bottom (60)

        let page = RawPage {
            index: 0,
            width: 200.0,
            height: 300.0,
            chars,
        };
        let paragraphs = paragraphs_for_page(&page);

        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0].lines.len(), 3);
        assert_eq!(paragraphs[1].lines.len(), 1);
    }
}
