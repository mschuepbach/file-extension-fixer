use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::naming::{compute_suggested_name, resolve_conflict};

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

fn rename_one(request: &RenameRequest, claimed: &mut HashSet<PathBuf>) -> RenameOutcome {
    let path = PathBuf::from(&request.path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    let suggested_name = compute_suggested_name(&path, &request.canonical_extension);
    let target = resolve_conflict(parent, &suggested_name, claimed, |p| p.exists());

    match std::fs::rename(&path, &target) {
        Ok(()) => RenameOutcome {
            path: request.path.clone(),
            new_path: Some(target.to_string_lossy().to_string()),
            error: None,
        },
        Err(err) => RenameOutcome {
            path: request.path.clone(),
            new_path: None,
            error: Some(err.to_string()),
        },
    }
}

#[tauri::command]
pub async fn apply_renames(
    app: AppHandle,
    items: Vec<RenameRequest>,
) -> Result<ApplySummary, String> {
    let mut renamed = 0usize;
    let mut failed = 0usize;
    let mut claimed: HashSet<PathBuf> = HashSet::new();

    for request in &items {
        let outcome = rename_one(request, &mut claimed);
        if outcome.error.is_some() {
            failed += 1;
        } else {
            renamed += 1;
        }

        app.emit("apply:progress", &outcome)
            .map_err(|err| err.to_string())?;
    }

    Ok(ApplySummary { renamed, failed })
}
