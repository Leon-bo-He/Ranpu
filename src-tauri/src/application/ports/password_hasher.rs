use thiserror::Error;

use crate::domain::identity::password::PasswordHash;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum PasswordHasherError {
    #[error("生成密码哈希失败：{0}")]
    HashFailed(String),
    #[error("校验密码哈希失败：{0}")]
    VerifyFailed(String),
}

/// 密码哈希器抽象 (PROMPT 第 96 行 PasswordHasher trait)。
///
/// 默认实现（feat/infra-crypto）用 argon2id。
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, plain: &str) -> Result<PasswordHash, PasswordHasherError>;
    fn verify(&self, plain: &str, hash: &PasswordHash) -> Result<bool, PasswordHasherError>;
}
