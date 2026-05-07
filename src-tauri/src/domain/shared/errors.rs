//! 跨上下文共享的领域错误。
//!
//! identity 上下文有自己的 `IdentityError`（位于 `domain/identity/errors.rs`），
//! 其它上下文的领域校验错误统一归到 `DomainError`。

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    // ---- formula: 内部色号 ----
    #[error("内部色号不能为空")]
    InternalColorCodeEmpty,
    #[error("内部色号最长 32 个字符（当前 {len}）")]
    InternalColorCodeTooLong { len: usize },
    #[error("内部色号不能包含空白字符")]
    InternalColorCodeHasWhitespace,

    // ---- formula: 客户色号 ----
    #[error("客户色号最长 64 个字符（当前 {len}）")]
    CustomerColorCodeTooLong { len: usize },
    #[error("客户色号不能为空字符串（如不需要请用 None）")]
    CustomerColorCodeEmpty,

    // ---- formula: 单位 ----
    #[error("未知的染料投料单位：{0}")]
    UnknownUnit(String),

    // ---- formula: 浴比 ----
    #[error("浴比必须是有限数")]
    LiquorRatioNotFinite,
    #[error("浴比必须大于 0（当前 {actual}）")]
    LiquorRatioMustBePositive { actual: f64 },
    #[error("配方含 g/L 单位的染料，但没有设置浴比")]
    LiquorRatioRequired,

    // ---- formula: 数量 ----
    #[error("百分比必须是有限数")]
    PercentageNotFinite,
    #[error("百分比必须大于 0（当前 {actual}）")]
    PercentageMustBePositive { actual: f64 },
    #[error("克数不能为负（当前 {actual}）")]
    GramsNegative { actual: f64 },
    #[error("克数必须是有限数")]
    GramsNotFinite,
    #[error("kg 数必须在 0.01 到 99999.99 之间（当前 {actual}）")]
    KilogramsOutOfRange { actual: f64 },
    #[error("染料投料量必须大于 0")]
    DyeAmountMustBePositive,
    #[error("染料投料量必须是有限数")]
    DyeAmountNotFinite,

    // ---- formula: item / 配方聚合 ----
    #[error("染料名称不能为空")]
    DyeNameEmpty,
    #[error("染料名称最长 64 个字符（当前 {len}）")]
    DyeNameTooLong { len: usize },
    #[error("染料编号最长 32 个字符（当前 {len}）")]
    DyeCodeTooLong { len: usize },
    #[error("配方至少要有一种染料")]
    FormulaMustHaveAtLeastOneItem,
    #[error("颜色俗称最长 64 个字符（当前 {len}）")]
    ColorNameTooLong { len: usize },
    #[error("配方说明最长 1024 个字符（当前 {len}）")]
    DescriptionTooLong { len: usize },

    // ---- workspace ----
    #[error("工作区名称不能为空")]
    WorkspaceNameEmpty,
    #[error("工作区名称最长 64 个字符（当前 {len}）")]
    WorkspaceNameTooLong { len: usize },

    // ---- cart ----
    #[error("购物车 kg 数必须大于 0")]
    CartTargetKgMustBePositive,
    #[error("未知的购物车配方来源类型：{0}")]
    UnknownSourceKind(String),

    // ---- audit ----
    #[error("审计日志的开始时间不能晚于结束时间")]
    AuditDateRangeInvalid,
}

pub type DomainResult<T> = Result<T, DomainError>;
