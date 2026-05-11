import { invoke } from './invoke';

export const cloudApi = {
  /// PUT 本地文件到指定 URL (WebDAV-style). 调用前需要把文件名拼到 URL
  /// 末尾, 例如 https://host/.../dav/files/<token>/configfile.ranpu.
  uploadFile: (localPath: string, uploadUrl: string) =>
    invoke<void>('cmd_upload_file_to_url', {
      local_path: localPath,
      upload_url: uploadUrl,
    }),
};
