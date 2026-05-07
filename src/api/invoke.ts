import { invoke as tauriInvoke } from '@tauri-apps/api/core';

/**
 * 后端 UiError 的 wire 格式（interfaces/tauri/error_mapping.rs）。
 */
export interface BackendError {
  code: string;
  message: string;
}

export class ApiError extends Error {
  public readonly code: string;
  constructor(err: BackendError) {
    super(err.message);
    this.code = err.code;
    this.name = 'ApiError';
  }
}

/**
 * 统一的 invoke 包装：把后端 UiError 转成 ApiError 异常。
 * 全部命令都返回 Promise<T>，错误以 ApiError 抛出，UI 直接 try/catch 拿 message 即可
 * （message 是中文用户文案）。
 */
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return (await tauriInvoke(cmd, args)) as T;
  } catch (raw: unknown) {
    if (raw && typeof raw === 'object' && 'code' in raw && 'message' in raw) {
      throw new ApiError(raw as BackendError);
    }
    if (typeof raw === 'string') {
      throw new ApiError({ code: 'unknown', message: raw });
    }
    throw new ApiError({ code: 'unknown', message: String(raw ?? '未知错误') });
  }
}
