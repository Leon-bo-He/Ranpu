use std::path::Path;

use chrono::Utc;

use crate::application::ports::batch_sheet_exporter::{
    BatchSheetContext, BatchSheetError, BatchSheetExporter, BatchSheetFormat,
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
        })
    }
}

fn render_csv(results: &[CalculationResult]) -> String {
    let mut buf = String::new();
    buf.push_str("内部色号,目标 kg,染料名称,染料编号,克数,单位,来源\n");
    for r in results {
        for line in &r.lines {
            buf.push_str(&format!(
                "{},{:.2},{},{},{:.2},{},{}\n",
                csv_escape(r.internal_color_code.as_str()),
                r.target_kg.value(),
                csv_escape(&line.dye_name),
                csv_escape(line.dye_code.as_deref().unwrap_or("")),
                line.grams.value(),
                line.unit_used.as_db_str(),
                r.source.display_label(),
            ));
        }
    }
    buf
}

fn render_html(results: &[CalculationResult], context: BatchSheetContext<'_>) -> String {
    // 算一次 date, 上面的 <title> 和下面的 "导出时间" 行都会用.
    let now = Utc::now();
    let date = now.format("%Y-%m-%d").to_string();
    // 文档 title 决定 WebView2 "Save as PDF" 时的默认文件名: 客户名-批次单-日期.
    // 文件名里只用安全字符 (Windows 禁 \ / : * ? " < > |); html_escape 防 XSS.
    let title = match context.workspace_name {
        Some(name) => format!("{}-批次单-{}", sanitize_for_filename(name), date),
        None => format!("批次单-{date}"),
    };

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    // 内嵌 CSP, 允许 inline script + inline style — 这是自包含打印预览页,
    // 没有外部资源, inline 安全. 不写 'self' 因为自定义协议下 'self' 含义
    // 模糊, 直接给最小集就够了.
    html.push_str(
        "<meta http-equiv=\"Content-Security-Policy\" \
         content=\"default-src 'none'; \
                  script-src 'unsafe-inline'; \
                  style-src 'unsafe-inline'; \
                  img-src data:; \
                  font-src data:;\">\n",
    );
    html.push_str(&format!("<title>{}</title>\n", html_escape(&title)));
    html.push_str(
        r#"<style>
  @page { size: A4; margin: 1.5cm; }
  body {
    font-family: "Microsoft YaHei", "PingFang SC", "Source Han Sans SC",
                 "Noto Sans CJK SC", system-ui, sans-serif;
    color: #1f1f1f;
    margin: 0;
    padding: 24px;
    /* 强制打印保留背景色 (th 灰底 / 表格分隔线), 等价于用户勾上
       "Background graphics" — 这条 CSS 直接覆盖那个勾选, 无论用户
       怎么设都按这渲染. */
    print-color-adjust: exact;
    -webkit-print-color-adjust: exact;
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
  .formula h2 .source {
    float: right;
    font-size: 12px;
    color: #888;
    font-weight: normal;
  }
  /* 所有批次表统一列宽; table-layout:fixed 让 col 宽度严格生效, 否则
     不同表的列宽会被各自内容撑出差异. */
  table { border-collapse: collapse; width: 100%; font-size: 13px; table-layout: fixed; }
  col.col-dye    { width: 50%; }
  col.col-code   { width: 18%; }
  col.col-grams  { width: 18%; }
  col.col-unit   { width: 14%; }
  th, td {
    border: 1px solid #ccc;
    padding: 6px 10px;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    word-wrap: break-word;
  }
  th { background: #f3f3f3; }
  th.num { text-align: right; }
  td.num { text-align: right; font-family: "Cascadia Mono", "JetBrains Mono", monospace; }
  td.unit { color: #888; font-size: 12px; }
  /* 顶部工具栏 — 仅屏幕显示, 打印时藏起. */
  .ranpu-toolbar {
    position: sticky;
    top: 0;
    z-index: 100;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 10px 24px;
    background: #fafafa;
    border-bottom: 1px solid #e5e5e5;
    margin: -24px -24px 24px;
  }
  .ranpu-toolbar button {
    font: inherit;
    padding: 6px 14px;
    border: 1px solid #d4d4d4;
    border-radius: 4px;
    background: #fff;
    cursor: pointer;
  }
  .ranpu-toolbar button:hover { background: #f0f0f0; }
  .ranpu-toolbar button.primary {
    background: #1f1f1f;
    color: #fff;
    border-color: #1f1f1f;
  }
  .ranpu-toolbar button.primary:hover { background: #000; }
  @media print {
    body { padding: 0; }
    .no-print, .ranpu-toolbar { display: none !important; }
  }
</style>
</head>
<body>
  <div class="ranpu-toolbar no-print">
    <button onclick="window.close()">关闭</button>
    <button class="primary" onclick="window.print()">打印 / 另存为 PDF</button>
  </div>
  <h1>染谱批次单</h1>
  <div class="sub">DYE FORMULA · BATCH SHEET</div>
  <div class="meta">
"#,
    );
    if let Some(name) = context.workspace_name {
        html.push_str(&format!(
            "    <div class=\"row\"><span class=\"label\">当前客户:</span><span class=\"value\">{}</span></div>\n",
            html_escape(name),
        ));
    }
    html.push_str("    <div class=\"row\"><span class=\"label\">导出时间:</span><span class=\"value\">");
    html.push_str(&now.format("%Y-%m-%d %H:%M UTC").to_string());
    html.push_str("</span></div>\n");
    html.push_str("  </div>\n");

    for r in results {
        html.push_str(r#"  <div class="formula">"#);
        html.push('\n');
        html.push_str(&format!(
            r#"    <h2>{} <span class="target">目标 {:.2} kg</span><span class="source">{}</span></h2>"#,
            html_escape(r.internal_color_code.as_str()),
            r.target_kg.value(),
            html_escape(r.source.display_label()),
        ));
        html.push('\n');
        html.push_str("    <table>\n");
        html.push_str("      <colgroup>\n");
        html.push_str("        <col class=\"col-dye\" />\n");
        html.push_str("        <col class=\"col-code\" />\n");
        html.push_str("        <col class=\"col-grams\" />\n");
        html.push_str("        <col class=\"col-unit\" />\n");
        html.push_str("      </colgroup>\n");
        html.push_str("      <thead><tr><th>染料</th><th>编号</th><th class=\"num\">克数</th><th>原始单位</th></tr></thead>\n");
        html.push_str("      <tbody>\n");
        for l in &r.lines {
            html.push_str(&format!(
                "        <tr><td>{}</td><td>{}</td><td class=\"num\">{:.2}</td><td class=\"unit\">{}</td></tr>\n",
                html_escape(&l.dye_name),
                html_escape(l.dye_code.as_deref().unwrap_or("—")),
                l.grams.value(),
                l.unit_used.display_label(),
            ));
        }
        html.push_str("      </tbody>\n");
        html.push_str("    </table>\n");
        html.push_str("  </div>\n");
    }
    html.push_str("</body>\n</html>\n");
    html
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
