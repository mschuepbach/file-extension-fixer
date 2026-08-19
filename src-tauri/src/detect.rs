use std::fs::File;
use std::io::Read;
use std::path::Path;

const JPEG_MAGIC: [u8; 3] = [0xFF, 0xD8, 0xFF];
const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// Reads the leading bytes of a file and determines its real format
/// from its magic-byte signature. Only the formats this tool supports
/// are recognized; anything else (including unreadable files) returns
/// `None` so callers can skip it entirely.
pub fn detect_extension(path: &Path) -> Option<&'static str> {
    let mut file = File::open(path).ok()?;
    let mut header = [0u8; 16];
    let read = file.read(&mut header).ok()?;
    let header = &header[..read];

    detect_from_bytes(header)
}

fn detect_from_bytes(header: &[u8]) -> Option<&'static str> {
    if header.len() >= 3 && header[..3] == JPEG_MAGIC {
        return Some("jpg");
    }

    if header.len() >= 8 && header[..8] == PNG_MAGIC {
        return Some("png");
    }

    // MP4 and MOV are both ISO Base Media File Format containers: bytes
    // 4..8 spell "ftyp" and bytes 8..12 hold the major brand. QuickTime
    // (.mov) uses the "qt  " brand; everything else with an ftyp box in
    // our supported set is treated as MP4.
    if header.len() >= 12 && &header[4..8] == b"ftyp" {
        let brand = &header[8..12];
        if brand == b"qt  " {
            return Some("mov");
        }
        return Some("mp4");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_jpeg() {
        let bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_from_bytes(&bytes), Some("jpg"));
    }

    #[test]
    fn detects_png() {
        assert_eq!(detect_from_bytes(&PNG_MAGIC), Some("png"));
    }

    #[test]
    fn detects_mp4() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x18];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"isom");
        assert_eq!(detect_from_bytes(&bytes), Some("mp4"));
    }

    #[test]
    fn detects_mov() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x14];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"qt  ");
        assert_eq!(detect_from_bytes(&bytes), Some("mov"));
    }

    #[test]
    fn returns_none_for_unrecognized() {
        let bytes = [0x50, 0x4B, 0x03, 0x04]; // zip
        assert_eq!(detect_from_bytes(&bytes), None);
    }

    #[test]
    fn returns_none_for_short_file() {
        let bytes = [0xFF];
        assert_eq!(detect_from_bytes(&bytes), None);
    }
}
