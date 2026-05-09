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
};
