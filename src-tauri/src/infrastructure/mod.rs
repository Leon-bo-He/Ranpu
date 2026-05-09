//! Infrastructure 层：实现 application 中的 ports（adapters）。
//!
//! SQLCipher、DPAPI、aes-gcm 都在这里。单用户解锁模型: 没有 argon2 (没有用户口令 hash).

pub mod clock_system;
pub mod crypto;
pub mod export;
pub mod persistence;
pub mod session;
