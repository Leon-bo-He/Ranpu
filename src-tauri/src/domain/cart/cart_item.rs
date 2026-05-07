use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::domain::formula::amounts::Kilograms;
use crate::domain::shared::errors::DomainError;
use crate::domain::shared::id::{CartItemId, FormulaId};

/// 购物车条目的配方来源种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKind {
    Default,
    Workspace,
}

impl SourceKind {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            SourceKind::Default => "default",
            SourceKind::Workspace => "workspace",
        }
    }

    pub const fn display_label(self) -> &'static str {
        match self {
            SourceKind::Default => "默认库",
            SourceKind::Workspace => "工作区",
        }
    }
}

impl FromStr for SourceKind {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(SourceKind::Default),
            "workspace" => Ok(SourceKind::Workspace),
            other => Err(DomainError::UnknownSourceKind(other.to_owned())),
        }
    }
}

impl fmt::Display for SourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

/// 购物车条目子实体。
#[derive(Debug, Clone, PartialEq)]
pub struct CartItem {
    id: Option<CartItemId>,
    source_kind: SourceKind,
    source_formula_id: FormulaId,
    target_kg: Kilograms,
    added_at: DateTime<Utc>,
}

impl CartItem {
    pub fn new(
        source_kind: SourceKind,
        source_formula_id: FormulaId,
        target_kg: Kilograms,
        added_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            source_kind,
            source_formula_id,
            target_kg,
            added_at,
        }
    }

    pub fn rehydrate(
        id: CartItemId,
        source_kind: SourceKind,
        source_formula_id: FormulaId,
        target_kg: Kilograms,
        added_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            source_kind,
            source_formula_id,
            target_kg,
            added_at,
        }
    }

    pub fn id(&self) -> Option<CartItemId> {
        self.id
    }
    pub fn source_kind(&self) -> SourceKind {
        self.source_kind
    }
    pub fn source_formula_id(&self) -> FormulaId {
        self.source_formula_id
    }
    pub fn target_kg(&self) -> Kilograms {
        self.target_kg
    }
    pub fn added_at(&self) -> DateTime<Utc> {
        self.added_at
    }

    pub fn assign_id(&mut self, id: CartItemId) {
        self.id = Some(id);
    }

    pub(crate) fn update_target_kg(&mut self, target_kg: Kilograms, now: DateTime<Utc>) {
        self.target_kg = target_kg;
        self.added_at = now;
    }

    /// 复合键判等（不含 Cart 自带的 user/workspace，因为同一 Cart 内已隐含）。
    pub fn matches(&self, source_kind: SourceKind, source_formula_id: FormulaId) -> bool {
        self.source_kind == source_kind && self.source_formula_id == source_formula_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t() -> DateTime<Utc> {
        Utc.timestamp_opt(0, 0).unwrap()
    }

    #[test]
    fn source_kind_round_trips_db_str() {
        for s in [SourceKind::Default, SourceKind::Workspace] {
            assert_eq!(SourceKind::from_str(s.as_db_str()).unwrap(), s);
        }
    }

    #[test]
    fn unknown_source_string_is_rejected() {
        assert!(SourceKind::from_str("private").is_err());
    }

    #[test]
    fn cart_item_matches_by_kind_and_formula_id() {
        let a = CartItem::new(
            SourceKind::Default,
            FormulaId::new(7),
            Kilograms::new(1.0).unwrap(),
            t(),
        );
        assert!(a.matches(SourceKind::Default, FormulaId::new(7)));
        assert!(!a.matches(SourceKind::Workspace, FormulaId::new(7)));
        assert!(!a.matches(SourceKind::Default, FormulaId::new(8)));
    }
}
