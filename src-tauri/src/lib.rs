mod apply;
mod detect;
mod events;
mod naming;
mod scan;

use apply::apply_renames;
use scan::{cancel_scan, scan_folder, ScanState};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder);
    });

    rx.recv()
        .ok()
        .flatten()
        .map(|path| path.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ScanState::default())
        .invoke_handler(tauri::generate_handler![
            pick_folder,
            scan_folder,
            cancel_scan,
            apply_renames
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
