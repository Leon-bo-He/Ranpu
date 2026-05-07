use std::str::FromStr;
use std::sync::Arc;

use rusqlite::{params, Row};

use crate::application::ports::cart_repository::CartRepository;
use crate::application::ports::errors::RepositoryError;
use crate::domain::cart::cart::Cart;
use crate::domain::cart::cart_item::{CartItem, SourceKind};
use crate::domain::formula::amounts::Kilograms;
use crate::domain::shared::id::{CartItemId, FormulaId, UserId, WorkspaceId};
use crate::infrastructure::persistence::sqlcipher::connection::SqliteConnection;
use crate::infrastructure::persistence::sqlcipher::row_mapping::{corrupt, parse_dt, rfc3339};

pub struct SqliteCartRepository {
    db: Arc<SqliteConnection>,
}

impl SqliteCartRepository {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

struct ItemRow {
    id: i64,
    source_kind: String,
    source_formula_id: i64,
    target_kg: f64,
    added_at: String,
}

impl ItemRow {
    fn from_row(r: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            source_kind: r.get(1)?,
            source_formula_id: r.get(2)?,
            target_kg: r.get(3)?,
            added_at: r.get(4)?,
        })
    }

    fn into_domain(self) -> Result<CartItem, RepositoryError> {
        let kind = SourceKind::from_str(&self.source_kind)
            .map_err(|e| corrupt("cart.source_kind", e))?;
        let target_kg =
            Kilograms::new(self.target_kg).map_err(|e| corrupt("cart.target_kg", e))?;
        let added_at = parse_dt(&self.added_at)?;
        Ok(CartItem::rehydrate(
            CartItemId::new(self.id),
            kind,
            FormulaId::new(self.source_formula_id),
            target_kg,
            added_at,
        ))
    }
}

impl CartRepository for SqliteCartRepository {
    fn load(
        &self,
        user_id: UserId,
        workspace_id: WorkspaceId,
    ) -> Result<Cart, RepositoryError> {
        let raws: Vec<ItemRow> = self.db.with(|c| {
            let mut stmt = c.prepare(
                "SELECT id, source_kind, source_formula_id, target_kg, added_at
                 FROM cart_items WHERE user_id = ?1 AND workspace_id = ?2 ORDER BY added_at, id",
            )?;
            let collected: rusqlite::Result<Vec<ItemRow>> = stmt
                .query_map(
                    params![user_id.value(), workspace_id.value()],
                    ItemRow::from_row,
                )?
                .collect();
            collected
        })?;
        let items: Result<Vec<_>, _> = raws.into_iter().map(ItemRow::into_domain).collect();
        Ok(Cart::rehydrate(user_id, workspace_id, items?))
    }

    fn save(&self, cart: &Cart) -> Result<(), RepositoryError> {
        self.db.with_tx(|tx| {
            tx.execute(
                "DELETE FROM cart_items WHERE user_id = ?1 AND workspace_id = ?2",
                params![cart.user_id().value(), cart.workspace_id().value()],
            )?;
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO cart_items
                        (user_id, workspace_id, source_kind, source_formula_id, target_kg, added_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )?;
                for item in cart.items() {
                    stmt.execute(params![
                        cart.user_id().value(),
                        cart.workspace_id().value(),
                        item.source_kind().as_db_str(),
                        item.source_formula_id().value(),
                        item.target_kg().value(),
                        rfc3339(item.added_at()),
                    ])?;
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn setup() -> (Arc<SqliteConnection>, UserId, WorkspaceId) {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        db.with(|c| {
            c.execute(
                "INSERT INTO users (username, password_hash, role, created_at) VALUES ('alice','h','user',?1)",
                params![rfc3339(Utc.timestamp_opt(0, 0).unwrap())],
            )?;
            let user_id = c.last_insert_rowid();
            c.execute(
                "INSERT INTO workspaces (name, created_at) VALUES ('客户A', ?1)",
                params![rfc3339(Utc.timestamp_opt(0, 0).unwrap())],
            )?;
            let ws_id = c.last_insert_rowid();
            Ok((user_id, ws_id))
        })
        .map(|(uid, wid)| (db, UserId::new(uid), WorkspaceId::new(wid)))
        .unwrap()
    }

    #[test]
    fn save_then_load_round_trips_items() {
        let (db, user_id, ws_id) = setup();
        let repo = SqliteCartRepository::new(db);
        let mut cart = Cart::new(user_id, ws_id);
        cart.add_or_update(
            SourceKind::Default,
            FormulaId::new(1),
            Kilograms::new(10.0).unwrap(),
            Utc.timestamp_opt(100, 0).unwrap(),
        );
        cart.add_or_update(
            SourceKind::Workspace,
            FormulaId::new(2),
            Kilograms::new(5.0).unwrap(),
            Utc.timestamp_opt(200, 0).unwrap(),
        );
        repo.save(&cart).unwrap();

        let loaded = repo.load(user_id, ws_id).unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn load_returns_empty_cart_when_no_items() {
        let (db, user_id, ws_id) = setup();
        let repo = SqliteCartRepository::new(db);
        let cart = repo.load(user_id, ws_id).unwrap();
        assert!(cart.is_empty());
    }
}
