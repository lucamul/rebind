use rebind_core::{Block, Document, Span};

/// Round-trips through rebind-core so the app<->core wiring is proven,
/// not just assumed. Delete once there's a real command to replace it.
#[tauri::command]
fn ping() -> String {
    let mut doc = Document::new();
    doc.push(Block::Heading { id: 1, level: 1, text: "rebind-core".into() });
    doc.push(Block::Paragraph { id: 2, spans: vec![Span::plain("core is reachable")] });
    format!("{} blocks from rebind-core, first: {:?}", doc.blocks.len(), doc.block(1))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![ping])
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
