import { useEffect } from 'react';

import { workspaceApi } from '@/api/workspace';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useSessionStore } from '@/store/session';
import { useWorkspacesStore } from '@/store/workspaces';

const NONE_VALUE = '__none__';

export function WorkspaceSwitcher() {
  const session = useSessionStore((s) => s.session);
  const setActiveWorkspace = useSessionStore((s) => s.setActiveWorkspace);
  const workspaces = useWorkspacesStore((s) => s.list);
  const refreshWorkspaces = useWorkspacesStore((s) => s.refresh);

  useEffect(() => {
    if (!session) return;
    refreshWorkspaces();
  }, [session, refreshWorkspaces]);

  const onChange = async (value: string) => {
    const workspaceId = value === NONE_VALUE ? null : Number(value);
    await workspaceApi.switch(workspaceId);
    setActiveWorkspace(workspaceId);
  };

  const current = session?.active_workspace_id;
  const value = current === null || current === undefined ? NONE_VALUE : String(current);

  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="w-[200px]">
        <SelectValue placeholder="选择工作区" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={NONE_VALUE}>未选择工作区</SelectItem>
        {workspaces.map((w) => (
          <SelectItem key={w.id} value={String(w.id)}>
            {w.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
