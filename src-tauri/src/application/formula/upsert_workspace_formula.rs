use crate::application::errors::AppResult;
use crate::application::formula::parse::parse_upsert;
use crate::application::formula::service::{FormulaService, FormulaUpsertInput};
use crate::application::session_guard::{ensure_active_workspace, ensure_admin};
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::shared::id::FormulaId;

impl FormulaService {
    pub fn upsert_workspace_formula(
        &self,
        input: FormulaUpsertInput,
    ) -> AppResult<FormulaId> {
        let snap = ensure_admin(&*self.session_store)?;
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let now = self.clock.now();
        let parsed = parse_upsert(input)?;
        let internal_code_str = parsed.internal.as_str().to_owned();
        let formula = match parsed.id {
            None => WorkspaceFormula::new(
                workspace_id,
                parsed.internal,
                parsed.customer,
                parsed.color_name,
                parsed.description,
                parsed.base_kg,
                parsed.ratio,
                parsed.notes,
                parsed.items,
                None,
                now,
            )?,
            Some(id) => WorkspaceFormula::rehydrate(
                id,
                workspace_id,
                parsed.internal,
                parsed.customer,
                parsed.color_name,
                parsed.description,
                parsed.base_kg,
                parsed.ratio,
                parsed.notes,
                parsed.items,
                None,
                now,
                now,
            )?,
        };
        let id = self.workspace_repo.upsert(&formula)?;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            Some(workspace_id),
            Action::WorkspaceFormulaUpserted,
            Some(internal_code_str),
            None,
            now,
        );
        self.audit_writer.record(&event)?;
        Ok(id)
    }
}
