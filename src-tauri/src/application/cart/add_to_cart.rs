use std::str::FromStr;

use crate::application::cart::service::CartService;
use crate::application::errors::AppResult;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::cart::cart::CartChange;
use crate::domain::cart::cart_item::SourceKind;
use crate::domain::formula::amounts::Kilograms;
use crate::domain::shared::id::FormulaId;

#[derive(Debug, Clone)]
pub struct AddToCartInput {
    pub source_kind: String,
    pub source_formula_id: FormulaId,
    pub target_kg: f64,
}

impl CartService {
    pub fn add_to_cart(&self, input: AddToCartInput) -> AppResult<()> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let source_kind = SourceKind::from_str(&input.source_kind)?;
        let target_kg = Kilograms::new(input.target_kg)?;
        let now = self.clock.now();

        let mut cart = self.cart_repo.load(workspace_id)?;
        let change = cart.add_or_update(source_kind, input.source_formula_id, target_kg, now);
        self.cart_repo.save(&cart)?;

        let action = match change {
            CartChange::Added => Action::CartItemAdded,
            CartChange::UpdatedKg => Action::CartItemKgUpdated,
        };
        let event = AuditEvent::new(
            Some(workspace_id),
            action,
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
