use std::sync::Arc;

use crate::application::ports::{
    AuditWriter, Clock, DefaultFormulaRepository, SessionStore, WorkspaceFormulaRepository,
};

#[derive(Clone)]
pub struct FormulaService {
    pub(super) default_repo: Arc<dyn DefaultFormulaRepository>,
    pub(super) workspace_repo: Arc<dyn WorkspaceFormulaRepository>,
    pub(super) audit_writer: Arc<dyn AuditWriter>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) session_store: Arc<dyn SessionStore>,
}

impl FormulaService {
    pub fn new(
        default_repo: Arc<dyn DefaultFormulaRepository>,
        workspace_repo: Arc<dyn WorkspaceFormulaRepository>,
        audit_writer: Arc<dyn AuditWriter>,
        clock: Arc<dyn Clock>,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            default_repo,
            workspace_repo,
            audit_writer,
            clock,
            session_store,
        }
    }
}

/// 共享 Input：用于 upsert default / workspace 两种聚合根的共同字段。
#[derive(Debug, Clone)]
pub struct FormulaUpsertInput {
    pub id: Option<crate::domain::shared::id::FormulaId>,
    pub internal_color_code: String,
    pub customer_color_code: Option<String>,
    pub color_name: Option<String>,
    pub description: Option<String>,
    pub base_weight_kg: Option<f64>,
    pub liquor_ratio: Option<f64>,
    pub notes: Option<String>,
    pub items: Vec<FormulaItemInput>,
}

#[derive(Debug, Clone)]
pub struct FormulaItemInput {
    pub dye_name: String,
    pub dye_code: Option<String>,
    pub amount: f64,
    pub unit: String,
    pub sort_order: u16,
}
