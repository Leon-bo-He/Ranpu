use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::application::session_guard::ensure_active;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::FormulaId;

impl FormulaService {
    pub fn delete_default_formula(&self, id: FormulaId) -> AppResult<()> {
        let _ = ensure_active(&*self.session_store)?;
        // 先取出 internal_color_code 以便随后镜像删除 (默认库 delete 后就拿不到了).
        let internal = self
            .default_repo
            .find_by_id(id)?
            .map(|f| {
                <crate::domain::formula::default_formula::DefaultFormula
                    as crate::domain::calculation::dye_calculator::CalculableFormula>::internal_color_code(&f)
                    .clone()
            });
        self.default_repo.delete(id)?;
        if let Some(code) = internal {
            self.mirror_default_delete_by_internal(&code)?;
        }
        let event = AuditEvent::new(
            None,
            Action::DefaultFormulaDeleted,
            Some(id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
