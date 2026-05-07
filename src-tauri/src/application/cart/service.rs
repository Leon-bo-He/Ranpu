use std::sync::Arc;

use crate::application::ports::{
    AuditWriter, BatchSheetExporter, CartRepository, Clock, DefaultFormulaRepository,
    SessionStore, WorkspaceFormulaRepository,
};
use crate::domain::calculation::dye_calculator::DyeCalculator;

#[derive(Clone)]
pub struct CartService {
    pub(super) cart_repo: Arc<dyn CartRepository>,
    pub(super) default_repo: Arc<dyn DefaultFormulaRepository>,
    pub(super) workspace_repo: Arc<dyn WorkspaceFormulaRepository>,
    pub(super) calculator: Arc<dyn DyeCalculator>,
    pub(super) batch_sheet_exporter: Arc<dyn BatchSheetExporter>,
    pub(super) audit_writer: Arc<dyn AuditWriter>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) session_store: Arc<dyn SessionStore>,
}

impl CartService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cart_repo: Arc<dyn CartRepository>,
        default_repo: Arc<dyn DefaultFormulaRepository>,
        workspace_repo: Arc<dyn WorkspaceFormulaRepository>,
        calculator: Arc<dyn DyeCalculator>,
        batch_sheet_exporter: Arc<dyn BatchSheetExporter>,
        audit_writer: Arc<dyn AuditWriter>,
        clock: Arc<dyn Clock>,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            cart_repo,
            default_repo,
            workspace_repo,
            calculator,
            batch_sheet_exporter,
            audit_writer,
            clock,
            session_store,
        }
    }
}
