//! Infrastructure 层：实现 application 中的 ports（adapters）。
//!
//! SQLCipher、DPAPI、argon2、aes-gcm 都在这里。

pub mod clock_system;
pub mod crypto;
pub mod export;
pub mod persistence;
pub mod session;
