# File Extension Fixer

A desktop app that scans a folder for media files whose extension doesn't match their actual
content — a photo saved as `.mp4`, a video saved as `.jpg` — and lets you review and fix them in
bulk.

Detection is based on file signatures (magic bytes), not the filename, so it works even if the
extension is completely wrong. Supported formats: jpg, png, gif, webp, bmp, tiff, heic, avif, mp4,
mov, webm, mkv, avi, 3gp, m4a, wav, flac, ogg, mp3.

## How it works

1. Pick a folder (drag-and-drop or browse) - subdirectories are scanned automatically.
2. Mismatches show up live as they're found, each with the file's real detected format.
3. Select the ones you want to fix (checkboxes, shift-click for a range) and apply.
4. If something goes wrong, undo restores the original filenames.

If a fixed name would collide with a file that already exists, a numbered suffix is added instead
(`photo (1).jpg`) - nothing gets silently overwritten.

## Development

Requires [Rust](https://www.rust-lang.org/tools/install) and [Node.js](https://nodejs.org/).

```bash
npm install
npm run tauri dev
```

Run the Rust test suite:

```bash
cd src-tauri
cargo test
```

Built with [Tauri](https://tauri.app/), React, and TypeScript.
