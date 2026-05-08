import { Loader2, Printer, X } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';

import { cartApi } from '@/api/cart';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';

/// 独立窗口里的批次单预览 + 打印.
///
/// 主窗口 cmd_open_print_preview 已经把 HTML stash 到 AppState; 我们
/// mount 时调 consumePrintPreview 取走, 灌到 iframe srcDoc 里.
///
/// 用户点 "打印 / 另存为 PDF" → iframe.contentWindow.print() → WebView2
/// 的打印预览只接管这个新窗口, 主窗口完全不受影响.
export function PrintPreviewWindow() {
  const [html, setHtml] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);

  useEffect(() => {
    cartApi
      .consumePrintPreview()
      .then((content) => {
        if (content === null || content.length === 0) {
          setError(
            '没有等待打印的批次单内容. 请关掉这个窗口, 回主窗口重新点 "预览 / 打印".',
          );
        } else {
          setHtml(content);
        }
      })
      .catch((e) => {
        const msg = e instanceof ApiError ? e.message : String(e);
        // eslint-disable-next-line no-console
        console.error('[print-preview] consumePrintPreview failed:', e);
        setError(`取批次单内容失败: ${msg}`);
      })
      .finally(() => setLoading(false));
  }, []);

  const onPrint = () => {
    iframeRef.current?.contentWindow?.focus();
    iframeRef.current?.contentWindow?.print();
  };

  // 关窗用 window.close() 兜底 — Tauri 的 WebviewWindow.close 万一不可用,
  // 浏览器 window.close() 仍然能关掉自己 (因为是被脚本打开的子窗口).
  const onClose = () => {
    try {
      window.close();
    } catch (e) {
      // eslint-disable-next-line no-console
      console.error('[print-preview] close failed:', e);
    }
  };

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex shrink-0 items-center justify-between border-b bg-card/50 px-4 py-2">
        <div className="text-sm font-medium">批次单预览</div>
        <div className="flex gap-2">
          <Button size="sm" variant="ghost" onClick={onClose}>
            <X className="mr-1 h-4 w-4" /> 关闭
          </Button>
          <Button size="sm" onClick={onPrint} disabled={html === null}>
            <Printer className="mr-1 h-4 w-4" /> 打印 / 另存为 PDF
          </Button>
        </div>
      </header>
      <div className="flex-1 overflow-hidden bg-neutral-100 p-4">
        {loading && (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" /> 正在加载批次单…
          </div>
        )}
        {error && !loading && (
          <div className="rounded-md border border-destructive/40 bg-destructive/5 p-4 text-sm text-destructive">
            {error}
          </div>
        )}
        {html && !loading && (
          <iframe
            ref={iframeRef}
            srcDoc={html}
            title="批次单预览"
            className="h-full w-full rounded-md border bg-white shadow-sm"
          />
        )}
      </div>
    </div>
  );
}

export default PrintPreviewWindow;
