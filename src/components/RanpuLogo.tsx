import { useId, type CSSProperties } from 'react';

export interface RanpuLogoProps {
  /** 图形像素尺寸（必填，正方形）。 */
  size: number;
  /** 是否在图形右侧渲染「染谱」中文。默认 false。 */
  withText?: boolean;
  /** 是否播放绘制动画。默认 false（移除 <animate> 元素）。 */
  animated?: boolean;
  /** 透传 className，方便外层做布局/对齐。 */
  className?: string;
  /** 透传 style。 */
  style?: CSSProperties;
}

/**
 * 染谱 Logo。
 * - SVG 内联渲染，便于条件性插入 <animate>。
 * - 渐变 id 通过 useId() 隔离，多实例同页面不冲突。
 * - withText 为 true 时，文字用 <span> 渲染（不嵌进 SVG）。
 */
export function RanpuLogo({
  size,
  withText = false,
  animated = false,
  className,
  style,
}: RanpuLogoProps) {
  const reactId = useId();
  const gradientId = `ranpu-spectrum-${reactId.replace(/:/g, '')}`;

  return (
    <span
      className={className}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: withText ? Math.max(6, Math.round(size * 0.25)) : 0,
        lineHeight: 1,
        ...style,
      }}
    >
      <svg
        width={size}
        height={size}
        viewBox="0 0 200 200"
        xmlns="http://www.w3.org/2000/svg"
        role="img"
        aria-label="染谱"
      >
        <defs>
          <linearGradient id={gradientId} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#FF4B2B" />
            <stop offset="50%" stopColor="#6A11CB" />
            <stop offset="100%" stopColor="#2575FC" />
          </linearGradient>
        </defs>

        <circle
          cx={100}
          cy={100}
          r={95}
          fill="none"
          stroke="#f0f0f0"
          strokeWidth={2}
        />

        <g transform="translate(40, 45)">
          <path
            d="M10,110 C10,10 110,110 110,10"
            stroke="#eee"
            strokeWidth={12}
            fill="none"
            strokeLinecap="round"
          />

          <path
            d="M10,110 C10,10 110,110 110,10"
            stroke={`url(#${gradientId})`}
            strokeWidth={10}
            fill="none"
            strokeLinecap="round"
            strokeDasharray={200}
            strokeDashoffset={animated ? 200 : 0}
          >
            {animated && (
              <animate
                attributeName="stroke-dashoffset"
                from="200"
                to="0"
                dur="2s"
                fill="freeze"
              />
            )}
          </path>

          <path
            d="M10,10 C10,110 110,10 110,110"
            stroke={`url(#${gradientId})`}
            strokeWidth={10}
            fill="none"
            strokeLinecap="round"
            opacity={0.6}
          />
        </g>
      </svg>

      {withText && (
        <span
          style={{
            fontFamily:
              'var(--font-serif, "Source Han Serif SC", "Noto Serif SC", "Songti SC", serif)',
            fontWeight: 500,
            letterSpacing: '3px',
            color: 'var(--color-text-primary, #1f1f1f)',
            fontSize: Math.round(size * 0.7),
            userSelect: 'none',
          }}
        >
          染谱
        </span>
      )}
    </span>
  );
}

export default RanpuLogo;
