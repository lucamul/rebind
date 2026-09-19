//! A [`Conflict`] is a decision the automatic recovery pass wasn't
//! confident enough to make silently. The whole point of rebind: get
//! the obvious 90% right automatically, and surface only genuine
//! ambiguity for a human to resolve — rather than guessing everything
//! and being wrong in ways nobody notices until the ebook ships.
//!
//! The desktop app renders these; resolving one produces a
//! [`Resolution`] that gets applied back onto the [`crate::Document`].

use crate::document::BlockId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictKind {
    /// A page boundary in the source PDF: is it a deliberate section
    /// break, or just where justified text ran out of room mid-thought?
    /// This is the single most consequential conflict kind — see the
    /// project README for why it can't be resolved from the PDF alone.
    PageBreak { before: BlockId, after: BlockId, guess: PageBreakGuess, confidence: f32 },

    /// A line that repeats across many pages at the same position:
    /// running header/footer (strip it) or genuine, repeated body text
    /// (keep it)?
    RepeatingLine { sample_text: String, seen_on_pages: usize, total_pages: usize },

    /// A span whose italic/bold status couldn't be read reliably from
    /// font metadata — subsetted/renamed fonts, or glyphs outlined with
    /// no text semantics left at all.
    AmbiguousStyle { block: BlockId, text: String },

    /// A line-end hyphen: rejoin ("exam-ple" -> "example") or keep as a
    /// genuinely hyphenated word ("self-esteem")?
    Hyphenation { block: BlockId, word_before: String, word_after: String },

    /// A block the automatic pass suspects is a heading, with an
    /// uncertain level (Part vs. Chapter vs. a plain emphasized line).
    HeadingLevel { block: BlockId, text: String, guess: u8 },
}

/// The automatic pass's best guess for a [`ConflictKind::PageBreak`],
/// carried along so the review UI can pre-select it and the user only
/// has to act on the ones it got wrong.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PageBreakGuess {
    /// The paragraph just continues on the next page.
    Continues,
    /// This looks like a deliberate new section (heading, poem, etc.).
    NewSection,
    /// No signal either way strong enough to guess.
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: u64,
    pub kind: ConflictKind,
}

/// The user's decision on one conflict, ready to apply back to the
/// document. Kept separate from `Conflict` itself so a resolution can
/// be logged, undone, or replayed independent of the conflict that
/// prompted it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub conflict_id: u64,
    pub choice: Choice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Choice {
    PageBreak(PageBreakGuess),
    /// `true` = strip as a running header/footer.
    KeepRepeatingLine(bool),
    Style { italic: bool, bold: bool },
    /// `true` = rejoin across the hyphen.
    Hyphenation(bool),
    HeadingLevel(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_break_conflict_carries_enough_context_to_render_without_the_source_pdf() {
        let c = Conflict {
            id: 1,
            kind: ConflictKind::PageBreak {
                before: 10,
                after: 11,
                guess: PageBreakGuess::Unknown,
                confidence: 0.4,
            },
        };

        match c.kind {
            ConflictKind::PageBreak { confidence, guess, .. } => {
                assert!(confidence < 0.5);
                assert_eq!(guess, PageBreakGuess::Unknown);
            }
            _ => panic!("wrong kind"),
        }
    }

    #[test]
    fn a_resolution_references_its_conflict_by_id_not_by_holding_it() {
        let r = Resolution { conflict_id: 42, choice: Choice::Hyphenation(true) };
        assert_eq!(r.conflict_id, 42);
    }
}
