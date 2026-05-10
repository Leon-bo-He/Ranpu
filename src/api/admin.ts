import { invoke } from './invoke';

export const adminApi = {
  /// 把整个数据库 (默认库 / 工作区 / 配方 / 批次清单 / 审计日志) 擦干净.
  /// 后端会先关 SQLCipher 连接 → 删 db 文件 → 异步触发 app.restart().
  /// 前端拿到 Ok 之后短暂内进程会被重启, 用户回到 boot 屏.
  resetDatabase: (passphrase: string, confirmText: string) =>
    invoke<void>('cmd_reset_database', {
      cmd: { passphrase, confirm_text: confirmText },
    }),
};
