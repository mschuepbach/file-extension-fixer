use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::events::Batcher;
use crate::naming::{compute_suggested_name, resolve_conflict};

const BATCH_SIZE: usize = 50;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameRequest {
    pub path: String,
    /// The detected canonical format (e.g. "jpg"), as previewed during
    /// scan. Apply recomputes the actual target itself rather than
    /// trusting a precomputed name from the frontend, so it can resolve
    /// conflicts authoritatively against live disk state and the rest of
    /// this batch.
    pub canonical_extension: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameOutcome {
    pub path: String,
    pub new_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySummary {
    pub renamed: usize,
    pub failed: usize,
}

/// Remembers the most recent successful apply so "undo" can reverse it.
/// Only ever holds one batch - a new apply replaces it, and undoing
/// consumes it.
#[derive(Default)]
pub struct UndoState {
    last_apply: Mutex<Vec<(PathBuf, PathBuf)>>,
}

fn rename(from: &Path, to: &Path) -> RenameOutcome {
    match std::fs::rename(from, to) {
        Ok(()) => RenameOutcome {
            path: from.to_string_lossy().to_string(),
            new_path: Some(to.to_string_lossy().to_string()),
            error: None,
        },
        Err(err) => RenameOutcome {
            path: from.to_string_lossy().to_string(),
            new_path: None,
            error: Some(err.to_string()),
        },
    }
}

/// Runs `operation` over every item in `items`, batching progress events
/// and emitting a terminal `{prefix}:complete` on the same channel - the
/// frontend waits for that event rather than this command's return
/// value, since invoke() and emit/listen have no ordering guarantee
/// between them (see events::Batcher).
async fn run_batch(
    app: &AppHandle,
    event_prefix: &str,
    items: impl Iterator<Item = (PathBuf, PathBuf)>,
    operation: impl Fn(&Path, &Path) -> RenameOutcome,
) -> Result<(ApplySummary, Vec<(PathBuf, PathBuf)>), String> {
    let mut renamed = 0usize;
    let mut failed = 0usize;
    let mut succeeded = Vec::new();
    let batcher: Batcher<RenameOutcome> = Batcher::new(BATCH_SIZE);
    let progress_event = format!("{event_prefix}:progress");

    for (from, to) in items {
        let outcome = operation(&from, &to);
        if outcome.error.is_some() {
            failed += 1;
        } else {
            renamed += 1;
            succeeded.push((from, to));
        }

        if let Some(batch) = batcher.push(outcome) {
            app.emit(&progress_event, &batch).map_err(|err| err.to_string())?;
        }
    }

    let remaining = batcher.flush();
    if !remaining.is_empty() {
        app.emit(&progress_event, &remaining).map_err(|err| err.to_string())?;
    }

    let summary = ApplySummary { renamed, failed };
    let _ = app.emit(&format!("{event_prefix}:complete"), &summary);

    Ok((summary, succeeded))
}

#[tauri::command]
pub async fn apply_renames(
    app: AppHandle,
    undo_state: State<'_, UndoState>,
    items: Vec<RenameRequest>,
) -> Result<ApplySummary, String> {
    let mut claimed: HashSet<PathBuf> = HashSet::new();

    let targets: Vec<(PathBuf, PathBuf)> = items
        .iter()
        .map(|request| {
            let from = PathBuf::from(&request.path);
            let parent = from.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
            let suggested_name = compute_suggested_name(&from, &request.canonical_extension);
            let to = resolve_conflict(&parent, &suggested_name, &mut claimed, |p| p.exists());
            (from, to)
        })
        .collect();

    let (summary, succeeded) = run_batch(&app, "apply", targets.into_iter(), |from, to| rename(from, to)).await?;
    *undo_state.last_apply.lock().unwrap() = succeeded;

    Ok(summary)
}

#[tauri::command]
pub async fn undo_last_apply(
    app: AppHandle,
    undo_state: State<'_, UndoState>,
) -> Result<ApplySummary, String> {
    let batch = std::mem::take(&mut *undo_state.last_apply.lock().unwrap());

    // Reverse each (original, renamed-to) pair back to (renamed-to,
    // original). Each is only safe if the renamed file is still there
    // and nothing new has taken the original name since the apply -
    // checked per-item so an unsafe one is reported as a failure rather
    // than silently skipped.
    let reversed = batch.into_iter().map(|(original, current)| (current, original));

    let (summary, _) = run_batch(&app, "apply", reversed, |current, original| {
        if !current.exists() {
            RenameOutcome {
                path: current.to_string_lossy().to_string(),
                new_path: None,
                error: Some("file no longer exists".into()),
            }
        } else if original.exists() {
            RenameOutcome {
                path: current.to_string_lossy().to_string(),
                new_path: None,
                error: Some("original filename is taken again".into()),
            }
        } else {
            rename(current, original)
        }
    })
    .await?;

    Ok(summary)
}
