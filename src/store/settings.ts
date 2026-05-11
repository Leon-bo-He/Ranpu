import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type IdleTimeoutMinutes = 5 | 10 | 30 | 60 | 0; // 0 = 关闭

/// 配方互导 URL 备份的默认 domain. 完整 PUT URL 由代码拼:
///   https://<domain>/public.php/dav/files/H9g9DTkFX3FLq8P/<filename>
/// path 部分固定 (这是 Nextcloud 分享 token, 改了等于换备份位置), 只有
/// domain 暴露给用户改.
export const DEFAULT_CLOUD_UPLOAD_DOMAIN = 'upload.1122888.xyz';

/// 备份 URL 的固定 path 段 (含 token). 不在 UI 里展示, 也不让用户改.
export const CLOUD_UPLOAD_PATH = '/public.php/dav/files/H9g9DTkFX3FLq8P';

interface SettingsState {
  idleTimeoutMinutes: IdleTimeoutMinutes;
  setIdleTimeoutMinutes: (m: IdleTimeoutMinutes) => void;
  /// 一个纱支包/筒的标准重量 (kg). 批次单 prompt 里用 总重量 / 单个重量
  /// 自动算每条配方的纱支个数. 默认 1.25 kg.
  singleYarnWeightKg: number;
  setSingleYarnWeightKg: (n: number) => void;
  /// 配方互导 URL 备份的 domain (不带 scheme / path). 默认 DEFAULT_CLOUD_UPLOAD_DOMAIN.
  cloudUploadDomain: string;
  setCloudUploadDomain: (domain: string) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      idleTimeoutMinutes: 10,
      setIdleTimeoutMinutes: (m) => set({ idleTimeoutMinutes: m }),
      singleYarnWeightKg: 1.25,
      setSingleYarnWeightKg: (n) =>
        set({
          // 限正数, 上限给个保守的 999 防误输; 非数字 / 0 / 负数都拒绝.
          singleYarnWeightKg:
            Number.isFinite(n) && n > 0 ? Math.min(n, 999) : 1.25,
        }),
      cloudUploadDomain: DEFAULT_CLOUD_UPLOAD_DOMAIN,
      setCloudUploadDomain: (domain) => {
        // 用户可能贴一个 "https://upload.1122888.xyz/" 进来, 这里只留 host.
        // 去掉前缀 scheme + 末尾斜杠 / 路径; 空字符串回退默认.
        const trimmed = domain.trim();
        if (!trimmed) {
          set({ cloudUploadDomain: DEFAULT_CLOUD_UPLOAD_DOMAIN });
          return;
        }
        const stripped = trimmed
          .replace(/^https?:\/\//i, '')
          .split('/')[0]
          .trim();
        set({
          cloudUploadDomain: stripped || DEFAULT_CLOUD_UPLOAD_DOMAIN,
        });
      },
    }),
    { name: 'ranpu-settings' },
  ),
);
