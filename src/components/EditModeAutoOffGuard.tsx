import { useEffect } from 'react';

import { IDLE_AUTO_OFF_MS, useEditModeStore } from '@/store/editMode';

/// 每 60s 扫一次 editMode 三个开关; 超 30min 没动作就自动 OFF.
/// 没 UI, 仅副作用. 挂在 App.tsx 主面板分支即可.
export function EditModeAutoOffGuard() {
  const sweepIdle = useEditModeStore((s) => s.sweepIdle);

  useEffect(() => {
    const tick = () => sweepIdle(IDLE_AUTO_OFF_MS);
    tick();
    const id = window.setInterval(tick, 60_000);
    return () => window.clearInterval(id);
  }, [sweepIdle]);

  return null;
}
