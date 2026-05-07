-- ===========================================================
-- 染谱 Ranpu - SQLCipher schema
-- 与 PROMPT 第 156-202 行 完全对齐。
-- 所有时间列存 RFC3339 字符串（chrono 默认 TEXT 序列化）。
-- ===========================================================

PRAGMA foreign_keys = ON;

-- ---------- 用户 ----------
CREATE TABLE IF NOT EXISTS users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    username        TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    role            TEXT NOT NULL CHECK(role IN ('admin','user')),
    is_active       INTEGER NOT NULL DEFAULT 1,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until    TEXT,
    created_at      TEXT NOT NULL,
    last_login      TEXT
);

-- ---------- 工作区 ----------
CREATE TABLE IF NOT EXISTS workspaces (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT NOT NULL UNIQUE,
    description         TEXT,
    created_by_user_id  INTEGER REFERENCES users(id) ON DELETE SET NULL,
    created_at          TEXT NOT NULL
);

-- ---------- 默认配方库 ----------
CREATE TABLE IF NOT EXISTS default_formulas (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    internal_color_code TEXT NOT NULL UNIQUE,
    customer_color_code TEXT,
    color_name          TEXT,
    description         TEXT,
    base_weight_kg      REAL,
    liquor_ratio        REAL,
    notes               TEXT,
    created_by_user_id  INTEGER REFERENCES users(id) ON DELETE SET NULL,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS default_formula_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    formula_id  INTEGER NOT NULL REFERENCES default_formulas(id) ON DELETE CASCADE,
    dye_name    TEXT NOT NULL,
    dye_code    TEXT,
    percentage  REAL NOT NULL,
    unit        TEXT NOT NULL CHECK(unit IN ('pct_owf','g_per_kg','g_per_L')),
    sort_order  INTEGER NOT NULL DEFAULT 0
);

-- ---------- 工作区配方 ----------
CREATE TABLE IF NOT EXISTS workspace_formulas (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id        INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    internal_color_code TEXT NOT NULL,
    customer_color_code TEXT,
    color_name          TEXT,
    description         TEXT,
    base_weight_kg      REAL,
    liquor_ratio        REAL,
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
    unit        TEXT NOT NULL CHECK(unit IN ('pct_owf','g_per_kg','g_per_L')),
    sort_order  INTEGER NOT NULL DEFAULT 0
);

-- ---------- 购物车 ----------
CREATE TABLE IF NOT EXISTS cart_items (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id             INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workspace_id        INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    source_kind         TEXT NOT NULL CHECK(source_kind IN ('default','workspace')),
    source_formula_id   INTEGER NOT NULL,
    target_kg           REAL NOT NULL CHECK(target_kg > 0),
    added_at            TEXT NOT NULL,
    UNIQUE(user_id, workspace_id, source_kind, source_formula_id)
);

-- ---------- 审计日志 ----------
CREATE TABLE IF NOT EXISTS audit_log (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    event_uuid              TEXT NOT NULL UNIQUE,
    user_id                 INTEGER REFERENCES users(id) ON DELETE SET NULL,
    workspace_context_id    INTEGER REFERENCES workspaces(id) ON DELETE SET NULL,
    action                  TEXT NOT NULL,
    target                  TEXT,
    details                 TEXT,
    occurred_at             TEXT NOT NULL
);

-- ---------- 索引 ----------
CREATE INDEX IF NOT EXISTS idx_workspace_formulas_ws_internal
    ON workspace_formulas(workspace_id, internal_color_code);
CREATE INDEX IF NOT EXISTS idx_workspace_formulas_ws_customer
    ON workspace_formulas(workspace_id, customer_color_code);
CREATE INDEX IF NOT EXISTS idx_default_formulas_internal
    ON default_formulas(internal_color_code);
CREATE INDEX IF NOT EXISTS idx_default_formulas_customer
    ON default_formulas(customer_color_code);
CREATE INDEX IF NOT EXISTS idx_cart_user_workspace
    ON cart_items(user_id, workspace_id);
CREATE INDEX IF NOT EXISTS idx_audit_user_time
    ON audit_log(user_id, occurred_at);
