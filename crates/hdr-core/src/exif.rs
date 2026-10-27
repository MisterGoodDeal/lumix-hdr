use std::path::Path;

use crate::error::HdrError;

const SOI: [u8; 2] = [0xFF, 0xD8];
const APP1: u8 = 0xE1;
const EXIF_HEADER: &[u8] = b"Exif\0\0";

pub fn copy_exif_jpeg(reference: &Path, jpeg_bytes: &[u8]) -> Result<Vec<u8>, HdrError> {
    let source = std::fs::read(reference).map_err(|source| HdrError::Io {
        path: reference.to_path_buf(),
        source,
    })?;
    let Some(exif_segment) = first_exif_app1(&source) else {
        return Ok(jpeg_bytes.to_vec());
    };
    splice_exif_app1(jpeg_bytes, exif_segment)
}

fn first_exif_app1(jpeg: &[u8]) -> Option<&[u8]> {
    if jpeg.len() < 4 || jpeg[0..2] != SOI {
        return None;
    }
    let mut offset = 2;
    while offset + 4 <= jpeg.len() {
        if jpeg[offset] != 0xFF {
            return None;
        }
        let marker = jpeg[offset + 1];
        if marker == 0xDA {
            return None;
        }
        if marker == 0xD9 {
            return None;
        }
        if marker == 0x00 || marker == 0x01 || (0xD0..=0xD8).contains(&marker) {
            offset += 2;
            continue;
        }
        let length = u16::from_be_bytes([jpeg[offset + 2], jpeg[offset + 3]]) as usize;
        let end = offset.checked_add(2 + length)?;
        if end > jpeg.len() {
            return None;
        }
        if marker == APP1 {
            let payload = &jpeg[offset + 4..end];
            if payload.starts_with(EXIF_HEADER) {
                return Some(&jpeg[offset..end]);
            }
        }
        offset = end;
    }
    None
}

fn splice_exif_app1(jpeg: &[u8], exif_segment: &[u8]) -> Result<Vec<u8>, HdrError> {
    if jpeg.len() < 2 || jpeg[0..2] != SOI {
        return Err(HdrError::Exif("JPEG de sortie invalide".to_string()));
    }
    let mut output = Vec::with_capacity(jpeg.len() + exif_segment.len());
    output.extend_from_slice(&SOI);
    output.extend_from_slice(exif_segment);
    let mut offset = 2;
    while offset + 4 <= jpeg.len() {
        if jpeg[offset] != 0xFF {
            output.extend_from_slice(&jpeg[offset..]);
            return Ok(output);
        }
        let marker = jpeg[offset + 1];
        if marker == 0xDA {
            output.extend_from_slice(&jpeg[offset..]);
            return Ok(output);
        }
        if marker == 0x00 || marker == 0x01 || (0xD0..=0xD8).contains(&marker) {
            output.extend_from_slice(&jpeg[offset..offset + 2]);
            offset += 2;
            continue;
        }
        let length = u16::from_be_bytes([jpeg[offset + 2], jpeg[offset + 3]]) as usize;
        let end = offset
            .checked_add(2 + length)
            .ok_or_else(|| HdrError::Exif("segment JPEG tronqué".to_string()))?;
        if end > jpeg.len() {
            return Err(HdrError::Exif("segment JPEG tronqué".to_string()));
        }
        let skip_existing_exif = marker == APP1 && jpeg[offset + 4..end].starts_with(EXIF_HEADER);
        if !skip_existing_exif {
            output.extend_from_slice(&jpeg[offset..end]);
        }
        offset = end;
    }
    if offset < jpeg.len() {
        output.extend_from_slice(&jpeg[offset..]);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_jpeg() -> Vec<u8> {
        let mut bytes = Vec::from(SOI);
        bytes.extend_from_slice(&[0xFF, 0xD9]);
        bytes
    }

    #[test]
    fn splice_inserts_app1_after_soi() {
        let jpeg = minimal_jpeg();
        let segment = {
            let mut s = vec![0xFF, APP1, 0x00, 0x08];
            s.extend_from_slice(EXIF_HEADER);
            s
        };
        let out = splice_exif_app1(&jpeg, &segment).expect("splice");
        assert_eq!(&out[0..2], &SOI);
        assert_eq!(&out[2..4], &[0xFF, APP1]);
        assert_eq!(&out[out.len() - 2..], &[0xFF, 0xD9]);
    }
}
