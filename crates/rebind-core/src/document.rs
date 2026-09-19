//! The recovered document model: a flat, ordered list of blocks with
//! enough structure to compile to either print (page-aware) or reflow
//! (EPUB) output, without needing to re-derive anything from the source
//! PDF. This is the thing a human's conflict resolutions ultimately
//! shape — the "clean source" the rest of the pipeline builds from.

use serde::{Deserialize, Serialize};

/// A stable id for a block, so conflicts and resolutions can reference
/// a specific block even as the surrounding document changes.
pub type BlockId = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Block {
    /// A structural division: front matter, a Part, a Chapter. `level`
    /// mirrors the print hierarchy (1 = Part, 2 = Chapter, ...).
    Heading { id: BlockId, level: u8, text: String },
    /// Ordinary reflowable prose.
    Paragraph { id: BlockId, spans: Vec<Span> },
    /// A verse/poem line — centered, never reflowed with neighbors.
    VerseLine { id: BlockId, spans: Vec<Span> },
    /// A blank line / stanza break within verse.
    VerseBreak { id: BlockId },
    /// A deliberate scene break within prose (e.g. "* * *").
    SceneBreak { id: BlockId },
}

impl Block {
    pub fn id(&self) -> BlockId {
        match self {
            Block::Heading { id, .. }
            | Block::Paragraph { id, .. }
            | Block::VerseLine { id, .. }
            | Block::VerseBreak { id }
            | Block::SceneBreak { id } => *id,
        }
    }
}

/// A run of text with uniform formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    pub italic: bool,
    pub bold: bool,
}

impl Span {
    pub fn plain(text: impl Into<String>) -> Self {
        Self { text: text.into(), italic: false, bold: false }
    }
}

/// A recovered document: an ordered sequence of blocks. Conflicts live
/// separately (see [`crate::conflict`]) so a document can be inspected
/// or rendered at any point in the review process, fully resolved or
/// not.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Document {
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn block(&self, id: BlockId) -> Option<&Block> {
        self.blocks.iter().find(|b| b.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_holds_blocks_in_order() {
        let mut doc = Document::new();
        doc.push(Block::Heading { id: 1, level: 1, text: "One".into() });
        doc.push(Block::Paragraph { id: 2, spans: vec![Span::plain("Hello.")] });

        assert_eq!(doc.blocks.len(), 2);
        assert_eq!(doc.blocks[0].id(), 1);
    }

    #[test]
    fn block_looks_itself_up_by_id() {
        let mut doc = Document::new();
        doc.push(Block::SceneBreak { id: 7 });

        assert!(matches!(doc.block(7), Some(Block::SceneBreak { .. })));
        assert!(doc.block(99).is_none());
    }
}
