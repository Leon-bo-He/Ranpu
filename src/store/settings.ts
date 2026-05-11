import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type IdleTimeoutMinutes = 5 | 10 | 30 | 60 | 0; // 0 = 关闭

/// 配方互导云端上传的默认 URL 前缀 (WebDAV PUT 把文件名拼在末尾). 用户
/// 可以在设置里改成自己的 share 链接.
export const DEFAULT_CLOUD_UPLOAD_URL =
  'https://upload.1122888.xyz/public.php/dav/files/H9g9DTkFX3FLq8P';

interface SettingsState {
  idleTimeoutMinutes: IdleTimeoutMinutes;
  setIdleTimeoutMinutes: (m: IdleTimeoutMinutes) => void;
  /// 一个纱支包/筒的标准重量 (kg). 批次单 prompt 里用 总重量 / 单个重量
  /// 自动算每条配方的纱支个数. 默认 1.25 kg.
  singleYarnWeightKg: number;
  setSingleYarnWeightKg: (n: number) => void;
  /// 配方互导云端上传的 URL 前缀, 默认 DEFAULT_CLOUD_UPLOAD_URL.
  /// 真实上传 URL = 这个 + "/" + 文件名.
  cloudUploadUrl: string;
  setCloudUploadUrl: (url: string) => void;
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
      cloudUploadUrl: DEFAULT_CLOUD_UPLOAD_URL,
      setCloudUploadUrl: (url) => {
        const trimmed = url.trim();
        // 空字符串回退默认; 否则去掉末尾斜杠.
        const cleaned = trimmed
          ? trimmed.replace(/\/+$/, '')
          : DEFAULT_CLOUD_UPLOAD_URL;
        set({ cloudUploadUrl: cleaned });
      },
    }),
    { name: 'ranpu-settings' },
  ),
);
