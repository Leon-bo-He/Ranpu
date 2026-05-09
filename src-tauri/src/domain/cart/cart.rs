use chrono::{DateTime, Utc};

use crate::domain::cart::cart_item::{CartItem, SourceKind};
use crate::domain::formula::amounts::Kilograms;
use crate::domain::shared::id::{FormulaId, WorkspaceId};

/// Cart 聚合根。
///
/// 单用户解锁模型: 复合键就是 workspace_id 一个 — 切 workspace 看到不同购物车.
///
/// 不变量：同一 cart 不能重复添加同一 (source_kind, source_formula_id)，
/// 二次添加视作更新 target_kg.
#[derive(Debug, Clone, PartialEq)]
pub struct Cart {
    workspace_id: WorkspaceId,
    items: Vec<CartItem>,
}

/// `add_or_update` 的返回值，让上层用例知道是新增还是更新（用于审计写入）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartChange {
    Added,
    UpdatedKg,
}

impl Cart {
    pub fn new(workspace_id: WorkspaceId) -> Self {
        Self {
            workspace_id,
            items: Vec::new(),
        }
    }

    pub fn rehydrate(workspace_id: WorkspaceId, items: Vec<CartItem>) -> Self {
        Self {
            workspace_id,
            items,
        }
    }

    pub fn workspace_id(&self) -> WorkspaceId {
        self.workspace_id
    }
    pub fn items(&self) -> &[CartItem] {
        &self.items
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 加入或更新条目：若同 (source_kind, source_formula_id) 已存在，
    /// 更新它的 target_kg 与 added_at；否则追加新条目。
    pub fn add_or_update(
        &mut self,
        source_kind: SourceKind,
        source_formula_id: FormulaId,
        target_kg: Kilograms,
        now: DateTime<Utc>,
    ) -> CartChange {
        if let Some(existing) = self
            .items
            .iter_mut()
            .find(|i| i.matches(source_kind, source_formula_id))
        {
            existing.update_target_kg(target_kg, now);
            CartChange::UpdatedKg
        } else {
            self.items
                .push(CartItem::new(source_kind, source_formula_id, target_kg, now));
            CartChange::Added
        }
    }

    /// 仅更新已存在的条目；不存在返回 false。
    pub fn update_kg(
        &mut self,
        source_kind: SourceKind,
        source_formula_id: FormulaId,
        target_kg: Kilograms,
        now: DateTime<Utc>,
    ) -> bool {
        if let Some(existing) = self
            .items
            .iter_mut()
            .find(|i| i.matches(source_kind, source_formula_id))
        {
            existing.update_target_kg(target_kg, now);
            true
        } else {
            false
        }
    }

    /// 移除条目；不存在返回 false。
    pub fn remove(&mut self, source_kind: SourceKind, source_formula_id: FormulaId) -> bool {
        let before = self.items.len();
        self.items
            .retain(|i| !i.matches(source_kind, source_formula_id));
        before != self.items.len()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(s: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(s, 0).unwrap()
    }

    fn kg(v: f64) -> Kilograms {
        Kilograms::new(v).unwrap()
    }

    fn make() -> Cart {
        Cart::new(WorkspaceId::new(2))
    }

    #[test]
    fn new_cart_is_empty() {
        let c = make();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn add_or_update_appends_new_item() {
        let mut c = make();
        let r = c.add_or_update(SourceKind::Default, FormulaId::new(7), kg(10.0), t(100));
        assert_eq!(r, CartChange::Added);
        assert_eq!(c.len(), 1);
        assert_eq!(c.items()[0].target_kg().value(), 10.0);
    }

    #[test]
    fn second_add_with_same_key_updates_kg_not_pushes_new() {
        let mut c = make();
        c.add_or_update(SourceKind::Default, FormulaId::new(7), kg(10.0), t(100));
        let r = c.add_or_update(SourceKind::Default, FormulaId::new(7), kg(25.0), t(200));
        assert_eq!(r, CartChange::UpdatedKg);
        assert_eq!(c.len(), 1);
        assert_eq!(c.items()[0].target_kg().value(), 25.0);
        assert_eq!(c.items()[0].added_at(), t(200));
    }

    #[test]
    fn workspace_and_default_with_same_id_coexist() {
        let mut c = make();
        c.add_or_update(SourceKind::Default, FormulaId::new(7), kg(1.0), t(0));
        c.add_or_update(SourceKind::Workspace, FormulaId::new(7), kg(2.0), t(0));
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn update_kg_returns_false_when_missing() {
        let mut c = make();
        assert!(!c.update_kg(SourceKind::Default, FormulaId::new(99), kg(5.0), t(0)));
    }

    #[test]
    fn update_kg_changes_existing_item() {
        let mut c = make();
        c.add_or_update(SourceKind::Workspace, FormulaId::new(3), kg(1.0), t(0));
        assert!(c.update_kg(SourceKind::Workspace, FormulaId::new(3), kg(7.5), t(50)));
        assert_eq!(c.items()[0].target_kg().value(), 7.5);
        assert_eq!(c.items()[0].added_at(), t(50));
    }

    #[test]
    fn remove_returns_true_when_removed_false_when_missing() {
        let mut c = make();
        c.add_or_update(SourceKind::Default, FormulaId::new(1), kg(1.0), t(0));
        assert!(c.remove(SourceKind::Default, FormulaId::new(1)));
        assert!(!c.remove(SourceKind::Default, FormulaId::new(1)));
        assert!(c.is_empty());
    }

    #[test]
    fn clear_drops_all_items() {
        let mut c = make();
        c.add_or_update(SourceKind::Default, FormulaId::new(1), kg(1.0), t(0));
        c.add_or_update(SourceKind::Workspace, FormulaId::new(1), kg(2.0), t(0));
        c.clear();
        assert!(c.is_empty());
    }
}
