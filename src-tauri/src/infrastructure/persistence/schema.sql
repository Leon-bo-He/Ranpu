-- ===========================================================
-- 染谱 Ranpu - SQLCipher schema (单用户解锁模型)
--
-- 不维护 users / 角色 / 登录 — 一道应用解锁密码即进系统.
-- 配方只保留 4 个核心字段 + 染料明细:
--   internal_color_code (内部色号), customer_color_code (客户色号),
--   color_family (色系, dropdown 可选已有或输入新), notes (备注),
--   外加 default/workspace_formula_items 表里的染料明细.
-- 计算单位只剩 pct_owf / g_per_kg, g_per_L 因没了 liquor_ratio 一并去掉.
-- 所有时间列存 RFC3339 字符串 (chrono 默认 TEXT 序列化).
-- 老 v1.0.x DB 由 connection.rs 的 run_migrations 自动平迁.
-- ===========================================================

PRAGMA foreign_keys = ON;

-- ---------- 工作区 ----------
CREATE TABLE IF NOT EXISTS workspaces (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT NOT NULL UNIQUE,
    description  TEXT,
    created_at   TEXT NOT NULL,
    kind         TEXT NOT NULL DEFAULT 'normal' CHECK(kind IN ('normal','system_mirror'))
);

-- ---------- 默认配方库 ----------
CREATE TABLE IF NOT EXISTS default_formulas (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    internal_color_code TEXT NOT NULL UNIQUE,
    customer_color_code TEXT,
    color_family        TEXT,
    notes               TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS default_formula_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    formula_id  INTEGER NOT NULL REFERENCES default_formulas(id) ON DELETE CASCADE,
    dye_name    TEXT NOT NULL,
    dye_code    TEXT,
    percentage  REAL NOT NULL,
    unit        TEXT NOT NULL CHECK(unit IN ('pct_owf','g_per_kg')),
    sort_order  INTEGER NOT NULL DEFAULT 0
);

-- ---------- 工作区配方 ----------
CREATE TABLE IF NOT EXISTS workspace_formulas (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id        INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    internal_color_code TEXT NOT NULL,
    customer_color_code TEXT,
    color_family        TEXT,
    notes               TEXT,
    source_default_id   INTEGER REFERENCES default_formulas(id) ON DELETE SET NULL,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    UNIQUE(workspace_id, internal_color_code)
);

CREATE TABLE IF NOT EXISTS workspace_formula_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    formula_id  INTEGER NOT NULL REFERENCES workspace_formulas(id) ON DELETE CASCADE,
    dye_name    TEXT NOT NULL,
    dye_code    TEXT,
    percentage  REAL NOT NULL,
    unit        TEXT NOT NULL CHECK(unit IN ('pct_owf','g_per_kg')),
    sort_order  INTEGER NOT NULL DEFAULT 0
);

-- ---------- 批次清单 ----------
CREATE TABLE IF NOT EXISTS cart_items (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id      INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    source_kind       TEXT NOT NULL CHECK(source_kind IN ('default','workspace')),
    source_formula_id INTEGER NOT NULL,
    target_kg         REAL NOT NULL CHECK(target_kg > 0),
    added_at          TEXT NOT NULL,
    UNIQUE(workspace_id, source_kind, source_formula_id)
);

-- ---------- 审计日志 ----------
CREATE TABLE IF NOT EXISTS audit_log (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    event_uuid           TEXT NOT NULL UNIQUE,
    workspace_context_id INTEGER REFERENCES workspaces(id) ON DELETE SET NULL,
    action               TEXT NOT NULL,
    target               TEXT,
    details              TEXT,
    occurred_at          TEXT NOT NULL
);

-- ---------- 索引 ----------
-- color_family 索引在 connection.rs run_migrations 里建, 因为 schema.sql
-- 这一段在老 DB 上跑时, ALTER TABLE ADD COLUMN color_family 还没执行,
-- 直接 CREATE INDEX ON ...(color_family) 会 "no such column" 报错.
CREATE INDEX IF NOT EXISTS idx_workspace_formulas_ws_internal
    ON workspace_formulas(workspace_id, internal_color_code);
CREATE INDEX IF NOT EXISTS idx_workspace_formulas_ws_customer
    ON workspace_formulas(workspace_id, customer_color_code);
CREATE INDEX IF NOT EXISTS idx_default_formulas_internal
    ON default_formulas(internal_color_code);
CREATE INDEX IF NOT EXISTS idx_default_formulas_customer
    ON default_formulas(customer_color_code);
CREATE INDEX IF NOT EXISTS idx_cart_workspace
    ON cart_items(workspace_id);
CREATE INDEX IF NOT EXISTS idx_audit_time
    ON audit_log(occurred_at);
