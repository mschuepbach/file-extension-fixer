use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameRequest {
    pub path: String,
    pub new_extension: String,
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

fn rename_one(request: &RenameRequest) -> RenameOutcome {
    let path = PathBuf::from(&request.path);

    let new_path = path.with_extension(&request.new_extension);

    if new_path.exists() {
        return RenameOutcome {
            path: request.path.clone(),
            new_path: None,
            error: Some("a file with the new name already exists".into()),
        };
    }

    match std::fs::rename(&path, &new_path) {
        Ok(()) => RenameOutcome {
            path: request.path.clone(),
            new_path: Some(new_path.to_string_lossy().to_string()),
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

    for request in &items {
        let outcome = rename_one(request);
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
