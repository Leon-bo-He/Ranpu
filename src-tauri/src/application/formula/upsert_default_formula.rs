use crate::application::errors::AppResult;
use crate::application::formula::parse::parse_upsert;
use crate::application::formula::service::{FormulaService, FormulaUpsertInput};
use crate::application::session_guard::ensure_active;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::shared::id::FormulaId;

impl FormulaService {
    pub fn upsert_default_formula(&self, input: FormulaUpsertInput) -> AppResult<FormulaId> {
        let _ = ensure_active(&*self.session_store)?;
        let now = self.clock.now();
        let parsed = parse_upsert(input)?;
        let internal_code_str = parsed.internal.as_str().to_owned();
        let formula = match parsed.id {
            None => DefaultFormula::new(
                parsed.internal,
                parsed.customer,
                parsed.color_name,
                parsed.description,
                parsed.base_kg,
                parsed.ratio,
                parsed.notes,
                parsed.items,
                now,
            )?,
            Some(id) => DefaultFormula::rehydrate(
                id,
                parsed.internal,
                parsed.customer,
                parsed.color_name,
                parsed.description,
                parsed.base_kg,
                parsed.ratio,
                parsed.notes,
                parsed.items,
                now,
                now,
            )?,
        };
        let id = self.default_repo.upsert(&formula)?;
        // 同步到系统内置 "通用" 镜像工作区.
        if let Some(persisted) = self.default_repo.find_by_id(id)? {
            self.mirror_default_upsert(&persisted, now)?;
        }
        let event = AuditEvent::new(
            None,
            Action::DefaultFormulaUpserted,
            Some(internal_code_str),
            None,
            now,
        );
        self.audit_writer.record(&event)?;
        Ok(id)
    }
}
