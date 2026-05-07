use std::str::FromStr;

use crate::application::cart::service::CartService;
use crate::application::errors::AppResult;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::cart::cart_item::SourceKind;
use crate::domain::shared::id::FormulaId;

#[derive(Debug, Clone)]
pub struct RemoveFromCartInput {
    pub source_kind: String,
    pub source_formula_id: FormulaId,
}

impl CartService {
    pub fn remove_from_cart(&self, input: RemoveFromCartInput) -> AppResult<bool> {
        let (snap, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let source_kind = SourceKind::from_str(&input.source_kind)?;

        let mut cart = self.cart_repo.load(snap.user_id(), workspace_id)?;
        let removed = cart.remove(source_kind, input.source_formula_id);
        if removed {
            self.cart_repo.save(&cart)?;
            let event = AuditEvent::new(
                Some(snap.user_id()),
                Some(workspace_id),
                Action::CartItemRemoved,
                Some(format!(
                    "{}:{}",
                    source_kind.as_db_str(),
                    input.source_formula_id
                )),
                None,
                self.clock.now(),
            );
            self.audit_writer.record(&event)?;
        }
        Ok(removed)
    }
}
