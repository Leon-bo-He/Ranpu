use crate::application::calculation::formula_resolver::ResolvedFormula;
use crate::application::calculation::service::CalculationService;
use crate::application::errors::AppResult;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::calculation::dye_calculator::CalculationResult;
use crate::domain::formula::amounts::Kilograms;
use crate::domain::formula::internal_color_code::InternalColorCode;

#[derive(Debug, Clone)]
pub struct CalculateDyeAmountsInput {
    pub internal_color_code: String,
    pub target_kg: f64,
}

impl CalculationService {
    pub fn calculate_dye_amounts(
        &self,
        input: CalculateDyeAmountsInput,
    ) -> AppResult<CalculationResult> {
        let (snap, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let code = InternalColorCode::new(input.internal_color_code)?;
        let target = Kilograms::new(input.target_kg)?;
        let resolved = self.resolve_by_internal_code(workspace_id, &code)?;
        let formula_id = match &resolved {
            ResolvedFormula::Workspace(f) => f.id(),
            ResolvedFormula::Default(f) => f.id(),
        };
        let mut result = self.calculator.calculate(
            resolved.as_calculable(),
            target,
            resolved.source(),
        )?;
        result.formula_id = formula_id;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            Some(workspace_id),
            Action::CalculationPerformed,
            Some(code.into_string()),
            Some(format!("target_kg={:.2}", target.value())),
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(result)
    }
}
