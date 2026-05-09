//! 开发环境专用大体量种子: 255 条默认配方 + 20 个客户工作区, 每个客户
//! 5-20 条带 customer color code 的工作区配方.
//!
//! ⚠️ 本模块整文件被 `#[cfg(feature = "dev-seed")]` 网住, 默认 feature
//! 关闭, 因此 `tauri build` / `cargo build --release` 都不会把它编进
//! 生产构件里. 启动时还要再加一道 `RANPU_DEV_SEED=1` 环境变量门槛,
//! 防止开发模式下意外灌库.
//!
//! 触发方式:
//! ```bash
//! RANPU_DEV_SEED=1 cargo tauri dev --features dev-seed
//! ```
//! 二次启动幂等: 默认配方 / 客户配方先按 internal_color_code 查存在,
//! 已有就跳过 (repo.upsert 在 id=None 时走 INSERT, 直接 upsert 会撞
//! UNIQUE 约束). 工作区按名跳过.
//!
//! 这些数据完全确定 (无随机种子), 同一次构建多次运行结果一致.
//!
//! 不写测试: 整模块仅在开发 feature 下编译, 跑一次能进库就达成目的.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::application::ports::default_formula_repository::DefaultFormulaRepository;
use crate::application::ports::errors::RepositoryError;
use crate::application::ports::workspace_formula_repository::WorkspaceFormulaRepository;
use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::unit::Unit;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::workspace::workspace::{Workspace, WorkspaceName};

const TARGET_DEFAULT_FORMULAS: usize = 255;
const TARGET_WORKSPACES: usize = 20;

/// 入口: 灌默认库, 再灌 20 个客户工作区. 任一阶段失败即向上抛.
pub fn run(
    workspace_repo: &Arc<dyn WorkspaceRepository>,
    default_repo: &Arc<dyn DefaultFormulaRepository>,
    workspace_formula_repo: &Arc<dyn WorkspaceFormulaRepository>,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    seed_defaults(default_repo.as_ref(), now)?;
    seed_customers(
        workspace_repo.as_ref(),
        workspace_formula_repo.as_ref(),
        now,
    )?;
    Ok(())
}

// =====================================================================
// 默认配方库 — 255 条
// =====================================================================

fn seed_defaults(
    repo: &dyn DefaultFormulaRepository,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    let mut count = 0usize;
    for family in FAMILIES {
        for n in 1..=family.size {
            let code = InternalColorCode::new(format!("{}-{:03}", family.prefix, n))
                .expect("dev seed internal code valid");
            if repo.find_by_internal_code(&code)?.is_some() {
                count += 1;
                continue;
            }
            let formula = build_default(family, n, now);
            repo.upsert(&formula)?;
            count += 1;
        }
    }
    debug_assert_eq!(count, TARGET_DEFAULT_FORMULAS);
    Ok(())
}

#[derive(Clone, Copy)]
struct Family {
    prefix: &'static str,
    /// 色系名 (1.0.7+ 字段 color_family)
    family_name: &'static str,
    size: usize,
    palette: u8,
}

const FAMILIES: &[Family] = &[
    Family { prefix: "RD", family_name: "红色系", size: 40, palette: 0 },
    Family { prefix: "OD", family_name: "橙色系", size: 30, palette: 1 },
    Family { prefix: "YD", family_name: "黄色系", size: 35, palette: 2 },
    Family { prefix: "GD", family_name: "绿色系", size: 40, palette: 3 },
    Family { prefix: "BD", family_name: "蓝色系", size: 50, palette: 4 },
    Family { prefix: "PD", family_name: "紫色系", size: 25, palette: 5 },
    Family { prefix: "ND", family_name: "棕色系", size: 20, palette: 6 },
    Family { prefix: "KD", family_name: "黑灰系", size: 15, palette: 7 },
];

fn build_default(family: &Family, n: usize, now: DateTime<Utc>) -> DefaultFormula {
    let internal_code = format!("{}-{:03}", family.prefix, n);
    let items = build_items(family.palette, n);

    let customer_color = if n % 3 == 0 {
        Some(
            CustomerColorCode::new(format!("CUST-{}-{:03}", family.prefix, n))
                .expect("dev seed customer code valid"),
        )
    } else {
        None
    };

    let notes = if n % 5 == 0 {
        Some(format!("活性染料组 {} 风格, 注意 pH.", family.palette))
    } else {
        None
    };

    DefaultFormula::new(
        InternalColorCode::new(internal_code).expect("dev seed internal code valid"),
        customer_color,
        Some(family.family_name.into()),
        notes,
        items,
        now,
    )
    .expect("dev seed default formula invariants ok")
}

/// 按 palette + n 拼 1-3 条 FormulaItem, 覆盖两种 Unit (pct_owf / g_per_kg).
fn build_items(palette: u8, n: usize) -> Vec<FormulaItem> {
    let style = (palette as usize + n) % 3;
    let dyes = palette_dyes(palette);

    let mut items = Vec::new();
    let mut sort = 0u16;

    items.push(
        FormulaItem::new(
            dyes[0].0,
            Some(dyes[0].1.into()),
            0.5 + ((n % 7) as f64) * 0.5,
            Unit::PctOwf,
            sort,
        )
        .expect("dev seed item valid"),
    );
    sort += 1;

    if style >= 1 {
        items.push(
            FormulaItem::new(
                dyes[1].0,
                Some(dyes[1].1.into()),
                0.1 + ((n % 5) as f64) * 0.3,
                Unit::PctOwf,
                sort,
            )
            .expect("dev seed item valid"),
        );
        sort += 1;
    }

    if style >= 2 {
        items.push(
            FormulaItem::new(
                "元明粉",
                None,
                30.0 + ((n % 4) as f64) * 10.0,
                Unit::GramsPerKg,
                sort,
            )
            .expect("dev seed item valid"),
        );
    }

    items
}

fn palette_dyes(palette: u8) -> [(&'static str, &'static str); 2] {
    match palette % 8 {
        0 => [
            ("C.I. Reactive Red 195", "RR195"),
            ("C.I. Reactive Yellow 145", "RY145"),
        ],
        1 => [
            ("C.I. Reactive Orange 16", "RO16"),
            ("C.I. Reactive Yellow 145", "RY145"),
        ],
        2 => [
            ("C.I. Reactive Yellow 145", "RY145"),
            ("C.I. Reactive Orange 16", "RO16"),
        ],
        3 => [
            ("C.I. Reactive Blue 19", "RB19"),
            ("C.I. Reactive Yellow 145", "RY145"),
        ],
        4 => [
            ("C.I. Reactive Blue 19", "RB19"),
            ("C.I. Reactive Black 5", "RB5"),
        ],
        5 => [
            ("C.I. Reactive Red 195", "RR195"),
            ("C.I. Reactive Blue 19", "RB19"),
        ],
        6 => [
            ("C.I. Reactive Brown 18", "RBr18"),
            ("C.I. Reactive Yellow 145", "RY145"),
        ],
        _ => [
            ("C.I. Reactive Black 5", "RB5"),
            ("C.I. Reactive Blue 19", "RB19"),
        ],
    }
}

// =====================================================================
// 客户工作区 — 20 个 × 5-20 条
// =====================================================================

fn seed_customers(
    workspace_repo: &dyn WorkspaceRepository,
    workspace_formula_repo: &dyn WorkspaceFormulaRepository,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    for i in 1..=TARGET_WORKSPACES {
        let name = format!("Dev客户{:02}", i);
        let workspace = match workspace_repo.find_by_name(&name)? {
            Some(w) => w,
            None => {
                let w = Workspace::new(
                    WorkspaceName::new(&name).expect("dev seed ws name valid"),
                    Some(format!("开发种子客户 #{:02}, 用于配方分页 / 搜索压测.", i)),
                    now,
                )
                .expect("dev seed workspace invariants ok");
                let id = workspace_repo.insert(&w)?;
                workspace_repo
                    .find_by_id(id)?
                    .ok_or_else(|| RepositoryError::Backend("dev seed: 工作区写后取不到".into()))?
            }
        };
        let workspace_id = workspace
            .id()
            .ok_or_else(|| RepositoryError::Backend("dev seed: 工作区缺 id".into()))?;

        let count = 5 + (i * 7) % 16;
        for j in 1..=count {
            let code = InternalColorCode::new(format!("WS{:02}-{:03}", i, j))
                .expect("dev seed ws internal code valid");
            if workspace_formula_repo
                .find_by_internal_code(workspace_id, &code)?
                .is_some()
            {
                continue;
            }
            let formula = build_workspace_formula(i, j, workspace_id, now);
            workspace_formula_repo.upsert(&formula)?;
        }
    }
    Ok(())
}

fn build_workspace_formula(
    customer_idx: usize,
    formula_idx: usize,
    workspace_id: crate::domain::shared::id::WorkspaceId,
    now: DateTime<Utc>,
) -> WorkspaceFormula {
    let internal = format!("WS{:02}-{:03}", customer_idx, formula_idx);
    let customer_code = format!("C{:02}-{:04}", customer_idx, formula_idx * 17 % 9999);
    let palette = ((customer_idx + formula_idx) % 8) as u8;
    let items = build_items(palette, formula_idx);

    // family 跟着 palette 走, 保持一致.
    let family = FAMILIES[palette as usize % FAMILIES.len()].family_name;

    let notes = if formula_idx % 6 == 0 {
        Some("客户车间偏酸性水, 染色后需充分皂洗.".to_string())
    } else {
        None
    };

    WorkspaceFormula::new(
        workspace_id,
        InternalColorCode::new(internal).expect("dev seed ws internal code valid"),
        Some(CustomerColorCode::new(customer_code).expect("dev seed customer code valid")),
        Some(family.into()),
        notes,
        items,
        None,
        now,
    )
    .expect("dev seed workspace formula invariants ok")
}
