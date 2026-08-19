use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use walkdir::WalkDir;

use crate::detect::detect_extension;
use crate::events::Batcher;
use crate::naming::compute_suggested_name;

/// Shared across scans so a "cancel" click can reach the running rayon
/// pool. Only one scan runs at a time (the UI blocks rescanning while
/// scanning), so a single flag is enough.
#[derive(Default)]
pub struct ScanState {
    cancel_requested: Arc<AtomicBool>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mismatch {
    /// Absolute path, used as the identifier for later apply calls.
    pub path: String,
    /// Path relative to the scanned folder, for display.
    pub relative_path: String,
    pub current_extension: String,
    pub detected_extension: String,
    /// The filename this file would be renamed to. Provisional: computed
    /// against the current disk state at scan time, not re-checked
    /// against the rest of the batch. The authoritative rename target
    /// (with real conflict numbering) is only decided at apply time.
    pub suggested_name: String,
    /// Whether `suggested_name` currently collides with an existing file.
    pub has_conflict: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub total_scanned: usize,
    pub mismatches_found: usize,
    pub cancelled: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub scanned: usize,
}

/// How many mismatches to buffer before flushing a batch event - see
/// `events::Batcher` for why this matters.
const BATCH_SIZE: usize = 50;

fn to_mismatch(entry_path: &std::path::Path, root: &std::path::Path) -> Option<Mismatch> {
    let format = detect_extension(entry_path)?;

    let current_extension = entry_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if format.accepted.contains(&current_extension.as_str()) {
        return None;
    }

    let relative_path = entry_path
        .strip_prefix(root)
        .unwrap_or(entry_path)
        .to_string_lossy()
        .replace('\\', "/");

    let suggested_name = compute_suggested_name(entry_path, format.canonical);
    let has_conflict = entry_path
        .parent()
        .map(|parent| parent.join(&suggested_name).exists())
        .unwrap_or(false);

    Some(Mismatch {
        path: entry_path.to_string_lossy().to_string(),
        relative_path,
        current_extension,
        detected_extension: format.canonical.to_string(),
        suggested_name,
        has_conflict,
    })
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, ScanState>) {
    state.cancel_requested.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub async fn scan_folder(
    app: AppHandle,
    state: State<'_, ScanState>,
    folder: String,
) -> Result<ScanSummary, String> {
    let root = std::path::PathBuf::from(&folder);
    if !root.is_dir() {
        return Err(format!("{folder} is not a directory"));
    }

    state.cancel_requested.store(false, Ordering::Relaxed);
    let cancel_requested = state.cancel_requested.clone();

    // Only descend into the file types we recognize; anything else is
    // read as a plain filesystem walk (cheap - we stat, we don't open).
    let candidates: Vec<std::path::PathBuf> = WalkDir::new(&root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect();

    let scanned = AtomicUsize::new(0);
    let mismatches_found = AtomicUsize::new(0);
    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let batcher: Batcher<Mismatch> = Batcher::new(BATCH_SIZE);

    // Throttle progress events so huge folders don't flood the frontend.
    let progress_every = (candidates.len() / 200).max(1);

    let outcome = candidates.par_iter().try_for_each(|path| {
        if cancel_requested.load(Ordering::Relaxed) {
            return Err(());
        }

        if let Some(mismatch) = to_mismatch(path, &root) {
            mismatches_found.fetch_add(1, Ordering::Relaxed);

            if let Some(batch) = batcher.push(mismatch) {
                if let Err(err) = app.emit("scan:mismatches-found", &batch) {
                    errors.lock().unwrap().push(err.to_string());
                }
            }
        }

        let count = scanned.fetch_add(1, Ordering::Relaxed) + 1;
        if count % progress_every == 0 {
            let _ = app.emit("scan:progress", ScanProgress { scanned: count });
        }

        Ok(())
    });

    let remaining = batcher.flush();
    if !remaining.is_empty() {
        if let Err(err) = app.emit("scan:mismatches-found", &remaining) {
            errors.lock().unwrap().push(err.to_string());
        }
    }

    if let Some(err) = errors.lock().unwrap().first() {
        return Err(err.clone());
    }

    let final_scanned = scanned.load(Ordering::Relaxed);
    let _ = app.emit("scan:progress", ScanProgress { scanned: final_scanned });

    let summary = ScanSummary {
        total_scanned: final_scanned,
        mismatches_found: mismatches_found.load(Ordering::Relaxed),
        cancelled: outcome.is_err(),
    };

    // Emitted last, on the same event channel as the mismatch batches
    // above, so the frontend has a reliable "the stream is done" signal
    // to wait for instead of inferring it from this command's return
    // value (a different, unordered channel).
    let _ = app.emit("scan:complete", &summary);

    Ok(summary)
}
