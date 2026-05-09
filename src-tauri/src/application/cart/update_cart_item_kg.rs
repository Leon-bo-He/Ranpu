use std::str::FromStr;

use crate::application::cart::service::CartService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::errors::RepositoryError;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::cart::cart_item::SourceKind;
use crate::domain::formula::amounts::Kilograms;
use crate::domain::shared::id::FormulaId;

#[derive(Debug, Clone)]
pub struct UpdateCartItemKgInput {
    pub source_kind: String,
    pub source_formula_id: FormulaId,
    pub target_kg: f64,
}

impl CartService {
    pub fn update_cart_item_kg(&self, input: UpdateCartItemKgInput) -> AppResult<()> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let source_kind = SourceKind::from_str(&input.source_kind)?;
        let target_kg = Kilograms::new(input.target_kg)?;
        let now = self.clock.now();

        let mut cart = self.cart_repo.load(workspace_id)?;
        let updated = cart.update_kg(source_kind, input.source_formula_id, target_kg, now);
        if !updated {
            return Err(AppError::Repository(RepositoryError::NotFound));
        }
        self.cart_repo.save(&cart)?;

        let event = AuditEvent::new(
            Some(workspace_id),
            Action::CartItemKgUpdated,
            Some(format!(
                "{}:{}",
                source_kind.as_db_str(),
                input.source_formula_id
            )),
            Some(format!("target_kg={:.2}", target_kg.value())),
            now,
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
