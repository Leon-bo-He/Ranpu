import { pinyin } from 'pinyin-pro';

/// 把字符串里的中文字符转成拼音首字母 (拼接, 不带音调). 非中文字符保留原样.
/// 例: "博奥 30/2" → "ba 30/2".
export function pinyinFirstLetters(s: string): string {
  return pinyin(s, {
    pattern: 'first',
    toneType: 'none',
    separator: '',
    nonZh: 'consecutive',
  }).toLowerCase();
}

/// 字符串模糊匹配: 直接子串 + 拼音首字母子串都尝试. query 已 trim 并大小
/// 写无关. 用于纱支厂名 / 规格搜索.
export function matchOption(query: string, option: string): boolean {
  const q = query.toLowerCase();
  if (!q) return true;
  if (option.toLowerCase().includes(q)) return true;
  if (pinyinFirstLetters(option).includes(q)) return true;
  return false;
}
