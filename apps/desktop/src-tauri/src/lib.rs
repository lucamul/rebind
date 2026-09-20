use rebind_core::{recover, Block, ConflictKind, PageBreakGuess, PdfSource};
use serde::Serialize;

/// One recovered paragraph, in final reading order — page breaks
/// already resolved (merged away where confident, left as separate
/// blocks otherwise). This is `recover()`'s output made visible.
#[derive(Debug, Serialize)]
struct BlockView {
    id: u64,
    text: String,
    italic: bool,
    bold: bool,
}

/// A page-break boundary the recovery pass wasn't confident enough to
/// resolve silently. No way to actually resolve one from the UI yet —
/// this is just making the conflict visible, the same way `BlockView`
/// made paragraph recovery visible before it.
#[derive(Debug, Serialize)]
struct ConflictView {
    id: u64,
    guess: String,
    confidence: f32,
    before_block: u64,
    after_block: u64,
}

#[derive(Debug, Serialize)]
struct PdfInspection {
    page_count: usize,
    blocks: Vec<BlockView>,
    conflicts: Vec<ConflictView>,
}

#[tauri::command]
fn open_pdf(path: String) -> Result<PdfInspection, String> {
    let source = PdfSource::bind().map_err(|e| e.to_string())?;
    let raw_pages = source.load(&path).map_err(|e| e.to_string())?;
    let page_count = raw_pages.len();

    let recovered = recover(&raw_pages);

    let blocks = recovered
        .document
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Paragraph { id, spans } => Some(BlockView {
                id: *id,
                text: spans[0].text.clone(),
                italic: spans[0].italic,
                bold: spans[0].bold,
            }),
            // recover() doesn't produce any other Block variant yet.
            _ => None,
        })
        .collect();

    let conflicts = recovered
        .conflicts
        .iter()
        .filter_map(|c| match c.kind {
            ConflictKind::PageBreak {
                before,
                after,
                guess,
                confidence,
            } => Some(ConflictView {
                id: c.id,
                guess: match guess {
                    PageBreakGuess::Continues => "continues",
                    PageBreakGuess::NewSection => "new_section",
                    PageBreakGuess::Unknown => "unknown",
                }
                .to_string(),
                confidence,
                before_block: before,
                after_block: after,
            }),
            // recover() only ever produces PageBreak conflicts today.
            _ => None,
        })
        .collect();

    Ok(PdfInspection {
        page_count,
        blocks,
        conflicts,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![open_pdf])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
