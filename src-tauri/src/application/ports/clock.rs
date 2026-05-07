use chrono::{DateTime, Utc};

/// 抽象时钟，便于测试时注入固定时间。
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}
