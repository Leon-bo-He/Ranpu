//! 主密钥存储。Windows 走 DPAPI；其它平台走带 marker 的明文文件作 dev fallback。
//!
//! 文件路径建议传 `%APPDATA%\Ranpu\keystore.bin`（PROMPT 第 207 行）。

use std::path::PathBuf;

use crate::application::ports::key_store::{KeyStore, KeyStoreError};

pub struct OsKeyStore {
    path: PathBuf,
}

impl OsKeyStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use windows::core::PWSTR;
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
    };

    // windows-rs 0.58 不再 export LocalFree（被 Result<()> 化的版本砍掉了），
    // 直接 link kernel32 的原生符号绕过版本摆动。LocalFree 释放
    // CryptProtectData/CryptUnprotectData 通过 LocalAlloc 分配的输出缓冲。
    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(hmem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    }

    pub fn protect(plain: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
        unsafe {
            let mut input = CRYPT_INTEGER_BLOB {
                cbData: plain.len() as u32,
                pbData: plain.as_ptr() as *mut u8,
            };
            let mut output = CRYPT_INTEGER_BLOB::default();
            CryptProtectData(
                &mut input,
                PWSTR::null(),
                None,
                None,
                None,
                0,
                &mut output,
            )
            .map_err(|e| KeyStoreError::Crypto(e.message()))?;
            let bytes =
                std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
            LocalFree(output.pbData as _);
            Ok(bytes)
        }
    }

    pub fn unprotect(encrypted: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
        unsafe {
            let mut input = CRYPT_INTEGER_BLOB {
                cbData: encrypted.len() as u32,
                pbData: encrypted.as_ptr() as *mut u8,
            };
            let mut output = CRYPT_INTEGER_BLOB::default();
            CryptUnprotectData(
                &mut input,
                None,
                None,
                None,
                None,
                0,
                &mut output,
            )
            .map_err(|e| KeyStoreError::Crypto(e.message()))?;
            let bytes =
                std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
            LocalFree(output.pbData as _);
            Ok(bytes)
        }
    }
}

/// 非 Windows 平台用一个明文文件 + magic 作 dev fallback。
/// 这条警告不要在生产构建上忽略：Windows 才有真正的 DPAPI 保护。
#[cfg(not(windows))]
mod fallback_impl {
    use super::*;
    pub const FALLBACK_MAGIC: &[u8; 8] = b"RNPU-DEV";

    pub fn protect(plain: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
        let mut buf = Vec::with_capacity(FALLBACK_MAGIC.len() + plain.len());
        buf.extend_from_slice(FALLBACK_MAGIC);
        buf.extend_from_slice(plain);
        Ok(buf)
    }

    pub fn unprotect(raw: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
        if raw.len() < FALLBACK_MAGIC.len() || &raw[..FALLBACK_MAGIC.len()] != FALLBACK_MAGIC {
            return Err(KeyStoreError::Crypto(
                "keystore.bin 头部签名不匹配（dev fallback）".into(),
            ));
        }
        Ok(raw[FALLBACK_MAGIC.len()..].to_vec())
    }
}

#[cfg(windows)]
use windows_impl::{protect, unprotect};
#[cfg(not(windows))]
use fallback_impl::{protect, unprotect};

impl KeyStore for OsKeyStore {
    fn load(&self) -> Result<Vec<u8>, KeyStoreError> {
        if !self.path.exists() {
            return Err(KeyStoreError::NotFound(
                self.path.to_string_lossy().into_owned(),
            ));
        }
        let raw = std::fs::read(&self.path).map_err(|e| KeyStoreError::Io(e.to_string()))?;
        unprotect(&raw)
    }

    fn save(&self, secret: &[u8]) -> Result<(), KeyStoreError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| KeyStoreError::Io(e.to_string()))?;
        }
        let protected = protect(secret)?;
        std::fs::write(&self.path, protected).map_err(|e| KeyStoreError::Io(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn save_then_load_round_trips() {
        let path = env::temp_dir().join(format!("ranpu-keystore-{}.bin", uuid::Uuid::new_v4()));
        let store = OsKeyStore::new(path.clone());
        let secret = vec![0xAB; 32];
        store.save(&secret).unwrap();
        let recovered = store.load().unwrap();
        assert_eq!(recovered, secret);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_is_not_found() {
        let path = env::temp_dir().join(format!("ranpu-missing-{}.bin", uuid::Uuid::new_v4()));
        let store = OsKeyStore::new(path);
        let err = store.load().unwrap_err();
        assert!(matches!(err, KeyStoreError::NotFound(_)));
    }
}
