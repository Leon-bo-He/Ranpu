use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("找不到密钥库：{0}")]
    NotFound(String),
    #[error("密钥库读写错误：{0}")]
    Io(String),
    #[error("密钥库加解密错误：{0}")]
    Crypto(String),
}

/// DB 主密钥的 OS 级保护抽象。
///
/// 默认实现（feat/infra-crypto，仅 Windows）用 DPAPI；其它平台用一个
/// 占位的明文文件实现，便于本机开发，但不可用于发布构建。
pub trait KeyStore: Send + Sync {
    /// 取出主密钥；首次调用如不存在应返回 NotFound，由调用方 ensure_or_init。
    fn load(&self) -> Result<Vec<u8>, KeyStoreError>;
    /// 保存（覆写）主密钥。
    fn save(&self, secret: &[u8]) -> Result<(), KeyStoreError>;
}
