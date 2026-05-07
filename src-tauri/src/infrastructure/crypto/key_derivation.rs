//! PBKDF2-SHA256 600k 轮派生工具。
//!
//! 用于：
//! 1) 把「主密钥(32B) + 启动口令」派生 SQLCipher 用的 32 字节 key（输出 64 hex）
//! 2) 把「.ydaexp 导出口令 + salt(16B)」派生 32B AES 密钥

use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha256;

pub const PBKDF2_ROUNDS: u32 = 600_000;

/// 用 master_key + boot_passphrase 派生 32 字节，再 hex 化成 64 字符。
pub fn derive_db_key_hex(master_key: &[u8], boot_passphrase: &str) -> String {
    let mut combined = Vec::with_capacity(master_key.len() + boot_passphrase.len());
    combined.extend_from_slice(master_key);
    combined.extend_from_slice(boot_passphrase.as_bytes());
    let mut out = [0_u8; 32];
    pbkdf2::<Hmac<Sha256>>(&combined, b"ranpu-db-key", PBKDF2_ROUNDS, &mut out)
        .expect("pbkdf2 length parameters fixed at compile time");
    hex_encode(&out)
}

/// 用 passphrase + salt 派生 32 字节 AES-GCM key。
pub fn derive_export_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let mut out = [0_u8; 32];
    pbkdf2::<Hmac<Sha256>>(passphrase.as_bytes(), salt, PBKDF2_ROUNDS, &mut out)
        .expect("pbkdf2 length parameters fixed at compile time");
    out
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_db_key_is_deterministic() {
        let master = b"0123456789abcdef0123456789abcdef";
        let a = derive_db_key_hex(master, "boot-pw");
        let b = derive_db_key_hex(master, "boot-pw");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn db_key_changes_with_passphrase() {
        let master = b"0123456789abcdef0123456789abcdef";
        let a = derive_db_key_hex(master, "p1");
        let b = derive_db_key_hex(master, "p2");
        assert_ne!(a, b);
    }

    #[test]
    fn export_key_changes_with_salt() {
        let a = derive_export_key("pw", &[1; 16]);
        let b = derive_export_key("pw", &[2; 16]);
        assert_ne!(a, b);
    }
}
