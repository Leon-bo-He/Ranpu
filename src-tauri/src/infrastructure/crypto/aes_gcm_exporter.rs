use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;

use crate::application::ports::encrypted_exporter::{
    EncryptedExporter, EncryptedImporter, ExportError,
};
use crate::infrastructure::crypto::key_derivation::derive_export_key;

/// `.ranpu` 文件头格式：
///   MAGIC(4)='RNP1' | VERSION(1) | SALT(16) | NONCE(12) | 密文 + TAG(16)
/// AAD 取 MAGIC。
///
/// (历史: 第一版用过 'YDA1' / .ydaexp, 与品牌 「染谱 Ranpu」 不一致, 已统一改为
/// RNP1 / .ranpu. PROMPT 第 141 行的旧规约保留为文档历史.)
pub const MAGIC: &[u8; 4] = b"RNP1";
pub const VERSION: u8 = 1;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const HEADER_LEN: usize = 4 + 1 + SALT_LEN + NONCE_LEN; // = 33

pub struct RanpuExporter;

impl RanpuExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RanpuExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptedExporter for RanpuExporter {
    fn export_to_file(
        &self,
        plaintext: &[u8],
        passphrase: &str,
        out_path: &Path,
    ) -> Result<(), ExportError> {
        let mut salt = [0_u8; SALT_LEN];
        let mut nonce_bytes = [0_u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let key = derive_export_key(passphrase, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| ExportError::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext,
                    aad: MAGIC,
                },
            )
            .map_err(|e| ExportError::Crypto(e.to_string()))?;

        let mut buf = Vec::with_capacity(HEADER_LEN + ciphertext.len());
        buf.extend_from_slice(MAGIC);
        buf.push(VERSION);
        buf.extend_from_slice(&salt);
        buf.extend_from_slice(&nonce_bytes);
        buf.extend_from_slice(&ciphertext);

        std::fs::write(out_path, buf).map_err(|e| ExportError::Io(e.to_string()))?;
        Ok(())
    }
}

impl EncryptedImporter for RanpuExporter {
    fn import_from_file(
        &self,
        in_path: &Path,
        passphrase: &str,
    ) -> Result<Vec<u8>, ExportError> {
        let raw = std::fs::read(in_path).map_err(|e| ExportError::Io(e.to_string()))?;
        if raw.len() < HEADER_LEN {
            return Err(ExportError::Format("文件长度不足以容纳头部".into()));
        }
        if &raw[..4] != MAGIC {
            return Err(ExportError::Format("文件签名不匹配，可能不是 .ranpu".into()));
        }
        if raw[4] != VERSION {
            return Err(ExportError::Format(format!(
                "不支持的版本号：{}",
                raw[4]
            )));
        }
        let salt = &raw[5..5 + SALT_LEN];
        let nonce_bytes = &raw[5 + SALT_LEN..5 + SALT_LEN + NONCE_LEN];
        let ciphertext = &raw[HEADER_LEN..];

        let key = derive_export_key(passphrase, salt);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| ExportError::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext,
                    aad: MAGIC,
                },
            )
            .map_err(|_| ExportError::WrongPassphrase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn round_trip_export_then_import() {
        let path = env::temp_dir().join(format!("ranpu-test-{}.ranpu", uuid::Uuid::new_v4()));
        let exporter = RanpuExporter::new();
        exporter
            .export_to_file(b"hello, dyer", "topsecret", &path)
            .unwrap();
        let recovered = exporter.import_from_file(&path, "topsecret").unwrap();
        assert_eq!(recovered, b"hello, dyer");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let path = env::temp_dir().join(format!("ranpu-test-{}.ranpu", uuid::Uuid::new_v4()));
        let exporter = RanpuExporter::new();
        exporter.export_to_file(b"x", "right", &path).unwrap();
        let err = exporter.import_from_file(&path, "wrong").unwrap_err();
        assert!(matches!(err, ExportError::WrongPassphrase));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn corrupted_magic_is_format_error() {
        let path = env::temp_dir().join(format!("ranpu-test-{}.ranpu", uuid::Uuid::new_v4()));
        let exporter = RanpuExporter::new();
        exporter.export_to_file(b"x", "pw", &path).unwrap();
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[0] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();
        let err = exporter.import_from_file(&path, "pw").unwrap_err();
        assert!(matches!(err, ExportError::Format(_)));
        let _ = std::fs::remove_file(&path);
    }
}
