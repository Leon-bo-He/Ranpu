import { invoke } from './invoke';
import type { BootStatusView, SessionView } from './types';

export const bootApi = {
  status: () => invoke<BootStatusView>('cmd_boot_status'),

  bootApp: (bootPassphrase: string) =>
    invoke<SessionView>('cmd_boot_app', {
      cmd: { boot_passphrase: bootPassphrase },
    }),

  setupFirstRun: (bootPassphrase: string) =>
    invoke<SessionView>('cmd_setup_first_run', {
      cmd: { boot_passphrase: bootPassphrase },
    }),

  lockSession: () => invoke<void>('cmd_lock_session'),

  unlockSession: (passphrase: string) =>
    invoke<SessionView>('cmd_unlock_session', { cmd: { passphrase } }),

  /// 仅做口令校验, 不动 session 状态. 用于设置页开启高权限 toggle 前的
  /// 二次确认. 接受用户口令或内置 master 口令; 不通过抛 ApiError.
  verifyBootPassphrase: (passphrase: string) =>
    invoke<void>('cmd_verify_boot_passphrase', { cmd: { passphrase } }),
};
