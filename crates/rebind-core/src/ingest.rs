//! Raw PDF ingestion: PDFium binding plus per-page text and geometry
//! extraction. This stays deliberately low-level — turning [`RawPage`]s
//! into a [`crate::Document`] (paragraphs, headings, conflicts) is a
//! later, separate pass. Nothing here decides what anything *means*,
//! only what's on the page and where.

use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("could not load the PDFium library: {0}")]
    PdfiumBinding(String),
    #[error("could not open PDF: {0}")]
    OpenDocument(String),
}

/// One glyph with its position on the page, carried through so a later
/// pass can reconstruct paragraphs, detect repeated headers/footers by
/// position, and infer style from the font name — none of which is
/// decided here.
#[derive(Debug, Clone)]
pub struct RawChar {
    pub text: char,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub font_name: String,
    pub font_size: f32,
}

#[derive(Debug, Clone)]
pub struct RawPage {
    pub index: usize,
    pub width: f32,
    pub height: f32,
    pub chars: Vec<RawChar>,
}

pub struct PdfSource {
    pdfium: &'static Pdfium,
}

impl PdfSource {
    /// Binds to the PDFium library. Looks for a vendored copy next to
    /// the workspace first (see `scripts/fetch-pdfium.sh`), then falls
    /// back to whatever's on the system.
    ///
    /// The actual bind only ever happens once per process, behind a
    /// `OnceLock` — every caller (each test in a parallel test binary,
    /// every command the desktop app handles) shares one `Pdfium`
    /// instance. `Pdfium` is `Send + Sync` but not `Clone`, and more
    /// importantly, the native library itself isn't guaranteed to
    /// tolerate concurrent initialization from multiple threads; doing
    /// the whole bind inside `get_or_init` — not just caching its
    /// result — is what actually serializes that, not an afterthought.
    pub fn bind() -> Result<Self, IngestError> {
        static PDFIUM: OnceLock<Result<Pdfium, String>> = OnceLock::new();

        let result = PDFIUM.get_or_init(|| {
            let bindings = Self::vendored_library_path()
                .and_then(|path| Pdfium::bind_to_library(path).ok())
                .or_else(|| Pdfium::bind_to_system_library().ok())
                .ok_or_else(|| {
                    "no PDFium library found (run scripts/fetch-pdfium.sh)".to_string()
                })?;
            Ok(Pdfium::new(bindings))
        });

        match result {
            Ok(pdfium) => Ok(Self { pdfium }),
            Err(e) => Err(IngestError::PdfiumBinding(e.clone())),
        }
    }

    fn vendored_library_path() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var("PDFIUM_LIB_DIR") {
            return Some(Pdfium::pdfium_platform_library_name_at_path(&dir));
        }
        // crates/rebind-core -> workspace root -> vendor/pdfium/lib
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let candidate = manifest_dir.join("../../vendor/pdfium/lib");
        candidate
            .is_dir()
            .then(|| Pdfium::pdfium_platform_library_name_at_path(&candidate))
    }

    /// Loads every page of the PDF at `path` into [`RawPage`]s.
    pub fn load(&self, path: impl AsRef<Path>) -> Result<Vec<RawPage>, IngestError> {
        let document = self
            .pdfium
            .load_pdf_from_file(path.as_ref(), None)
            .map_err(|e| IngestError::OpenDocument(e.to_string()))?;

        let mut pages = Vec::new();
        for (index, page) in document.pages().iter().enumerate() {
            let width = page.width().value;
            let height = page.height().value;
            let text = page
                .text()
                .map_err(|e| IngestError::OpenDocument(e.to_string()))?;

            let chars = text
                .chars()
                .iter()
                .filter_map(|c| {
                    let bounds = c.tight_bounds().ok()?;
                    Some(RawChar {
                        text: c.unicode_char()?,
                        left: bounds.left().value,
                        top: bounds.top().value,
                        right: bounds.right().value,
                        bottom: bounds.bottom().value,
                        font_name: c.font_name(),
                        font_size: c.unscaled_font_size().value,
                    })
                })
                .collect();

            pages.push(RawPage {
                index,
                width,
                height,
                chars,
            });
        }
        Ok(pages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindable() -> bool {
        PdfSource::vendored_library_path().is_some()
    }

    #[test]
    fn binds_to_the_vendored_library_when_present() {
        if !bindable() {
            eprintln!("skipping: run scripts/fetch-pdfium.sh first");
            return;
        }
        assert!(PdfSource::bind().is_ok());
    }
}
