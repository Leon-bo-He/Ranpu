帮我从零搭建一个 Windows 离线染纱配色软件，项目名「染谱」（英文 Ranpu）。
技术栈固定为：
  Tauri 2 + React + TypeScript + Tailwind CSS + shadcn/ui + Rust + rusqlite(bundled-sqlcipher)
架构采用 DDD（领域驱动设计）+ Hexagonal/Ports-and-Adapters。
所有 UI 文案、错误提示、警告、按钮、日期格式 全部用中文，措辞通俗（不要"鉴权失败""非法实体"这种生硬翻译，用"账号或密码不对""配方里至少要有一种染料"这种）。

═══════════════════════════════════════════════════════
【项目品牌与 Logo】
═══════════════════════════════════════════════════════
- 应用名：染谱
- 英文名：Ranpu
- 副标题（用于解锁 / 首次启动页副标题、About 对话框、README 顶部）：DYE FORMULA

- Logo SVG（纯图形，不含任何文字。保存为 src/assets/logo.svg，并复制一份到 src-tauri/icons/source-logo.svg）：

<svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="染谱">
  <defs>
    <linearGradient id="spectrumGradient" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FF4B2B"/>
      <stop offset="50%" stop-color="#6A11CB"/>
      <stop offset="100%" stop-color="#2575FC"/>
    </linearGradient>
  </defs>
  <circle cx="100" cy="100" r="95" fill="none" stroke="#f0f0f0" stroke-width="2"/>
  <g transform="translate(40, 45)">
    <path d="M10,110 C10,10 110,110 110,10"
          stroke="#eee" stroke-width="12" fill="none" stroke-linecap="round"/>
    <path d="M10,110 C10,10 110,110 110,10"
          stroke="url(#spectrumGradient)" stroke-width="10" fill="none"
          stroke-linecap="round"
          stroke-dasharray="200" stroke-dashoffset="0">
      <animate attributeName="stroke-dashoffset" from="200" to="0" dur="2s" fill="freeze"/>
    </path>
    <path d="M10,10 C10,110 110,10 110,110"
          stroke="url(#spectrumGradient)" stroke-width="10" fill="none"
          stroke-linecap="round" opacity="0.6"/>
  </g>
</svg>

- Tauri 应用图标：用上述 SVG 通过 `npm run tauri icon -- src-tauri/icons/source-logo.svg` 生成多尺寸 PNG + Windows ICO，自动放入 src-tauri/icons/，并在 tauri.conf.json 中引用。
- 重要：tauri icon 命令底层用 resvg，不会执行 SMIL 动画，会按 SVG 静态属性光栅化。本 SVG 的 stroke-dashoffset 静态值为 0，因此图标里曲线会是完整画好的状态，不需要单独维护"无动画版"。

- 封装成 React 组件 <RanpuLogo />，props：
  · size: number — 像素尺寸（必填）
  · withText?: boolean — 是否在图形右侧渲染"染谱"中文字（默认 false）。文字用 React <span> 渲染，不要塞进 SVG。字体 var(--font-serif)（思源宋体备选 system-ui serif），字重 500，字间距 3px，颜色用 CSS 变量 var(--color-text-primary)，跟随系统深浅模式。
  · animated?: boolean — 是否播放绘制动画（默认 false）。为 true 时保留 SVG 内的 <animate> 元素；为 false 时组件渲染时移除 <animate>，只保留静态曲线。

- Logo 出现位置（明确传参）：
  · BootScreen / FirstRunSetup 页：<RanpuLogo size={120} withText={false} animated={true} /> 居中，下方另起一行单独渲染"染谱"中文（24px 宋体）+ 再下一行"DYE FORMULA"（10px sans，字间距 2px，颜色 var(--color-text-tertiary)）
  · 主界面顶栏左侧：<RanpuLogo size={28} withText={true} animated={false} />
  · 锁屏遮罩：<RanpuLogo size={80} withText={false} animated={false} /> 居中，下方"染谱"中文小字
  · 关于对话框：<RanpuLogo size={64} withText={true} animated={false} /> + 版本号
  · README.md 顶部：直接引用 src/assets/logo.svg
  · 应用窗口图标 + 任务栏图标 + 系统通知图标：由 tauri icon 命令生成的位图

═══════════════════════════════════════════════════════
【认证 / 解锁模型】
═══════════════════════════════════════════════════════
单用户车间机器人模型 — 不维护用户表 / 角色 / 登录, 只有一道**应用解锁密码**:

- 主密钥: 32 字节随机生成, Windows DPAPI 保护后存 %APPDATA%\Ranpu\keystore.bin.
- 应用解锁密码: 用户脑里 / 自己保管, 不写任何文件. 启动时输入它, 后端用
  PBKDF2-SHA256(主密钥 + 密码, 600k 轮) 派生 SQLCipher PRAGMA key, 解开 ranpu.db.
- 首次启动 (keystore.bin 不存在): 引导用户设置应用解锁密码 (≥ 8 位), 后端生成
  主密钥 + 写 keystore.bin + 派生 PRAGMA key + 创建空 DB. 第一次解锁直接进系统.
- 之后每次启动: 输入正确密码 → 进系统; 输错 → SQLCipher 解密失败 → 后端返回
  AppError::BootPassphraseIncorrect → UI 显示 "解锁密码不对, 请重试". 没有"试 5 次
  锁定"概念 (单用户场景, 用户自己想试多少次都行; 暴力破解者拿到本机文件也得对着
  PBKDF2 600k 轮硬算).
- **不要 users 表, 不要角色, 不要 admin / user 之分**. 任何能进系统的操作都一律
  放开 — 创建 / 删除 / 编辑 default 配方, 导出审计, 都没有权限门.

会话 (Session) 简化成纯内存状态 (InMemorySessionStore):
  AppRunState { locked: bool, active_workspace_id: Option<i64>, last_activity_at: DateTime<Utc> }
登录后默认无激活 workspace; 必须激活某个才能操作 workspace 配方 / 做计算 / 用批次清单.

═══════════════════════════════════════════════════════
【启动 / 解锁体验细节】
═══════════════════════════════════════════════════════
- BootScreen 一个密码输入框 + "解锁" 按钮; 按下后立即进入 loading: 按钮 disabled +
  spinner + 文案 "正在解锁…".
- 解锁失败 → "解锁密码不对, 请重试" (没有"剩余次数"提示).
- 解锁成功 → 进 Workspace 选择 (顶栏 WorkspacePicker 就位, 默认未选择 workspace).
- 首次启动走 FirstRunSetup: 设置 ≥ 8 位解锁密码 + 二次确认 + 提示"密码丢失就开不
  了应用, 数据无法找回". 设完直接进系统.

═══════════════════════════════════════════════════════
【自动锁屏与手动锁屏】
═══════════════════════════════════════════════════════
- 默认空闲 10 分钟自动锁屏 (鼠标 / 键盘无活动); 设置页可调 5 / 10 / 30 / 60 分钟
  或关闭.
- 顶栏右侧固定 [锁定] 按钮, 点击立即锁屏.
- 锁屏语义: 内存状态保留 (active_workspace_id / 批次清单 / 未保存编辑都不丢);
  UI 全屏遮罩 + 染谱 logo + 密码框 + [解锁] 按钮.
- 解锁: 重新输 app 解锁密码 (跟启动那个同一个), 后端 PBKDF2 派生一次 PRAGMA key
  对照 (派生 + 一条 SELECT count(*) 探测), 一致则恢复; 不一致 → "解锁密码不对".
  错多少次都不强制登出 (没有"用户"可登出).
- 锁屏纯前端 + 后端 SessionStore.locked 标志; 锁屏期间所有非 unlock_session 的
  #[tauri::command] 都返回 "会话已锁定" 拒绝执行.

═══════════════════════════════════════════════════════
【DDD 设计要点 — 限界上下文】
═══════════════════════════════════════════════════════
**没有 Identity 上下文** (单用户模型, 解锁密码就够). 应用启动 / 解锁通过
infrastructure 层的 OsKeyStore + key_derivation, application 层只暴露 boot /
unlock_session / lock_session 命令; 不需要领域聚合或值对象.

1) Workspace 上下文
   - 聚合根：Workspace（id, name, description, created_at, kind: Normal | SystemMirror）
   - 值对象：WorkspaceId, WorkspaceName

2) Formula 上下文
   - 聚合根 1：DefaultFormula（含 FormulaItem 子实体）
   - 聚合根 2：WorkspaceFormula（含 FormulaItem 子实体；通过 source_default_id 引用 DefaultFormula 的 ID）
   - 值对象：
     · InternalColorCode（内部色号，1–32 字符，不含空白，全局/工作区内唯一）
     · CustomerColorCode（客户色号，可空，1–64 字符；不强制唯一）
     · ColorName（颜色俗称如「藏青」「玫红」）
     · DyeAmount { value: f64, unit: Unit }
     · Unit 枚举（PctOwf | GramsPerKg | GramsPerL）
     · LiquorRatio（>0 的 f64，按 1:N 中的 N 存）
     · Percentage, Grams, Kilograms
   - 不变量：每个聚合至少 1 个 item；GramsPerL 必须配方有 LiquorRatio；InternalColorCode 在 default 库全局唯一，在 workspace 内按 (workspace_id, internal_color_code) 唯一。

3) Calculation 上下文
   - 领域服务：DyeCalculator
     · 输入：Formula(trait) + Kilograms
     · 输出：Vec<CalculationLine { dye_name, dye_code, grams, unit_used }>
     · 计算规则：
        - pct_owf  → grams = target_kg * 1000 * pct / 100
        - g_per_kg → grams = target_kg * pct
        - g_per_L  → grams = target_kg * liquor_ratio * pct
   - 应用服务 FormulaResolver：按内部色号查询，先查激活 workspace，找不到再 fallback 到 default 库；同时支持按客户色号查询（返回多个匹配让用户选）。

4) Cart 上下文 (UI 文案叫 "批次清单", 代码内部仍叫 cart)
   - 聚合根：Cart (workspace_id 单键, 含 CartItem 集合)
   - 子实体：CartItem { source_kind: Default|Workspace, source_formula_id, target_kg, added_at }
   - 用例：add_to_cart、remove_from_cart、update_cart_item_kg、clear_cart、list_cart_with_calculations、export_cart_as_batch_sheet、preview_cart_as_batch_sheet_html
   - 不变量：同一 workspace 的 cart 不能重复添加同一 (source_kind, source_formula_id)，二次添加视作更新 target_kg；切换 workspace 时显示该 workspace 对应的 cart。

5) Audit 上下文
   - 实体：AuditEvent (workspace_context_id, action, target, details, occurred_at)
   - 领域服务 trait：AuditWriter
   - 应用服务：list_audit_events(filter), export_audit_log(filter, encryption_passphrase)
   - 没有 user_id (单用户车间机器, 主体始终是这台机器的解锁人; 想区分多人操作请上多用户系统).
   - 导出范围筛选: 起止日期 (必填), 动作类型筛选 (可选多选); 输出格式两种:
     · 加密 .ranpu (默认, AES-256-GCM + PBKDF2, 独立口令)
     · 明文 CSV (需二次确认弹窗 "日志包含敏感操作记录, 确定明文导出?")

6) Backup 上下文
   - 领域服务 trait：EncryptedExporter / EncryptedImporter
   - infra 实现：VACUUM INTO 临时文件 + AES-256-GCM + PBKDF2 → .ranpu
   - 文件头：MAGIC(4)='RNP1' | VERSION(1) | SALT(16) | NONCE(12) | 密文+TAG，AAD 用 MAGIC
   - 三种用户场景共用本格式：完整 DB 备份、配方加密分发（库互导）、审计日志加密导出。

═══════════════════════════════════════════════════════
【架构铁律 — 必须严格遵守】
═══════════════════════════════════════════════════════
- domain/ 层零外部依赖，禁止 import rusqlite、tauri、tokio、serde；只允许 std + chrono + thiserror + uuid。
- application/ 层定义 trait（ports）和 use case 编排，禁止 import infrastructure。
- infrastructure/ 实现 application 的 trait（adapters）；SQLCipher、DPAPI、PBKDF2、aes-gcm 都在这里。
- interfaces/tauri/ 只做 DTO 转换 + 调用 application + 权限检查；每个 #[tauri::command] ≤ 30 行。
- main.rs 是 composition root：构造所有 adapter，注入到 application service，注册成 Tauri State。
- 仓储 trait 返回 domain 类型，绝不返回 sqlite Row 或 serde_json::Value。
- 保持轻量：值对象只在上述列表里包，不要每个原始类型都包 newtype；不要写「贫血模型」（业务规则放领域对象的方法里）；不要给只有一个实现的 trait 强加抽象层。

═══════════════════════════════════════════════════════
【数据 schema（infrastructure/persistence/schema.sql）】
═══════════════════════════════════════════════════════
**没有 users 表** — 单用户车间模型, 解锁密码就是身份本身.

workspaces(id PK, name UNIQUE, description, created_at,
           kind TEXT NOT NULL DEFAULT 'normal' CHECK(kind IN ('normal','system_mirror')))

default_formulas(
  id PK,
  internal_color_code TEXT UNIQUE NOT NULL,
  customer_color_code TEXT,
  color_name, description,
  base_weight_kg, liquor_ratio, notes,
  created_at, updated_at
)
default_formula_items(id PK, formula_id FK CASCADE,
  dye_name, dye_code, percentage, unit CHECK in ('pct_owf','g_per_kg','g_per_L'), sort_order)

workspace_formulas(
  id PK, workspace_id FK CASCADE,
  internal_color_code TEXT NOT NULL,
  customer_color_code TEXT,
  color_name, description,
  base_weight_kg, liquor_ratio, notes,
  source_default_id FK SET NULL, created_at, updated_at,
  UNIQUE(workspace_id, internal_color_code)
)
workspace_formula_items(...)  -- 同 default_formula_items 结构

cart_items(
  id PK, workspace_id FK CASCADE,
  source_kind TEXT CHECK in ('default','workspace'),
  source_formula_id INTEGER NOT NULL,
  target_kg REAL NOT NULL CHECK(target_kg > 0),
  added_at,
  UNIQUE(workspace_id, source_kind, source_formula_id)
)

audit_log(id PK, workspace_context_id FK SET NULL,
          action, target, details, occurred_at)

索引：
  idx_workspace_formulas_ws_internal  on workspace_formulas(workspace_id, internal_color_code)
  idx_workspace_formulas_ws_customer  on workspace_formulas(workspace_id, customer_color_code)
  idx_default_formulas_internal       on default_formulas(internal_color_code)
  idx_default_formulas_customer       on default_formulas(customer_color_code)
  idx_cart_workspace                  on cart_items(workspace_id)
  idx_audit_time                      on audit_log(occurred_at)

═══════════════════════════════════════════════════════
【加密设计】
═══════════════════════════════════════════════════════
- 数据库主密钥: 32 字节随机生成, Windows DPAPI (windows crate, Win32_Security_Cryptography)
  保护后存 %APPDATA%\Ranpu\keystore.bin.
- SQLCipher PRAGMA key 由「主密钥 + 应用解锁密码」PBKDF2-SHA256(600k 轮) 派生 64 hex.
  解锁密码首次启动时设置, 之后每次启动 / 锁屏解锁都要输入.
- 导出口令独立于解锁密码; .ranpu 文件用 AES-256-GCM + PBKDF2(600k 轮).
- **不需要 argon2**: 之前版本用 argon2 哈希用户登录密码, 但单用户模型已经把 users
  表删了, 解锁密码不存任何地方 (PBKDF2 派生 PRAGMA key 即解密 DB; 输错的话
  SQLCipher 直接 file is encrypted error).

═══════════════════════════════════════════════════════
【应用层用例（每个一个文件）】
═══════════════════════════════════════════════════════
boot:        boot_status, boot_app(passphrase), setup_first_run(passphrase)
session:     lock_session, unlock_session(passphrase)
workspace:   create_workspace, rename_workspace, update_workspace_description,
             list_workspaces, switch_active_workspace, delete_workspace
formula:     list_default_formulas(关键词搜内部 / 客户色号 / 颜色俗称都匹配),
             upsert_default_formula, delete_default_formula,
             list_workspace_formulas, upsert_workspace_formula, delete_workspace_formula,
             copy_default_to_active_workspace, batch_copy_default_to_active_workspace,
             export_library_archive, preview_library_archive, import_library_archive
             (一律放开, 不再有 admin only 守卫)
calculation: calculate_dye_amounts, search_by_customer_code → 让 UI 挑一条
cart:        add_to_cart, remove_from_cart, update_cart_item_kg, clear_cart,
             list_cart_with_calculations, export_cart_as_batch_sheet,
             preview_cart_as_batch_sheet_html (返回 HTML 字符串给 iframe)
backup:      export_encrypted_backup, import_encrypted_backup
audit:       list_audit_events(filter),
             export_audit_log(date_from, date_to, action_filter?, format=encrypted|csv, passphrase?)

═══════════════════════════════════════════════════════
【项目结构】
═══════════════════════════════════════════════════════
src-tauri/src/
├── domain/
│   ├── workspace/{workspace.rs, mod.rs}        # 含 WorkspaceKind enum
│   ├── formula/{default_formula.rs, workspace_formula.rs, formula_item.rs,
│   │            internal_color_code.rs, customer_color_code.rs,
│   │            unit.rs, liquor_ratio.rs, amounts.rs, mod.rs}
│   ├── calculation/{dye_calculator.rs, mod.rs}
│   ├── cart/{cart.rs, cart_item.rs, mod.rs}
│   ├── audit/{audit_event.rs, mod.rs}
│   ├── session/{session.rs, mod.rs}             # 内存 lock 状态 + active_workspace_id
│   ├── shared/{id.rs, errors.rs, mod.rs}
│   └── mod.rs
├── application/
│   ├── ports/  (repository + service traits)
│   ├── session/  workspace/  formula/  calculation/  cart/  backup/  audit/
│   └── mod.rs
├── infrastructure/
│   ├── persistence/sqlcipher/{connection.rs, workspace_repo.rs,
│   │                          default_formula_repo.rs, workspace_formula_repo.rs,
│   │                          cart_repo.rs, audit_repo.rs}
│   ├── persistence/schema.sql
│   ├── persistence/seed.rs                     # 5 默认配方 + ensure 通用 system_mirror
│   ├── persistence/dev_seed.rs                 # cfg(feature="dev-seed"), 见 K
│   ├── crypto/{dpapi_keystore.rs, key_derivation.rs, aes_gcm_exporter.rs}
│   ├── session/in_memory_session_store.rs      # locked + active_workspace_id
│   └── mod.rs
├── interfaces/tauri/{commands.rs, dto.rs, error_mapping.rs, state.rs, lock_guard.rs, boot.rs}
└── main.rs

注意没有 identity 目录, 没有 user_repo.rs, 没有 argon2_hasher.rs (单用户模型不需要).

src/
├── assets/{logo.svg}
├── components/
│   ├── RanpuLogo.tsx
│   ├── TopBar.tsx
│   ├── Sidebar.tsx                # 左侧导航 (含 "关于" 项红点)
│   ├── WorkspacePicker.tsx        # 顶栏 combobox (取代 Switcher + 搜索)
│   ├── LockOverlay.tsx
│   ├── IdleDetector.tsx
│   ├── FormulaEditor.tsx
│   ├── FormulaCard.tsx
│   ├── ConfirmDialog.tsx          # 统一确认弹窗替代 window.confirm
│   └── ui/                        # shadcn/ui 基础组件
├── pages/
│   ├── BootScreen.tsx             # 输应用解锁密码
│   ├── FirstRunSetup.tsx          # 首次启动设密码
│   ├── Dashboard.tsx
│   ├── DefaultLibrary.tsx
│   ├── WorkspaceFormulas.tsx
│   ├── Calculator.tsx
│   ├── Cart.tsx
│   ├── WorkspaceManagement.tsx
│   ├── AuditLog.tsx
│   ├── LibraryTransfer.tsx        # 加密 .ranpu 库互导
│   ├── About.tsx
│   └── Settings.tsx
├── store/                         # zustand
│   ├── session.ts                 # active_workspace_id + locked
│   ├── workspaces.ts              # 缓存全部工作区
│   └── update.ts                  # 更新检查状态
├── api/                           # 按上下文一个文件 (boot, workspace, formula, calculation, cart, audit, backup, types, invoke)
└── App.tsx                        # boot gate state machine

不再需要: Login.tsx, UserManagement.tsx (没有用户).

═══════════════════════════════════════════════════════
【UI 关键交互细节】
═══════════════════════════════════════════════════════
- TopBar 永远显示: 左侧 <RanpuLogo size={28} withText={true} />, 接着 WorkspacePicker
  combobox (含 "未选择工作区" 项), 右侧 [锁定] 按钮. **没有用户名 / 登出按钮**
  (单用户模型, 想登出 = 想锁屏).
- 配方列表每条卡片明显展示: 内部色号 (粗体) + 客户色号 (旁边小字标签 "客户: xxx"),
  颜色俗称做副标题.
- 搜索框统一支持模糊匹配「内部色号 / 客户色号 / 颜色俗称」三字段.
- DefaultLibrary 与 WorkspaceFormulas 页: 每条配方右侧按钮 — 「加入批次清单」(所
  有人都能用) + 「复制到当前工作区」(没激活 workspace 时禁用, 提示 "请先选择工作
  区"). **没有 admin only 守卫** — 任何能进系统的人 (= 知道解锁密码的人) 都能增
  删改默认配方.
- Calculator 页: 色号输入 + kg 输入 → 表格显示染料名称 / 染料编号 / 克数; 明确
  角标「来自当前工作区」或「来自默认库」.
- Cart 页: 表格列出所有批次清单条目 (色号 / 客户色号 / 目标 kg / 染料明细总克数),
  支持修改 kg 后重算 / 一键清空 / 导出 CSV / 应用内 "预览 / 打印" (Dialog + iframe).
- 锁屏遮罩: 全屏 var(--color-background-primary) 半透明遮罩 + 居中 logo + 密码框 +
  「解锁」按钮 + 错误提示 (没有"剩余次数"提示, 单用户模型可以无限重试).
- 所有日期显示格式: YYYY-MM-DD HH:mm (24 小时制); 不要 ISO 8601 带 T 和 Z 的.
- 所有金额 / 克数显示保留 2 位小数; kg 输入框最大 99999.99, 最小 0.01.

═══════════════════════════════════════════════════════
【种子数据 (infrastructure/persistence/seed.rs)】
═══════════════════════════════════════════════════════
首次启动 schema 建好后插入：
- 3 个 workspace：「客户A」「客户B」「客户C」
- 5 条 default 配方（提供合理的中文颜色名 + 真实风格染料组合，比如「藏青 N-2024」配 C.I. Reactive Blue 19 + Reactive Black 5；「玫红 R-105」配 Reactive Red 195 等），每条 2–4 种染料，单位混合 pct_owf / g_per_kg。
- 不创建任何 user (没有 users 表). 首次启动走 FirstRunSetup, 引导设置应用解锁密码 (≥ 8 位).

═══════════════════════════════════════════════════════
【交付内容】
═══════════════════════════════════════════════════════
1. 完整可编译运行的项目，npm install && cargo tauri dev 启动。
2. 单元测试：domain/ 每个值对象、聚合不变量、DyeCalculator 三种单位计算各 ≥2 个测试（含边界 + 错误路径）。
3. README.md 顶部嵌入 logo（引用 src/assets/logo.svg），写清：领域模型图（mermaid）、各层职责、本地开发流程、打包命令、首次启动流程、加密设计审计要点、所有快捷键、Git 分支策略。
4. tauri.conf.json 完整配置：window 标题"染谱 Ranpu"、最小尺寸 1024x768、icon 路径正确。
5. 提供 .gitignore 和 CONTRIBUTING.md（说明 DDD 边界 + Git 分支约定）。

═══════════════════════════════════════════════════════
【Git 工作流 — 必须严格遵守】
═══════════════════════════════════════════════════════
零号步骤，在生成任何代码之前先初始化仓库：
  git init
  git branch -M main
  echo "node_modules/\ntarget/\ndist/\n*.db\nkeystore.bin\n.DS_Store" > .gitignore
  git add .gitignore
  git commit -m "chore: initialize repository"

之后每完成一个重大特性（划分见下），按以下流程：

1. 从最新 main 切特性分支：
     git checkout main
     git checkout -b feat/<feature-name>

2. 在分支上增量开发，每个有意义的小步都提交（不要憋到分支结束才一次大 commit）。
   commit message 用 Conventional Commits：
     feat(domain): add InternalColorCode value object
     feat(formula): implement upsert_default_formula use case
     test(calc): cover g_per_L unit edge cases
     fix(ui): correct workspace switcher disabled state
     refactor(repo): extract row mapping helper
     chore(deps): bump rusqlite to 0.31

3. 特性完成、`cargo test` 全绿、`cargo clippy -- -D warnings` 无报错、前端 `npm run typecheck` 通过后，回 main 用 --no-ff 合并保留分支历史：
     git checkout main
     git merge --no-ff feat/<feature-name> -m "merge: <feature-name>"

4. 如果配置了 origin remote，每次 commit 后 push 当前分支，每次 merge 后 push main：
     git push -u origin feat/<feature-name>
     git push origin main
   如果没有 remote，跳过 push 步骤，仅本地分支操作。

特性分支划分（每个对应一条 feat/* 分支，按顺序推进，前一条合并 main 后才开下一条）：

  feat/initial-scaffold     — Tauri 2 + React + TS + Tailwind + shadcn/ui 初始化、tauri.conf.json、Cargo.toml、package.json、tsconfig、prettier/eslint、空的目录骨架、logo.svg 落位
  feat/domain-layer         — 所有 domain/ 下的值对象、聚合、领域服务，附完整单元测试，零外部依赖（除 chrono/thiserror/uuid）
  feat/application-ports    — application/ports/ 下所有 trait、application/ 下所有 use case 编排
  feat/infra-persistence    — SQLCipher 连接、schema.sql、所有 *_repo.rs，附 seed.rs
  feat/infra-crypto         — DPAPI keystore、PBKDF2 派生 SQLCipher key、AES-GCM 导出
  feat/interfaces-tauri     — 所有 #[tauri::command]、DTO、error 映射、lock_guard、main.rs composition root
  feat/ui-design-system     — RanpuLogo、TopBar、LockOverlay、IdleDetector、FormulaCard、shadcn/ui 组件落位
  feat/ui-boot              — BootScreen (输解锁密码), FirstRunSetup (首次设密码), Settings 页
  feat/ui-formula           — DefaultLibrary、WorkspaceFormulas、FormulaEditor、Calculator、Cart 页
  feat/ui-admin             — WorkspaceManagement, AuditLog (含导出 UI), LibraryTransfer, About 页
  feat/seed-and-polish      — seed 真实数据、README、CONTRIBUTING.md、最终联调

不要在 main 直接提交（除了零号步骤的初始化 commit）。

═══════════════════════════════════════════════════════
【流程】
═══════════════════════════════════════════════════════
请第一步只输出（先不要写代码、先不要 git init）：
(a) 完整文件清单（按上述项目结构，每个文件一行职责说明）；
(b) 三份关键 trait 的签名草稿：UserRepository、WorkspaceFormulaRepository、DyeCalculator；
(c) 三个关键值对象的 Rust 代码：InternalColorCode、Unit、LiquorRatio；
(d) RanpuLogo.tsx 组件的代码草稿（含 size / withText / animated 三个 prop 的实现）。

我确认无误后你再执行【Git 工作流】里的零号步骤初始化仓库，并从 feat/initial-scaffold 开始推进。

═══════════════════════════════════════════════════════════════════════════════
【v1.0.x 迭代后追加 — 必须按此最终状态实现】
═══════════════════════════════════════════════════════════════════════════════

以下是 1.0.0 之后陆续迭代落定的功能 / 修正 / 决策。生成时这些必须包含在最终
代码里，**视作 MVP 一部分**，不要作为后续迭代再做。

────────────────────────────────────────────────
A. 单实例 (tauri-plugin-single-instance)
────────────────────────────────────────────────
- Cargo.toml 加 tauri-plugin-single-instance = "2"
- lib.rs Builder 最先注册插件 (越早拦截越好); 第二个进程启动时插件让它退出,
  把 (argv, cwd) 推回老实例; 老实例回调里 unminimize + show + set_focus 主窗口.
- 双击 .ranpu 文件传参时, _argv 暂时不消费 (用户没要求 deep-link).

────────────────────────────────────────────────
B. 自动更新 (tauri-plugin-updater)
────────────────────────────────────────────────
- tauri.conf.json:
    plugins.updater.endpoints = ["https://github.com/<owner>/<repo>/releases/latest/download/latest.json"]
    plugins.updater.pubkey = <minisign pubkey base64, 由 tauri signer generate 生成>
    plugins.updater.windows.installMode = "passive"
    bundle.createUpdaterArtifacts = true
    bundle.windows.nsis.installMode = "both"   ← 不用 currentUser, 让用户选
- App.tsx mount 时静默跑一次 useUpdateStore.runCheck() (zustand store):
    interface UpdateState { pending, checking, hasChecked, error, runCheck }
- 命中后 pending 设上, 触发两处 UI 提示:
    1. 侧栏 "关于" 项右边贴 h-2 w-2 红点 (Sidebar 订阅 store.pending)
    2. About 页按钮文案变 "有新版本 X.Y.Z" + 红点 (用 ring-2 ring-background 让暗色也看得清)
- 用户点 About 页按钮 → ConfirmDialog 描述只两行 (无末尾句号):
    "当前 X → 新版本 Y"
    "点击立即更新会下载并安装然后自动重启应用"
  → "立即更新" 走 pending.downloadAndInstall() + plugin-process relaunch().
- **不要弹 toast** — 启动时不在右下角弹 "发现新版本" 卡片. 只走静默检查 + 红点 +
  About 页按钮. 用户嫌 toast 噪音.
- 不监听 afterprint 自动收, 不需要.

────────────────────────────────────────────────
C. 顶栏: 单一 WorkspacePicker (合并 Switcher + Search)
────────────────────────────────────────────────
- 删除独立的 WorkspaceSwitcher 和独立的搜索框 — 用一个 combobox 取代.
- src/components/WorkspacePicker.tsx:
    · 触发按钮: <FolderClosed/> + 当前工作区名 + <ChevronDown/>; 200px 宽
    · 点开 320px popover, 顶部 Input (300ms 防抖) + 下方过滤列表
    · 过滤策略: name 或 description 大小写无关 substring; 数据走客户端过滤
      useWorkspacesStore (zustand 缓存全部工作区, 不再打 IPC 查)
    · 列表项: Check 图标标记当前 workspace + name + (右上小标"系统"标 system_mirror) +
      "当前" 角标 + description 一行截断
    · 顶部固定一项 "未选择工作区" (登录后默认状态, 让用户能临时退出工作区上下文)
    · 选中具体工作区 → cmd_switch_workspace → setActiveWorkspace → navigate(/workspace-formulas)
    · 选 "未选择工作区" → 仅切换不跳转
    · ESC 关闭 popover, 点击外部关闭, popover 打开时自动 focus 输入框
- TopBar 结构: <Logo> <WorkspacePicker/> ... <用户名> <锁定> <登出>

────────────────────────────────────────────────
D. 搜索框防抖 (WorkspaceFormulas / DefaultLibrary)
────────────────────────────────────────────────
- 把现有色号搜索改成 300ms 防抖自动触发, 删 "搜索" 按钮 + onKeyDown=Enter:
    const [keyword, setKeyword] = useState('')
    const [debouncedKeyword, setDebouncedKeyword] = useState('')
    useEffect(() => {
      const t = setTimeout(() => setDebouncedKeyword(keyword), 300)
      return () => clearTimeout(t)
    }, [keyword])
    useEffect(() => { load() }, [debouncedKeyword, ...其它依赖])

────────────────────────────────────────────────
E. 审计日志页前端展示限 50 条
────────────────────────────────────────────────
- AuditLog 页 list 调用 limit: 50 (后端按 occurred_at DESC, id DESC 排序).
- 全量审计仍由 "导出" 走加密 .ranpu / 明文 CSV, 显示限制不影响合规.

────────────────────────────────────────────────
F. 系统镜像工作区 "通用" + WorkspaceKind
────────────────────────────────────────────────
- WorkspaceKind enum: Normal | SystemMirror, schema 存 'normal' / 'system_mirror'
- workspaces 表加 kind TEXT NOT NULL DEFAULT 'normal' CHECK(...)
- 老 DB 兼容: connection.rs run_migrations() 用 PRAGMA table_info 探测后 ALTER TABLE
  ADD COLUMN kind 加进去 (幂等, 启动每次跑).
- WorkspaceRepository 加 find_system_mirror() 返回唯一一条 system_mirror.
- seed::ensure_system_mirror(workspace_repo, default_repo, workspace_formula_repo, now):
    每次启动跑一次. 没有该工作区 → 创建名为 "通用" 的 SystemMirror; 名字撞 normal
    工作区 → 把 normal 那个改名 "通用-用户" 让位; 默认库 ↔ 通用 工作区做全量同步
    (默认库新增/修改 → upsert 到镜像; 默认库已删除的 → 从镜像里删).
- "通用" 工作区不能直接增删改配方 — FormulaService::reject_if_system_mirror 在所有
  workspace 写操作 (新建 / 修改 / 删除 / 批量复制 default→ws / 导入) 上调一次, 命中
  返回 AppError::Internal("通用工作区为系统内置, 配方与默认配方库自动同步, 不能在
  此工作区直接增删改").
- "通用" 工作区不能重命名 / 删除 (WorkspaceService 加同样 guard).
- WorkspaceFormulasPage 检测 active.kind === 'system_mirror' → 顶部 Lock 图标 +
  amber banner 提示, 不显示 "新建配方" / "编辑" / "删除" 按钮.

────────────────────────────────────────────────
G. 客户名 / 红点 / 默认库源标
────────────────────────────────────────────────
- FormulaSource::DefaultFallback display_label() 返回 "来自默认库" (无 fallback 后缀).
- enum variant 名 DefaultFallback / DTO source: "default_fallback" 不变 (内部命名).

────────────────────────────────────────────────
H. 染料明细数量输入 bug fix
────────────────────────────────────────────────
- FormulaEditor.tsx items state 用本地 ItemForm 类型, amount 字段是 string 不是 number.
  提交时 parseFloat → number. 否则用户敲 "0." → Number("0.")=0 → render "0" → 吞小数点.
- blankItem 默认 unit: 'g_per_kg' (车间最常用), 不是 pct_owf.

────────────────────────────────────────────────
I. 批次单 "导出 HTML" → 应用内 "预览 / 打印"
────────────────────────────────────────────────
- 接口层: BatchSheetExporter trait 加 render(results, ctx, format) -> String 方法,
  不落盘. export() 重构为 render() + write 保持向后兼容.
- 应用层: 新 use case CartService::preview_cart_as_batch_sheet_html — 复用
  list_cart_with_calculations + filter_map(calculation.ok), 不写审计 (纯渲染, 用户的
  打印动作我们看不到, 假装记录没意义).
- 命令: cmd_preview_cart_as_batch_sheet_html → CmdResult<String>, 注册到 lib.rs.
- 前端 cart api: previewHtml() => invoke<string>('cmd_preview_cart_as_batch_sheet_html')
- Cart 页 "导出 HTML" 按钮 → "预览 / 打印":
    点击 → previewHtml() → setPreviewHtml(html)
    Dialog (max-w-5xl, h-[90vh], flex column, p-0):
      Header: "批次单预览"
      <iframe ref srcDoc={html} className="flex-1 border-0 bg-white" />
      Footer: "关闭" + "打印 / 另存为 PDF" 按钮
- onPrintPreview 关键: PDF 默认文件名要 = <客户名>-批次单-<YYYY-MM-DD>:
    Chrome / WebView2 给 iframe 调 print() 时, "Save as PDF" 默认文件名取的是
    *主窗口 document.title* 而不是 iframe <title>. 调 print 前临时改主窗口
    title, 在 iframe.contentWindow afterprint 监听里还原:

    const date = new Date().toISOString().slice(0, 10);
    const printTitle = workspaceName
      ? `${sanitizeForFilename(workspaceName)}-批次单-${date}`
      : `批次单-${date}`;
    const original = document.title;
    document.title = printTitle;
    const restore = () => { document.title = original; ifWin.removeEventListener('afterprint', restore); };
    ifWin.addEventListener('afterprint', restore);
    ifWin.focus(); ifWin.print();

- "导出 CSV" 按钮保留, 走原 save() + cmd_export_cart('csv').
- ⚠️ 不要尝试用独立 Tauri WebviewWindow 显示预览 — 在 ARM Win11 + WebView2 实测
  webview 进程初始化卡死 (白屏 + 不响应), 已放弃此路线. 用 in-window Dialog +
  iframe srcDoc.

────────────────────────────────────────────────
J. 批次单 HTML 模板细节 (render_html in batch_sheet_csv.rs)
────────────────────────────────────────────────
- <title> 动态生成: <workspace_name>-批次单-<YYYY-MM-DD>; 没工作区时退化 "批次单-<日期>".
  workspace_name 先过 sanitize_for_filename (替 \ / : * ? " < > | + 控制字符为下划线).
- @page { size: A4; margin: 1.5cm 2cm; } — 上下 1.5cm 给足空间, 左右 2cm 留余量
  (实体打印机硬边距通常 6-8mm, 表格 width:100% 的右边框 1cm 不够会被裁).
- @media print { body { padding: 0; print-color-adjust: exact; -webkit-print-color-adjust: exact; } }
  — print-color-adjust 必须放 @media print 内, 不要放 body. 老版 WebView2 (尤其
  ARM 上的) 解析这条新属性时会连带跳过后续 CSS 规则, 导致预览样式失效.
- 不写 "提示: 在浏览器中按 Ctrl+P..." 这种提示行 (应用内已有打印按钮).
- 表格用 table-layout: fixed + colgroup 固定 50/18/18/14 列宽; border-collapse: collapse
  + th, td { border: 1px solid #ccc }.

────────────────────────────────────────────────
K. dev-seed 大体量种子 (开发专用, 不进生产端)
────────────────────────────────────────────────
- src-tauri/Cargo.toml:
    [features]
    default = []
    dev-seed = []
- src-tauri/src/infrastructure/persistence/dev_seed.rs (整个文件 #[cfg(feature = "dev-seed")] 网住).
  内容: 8 个色系 (RD/OD/YD/GD/BD/PD/ND/KD) 凑 255 条 default formula + 20 个
  Dev客户NN 工作区, 每客户 5..=20 条带客户色号配方 (count 按 5 + (i*7)%16 算).
  二次启动幂等: 写库前先 find_by_internal_code 已存在则跳过 (repo.upsert(id=None) 走
  INSERT, 直接 upsert 撞 default_formulas.internal_color_code UNIQUE).
- boot.rs run_if_empty 之后 / ensure_system_mirror 之前调用, 双重门:
    #[cfg(feature = "dev-seed")] { if env RANPU_DEV_SEED == "1" { dev_seed::run(...) } }
- README 写明用法: RANPU_DEV_SEED=1 cargo tauri dev --features dev-seed; 生产
  build (tauri build, cargo build --release) 默认 feature 关 → 整模块 cfg 剔掉.

────────────────────────────────────────────────
L. CI / Release workflow
────────────────────────────────────────────────
- .github/workflows/release.yml:
    on: push tags v*, workflow_dispatch
    runs-on: windows-latest
    steps:
      - actions/checkout@v5 fetch-depth: 0  ← 拉全 tag 历史
      - actions/setup-node@v5 node 20 cache: npm
      - dtolnay/rust-toolchain@stable target x86_64-pc-windows-msvc
      - Swatinem/rust-cache@v2 workspaces: src-tauri -> target
      - npm ci
      - id: notes — 调 gh api -X POST repos/${{ github.repository }}/releases/generate-notes
          -f tag_name='${{ github.ref_name }}' --jq .body
        把输出塞 GITHUB_OUTPUT (multiline heredoc).
      - tauri-apps/tauri-action@v0:
          env: GITHUB_TOKEN, TAURI_SIGNING_PRIVATE_KEY, TAURI_SIGNING_PRIVATE_KEY_PASSWORD
          tagName, releaseName "染谱 Ranpu ${{ github.ref_name }}"
          releaseBody: ${{ steps.notes.outputs.body }}     ← 自动 changelog, 不再手写
          args: --target x86_64-pc-windows-msvc
- 不要把 releaseBody 写死成"首次安装步骤", 那段会同时进 GitHub release 正文 *和*
  latest.json notes, 老用户更新提示框看到首次安装步骤就尴尬了.

────────────────────────────────────────────────
M. 启动门 / FirstRunSetup / BootScreen (单用户解锁模型)
────────────────────────────────────────────────
- App.tsx 是 boot gate, 状态机 GateState: 'checking' | 'first-run' | 'boot' | 'app' | 'error'.
  没有 'login' state (没有用户).
- bootApi.status() → BootStatusView { keystore_exists, db_initialized }
  路由判断:
    !keystore_exists → first-run (引导设解锁密码 ≥ 8 位 + 二次确认)
    !db_initialized → boot (输解锁密码)
    db_initialized → app
- BootScreen 单输入框 + [解锁] 按钮; 后端 SqliteConnection::open 捕 "file is encrypted
  or is not a database" → AppError::BootPassphraseIncorrect → UI 显示 "解锁密码不对,
  请重试". 无锁定 / 无次数限制.
- FirstRunSetup 流程: 设密码 + 再次输入确认 + 提示 "密码丢失后无法找回任何数据";
  完成 → boot_app 直接进 app.

────────────────────────────────────────────────
N. 其它前端 store / 文件结构变化
────────────────────────────────────────────────
- src/store/:
    session.ts   — { active_workspace_id: number | null, locked: boolean,
                     setActiveWorkspace, setLocked, clear }
                   注意: 没有 user 字段 / username / role.
    workspaces.ts — { list, loaded, refresh, clear } (zustand 缓存全部工作区, WorkspacePicker
                    + WorkspaceFormulas 等多页面订阅; 任何会改工作区集合的页面操作完调 refresh)
    update.ts    — { pending, checking, hasChecked, error, runCheck } (上节 B)
- src/api/ 按上下文一个文件: boot, workspace, formula, calculation, cart, audit,
  backup, types, invoke (后者封装 invoke + ApiError). **没有 identity.ts**.
- src/components/:
    新增 ConfirmDialog.tsx — 统一确认弹窗, 取代 window.confirm; 危险操作 destructive 红按钮.
    新增 Sidebar.tsx — 左侧 200px 导航; 没激活 workspace 时 formula/calculator/cart 项
                       disabled 灰掉提示 "请先在顶栏选择工作区"; 订阅 useUpdateStore.pending,
                       在 "/about" 项右侧贴红点.
    新增 WorkspacePicker.tsx (上节 C, 取代旧的 WorkspaceSwitcher.tsx).
    保留 RanpuLogo / TopBar / FormulaCard / FormulaEditor / IdleDetector / LockOverlay.
    **不要** 创建 CartDrawer.tsx, UpdateNotifier.tsx — 都不用.
- src/pages/: BootScreen, FirstRunSetup, Dashboard, DefaultLibrary, WorkspaceFormulas,
  Calculator, Cart, WorkspaceManagement, AuditLog, LibraryTransfer, About, Settings.
  **没有 Login.tsx / UserManagement.tsx**.

────────────────────────────────────────────────
O. 命名 / 文案口径
────────────────────────────────────────────────
- "购物车" 业务上叫 "批次清单" — UI 文案统一用 "批次清单", 但代码内部 module / repo /
  数据库表名仍叫 cart / cart_items (不重命名, 减少改动面).
- 加密导出包扩展名: .ranpu (不是 .ydaexp); MAGIC = "RNP1".
- 日期格式 YYYY-MM-DD HH:mm (24 小时), 不要 ISO 8601 带 T/Z 的.

────────────────────────────────────────────────
P. 不要做的事 (经验教训)
────────────────────────────────────────────────
- 不要把 `print-color-adjust: exact` 放在 body 块里 — 会让 ARM WebView2 老版本
  解析失败连带跳后续规则.
- 不要尝试用独立 Tauri WebviewWindow 做打印预览 — Parallels ARM Win11 上 webview
  实例初始化挂死 (白屏 + 不响应 + 无法关闭). 哪种 URL 形式 (App / External /
  CustomProtocol / data:) 都救不了, 卡在 webview 进程层.
- 不要给打印对话框加 toast 提示 / 自动隐藏 — 用户嫌噪音, 不弹任何更新 toast,
  只走红点 + About 按钮提示.
- 不要把"首次安装步骤"塞 release.yml 的 releaseBody — 那段会变成老用户更新
  对话框看到的内容, 跟"本版改了什么"无关. 用 gh api generate-notes 自动出
  changelog 才对.
