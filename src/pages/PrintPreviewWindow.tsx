import { Printer, X } from 'lucide-react';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
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
/// 的打印预览只接管这个新窗口, 主窗口完全不受影响 (相比之前的 Dialog
/// 内嵌 iframe 方案, 那个会把整个主窗口替换成 Edge 打印 UI).
export function PrintPreviewWindow() {
  const [html, setHtml] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);

  useEffect(() => {
    cartApi
      .consumePrintPreview()
      .then((content) => {
        if (content === null) {
          setError('没有等待打印的批次单内容. 请回主窗口重新点 "预览 / 打印".');
        } else {
          setHtml(content);
        }
      })
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)));
  }, []);

  const onPrint = () => {
    iframeRef.current?.contentWindow?.focus();
    iframeRef.current?.contentWindow?.print();
  };

  const onClose = () => {
    void getCurrentWebviewWindow().close();
  };

  return (
    <div className="flex h-screen flex-col bg-background">
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
        {error && (
          <div className="rounded-md border border-destructive/40 bg-destructive/5 p-4 text-sm text-destructive">
            {error}
          </div>
        )}
        {html && (
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
