import { invoke } from './invoke';
import type { BootStatusView, SessionView } from './types';

export const bootApi = {
  status: () => invoke<BootStatusView>('cmd_boot_status'),

  bootApp: (bootPassphrase: string) =>
    invoke<BootStatusView>('cmd_boot_app', {
      cmd: { boot_passphrase: bootPassphrase },
    }),

  setupFirstRun: (bootPassphrase: string, username: string, password: string) =>
    invoke<SessionView>('cmd_setup_first_run', {
      cmd: { boot_passphrase: bootPassphrase, username, password },
    }),
};
