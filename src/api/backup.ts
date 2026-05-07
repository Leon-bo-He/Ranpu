import { invoke } from './invoke';

export const backupApi = {
  exportEncrypted: (passphrase: string, outPath: string) =>
    invoke<void>('cmd_export_backup', { cmd: { passphrase, out_path: outPath } }),

  importEncrypted: (passphrase: string, inPath: string) =>
    invoke<void>('cmd_import_backup', { cmd: { passphrase, in_path: inPath } }),
};
