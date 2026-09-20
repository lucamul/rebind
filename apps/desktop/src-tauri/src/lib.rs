use rebind_core::{paragraphs_for_page, PdfSource};
use serde::Serialize;

/// One recovered paragraph, as the UI shows it — flattened text plus
/// whatever style is uniform across the whole paragraph. This is the
/// `layout` module's output made visible; the review UI itself
/// (conflicts, resolutions) doesn't exist yet.
#[derive(Debug, Serialize)]
struct ParagraphView {
    text: String,
    italic: bool,
    bold: bool,
}

#[derive(Debug, Serialize)]
struct PageView {
    index: usize,
    paragraphs: Vec<ParagraphView>,
}

#[derive(Debug, Serialize)]
struct PdfInspection {
    page_count: usize,
    pages: Vec<PageView>,
}

#[tauri::command]
fn open_pdf(path: String) -> Result<PdfInspection, String> {
    let source = PdfSource::bind().map_err(|e| e.to_string())?;
    let raw_pages = source.load(&path).map_err(|e| e.to_string())?;

    let pages = raw_pages
        .iter()
        .map(|page| {
            let paragraphs = paragraphs_for_page(page)
                .into_iter()
                .map(|p| ParagraphView {
                    text: p.text(),
                    italic: !p.lines.is_empty() && p.lines.iter().all(|l| l.is_italic()),
                    bold: !p.lines.is_empty() && p.lines.iter().all(|l| l.is_bold()),
                })
                .collect();
            PageView {
                index: page.index,
                paragraphs,
            }
        })
        .collect();

    Ok(PdfInspection {
        page_count: raw_pages.len(),
        pages,
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
