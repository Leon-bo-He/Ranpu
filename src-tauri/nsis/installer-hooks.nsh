; 清理旧版「Ranpu」快捷方式。
; productName 从 Ranpu 改为 染谱（v1.1.10）后，NSIS 只会管理新名称下的
; 快捷方式，旧的 Ranpu.lnk 不会被自动删除，这里手动清理。
!macro customInstall
  ; 开始菜单（当前用户 + 全局安装两种路径都尝试）
  Delete "$SMPROGRAMS\Ranpu\Ranpu.lnk"
  RMDir  "$SMPROGRAMS\Ranpu"
  Delete "$COMMONPROGRAMS\Ranpu\Ranpu.lnk"
  RMDir  "$COMMONPROGRAMS\Ranpu"
  ; 桌面快捷方式
  Delete "$DESKTOP\Ranpu.lnk"
  Delete "$COMMONDESKTOP\Ranpu.lnk"
!macroend
