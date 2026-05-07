//! Infrastructure 层：实现 application 中的 ports（adapters）。
//!
//! SQLCipher、DPAPI、argon2、aes-gcm 都在这里。
//! 子模块在后续 feat/infra-* 分支填充。

pub mod persistence;
