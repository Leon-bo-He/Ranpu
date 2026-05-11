import { invoke } from './invoke';

export const cloudApi = {
  /// PUT 本地文件到指定 URL (WebDAV-style). 调用前需要把文件名拼到 URL
  /// 末尾, 例如 https://host/.../dav/files/<token>/configfile.ranpu.
  uploadFile: (localPath: string, uploadUrl: string) =>
    // Tauri 2 默认 rename_all = camelCase, 后端的 local_path / upload_url
    // 在 IPC 边界变成 localPath / uploadUrl. 这里也要用 camelCase, 否则
    // 报 "missing required key localPath".
    invoke<void>('cmd_upload_file_to_url', {
      localPath,
      uploadUrl,
    }),
};
