import { invoke } from './invoke';
import type { WorkspaceView } from './types';

export const workspaceApi = {
  list: () => invoke<WorkspaceView[]>('cmd_list_workspaces'),

  create: (name: string, description?: string) =>
    invoke<number>('cmd_create_workspace', {
      cmd: { name, description: description ?? null },
    }),

  rename: (workspaceId: number, newName: string) =>
    invoke<void>('cmd_rename_workspace', {
      cmd: { workspace_id: workspaceId, new_name: newName },
    }),

  switch: (workspaceId: number | null) =>
    invoke<void>('cmd_switch_workspace', {
      cmd: { workspace_id: workspaceId },
    }),

  remove: (workspaceId: number) => invoke<void>('cmd_delete_workspace', { workspaceId }),
};
