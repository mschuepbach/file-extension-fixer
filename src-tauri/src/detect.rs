use std::fs::File;
use std::io::Read;
use std::path::Path;

const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const TIFF_LE_MAGIC: [u8; 4] = [0x49, 0x49, 0x2A, 0x00];
const TIFF_BE_MAGIC: [u8; 4] = [0x4D, 0x4D, 0x00, 0x2A];

/// A recognized format: `canonical` is the extension suggested on rename,
/// `accepted` is every extension that should NOT be flagged as a mismatch
/// (e.g. a jpg-detected file named `.jpeg` is already fine).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FormatMatch {
    pub canonical: &'static str,
    pub accepted: &'static [&'static str],
}

/// Reads the leading bytes of a file and determines its real format
/// from its magic-byte signature. Only the formats this tool supports
/// are recognized; anything else (including unreadable files) returns
/// `None` so callers can skip it entirely.
pub fn detect_extension(path: &Path) -> Option<FormatMatch> {
    let mut file = File::open(path).ok()?;
    let mut header = [0u8; 16];
    let read = file.read(&mut header).ok()?;
    let header = &header[..read];

    detect_from_bytes(header)
}

fn detect_from_bytes(header: &[u8]) -> Option<FormatMatch> {
    if header.len() >= 3 && header[..3] == [0xFF, 0xD8, 0xFF] {
        return Some(FormatMatch { canonical: "jpg", accepted: &["jpg", "jpeg"] });
    }

    if header.len() >= 8 && header[..8] == PNG_MAGIC {
        return Some(FormatMatch { canonical: "png", accepted: &["png"] });
    }

    if header.len() >= 4 && &header[..4] == b"GIF8" {
        return Some(FormatMatch { canonical: "gif", accepted: &["gif"] });
    }

    if header.len() >= 4
        && (header[..4] == TIFF_LE_MAGIC || header[..4] == TIFF_BE_MAGIC)
    {
        return Some(FormatMatch { canonical: "tiff", accepted: &["tiff", "tif"] });
    }

    if header.len() >= 2 && &header[..2] == b"BM" {
        return Some(FormatMatch { canonical: "bmp", accepted: &["bmp"] });
    }

    if header.len() >= 4 && &header[..4] == b"fLaC" {
        return Some(FormatMatch { canonical: "flac", accepted: &["flac"] });
    }

    if header.len() >= 4 && &header[..4] == b"OggS" {
        return Some(FormatMatch { canonical: "ogg", accepted: &["ogg", "oga", "opus"] });
    }

    // RIFF containers: bytes 0..4 "RIFF", size, then a 4-byte form type
    // at 8..12 that tells us what's actually inside.
    if header.len() >= 12 && &header[..4] == b"RIFF" {
        return match &header[8..12] {
            b"WEBP" => Some(FormatMatch { canonical: "webp", accepted: &["webp"] }),
            b"WAVE" => Some(FormatMatch { canonical: "wav", accepted: &["wav"] }),
            b"AVI " => Some(FormatMatch { canonical: "avi", accepted: &["avi"] }),
            _ => None,
        };
    }

    // ISO Base Media File Format containers: bytes 4..8 spell "ftyp" and
    // bytes 8..12 hold the major brand, which tells mp4/mov/heic/avif/m4a
    // apart. Brands outside our known list still get treated as mp4,
    // since that covers the overwhelming majority of ftyp-boxed video.
    if header.len() >= 12 && &header[4..8] == b"ftyp" {
        let brand = &header[8..12];
        return Some(match brand {
            b"qt  " => FormatMatch { canonical: "mov", accepted: &["mov", "qt"] },
            b"M4A " | b"M4B " => FormatMatch { canonical: "m4a", accepted: &["m4a", "m4b"] },
            b"3gp4" | b"3gp5" | b"3gp6" | b"3g2a" => {
                FormatMatch { canonical: "3gp", accepted: &["3gp", "3g2"] }
            }
            b"avif" | b"avis" => FormatMatch { canonical: "avif", accepted: &["avif"] },
            b"heic" | b"heix" | b"heim" | b"heis" | b"hevc" | b"hevx" | b"hevm" | b"hevs"
            | b"mif1" | b"msf1" => FormatMatch { canonical: "heic", accepted: &["heic", "heif"] },
            _ => FormatMatch { canonical: "mp4", accepted: &["mp4", "m4v"] },
        });
    }

    // WebM and Matroska share the EBML container signature; we don't
    // distinguish them further and suggest the more common ".webm".
    if header.len() >= 4 && header[..4] == [0x1A, 0x45, 0xDF, 0xA3] {
        return Some(FormatMatch { canonical: "webm", accepted: &["webm", "mkv"] });
    }

    // MP3: an ID3v2-tagged file starts with "ID3"; an untagged file
    // starts directly with an MPEG frame sync (11 set bits).
    if header.len() >= 3 && &header[..3] == b"ID3" {
        return Some(FormatMatch { canonical: "mp3", accepted: &["mp3"] });
    }
    if header.len() >= 2 && header[0] == 0xFF && (header[1] & 0xE0) == 0xE0 {
        return Some(FormatMatch { canonical: "mp3", accepted: &["mp3"] });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_jpeg() {
        let bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("jpg"));
    }

    #[test]
    fn detects_png() {
        assert_eq!(detect_from_bytes(&PNG_MAGIC).map(|f| f.canonical), Some("png"));
    }

    #[test]
    fn detects_gif() {
        let bytes = b"GIF89a\x00\x00";
        assert_eq!(detect_from_bytes(bytes).map(|f| f.canonical), Some("gif"));
    }

    #[test]
    fn detects_bmp() {
        let bytes = b"BM\x00\x00\x00\x00";
        assert_eq!(detect_from_bytes(bytes).map(|f| f.canonical), Some("bmp"));
    }

    #[test]
    fn detects_tiff_little_endian() {
        assert_eq!(detect_from_bytes(&TIFF_LE_MAGIC).map(|f| f.canonical), Some("tiff"));
    }

    #[test]
    fn detects_tiff_big_endian() {
        assert_eq!(detect_from_bytes(&TIFF_BE_MAGIC).map(|f| f.canonical), Some("tiff"));
    }

    #[test]
    fn detects_flac() {
        let bytes = b"fLaC\x00\x00\x00\x00";
        assert_eq!(detect_from_bytes(bytes).map(|f| f.canonical), Some("flac"));
    }

    #[test]
    fn detects_ogg() {
        let bytes = b"OggS\x00\x00\x00\x00";
        assert_eq!(detect_from_bytes(bytes).map(|f| f.canonical), Some("ogg"));
    }

    #[test]
    fn detects_webp() {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(b"WEBP");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("webp"));
    }

    #[test]
    fn detects_wav() {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(b"WAVE");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("wav"));
    }

    #[test]
    fn detects_avi() {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(b"AVI ");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("avi"));
    }

    #[test]
    fn detects_mp4() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x18];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"isom");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("mp4"));
    }

    #[test]
    fn detects_mov() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x14];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"qt  ");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("mov"));
    }

    #[test]
    fn detects_heic() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x18];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"heic");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("heic"));
    }

    #[test]
    fn detects_avif() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x18];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"avif");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("avif"));
    }

    #[test]
    fn detects_m4a() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x18];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"M4A ");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("m4a"));
    }

    #[test]
    fn detects_3gp() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x18];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"3gp4");
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("3gp"));
    }

    #[test]
    fn detects_webm() {
        let bytes = [0x1A, 0x45, 0xDF, 0xA3, 0x00, 0x00];
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("webm"));
    }

    #[test]
    fn detects_mp3_with_id3_tag() {
        let bytes = b"ID3\x03\x00\x00\x00";
        assert_eq!(detect_from_bytes(bytes).map(|f| f.canonical), Some("mp3"));
    }

    #[test]
    fn detects_mp3_frame_sync() {
        let bytes = [0xFF, 0xFB, 0x90, 0x00];
        assert_eq!(detect_from_bytes(&bytes).map(|f| f.canonical), Some("mp3"));
    }

    #[test]
    fn accepts_jpeg_as_equivalent_to_jpg() {
        let bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        let format = detect_from_bytes(&bytes).unwrap();
        assert!(format.accepted.contains(&"jpeg"));
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
