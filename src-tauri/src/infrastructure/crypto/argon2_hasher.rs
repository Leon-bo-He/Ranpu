use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash as A2Hash, PasswordHasher as A2Hasher, PasswordVerifier};

use crate::application::ports::password_hasher::{PasswordHasher, PasswordHasherError};
use crate::domain::identity::password::PasswordHash;

/// argon2id 默认参数 (m=19MiB, t=2, p=1) 的 PasswordHasher 实现。
pub struct Argon2PasswordHasher {
    inner: Argon2<'static>,
}

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self {
            inner: Argon2::default(),
        }
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, plain: &str) -> Result<PasswordHash, PasswordHasherError> {
        let salt = SaltString::generate(&mut OsRng);
        let phc = self
            .inner
            .hash_password(plain.as_bytes(), &salt)
            .map_err(|e| PasswordHasherError::HashFailed(e.to_string()))?;
        Ok(PasswordHash::from_phc_string(phc.to_string()))
    }

    fn verify(&self, plain: &str, hash: &PasswordHash) -> Result<bool, PasswordHasherError> {
        let parsed = A2Hash::new(hash.as_str())
            .map_err(|e| PasswordHasherError::VerifyFailed(e.to_string()))?;
        match self.inner.verify_password(plain.as_bytes(), &parsed) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(PasswordHasherError::VerifyFailed(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_hash_and_verify() {
        let h = Argon2PasswordHasher::new();
        let hash = h.hash("hunter2").unwrap();
        assert!(h.verify("hunter2", &hash).unwrap());
        assert!(!h.verify("wrong", &hash).unwrap());
    }

    #[test]
    fn each_hash_uses_fresh_salt() {
        let h = Argon2PasswordHasher::new();
        let a = h.hash("same").unwrap();
        let b = h.hash("same").unwrap();
        assert_ne!(a.as_str(), b.as_str());
        assert!(h.verify("same", &a).unwrap());
        assert!(h.verify("same", &b).unwrap());
    }
}
