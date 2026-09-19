use rebind_core::PdfSource;
use serde::Serialize;

/// What the review UI shows immediately after opening a PDF, before any
/// structure recovery has run — just proof the file loaded and a sense
/// of its shape.
#[derive(Debug, Serialize)]
struct PdfSummary {
    page_count: usize,
    first_page_width: f32,
    first_page_height: f32,
    first_page_char_count: usize,
}

#[tauri::command]
fn open_pdf(path: String) -> Result<PdfSummary, String> {
    let source = PdfSource::bind().map_err(|e| e.to_string())?;
    let pages = source.load(&path).map_err(|e| e.to_string())?;
    let first = pages.first().ok_or("PDF has no pages")?;

    Ok(PdfSummary {
        page_count: pages.len(),
        first_page_width: first.width,
        first_page_height: first.height,
        first_page_char_count: first.chars.len(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
