use chrono::{DateTime, Utc};

use crate::application::ports::clock::Clock;

/// 系统墙钟实现。
pub struct SystemClock;

impl SystemClock {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
