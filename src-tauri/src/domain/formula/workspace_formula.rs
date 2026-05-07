use chrono::{DateTime, Utc};

use crate::domain::calculation::dye_calculator::CalculableFormula;
use crate::domain::formula::amounts::Kilograms;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::default_formula::{
    check_invariants, normalize_color_name, normalize_description, normalize_notes,
};
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::liquor_ratio::LiquorRatio;
use crate::domain::shared::errors::DomainResult;
use crate::domain::shared::id::{FormulaId, WorkspaceId};

/// 工作区配方聚合根。
///
/// 不变量与 DefaultFormula 一致；额外携带：
/// - `workspace_id`: 所属工作区
/// - `source_default_id`: 若由 admin 从 default 库复制而来，记录来源 ID
///
/// 唯一性：(workspace_id, internal_color_code) 由仓储/索引保障。
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceFormula {
    id: Option<FormulaId>,
    workspace_id: WorkspaceId,
    internal_color_code: InternalColorCode,
    customer_color_code: Option<CustomerColorCode>,
    color_name: Option<String>,
    description: Option<String>,
    base_weight_kg: Option<Kilograms>,
    liquor_ratio: Option<LiquorRatio>,
    notes: Option<String>,
    items: Vec<FormulaItem>,
    source_default_id: Option<FormulaId>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
impl WorkspaceFormula {
    pub fn new(
        workspace_id: WorkspaceId,
        internal_color_code: InternalColorCode,
        customer_color_code: Option<CustomerColorCode>,
        color_name: Option<String>,
        description: Option<String>,
        base_weight_kg: Option<Kilograms>,
        liquor_ratio: Option<LiquorRatio>,
        notes: Option<String>,
        items: Vec<FormulaItem>,
        source_default_id: Option<FormulaId>,
        created_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        let color_name = normalize_color_name(color_name)?;
        let description = normalize_description(description)?;
        let notes = normalize_notes(notes)?;
        check_invariants(&items, liquor_ratio)?;
        Ok(Self {
            id: None,
            workspace_id,
            internal_color_code,
            customer_color_code,
            color_name,
            description,
            base_weight_kg,
            liquor_ratio,
            notes,
            items,
            source_default_id,
            created_at,
            updated_at: created_at,
        })
    }

    pub fn rehydrate(
        id: FormulaId,
        workspace_id: WorkspaceId,
        internal_color_code: InternalColorCode,
        customer_color_code: Option<CustomerColorCode>,
        color_name: Option<String>,
        description: Option<String>,
        base_weight_kg: Option<Kilograms>,
        liquor_ratio: Option<LiquorRatio>,
        notes: Option<String>,
        items: Vec<FormulaItem>,
        source_default_id: Option<FormulaId>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        check_invariants(&items, liquor_ratio)?;
        Ok(Self {
            id: Some(id),
            workspace_id,
            internal_color_code,
            customer_color_code,
            color_name,
            description,
            base_weight_kg,
            liquor_ratio,
            notes,
            items,
            source_default_id,
            created_at,
            updated_at,
        })
    }

    pub fn id(&self) -> Option<FormulaId> {
        self.id
    }
    pub fn workspace_id(&self) -> WorkspaceId {
        self.workspace_id
    }
    pub fn customer_color_code(&self) -> Option<&CustomerColorCode> {
        self.customer_color_code.as_ref()
    }
    pub fn color_name(&self) -> Option<&str> {
        self.color_name.as_deref()
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn base_weight_kg(&self) -> Option<Kilograms> {
        self.base_weight_kg
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn source_default_id(&self) -> Option<FormulaId> {
        self.source_default_id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn assign_id(&mut self, id: FormulaId) {
        self.id = Some(id);
    }

    pub fn replace_items(
        &mut self,
        items: Vec<FormulaItem>,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        check_invariants(&items, self.liquor_ratio)?;
        self.items = items;
        self.updated_at = now;
        Ok(())
    }

    pub fn set_liquor_ratio(
        &mut self,
        ratio: Option<LiquorRatio>,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        check_invariants(&self.items, ratio)?;
        self.liquor_ratio = ratio;
        self.updated_at = now;
        Ok(())
    }

    pub fn rename_internal_code(
        &mut self,
        code: InternalColorCode,
        now: DateTime<Utc>,
    ) {
        self.internal_color_code = code;
        self.updated_at = now;
    }

    pub fn set_customer_color_code(
        &mut self,
        code: Option<CustomerColorCode>,
        now: DateTime<Utc>,
    ) {
        self.customer_color_code = code;
        self.updated_at = now;
    }
}

impl CalculableFormula for WorkspaceFormula {
    fn internal_color_code(&self) -> &InternalColorCode {
        &self.internal_color_code
    }
    fn liquor_ratio(&self) -> Option<LiquorRatio> {
        self.liquor_ratio
    }
    fn items(&self) -> &[FormulaItem] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::formula::unit::Unit;
    use crate::domain::shared::errors::DomainError;
    use chrono::TimeZone;

    fn t() -> DateTime<Utc> {
        Utc.timestamp_opt(0, 0).unwrap()
    }

    fn pct_item() -> FormulaItem {
        FormulaItem::new("dye", Some("DC".into()), 2.0, Unit::PctOwf, 0).unwrap()
    }

    fn gpl_item() -> FormulaItem {
        FormulaItem::new("salt", None, 5.0, Unit::GramsPerL, 1).unwrap()
    }

    #[test]
    fn new_with_workspace_id_and_source_default_id() {
        let f = WorkspaceFormula::new(
            WorkspaceId::new(7),
            InternalColorCode::new("WK-001").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![pct_item()],
            Some(FormulaId::new(42)),
            t(),
        )
        .unwrap();
        assert_eq!(f.workspace_id(), WorkspaceId::new(7));
        assert_eq!(f.source_default_id(), Some(FormulaId::new(42)));
    }

    #[test]
    fn invariants_apply_same_as_default_formula() {
        let err = WorkspaceFormula::new(
            WorkspaceId::new(7),
            InternalColorCode::new("WK").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![gpl_item()],
            None,
            t(),
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::LiquorRatioRequired));
    }

    #[test]
    fn rename_internal_code_updates_timestamp() {
        let mut f = WorkspaceFormula::new(
            WorkspaceId::new(1),
            InternalColorCode::new("OLD").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![pct_item()],
            None,
            t(),
        )
        .unwrap();
        let later = t() + chrono::Duration::seconds(30);
        f.rename_internal_code(InternalColorCode::new("NEW").unwrap(), later);
        assert_eq!(
            <WorkspaceFormula as CalculableFormula>::internal_color_code(&f).as_str(),
            "NEW",
        );
        assert_eq!(f.updated_at(), later);
    }
}
