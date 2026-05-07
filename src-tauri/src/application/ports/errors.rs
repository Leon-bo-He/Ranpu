use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum RepositoryError {
    #[error("没找到记录")]
    NotFound,
    #[error("数据冲突：{0}")]
    Conflict(String),
    #[error("数据库错误：{0}")]
    Backend(String),
}
