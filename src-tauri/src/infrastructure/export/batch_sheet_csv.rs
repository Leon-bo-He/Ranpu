use std::path::Path;

use chrono::Local;

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
    html.push_str(&format!("<title>{}</title>\n", html_escape(&title)));
    html.push_str(
        r#"<style>
  /* 打印边距: 上下 1.5cm 给足空间, 左右 2cm 留余量 — 实体打印机
     硬边距通常 6-8mm, 表格 width:100% 边框贴右沿, 太小会被裁掉. */
  @page { size: A4; margin: 1.5cm 2cm; }
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
  /* 所有批次表统一列宽: col 宽度走 inline style (而不是 col.col-X 选择器) —
     某些 WebView2 版本在 col 类型选择器后会把后续规则一起丢, 表现为
     表格 border / th 背景全失效. inline style 是兜底办法, 不依赖 CSS 解析器. */
  table { border-collapse: collapse; width: 100%; font-size: 13px; table-layout: fixed; }
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
  @media print {
    /* 强制保留背景色 (th 灰底 / 表格分隔线), 等价于用户勾上
       "Background graphics". 放在 @media print 里, 避免老版本 WebView2
       在 screen 渲染时解析这条新属性出错连带后续规则失效. */
    body {
      padding: 0;
      print-color-adjust: exact;
      -webkit-print-color-adjust: exact;
    }
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
    if let Some(vat) = context.vat_number {
        html.push_str(&format!(
            "    <div class=\"row\"><span class=\"label\">缸号:</span><span class=\"value\">{}</span></div>\n",
            html_escape(vat),
        ));
    }
    if let Some(yarn) = context.yarn_count {
        html.push_str(&format!(
            "    <div class=\"row\"><span class=\"label\">纱支:</span><span class=\"value\">{}</span></div>\n",
            html_escape(yarn),
        ));
    }
    html.push_str("    <div class=\"row\"><span class=\"label\">导出时间:</span><span class=\"value\">");
    html.push_str(&now.format("%Y-%m-%d %H:%M").to_string());
    html.push_str("</span></div>\n");
    html.push_str("  </div>\n");

    for r in results {
        html.push_str(r#"  <div class="formula">"#);
        html.push('\n');
        html.push_str(&format!(
            r#"    <h2>{} <span class="target">目标 {} kg</span></h2>"#,
            html_escape(r.internal_color_code.as_str()),
            format_amount(r.target_kg.value()),
        ));
        html.push('\n');
        html.push_str("    <table>\n");
        html.push_str("      <colgroup>\n");
        html.push_str("        <col style=\"width:50%\" />\n");
        html.push_str("        <col style=\"width:18%\" />\n");
        html.push_str("        <col style=\"width:18%\" />\n");
        html.push_str("        <col style=\"width:14%\" />\n");
        html.push_str("      </colgroup>\n");
        html.push_str("      <thead><tr><th>染料</th><th>编号</th><th class=\"num\">克数</th><th>原始单位</th></tr></thead>\n");
        html.push_str("      <tbody>\n");
        for l in &r.lines {
            html.push_str(&format!(
                "        <tr><td>{}</td><td>{}</td><td class=\"num\">{}</td><td class=\"unit\">{}</td></tr>\n",
                html_escape(&l.dye_name),
                html_escape(l.dye_code.as_deref().unwrap_or("—")),
                format_amount(l.grams.value()),
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
