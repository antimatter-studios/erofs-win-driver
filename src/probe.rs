//! EROFS-specific superblock detection.
//!
//! EROFS places its superblock at byte offset 1024 from the start of
//! the device (`EROFS_SUPER_OFFSET`). The first four bytes are the
//! magic number `EROFS_SUPER_MAGIC_V1 = 0xE0F5E1E2` (little-endian),
//! defined in `linux/fs/erofs/erofs_fs.h`.

const EROFS_SUPER_OFFSET: usize = 1024;

/// EROFS_SUPER_MAGIC_V1 = 0xE0F5E1E2, little-endian on disk.
const EROFS_MAGIC: [u8; 4] = [0xE2, 0xE1, 0xF5, 0xE0];

pub fn is_erofs(bytes: &[u8]) -> bool {
    if bytes.len() < EROFS_SUPER_OFFSET + 4 {
        return false;
    }
    bytes[EROFS_SUPER_OFFSET..EROFS_SUPER_OFFSET + 4] == EROFS_MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_BUF_LEN: usize = 1100;

    #[test]
    fn matches_magic_at_super_offset() {
        let mut buf = vec![0u8; TEST_BUF_LEN];
        buf[EROFS_SUPER_OFFSET..EROFS_SUPER_OFFSET + 4].copy_from_slice(&EROFS_MAGIC);
        assert!(is_erofs(&buf));
    }

    #[test]
    fn rejects_wrong_magic() {
        let mut buf = vec![0u8; TEST_BUF_LEN];
        buf[EROFS_SUPER_OFFSET..EROFS_SUPER_OFFSET + 4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(!is_erofs(&buf));
    }

    #[test]
    fn rejects_short_buffer() {
        let buf = vec![0xE2u8; 16];
        assert!(!is_erofs(&buf));
    }

    #[test]
    fn rejects_magic_at_wrong_offset() {
        let mut buf = vec![0u8; TEST_BUF_LEN];
        buf[0..4].copy_from_slice(&EROFS_MAGIC);
        assert!(!is_erofs(&buf));
    }
}
