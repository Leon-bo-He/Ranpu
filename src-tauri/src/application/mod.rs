//! Application 层：定义 ports（trait）+ 编排 use case。
//!
//! 严禁 import infrastructure；只能依赖 domain 与 std/chrono/thiserror/uuid。
//! 单用户解锁模型: 没有 identity 子目录 (没有用户 / 角色 / 登录).

pub mod audit;
pub mod backup;
pub mod calculation;
pub mod cart;
pub mod errors;
pub mod formula;
pub mod ports;
pub mod session_guard;
pub mod sync;
pub mod workspace;

pub use audit::AuditService;
pub use backup::BackupService;
pub use calculation::CalculationService;
pub use cart::CartService;
pub use errors::{AppError, AppResult};
pub use formula::FormulaService;
pub use workspace::WorkspaceService;
