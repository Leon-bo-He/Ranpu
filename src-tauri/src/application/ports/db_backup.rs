use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbBackupError {
    #[error("数据库快照读写错误：{0}")]
    Io(String),
    #[error("数据库快照失败：{0}")]
    Backend(String),
}

/// SQLCipher VACUUM INTO 抽象（PROMPT 第 140 行 EncryptedExporter 内部调用）。
///
/// 返回临时文件的字节流，由调用方继续走加密导出器。这样让加密管道与
/// SQLite 细节解耦，application 层不需要 import rusqlite。
pub trait DbSnapshot: Send + Sync {
    fn snapshot_bytes(&self) -> Result<Vec<u8>, DbBackupError>;
    /// 用快照字节流原子替换当前数据库（导入用）。
    fn restore_bytes(&self, bytes: &[u8]) -> Result<(), DbBackupError>;
}
