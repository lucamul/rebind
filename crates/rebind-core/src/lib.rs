//! rebind-core: the structure-recovery engine.
//!
//! Takes a fixed-layout PDF and reconstructs a reflowable, structured
//! document — headings, paragraphs, verse — flagging every decision it
//! isn't confident about as a [`Conflict`] for a human to resolve,
//! rather than guessing silently. See `conflict` for why that split
//! exists and what it costs to get wrong.
//!
//! This crate is UI-agnostic: the desktop app (`apps/desktop`) is the
//! only thing that knows about windows, clicks, or rendering. Nothing
//! here reaches outside "PDF bytes in, `Document` + `Conflict`s out".

pub mod conflict;
pub mod document;
pub mod ingest;

pub use conflict::{Choice, Conflict, ConflictKind, PageBreakGuess, Resolution};
pub use document::{Block, BlockId, Document, Span};
pub use ingest::{IngestError, PdfSource, RawChar, RawPage};
