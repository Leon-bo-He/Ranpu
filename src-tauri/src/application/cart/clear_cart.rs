use crate::application::cart::service::CartService;
use crate::application::errors::AppResult;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::audit::audit_event::{Action, AuditEvent};

impl CartService {
    pub fn clear_cart(&self) -> AppResult<()> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let mut cart = self.cart_repo.load(workspace_id)?;
        if cart.is_empty() {
            return Ok(());
        }
        cart.clear();
        self.cart_repo.save(&cart)?;
        let event = AuditEvent::new(
            Some(workspace_id),
            Action::CartCleared,
            None,
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
