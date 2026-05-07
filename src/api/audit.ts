import { invoke } from './invoke';
import type { AuditEventView } from './types';

export interface ListAuditArgs {
  from?: string;
  to?: string;
  user_ids?: number[];
  actions?: string[];
  limit?: number;
  offset?: number;
}

export interface ExportAuditArgs {
  from: string;
  to: string;
  user_ids?: number[];
  actions?: string[];
  format: 'encrypted' | 'csv';
  passphrase?: string;
  out_path: string;
}

export const auditApi = {
  list: (args: ListAuditArgs = {}) =>
    invoke<AuditEventView[]>('cmd_list_audit', {
      cmd: {
        from: args.from ?? null,
        to: args.to ?? null,
        user_ids: args.user_ids ?? null,
        actions: args.actions ?? null,
        limit: args.limit ?? null,
        offset: args.offset ?? null,
      },
    }),

  export: (args: ExportAuditArgs) =>
    invoke<void>('cmd_export_audit', {
      cmd: {
        from: args.from,
        to: args.to,
        user_ids: args.user_ids ?? null,
        actions: args.actions ?? null,
        format: args.format,
        passphrase: args.passphrase ?? null,
        out_path: args.out_path,
      },
    }),
};
