use std::path::Path;

use chrono::Utc;

use crate::application::ports::batch_sheet_exporter::{
    BatchSheetError, BatchSheetExporter, BatchSheetFormat,
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
        format: BatchSheetFormat,
        out_path: &Path,
    ) -> Result<(), BatchSheetError> {
        let buf = match format {
            BatchSheetFormat::Csv => render_csv(results),
            BatchSheetFormat::Html => render_html(results),
        };
        std::fs::write(out_path, buf).map_err(|e| BatchSheetError::Io(e.to_string()))?;
        Ok(())
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

fn render_html(results: &[CalculationResult]) -> String {
    let mut html = String::new();
    html.push_str(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>染谱批次单</title>
<style>
  @page { size: A4; margin: 1.5cm; }
  body {
    font-family: "Microsoft YaHei", "PingFang SC", "Source Han Sans SC",
                 "Noto Sans CJK SC", system-ui, sans-serif;
    color: #1f1f1f;
    margin: 0;
    padding: 24px;
  }
  h1 { font-size: 20px; letter-spacing: 4px; margin: 0 0 4px; }
  .sub { color: #888; font-size: 12px; letter-spacing: 2px; text-transform: uppercase; }
  .meta { color: #666; font-size: 12px; margin: 12px 0 24px; }
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
  table { border-collapse: collapse; width: 100%; font-size: 13px; }
  th, td { border: 1px solid #ccc; padding: 6px 10px; text-align: left; }
  th { background: #f3f3f3; }
  td.num { text-align: right; font-family: "Cascadia Mono", "JetBrains Mono", monospace; }
  td.unit { color: #888; font-size: 12px; }
  @media print {
    body { padding: 0; }
    .no-print { display: none; }
  }
</style>
</head>
<body>
  <h1>染谱批次单</h1>
  <div class="sub">DYE FORMULA · BATCH SHEET</div>
  <div class="meta">导出时间: "#,
    );
    html.push_str(&Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());
    html.push_str(
        r#"</div>
  <div class="no-print" style="margin-bottom:16px;color:#888;font-size:12px;">
    提示：在浏览器中按 Ctrl+P 可另存为 PDF 或直接打印。
  </div>
"#,
    );

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
        html.push_str("      <thead><tr><th>染料</th><th>编号</th><th style=\"text-align:right;\">克数</th><th>原始单位</th></tr></thead>\n");
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
