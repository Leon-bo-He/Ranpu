use std::path::Path;

use chrono::Local;

use crate::application::ports::batch_sheet_exporter::{
    BatchSheetContext, BatchSheetError, BatchSheetExporter, BatchSheetFormat, FormulaMeta,
};
use crate::domain::calculation::dye_calculator::CalculationResult;

/// 批次单导出器: 根据 BatchSheetFormat 选择 CSV 或打印友好的 HTML.
///
/// HTML 路径不直接生成 PDF, 因为生成合规 PDF 需要嵌入 CJK 字体 (>= 1MB),
/// 与桌面 app 「轻量、所有数据本地」 的取舍冲突. 用户在浏览器打开 HTML 后
/// Ctrl+P → 另存为 PDF 同样能拿到打印件, 还省下 app 内的字体管线.
pub struct BatchSheetCsvExporter;

impl BatchSheetCsvExporter {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for BatchSheetCsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchSheetExporter for BatchSheetCsvExporter {
    fn export(
        &self,
        results: &[CalculationResult],
        context: BatchSheetContext<'_>,
        format: BatchSheetFormat,
        out_path: &Path,
    ) -> Result<(), BatchSheetError> {
        let buf = self.render(results, context, format)?;
        std::fs::write(out_path, buf).map_err(|e| BatchSheetError::Io(e.to_string()))?;
        Ok(())
    }

    fn render(
        &self,
        results: &[CalculationResult],
        context: BatchSheetContext<'_>,
        format: BatchSheetFormat,
    ) -> Result<String, BatchSheetError> {
        Ok(match format {
            BatchSheetFormat::Csv => render_csv(results),
            BatchSheetFormat::Html => render_html(results, context),
            BatchSheetFormat::HtmlGrid => render_html_grid(results, context),
            BatchSheetFormat::HtmlA6Punch => render_html_a6_punch(results, context),
        })
    }
}

fn render_csv(results: &[CalculationResult]) -> String {
    let mut buf = String::new();
    buf.push_str("内部色号,目标 kg,染料名称,染料编号,克数,单位,来源\n");
    for r in results {
        for line in &r.lines {
            buf.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                csv_escape(r.internal_color_code.as_str()),
                format_amount(r.target_kg.value()),
                csv_escape(&line.dye_name),
                csv_escape(line.dye_code.as_deref().unwrap_or("")),
                format_amount(line.grams.value()),
                line.unit_used.as_db_str(),
                r.source.display_label(),
            ));
        }
    }
    buf
}

/// 数字最多保留 4 位小数, 末尾零自动去掉. 跟前端 lib/format.ts 的 formatAmount
/// 行为对齐, 避免前端显示 0.001 而打印 / CSV 看到 0.00.
///
/// 实现: 先 format!("{:.4}") 截到 4 位, 再砍末尾连续的 '0' 和孤立的 '.'.
fn format_amount(v: f64) -> String {
    if !v.is_finite() {
        return String::new();
    }
    let s = format!("{:.4}", v);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

fn render_html(results: &[CalculationResult], context: BatchSheetContext<'_>) -> String {
    // 算一次 local 时间, 上面 <title> 和下面 "导出时间" 行共用.
    // 用 Local 而非 Utc — 车间用户看到的是机器本地时间, 不需要在脑袋里转换时区.
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    // 文档 title 决定 WebView2 "Save as PDF" 时的默认文件名: 客户名-批次单-日期.
    // 工作区名先过 sanitize_for_filename 去掉 Windows 禁字符.
    let title = match context.workspace_name {
        Some(name) => format!("{}-批次单-{}", sanitize_for_filename(name), date),
        None => format!("批次单-{date}"),
    };

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    // Release 构建下父文档 CSP 严格 (style-src 'self' 'unsafe-inline'), 但
    // iframe srcdoc 文档继承时某些 WebView2 版本会把 'unsafe-inline' 丢掉,
    // 导致 <style> 块和 style="" 都被拦. Dev 构建用 devCsp 比较宽松所以
    // 看起来好的. 这里给 srcdoc 文档自己设一条本地 CSP, 不再依赖继承.
    html.push_str(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:\">\n",
    );
    html.push_str(&format!("<title>{}</title>\n", html_escape(&title)));
    html.push_str(
        r#"<style>
  @page { size: A4; margin: 1.5cm 2cm; }
  table { border-collapse: collapse; width: 100%; font-size: 13px; table-layout: fixed; }
  th, td {
    border: 1px solid #ccc;
    padding: 6px 10px;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    word-wrap: break-word;
  }
  th { background-color: #f3f3f3; }
  th.num { text-align: right; }
  td.num { text-align: right; font-variant-numeric: tabular-nums; font-weight: 700; color: #000; }
  body {
    font-family: "Microsoft YaHei", "PingFang SC", "Source Han Sans SC", "Noto Sans CJK SC", system-ui, sans-serif;
    color: #1f1f1f;
    margin: 0;
    padding: 24px;
  }
  h1 { font-size: 20px; letter-spacing: 4px; margin: 0 0 4px; }
  .sub { color: #888; font-size: 12px; letter-spacing: 2px; text-transform: uppercase; }
  .meta { color: #666; font-size: 12px; margin: 12px 0 24px; }
  .meta .label { color: #888; margin-right: 4px; }
  .meta .value { color: #1f1f1f; font-weight: 500; }
  .meta .row { margin-bottom: 2px; }
  .formula { page-break-inside: avoid; margin-bottom: 28px; }
  .formula h2 {
    font-size: 15px;
    margin: 0 0 8px;
    border-bottom: 2px solid #1f1f1f;
    padding-bottom: 4px;
  }
  .formula h2 .target { color: #555; font-weight: normal; margin-left: 8px; }
  .formula-meta { color: #666; font-size: 12px; margin: 4px 0 8px; }
  .formula-meta .label { color: #888; margin-right: 4px; }
  .formula-meta .value { color: #1f1f1f; font-weight: 500; margin-right: 16px; }
  @media print {
    body { padding: 0; print-color-adjust: exact; -webkit-print-color-adjust: exact; }
    .no-print { display: none; }
  }
</style>
</head>
<body>
  <h1>染谱批次单</h1>
  <div class="sub">DYE FORMULA · BATCH SHEET</div>
  <div class="meta">
"#,
    );
    if let Some(name) = context.workspace_name {
        html.push_str(&format!(
            "    <div class=\"row\"><span class=\"label\">客户:</span><span class=\"value\">{}</span></div>\n",
            html_escape(name),
        ));
    }
    html.push_str("    <div class=\"row\"><span class=\"label\">导出时间:</span><span class=\"value\">");
    html.push_str(&now.format("%Y-%m-%d %H:%M").to_string());
    html.push_str("</span></div>\n");
    html.push_str("  </div>\n");

    let empty_meta = FormulaMeta::default();
    for (idx, r) in results.iter().enumerate() {
        let meta = context.per_formula.get(idx).unwrap_or(&empty_meta);
        html.push_str(r#"  <div class="formula">"#);
        html.push('\n');
        html.push_str(&format!(
            r#"    <h2>{} <span class="target">目标 {} kg</span></h2>"#,
            html_escape(r.internal_color_code.as_str()),
            format_amount(r.target_kg.value()),
        ));
        html.push('\n');
        // 色系 / 缸号 / 纱支 写在 h2 下面一行. 纱支可能有多条变体,
        // 同一段内逐行展示 ("厂名 规格   N 个").
        if meta.color_family.is_some() || meta.vat_number.is_some() || !meta.yarns.is_empty() {
            html.push_str(r#"    <div class="formula-meta">"#);
            html.push('\n');
            if let Some(family) = meta.color_family {
                html.push_str(&format!(
                    "      <span class=\"label\">色系:</span><span class=\"value\">{}</span>\n",
                    html_escape(family),
                ));
            }
            if let Some(vat) = meta.vat_number {
                html.push_str(&format!(
                    "      <span class=\"label\">缸号:</span><span class=\"value\">{}</span>\n",
                    html_escape(vat),
                ));
            }
            for y in &meta.yarns {
                let label = format_yarn_line(y.mill, y.spec, y.count);
                if !label.is_empty() {
                    html.push_str(&format!(
                        "      <span class=\"label\">纱支:</span><span class=\"value\">{}</span>\n",
                        html_escape(&label),
                    ));
                }
            }
            html.push_str("    </div>\n");
        }
        // 表格 / th / td 的 border + th 灰底也 inline 一份, 兜底 CSS 解析
        // 万一在某条规则上 bail (历史上 col selector + Chinese / em-dash 注释
        // 都中过招). Inline style 不经过 CSS 解析器, 保证视觉永远在.
        let th_style = "border:1px solid #ccc;padding:6px 10px;text-align:left;background-color:#f3f3f3;";
        let th_num_style = "border:1px solid #ccc;padding:6px 10px;text-align:right;background-color:#f3f3f3;";
        let td_style = "border:1px solid #ccc;padding:6px 10px;text-align:left;";
        let td_num_style = "border:1px solid #ccc;padding:6px 10px;text-align:right;font-variant-numeric:tabular-nums;font-weight:700;color:#000;";
        html.push_str(
            "    <table style=\"border-collapse:collapse;width:100%;font-size:13px;table-layout:fixed;\">\n",
        );
        html.push_str("      <colgroup>\n");
        html.push_str("        <col style=\"width:55%\" />\n");
        html.push_str("        <col style=\"width:22%\" />\n");
        html.push_str("        <col style=\"width:23%\" />\n");
        html.push_str("      </colgroup>\n");
        html.push_str(&format!(
            "      <thead><tr><th style=\"{th_style}\">染料</th><th style=\"{th_style}\">编号</th><th style=\"{th_num_style}\">克数</th></tr></thead>\n",
        ));
        html.push_str("      <tbody>\n");
        for l in &r.lines {
            html.push_str(&format!(
                "        <tr><td style=\"{td_style}\">{}</td><td style=\"{td_style}\">{}</td><td style=\"{td_num_style}\">{}</td></tr>\n",
                html_escape(&l.dye_name),
                html_escape(l.dye_code.as_deref().unwrap_or("—")),
                format_amount(l.grams.value()),
            ));
        }
        html.push_str("      </tbody>\n");
        html.push_str("    </table>\n");
        html.push_str("  </div>\n");
    }
    html.push_str("</body>\n</html>\n");
    html
}

/// 四宫格 / 多页 A4 打印格式. 一页 2x2 = 4 格, 虚线分割; 染料 ≥ 8 的配方
/// 跨 2 行独享一整列. 没有顶部 meta 段 — 客户 / 缸号 / 纱支 / 色号 / 色系
/// 都进每个格子里.
const GRID_WIDE_THRESHOLD: usize = 8;
const GRID_CELLS_PER_PAGE: usize = 4;

fn render_html_grid(results: &[CalculationResult], context: BatchSheetContext<'_>) -> String {
    let now = Local::now();
    let date_full = now.format("%Y-%m-%d").to_string();
    let title = match context.workspace_name {
        Some(name) => format!("{}-批次单-{}", sanitize_for_filename(name), date_full),
        None => format!("批次单-{date_full}"),
    };

    // 把 results 切分到 page-cells: 每页最多 GRID_CELLS_PER_PAGE 个 slot.
    // 跨 2 行的 wide 配方算 2. CSS Grid auto-flow 自动布局, Rust 只负责按
    // 顺序输出 cell + 在累计达到上限时插入 page break.
    let total = results.len();

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:\">\n",
    );
    html.push_str(&format!("<title>{}</title>\n", html_escape(&title)));
    html.push_str(
        r#"<style>
  @page { size: A4; margin: 8mm; }
  body { font-family: "Microsoft YaHei", "PingFang SC", "Source Han Sans SC", "Noto Sans CJK SC", system-ui, sans-serif; color: #1f1f1f; margin: 0; padding: 0; }
  .grid-page { display: grid; grid-template-columns: repeat(2, 1fr); grid-template-rows: repeat(2, 1fr); width: 100%; height: 281mm; page-break-after: always; grid-auto-flow: row dense; }
  .grid-page:last-child { page-break-after: auto; }
  .cell { border: 1px dashed #999; padding: 10mm 9mm 16mm 9mm; box-sizing: border-box; overflow: hidden; position: relative; font-size: 19px; line-height: 1.6; }
  .cell.wide { grid-row: span 2; }
  .vat { font-size: 32px; font-weight: bold; line-height: 1.2; margin-bottom: 6px; }
  .meta-line { font-size: 20px; margin-bottom: 4px; }
  .divider { border: 0; border-top: 1.8px solid #1f1f1f; margin: 20px 0 20px; }
  .dye-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; padding: 6px 0; font-size: 24px; }
  .dye-row .name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: center; }
  .dye-row .grams { font-variant-numeric: tabular-nums; font-weight: 700; color: #000; text-align: left; }
  .yarn-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; font-size: 20px; margin-bottom: 4px; }
  .yarn-row .name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .yarn-row .count { font-variant-numeric: tabular-nums; white-space: nowrap; text-align: center; }
  .corner-l { position: absolute; bottom: 4mm; left: 6mm; font-size: 13px; color: #888; }
  .corner-r { position: absolute; bottom: 4mm; right: 6mm; font-size: 13px; color: #888; }
  @media print {
    body { print-color-adjust: exact; -webkit-print-color-adjust: exact; }
  }
</style>
</head>
<body>
"#,
    );

    // 把 results 塞进 grid-page. 一页 GRID_CELLS_PER_PAGE 个 cell-slot,
    // 宽配方占 2 个. 用 VecDeque + lookahead-swap: 当队首装不下时, 往后
    // 找第一个能塞进当前页剩余空间的配方, 调上来填. 这样宽配方留下的
    // 空隙不会浪费, 也不需要等 CSS dense (dense 只能跨同页, 跨页无效).
    use std::collections::VecDeque;
    let mut queue: VecDeque<(usize, &CalculationResult)> = results.iter().enumerate().collect();
    let mut cells_used = 0usize;
    let mut page_open = false;

    while let Some(&(_, peek)) = queue.front() {
        let wide = peek.lines.len() >= GRID_WIDE_THRESHOLD;
        let need = if wide { 2 } else { 1 };

        if !page_open {
            html.push_str("<div class=\"grid-page\">\n");
            page_open = true;
        }

        if cells_used + need > GRID_CELLS_PER_PAGE {
            // 队首塞不下. 往后找第一个能塞下的, 调到队首.
            let swap_pos = queue.iter().enumerate().skip(1).find_map(|(i, (_, f))| {
                let f_wide = f.lines.len() >= GRID_WIDE_THRESHOLD;
                let f_need = if f_wide { 2 } else { 1 };
                if cells_used + f_need <= GRID_CELLS_PER_PAGE { Some(i) } else { None }
            });
            if let Some(pos) = swap_pos {
                let item = queue.remove(pos).expect("found above");
                queue.push_front(item);
                continue; // 重新走顶端的 wide / need 计算
            }
            // 没人塞得下 → 翻页
            html.push_str("</div>\n");
            page_open = false;
            cells_used = 0;
            continue;
        }

        let (idx, r) = queue.pop_front().expect("front exists");
        let meta = context.per_formula.get(idx).cloned().unwrap_or_default();
        let class = if wide { "cell wide" } else { "cell" };
        html.push_str(&format!("  <div class=\"{class}\">\n"));

        // 第一排 缸号 (没填的话用 "—" 占位, 不空)
        let vat_display = meta.vat_number.unwrap_or("—");
        html.push_str(&format!(
            "    <div class=\"vat\">{}</div>\n",
            html_escape(vat_display),
        ));
        // 第二排 客户名
        let cust = context.workspace_name.unwrap_or("—");
        html.push_str(&format!(
            "    <div class=\"meta-line\">{}</div>\n",
            html_escape(cust),
        ));
        // 第三排 内部色号 + 色系
        let internal = r.internal_color_code.as_str();
        let third = match meta.color_family {
            Some(f) => format!("{} · {}", internal, f),
            None => internal.to_owned(),
        };
        html.push_str(&format!(
            "    <div class=\"meta-line\">{}</div>\n",
            html_escape(&third),
        ));
        // 纱支多行: 一条变体一行 ("厂名 规格   N 个"). 没填的话用 "—" 占
        // 一行作为 placeholder 维持视觉骨架.
        if meta.yarns.is_empty() {
            html.push_str(
                "    <div class=\"meta-line\">—</div>\n",
            );
        } else {
            for y in &meta.yarns {
                let name = format_yarn_name(y.mill, y.spec);
                let count = y.count.map(|c| format!("{c} 个"));
                html.push_str(&format!(
                    "    <div class=\"yarn-row\"><span class=\"name\">{}</span><span class=\"count\">{}</span></div>\n",
                    html_escape(if name.is_empty() { "—" } else { &name }),
                    html_escape(count.as_deref().unwrap_or("")),
                ));
            }
        }
        html.push_str("    <hr class=\"divider\" />\n");
        // 染料明细
        for l in &r.lines {
            let dye_label = match l.dye_code.as_deref() {
                Some(code) => format!("{} ({})", l.dye_name, code),
                None => l.dye_name.clone(),
            };
            html.push_str(&format!(
                "    <div class=\"dye-row\"><span class=\"name\">{}</span><span class=\"grams\">{}</span></div>\n",
                html_escape(&dye_label),
                format_amount(l.grams.value()),
            ));
        }
        // 左下角: 导出时间 (带年份)
        html.push_str(&format!(
            "    <div class=\"corner-l\">{}</div>\n",
            html_escape(&date_full),
        ));
        // 右下角: 客户名 · N/总数
        let counter_label = match context.workspace_name {
            Some(c) => format!("{} · {}/{}", c, idx + 1, total),
            None => format!("{}/{}", idx + 1, total),
        };
        html.push_str(&format!(
            "    <div class=\"corner-r\">{}</div>\n",
            html_escape(&counter_label),
        ));
        html.push_str("  </div>\n");

        cells_used += need;
        if cells_used >= GRID_CELLS_PER_PAGE {
            html.push_str("</div>\n");
            page_open = false;
            cells_used = 0;
        }
    }
    if page_open {
        html.push_str("</div>\n");
    }

    html.push_str("</body>\n</html>\n");
    html
}

/// 穿孔纸格式. 一条配方一张, 纸张 120×140mm 纵向, 左右各留 13mm 给孔洞
/// 区, 上下 8mm 行距 → 内容区 94×124mm. 没有边框 (纸边即是边框), 内容靠
/// vat 大字 + 分割线区隔. 全部字号加粗 — 染料最多 4 种, 字号 + 行距按 4
/// 项染料正好填满 124mm 高度调过, 4 条时基本无下方留白.
fn render_html_a6_punch(results: &[CalculationResult], context: BatchSheetContext<'_>) -> String {
    let now = Local::now();
    let date_full = now.format("%Y-%m-%d").to_string();
    let title = match context.workspace_name {
        Some(name) => format!("{}-批次单-{}", sanitize_for_filename(name), date_full),
        None => format!("批次单-{date_full}"),
    };
    let total = results.len();

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:\">\n",
    );
    html.push_str(&format!("<title>{}</title>\n", html_escape(&title)));
    html.push_str(
        r#"<style>
  @page { size: 120mm 140mm; margin: 8mm 13mm; }
  body { font-family: "Microsoft YaHei", "PingFang SC", "Source Han Sans SC", "Noto Sans CJK SC", system-ui, sans-serif; color: #1f1f1f; margin: 0; padding: 0; font-weight: bold; }
  /* min-height (而非 height): 短配方仍让 corner 锚到 124mm 底部, 长配方
     自然撑高溢到下一张物理纸, 比 overflow: hidden 裁掉行更安全. */
  .page { page-break-after: always; min-height: 124mm; box-sizing: border-box; position: relative; padding-bottom: 4mm; line-height: 1.6; }
  .page:last-child { page-break-after: auto; }
  .vat { font-size: 42px; line-height: 1.1; margin-bottom: 3px; }
  .meta-line { font-size: 26px; margin-bottom: 2px; }
  .divider { border: 0; border-top: 2px solid #1f1f1f; margin: 20px 0 20px; }
  .dye-row { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; padding: 2px 0; font-size: 28px; }
  .dye-row .name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: center; }
  .dye-row .grams { font-variant-numeric: tabular-nums; color: #000; text-align: left; }
  .yarn-row { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; font-size: 26px; margin-bottom: 2px; }
  .yarn-row .name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .yarn-row .count { font-variant-numeric: tabular-nums; white-space: nowrap; text-align: center; }
  .corner-l { position: absolute; bottom: 2mm; left: 0; font-size: 12px; color: #888; font-weight: normal; }
  .corner-r { position: absolute; bottom: 2mm; right: 0; font-size: 12px; color: #888; font-weight: normal; }
  @media print {
    body { print-color-adjust: exact; -webkit-print-color-adjust: exact; }
  }
</style>
</head>
<body>
"#,
    );

    let empty_meta = FormulaMeta::default();
    for (idx, r) in results.iter().enumerate() {
        let meta = context.per_formula.get(idx).unwrap_or(&empty_meta);
        html.push_str("  <div class=\"page\">\n");

        let vat_display = meta.vat_number.unwrap_or("—");
        html.push_str(&format!(
            "    <div class=\"vat\">{}</div>\n",
            html_escape(vat_display),
        ));
        let cust = context.workspace_name.unwrap_or("—");
        html.push_str(&format!(
            "    <div class=\"meta-line\">{}</div>\n",
            html_escape(cust),
        ));
        let internal = r.internal_color_code.as_str();
        let third = match meta.color_family {
            Some(f) => format!("{} · {}", internal, f),
            None => internal.to_owned(),
        };
        html.push_str(&format!(
            "    <div class=\"meta-line\">{}</div>\n",
            html_escape(&third),
        ));
        if meta.yarns.is_empty() {
            html.push_str("    <div class=\"meta-line\">—</div>\n");
        } else {
            for y in &meta.yarns {
                let name = format_yarn_name(y.mill, y.spec);
                let count = y.count.map(|c| format!("{c} 个"));
                html.push_str(&format!(
                    "    <div class=\"yarn-row\"><span class=\"name\">{}</span><span class=\"count\">{}</span></div>\n",
                    html_escape(if name.is_empty() { "—" } else { &name }),
                    html_escape(count.as_deref().unwrap_or("")),
                ));
            }
        }
        html.push_str("    <hr class=\"divider\" />\n");
        for l in &r.lines {
            let dye_label = match l.dye_code.as_deref() {
                Some(code) => format!("{} ({})", l.dye_name, code),
                None => l.dye_name.clone(),
            };
            html.push_str(&format!(
                "    <div class=\"dye-row\"><span class=\"name\">{}</span><span class=\"grams\">{}</span></div>\n",
                html_escape(&dye_label),
                format_amount(l.grams.value()),
            ));
        }
        html.push_str(&format!(
            "    <div class=\"corner-l\">{}</div>\n",
            html_escape(&date_full),
        ));
        let counter_label = match context.workspace_name {
            Some(c) => format!("{} · {}/{}", c, idx + 1, total),
            None => format!("{}/{}", idx + 1, total),
        };
        html.push_str(&format!(
            "    <div class=\"corner-r\">{}</div>\n",
            html_escape(&counter_label),
        ));
        html.push_str("  </div>\n");
    }

    html.push_str("</body>\n</html>\n");
    html
}

/// "厂名 规格" 拼字符串. 任一为空就只返回非空那部分; 全空返回空串.
fn format_yarn_name(mill: Option<&str>, spec: Option<&str>) -> String {
    match (mill, spec) {
        (Some(m), Some(s)) => format!("{m} {s}"),
        (Some(m), None) => m.to_owned(),
        (None, Some(s)) => s.to_owned(),
        (None, None) => String::new(),
    }
}

/// 标准格式 "色系/缸号/纱支" meta 段里, 一条纱支变体的单行文本:
/// "{厂名} {规格}   {count} 个". 全空返回空串, 调用方会跳过.
fn format_yarn_line(mill: Option<&str>, spec: Option<&str>, count: Option<&str>) -> String {
    let name = format_yarn_name(mill, spec);
    match (name.as_str(), count) {
        ("", None) => String::new(),
        ("", Some(c)) => format!("{c} 个"),
        (n, None) => n.to_owned(),
        (n, Some(c)) => format!("{n}    {c} 个"),
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Windows 文件名禁字符 \ / : * ? " < > | + 控制字符 → 下划线. 跟前端
/// Cart.tsx 同名工具同样的策略, 保证 PDF 默认文件名不会被 OS 拒收.
fn sanitize_for_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
                || c.is_control()
            {
                '_'
            } else {
                c
            }
        })
        .collect::<String>()
        .trim()
        .to_owned()
}
