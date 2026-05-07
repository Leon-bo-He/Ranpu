use std::path::Path;

use crate::application::ports::batch_sheet_exporter::{
    BatchSheetError, BatchSheetExporter, BatchSheetFormat,
};
use crate::domain::calculation::dye_calculator::CalculationResult;

/// 简单的批次单 CSV/PDF 导出实现。PDF 暂时落到 CSV（为了打包尺寸不引入 PDF 库）。
/// 后续 feat/seed-and-polish 再决定是否真的实装 PDF（用 printpdf 或 typst）。
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
        _format: BatchSheetFormat,
        out_path: &Path,
    ) -> Result<(), BatchSheetError> {
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
        std::fs::write(out_path, buf).map_err(|e| BatchSheetError::Io(e.to_string()))?;
        Ok(())
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}
