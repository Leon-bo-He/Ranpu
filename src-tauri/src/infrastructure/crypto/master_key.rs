//! 32 字节主密钥的生成与缓存。
//!
//! 首次启动：随机生成 → 用 KeyStore 写到 keystore.bin。
//! 后续启动：从 KeyStore 读出。配合 PBKDF2 派生 SQLCipher key。

use rand::RngCore;

use crate::application::ports::key_store::{KeyStore, KeyStoreError};

pub const MASTER_KEY_LEN: usize = 32;

/// 取主密钥；不存在则随机生成 + 持久化。
pub fn ensure_master_key(store: &dyn KeyStore) -> Result<[u8; MASTER_KEY_LEN], KeyStoreError> {
    match store.load() {
        Ok(bytes) if bytes.len() == MASTER_KEY_LEN => {
            let mut out = [0_u8; MASTER_KEY_LEN];
            out.copy_from_slice(&bytes);
            Ok(out)
        }
        Ok(_) => Err(KeyStoreError::Crypto("主密钥长度不正确".into())),
        Err(KeyStoreError::NotFound(_)) => {
            let mut bytes = [0_u8; MASTER_KEY_LEN];
            rand::thread_rng().fill_bytes(&mut bytes);
            store.save(&bytes)?;
            Ok(bytes)
        }
        Err(e) => Err(e),
    }
}
