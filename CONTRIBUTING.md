# 贡献指南

欢迎给染谱 (Ranpu) 提 Issue 或 Pull Request。本文记录架构约束与协作规范。

---

## 1. 架构铁律 (DDD + Hexagonal)

四层依赖方向 **不可逆**：

```
interfaces  ──>  application  ──>  domain  <──  infrastructure
                                       ^
                                       └── infrastructure 只通过 application 中的 trait 反向依赖 domain
```

具体到 Rust 代码：

| 层 | 允许 import | 禁止 import |
|---|---|---|
| `src-tauri/src/domain/**` | `std`, `chrono`, `thiserror`, `uuid` | 任何其它 crate；任何上层模块 |
| `src-tauri/src/application/**` | `domain::*`, `std`, `chrono`, `thiserror` | `infrastructure::*`, `interfaces::*`, `rusqlite`, `tauri`, `argon2` |
| `src-tauri/src/infrastructure/**` | `application::ports::*`, `domain::*`, 任何外部 crate | `interfaces::*` |
| `src-tauri/src/interfaces/**` | `application::*`, `infrastructure::*` (composition root), `tauri::*` | （无） |

校验方式：`cargo clippy --all-targets -- -D warnings` 全绿即可。如果你不慎在 `domain/` 写了 `use rusqlite::*`，编译就会拒绝（domain Cargo 没引这条 crate）。

值对象保持轻量：

- 只对 PROMPT 第 102–112 行 列出的字段包 newtype。
- 不要对每个 `String` 都包一层。
- 业务规则放在领域对象的方法里（不要写贫血模型）。
- 不要给只有一个实现的 trait 强加抽象层 (例外：仓储 + 加密因为有真/假两种实装)。

---

## 2. 提交规范 (Conventional Commits)

scope 用上下文 / 模块名：

| 模板 | 例子 |
|---|---|
| `feat(<scope>): ...` | `feat(formula): support customer color code search` |
| `fix(<scope>): ...` | `fix(ui): correct disabled state of copy button when no workspace` |
| `refactor(<scope>): ...` | `refactor(repo): extract shared row mapping helper` |
| `chore(deps): ...` | `chore(deps): bump rusqlite to 0.32.2` |
| `test(<scope>): ...` | `test(calc): cover g_per_L unit edge cases` |
| `docs: ...` | `docs: add audit export format spec` |

**不要在 main 直接提交**（零号 `chore: initialize repository`、`docs: …` 是例外）。

---

## 3. 分支约定

- 主线：`main`
- 特性分支：`feat/<scope>-<short-title>`（如 `feat/domain-layer`）
- 修复分支：`fix/<scope>-<short-title>`
- 重构分支：`refactor/<scope>-<short-title>`

合并：永远 `--no-ff`，保留分支拓扑：

```bash
git checkout main
git merge --no-ff feat/<branch> -m "merge: <branch>"
```

分支落地前必须：

```bash
cargo test  --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets --manifest-path src-tauri/Cargo.toml -- -D warnings
npm run typecheck
npm run lint
```

四件套全绿才允许合并。

---

## 4. 测试

| 层 | 测试在哪 | 覆盖什么 |
|---|---|---|
| `domain/` | `#[cfg(test)] mod tests` 内联 | 每个值对象（边界 + 错误路径）、聚合不变量、领域服务（DyeCalculator 三种单位 ≥2 个用例） |
| `infrastructure/persistence/` | 内联 + `SqliteConnection::open_in_memory()` | 仓储 round-trip、唯一约束、审计写读 |
| `infrastructure/crypto/` | 内联 | argon2 / PBKDF2 / AES-GCM round-trip、错口令、篡改文件头 |
| `application/` | 暂无单测；通过端到端 Tauri 命令验证 | — |
| `interfaces/tauri/` | 暂无 | — |

新加领域规则时，**先加测试再加实现**。

---

## 5. UI 文案

所有 UI 文案、错误提示、按钮、日期格式 **必须中文**，措辞通俗（PROMPT 第 5 行）：

✗ 不要：「鉴权失败」「非法实体」「IO 错误」  
✓ 而是：「账号或密码不对」「配方里至少要有一种染料」「文件读写出错: …」

后端的 `AppError` / `IdentityError` / `DomainError` 已经用中文写好；不要在 interfaces 层做二次翻译，直接 `e.to_string()` 透传。

---

## 6. 加密相关 PR 须知

修改 `infrastructure/crypto/` 任何文件，PR 描述必须包含：

1. 是否影响磁盘格式（`.ydaexp` / `keystore.bin`）。如果影响，需要给出迁移方案。
2. 是否改变了 KDF 参数 / 算法。如果是，必须 bump `VERSION` 字段。
3. 是否新增了对外可见的明文存储路径（默认拒绝）。

PR 必须有审阅人对照 README 「加密设计 — 审计要点」 一节核对。

---

## 7. 提交前 checklist

- [ ] 我阅读了 PROMPT.md，新代码符合限界上下文与不变量
- [ ] `cargo test` 全过
- [ ] `cargo clippy --all-targets -- -D warnings` 没有警告
- [ ] `npm run typecheck` `npm run lint` 没有警告
- [ ] 新增 UI 文案是中文且通俗
- [ ] 没有在 domain 引入新的外部依赖
- [ ] commit 消息符合 Conventional Commits
