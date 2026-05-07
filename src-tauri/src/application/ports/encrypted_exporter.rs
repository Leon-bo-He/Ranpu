use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("文件读写错误：{0}")]
    Io(String),
    #[error("加密错误：{0}")]
    Crypto(String),
    #[error("文件格式错误：{0}")]
    Format(String),
    #[error("口令不正确")]
    WrongPassphrase,
}

/// 加密导出器（PROMPT 第 138-141 行 .ydaexp 格式）。
///
/// 文件头：MAGIC(4)='YDA1' | VERSION(1) | SALT(16) | NONCE(12) | 密文+TAG
/// AAD = MAGIC。AES-256-GCM + PBKDF2(600k 轮)。
pub trait EncryptedExporter: Send + Sync {
    fn export_to_file(
        &self,
        plaintext: &[u8],
        passphrase: &str,
        out_path: &Path,
    ) -> Result<(), ExportError>;
}

pub trait EncryptedImporter: Send + Sync {
    fn import_from_file(
        &self,
        in_path: &Path,
        passphrase: &str,
    ) -> Result<Vec<u8>, ExportError>;
}
