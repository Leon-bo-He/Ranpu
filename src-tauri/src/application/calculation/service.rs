use std::sync::Arc;

use crate::application::ports::{
    AuditWriter, Clock, DefaultFormulaRepository, SessionStore, WorkspaceFormulaRepository,
};
use crate::domain::calculation::dye_calculator::DyeCalculator;

#[derive(Clone)]
pub struct CalculationService {
    pub(super) default_repo: Arc<dyn DefaultFormulaRepository>,
    pub(super) workspace_repo: Arc<dyn WorkspaceFormulaRepository>,
    pub(super) calculator: Arc<dyn DyeCalculator>,
    pub(super) audit_writer: Arc<dyn AuditWriter>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) session_store: Arc<dyn SessionStore>,
}

impl CalculationService {
    pub fn new(
        default_repo: Arc<dyn DefaultFormulaRepository>,
        workspace_repo: Arc<dyn WorkspaceFormulaRepository>,
        calculator: Arc<dyn DyeCalculator>,
        audit_writer: Arc<dyn AuditWriter>,
        clock: Arc<dyn Clock>,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            default_repo,
            workspace_repo,
            calculator,
            audit_writer,
            clock,
            session_store,
        }
    }
}
