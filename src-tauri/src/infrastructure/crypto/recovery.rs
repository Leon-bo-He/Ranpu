//! 恢复槽 (recovery slot): 一份 db_key_hex 用编译期内置的 master 口令
//! 加密 + 落到 `recovery.bin`. 用户忘记启动口令时, 系统自己拿这条内置
//! 口令把 db_key 解出来直接打开 SQLCipher, 不再走 派生 链路.
//!
//! 设计取舍:
//! - master 口令在二进制里是明文常量, 反汇编可以拿到. 这是产品方明确
//!   要的"管理员后门" — 接受其安全代价, 它本来就是为了应急.
//! - 文件内容仍走 AES-256-GCM (跟 .ranpu 导出复用同套 KDF + 算法), 不是
//!   裸明文; master 口令泄漏才会暴露 db_key.
//! - 写入时机: boot 成功 (即 db_key_hex 是有效的) 且 recovery.bin 不存
//!   在时, 一次性写入. 用户改启动口令 (没有此功能) 不需要更新; 备份恢复
//!   到新机后会重新 boot, 重新 ensure 一次.
//!
//! 文件布局: SALT(16) | NONCE(12) | 密文 (db_key_hex 64B) + TAG(16). 共
//! 16 + 12 + 64 + 16 = 108 字节.

use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;

use crate::infrastructure::crypto::key_derivation::derive_export_key;

/// 编译期内置后门口令. 任意时刻 (包括 boot / 锁屏解锁) 都能用它替代
/// 用户口令. 用户看不到, UI 不展示.
pub const RECOVERY_MASTER_PASSPHRASE: &str = "Adminadmin123!";

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const AAD: &[u8; 8] = b"RNP-RCV1";

#[derive(Debug)]
pub enum RecoveryError {
    Io(std::io::Error),
    Crypto,
    Format,
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::Io(e) => write!(f, "recovery io: {e}"),
            RecoveryError::Crypto => write!(f, "recovery 解密失败"),
            RecoveryError::Format => write!(f, "recovery 文件格式错误"),
        }
    }
}

impl From<std::io::Error> for RecoveryError {
    fn from(e: std::io::Error) -> Self {
        RecoveryError::Io(e)
    }
}

/// 把 db_key_hex 用 master 口令加密落盘. 已存在时直接覆写 (无害, 内容相同).
pub fn write_recovery(path: &Path, db_key_hex: &str) -> Result<(), RecoveryError> {
    let mut salt = [0_u8; SALT_LEN];
    let mut nonce_bytes = [0_u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_export_key(RECOVERY_MASTER_PASSPHRASE, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| RecoveryError::Crypto)?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: db_key_hex.as_bytes(),
                aad: AAD,
            },
        )
        .map_err(|_| RecoveryError::Crypto)?;

    let mut buf = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    buf.extend_from_slice(&salt);
    buf.extend_from_slice(&nonce_bytes);
    buf.extend_from_slice(&ciphertext);
    std::fs::write(path, buf)?;
    Ok(())
}

/// 用 master 口令尝试读出 db_key_hex. 文件不存在返回 Ok(None); 存在但解
/// 密失败 / 格式错误返回 Err.
pub fn read_recovery(path: &Path) -> Result<Option<String>, RecoveryError> {
    if !path.exists() {
        return Ok(None);
    }
    let buf = std::fs::read(path)?;
    if buf.len() < SALT_LEN + NONCE_LEN {
        return Err(RecoveryError::Format);
    }
    let salt = &buf[..SALT_LEN];
    let nonce_bytes = &buf[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &buf[SALT_LEN + NONCE_LEN..];

    let key = derive_export_key(RECOVERY_MASTER_PASSPHRASE, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| RecoveryError::Crypto)?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad: AAD,
            },
        )
        .map_err(|_| RecoveryError::Crypto)?;
    let s = String::from_utf8(plaintext).map_err(|_| RecoveryError::Format)?;
    Ok(Some(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use uuid::Uuid;

    fn tmp_path() -> std::path::PathBuf {
        temp_dir().join(format!("ranpu-recovery-{}.bin", Uuid::new_v4()))
    }

    #[test]
    fn round_trip() {
        let path = tmp_path();
        let key_hex = "abcdef0123456789".repeat(4);
        write_recovery(&path, &key_hex).unwrap();
        let got = read_recovery(&path).unwrap().unwrap();
        assert_eq!(got, key_hex);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_returns_none() {
        let path = tmp_path();
        let got = read_recovery(&path).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn tampered_file_fails() {
        let path = tmp_path();
        let key_hex = "deadbeef".repeat(8);
        write_recovery(&path, &key_hex).unwrap();
        let mut buf = std::fs::read(&path).unwrap();
        // 翻转最后一个 byte → AEAD 校验必须挂.
        let last = buf.len() - 1;
        buf[last] ^= 0xff;
        std::fs::write(&path, &buf).unwrap();
        assert!(read_recovery(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
