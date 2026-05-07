import { invoke } from './invoke';
import type { SessionView, UnlockOutcomeView, UserView } from './types';

export const identityApi = {
  login: (username: string, password: string) =>
    invoke<SessionView>('cmd_login', { cmd: { username, password } }),

  logout: () => invoke<void>('cmd_logout'),

  lockSession: () => invoke<void>('cmd_lock_session'),

  unlockSession: (password: string) =>
    invoke<UnlockOutcomeView>('cmd_unlock_session', { cmd: { password } }),

  changePassword: (oldPassword: string, newPassword: string) =>
    invoke<void>('cmd_change_password', {
      cmd: { old_password: oldPassword, new_password: newPassword },
    }),

  createUser: (username: string, password: string, role: 'admin' | 'user') =>
    invoke<number>('cmd_create_user', { cmd: { username, password, role } }),

  deactivateUser: (userId: number) =>
    invoke<void>('cmd_deactivate_user', { userId }),

  activateUser: (userId: number) =>
    invoke<void>('cmd_activate_user', { userId }),

  listUsers: () => invoke<UserView[]>('cmd_list_users'),
};
