use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::detect;

/// Computes the filename (not full path) a mismatched file should be
/// renamed to, given the format its content was detected as.
///
/// - If the current extension isn't one we recognize at all (`.dup3`,
///   `.bak`, no extension, ...), the canonical extension is appended -
///   nothing about the original name is touched or guessed at.
/// - If the current extension is a *known* extension (just the wrong
///   format), it's replaced. If that replacement would leave two
///   identical real extensions back to back (`vacation.png.mp4` -> a
///   naive replace gives `vacation.png.png`), the redundant one is
///   dropped instead. Only one level of lookback is checked, so
///   `vacation.png.png.mp4` normalizes to `vacation.png.png`, not a
///   fully recursive collapse.
pub fn compute_suggested_name(path: &Path, canonical: &str) -> String {
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

    let current_ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase());

    let Some(current_ext) = current_ext else {
        return format!("{file_name}.{canonical}");
    };

    if !detect::is_known_extension(&current_ext) {
        return format!("{file_name}.{canonical}");
    }

    let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();

    // Does the stem itself already end in an accepted extension for this
    // same format? If so the naive replace would just duplicate it.
    if let Some(prior_ext) = Path::new(&stem).extension().and_then(|e| e.to_str()) {
        let prior_ext = prior_ext.to_lowercase();
        if let Some(accepted) = detect::accepted_for(canonical) {
            if accepted.contains(&prior_ext.as_str()) {
                return stem;
            }
        }
    }

    format!("{stem}.{canonical}")
}

/// Resolves `parent/suggested_name` against collisions - both files
/// already on disk and other targets already claimed earlier in the same
/// batch - appending " (1)", " (2)", etc. until a free name is found.
/// `exists` is injected so this stays testable without touching the
/// filesystem.
pub fn resolve_conflict(
    parent: &Path,
    suggested_name: &str,
    claimed: &mut HashSet<PathBuf>,
    exists: impl Fn(&Path) -> bool,
) -> PathBuf {
    let base = parent.join(suggested_name);
    if !exists(&base) && !claimed.contains(&base) {
        claimed.insert(base.clone());
        return base;
    }

    let candidate_source = Path::new(suggested_name);
    let stem = candidate_source.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let ext = candidate_source.extension().map(|e| e.to_string_lossy().to_string());

    let mut n: usize = 1;
    loop {
        let candidate_name = match &ext {
            Some(ext) => format!("{stem} ({n}).{ext}"),
            None => format!("{stem} ({n})"),
        };
        let candidate = parent.join(candidate_name);
        if !exists(&candidate) && !claimed.contains(&candidate) {
            claimed.insert(candidate.clone());
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_when_current_extension_is_unknown() {
        let name = compute_suggested_name(Path::new("somefile.jpg.dup3"), "jpg");
        assert_eq!(name, "somefile.jpg.dup3.jpg");
    }

    #[test]
    fn appends_when_there_is_no_extension_at_all() {
        let name = compute_suggested_name(Path::new("IMG_1234"), "jpg");
        assert_eq!(name, "IMG_1234.jpg");
    }

    #[test]
    fn replaces_when_current_extension_is_a_known_but_wrong_format() {
        let name = compute_suggested_name(Path::new("clip.mov"), "mp4");
        assert_eq!(name, "clip.mp4");
    }

    #[test]
    fn collapses_a_redundant_duplicate_extension() {
        let name = compute_suggested_name(Path::new("vacation.png.mp4"), "png");
        assert_eq!(name, "vacation.png");
    }

    #[test]
    fn collapses_across_equivalent_spellings_of_the_same_format() {
        // "jpeg" and "jpg" are both accepted for the jpg format.
        let name = compute_suggested_name(Path::new("vacation.jpeg.mp4"), "jpg");
        assert_eq!(name, "vacation.jpeg");
    }

    #[test]
    fn does_not_collapse_different_formats() {
        let name = compute_suggested_name(Path::new("vacation.jpg.mp4"), "png");
        assert_eq!(name, "vacation.jpg.png");
    }

    #[test]
    fn only_collapses_one_level() {
        let name = compute_suggested_name(Path::new("vacation.png.png.mp4"), "png");
        assert_eq!(name, "vacation.png.png");
    }

    #[test]
    fn resolve_conflict_returns_base_when_free() {
        let mut claimed = HashSet::new();
        let result = resolve_conflict(Path::new("/dir"), "photo.jpg", &mut claimed, |_| false);
        assert_eq!(result, PathBuf::from("/dir/photo.jpg"));
    }

    #[test]
    fn resolve_conflict_numbers_when_target_exists_on_disk() {
        let mut claimed = HashSet::new();
        let existing = PathBuf::from("/dir/photo.jpg");
        let result = resolve_conflict(Path::new("/dir"), "photo.jpg", &mut claimed, move |p| p == existing);
        assert_eq!(result, PathBuf::from("/dir/photo (1).jpg"));
    }

    #[test]
    fn resolve_conflict_numbers_when_target_already_claimed_in_batch() {
        let mut claimed = HashSet::new();
        claimed.insert(PathBuf::from("/dir/photo.jpg"));
        let result = resolve_conflict(Path::new("/dir"), "photo.jpg", &mut claimed, |_| false);
        assert_eq!(result, PathBuf::from("/dir/photo (1).jpg"));
    }

    #[test]
    fn resolve_conflict_keeps_incrementing_past_multiple_collisions() {
        let mut claimed = HashSet::new();
        let taken = [
            PathBuf::from("/dir/photo.jpg"),
            PathBuf::from("/dir/photo (1).jpg"),
            PathBuf::from("/dir/photo (2).jpg"),
        ];
        let result = resolve_conflict(Path::new("/dir"), "photo.jpg", &mut claimed, move |p| {
            taken.contains(&p.to_path_buf())
        });
        assert_eq!(result, PathBuf::from("/dir/photo (3).jpg"));
    }

    #[test]
    fn resolve_conflict_handles_no_extension() {
        let mut claimed = HashSet::new();
        let existing = PathBuf::from("/dir/README");
        let result = resolve_conflict(Path::new("/dir"), "README", &mut claimed, move |p| p == existing);
        assert_eq!(result, PathBuf::from("/dir/README (1)"));
    }
}
