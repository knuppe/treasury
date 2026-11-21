import { useId } from 'react';

export interface SparklineProps {
  /** Pontos da série (só para desenho; nunca usados em cálculo). */
  values: readonly number[];
  /** Descrição para leitores de tela (ex.: "Patrimônio subindo 4,2% em 12 meses"). */
  label: string;
  width?: number;
  height?: number;
  color?: string;
  /** Preenche a área sob a linha. */
  area?: boolean;
  className?: string;
}

/** Gráfico mínimo de linha em SVG, para o corpo dos cards de instrumento. */
export function Sparkline({
  values,
  label,
  width = 160,
  height = 36,
  color = 'var(--accent)',
  area = true,
  className,
}: SparklineProps) {
  const gradientId = useId();
  if (values.length < 2) return null;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const pad = 2;
  const points = values.map((v, i) => {
    const x = 3 + (i / (values.length - 1)) * (width - 6);
    const y = pad + (1 - (v - min) / span) * (height - pad * 2);
    return [x, y] as const;
  });
  const line = points
    .map(([x, y], i) => `${i ? 'L' : 'M'}${x.toFixed(1)} ${y.toFixed(1)}`)
    .join('');
  const last = points[points.length - 1]!;
  return (
    <svg
      role="img"
      aria-label={label}
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      className={className}
      preserveAspectRatio="none"
    >
      {area ? (
        <>
          <defs>
            <linearGradient id={gradientId} x1="0" x2="0" y1="0" y2="1">
              <stop offset="0%" stopColor={color} stopOpacity="0.22" />
              <stop offset="100%" stopColor={color} stopOpacity="0" />
            </linearGradient>
          </defs>
          <path d={`${line}L${width} ${height}L0 ${height}Z`} fill={`url(#${gradientId})`} />
        </>
      ) : null}
      <path
        d={line}
        fill="none"
        stroke={color}
        strokeWidth="1.5"
        vectorEffect="non-scaling-stroke"
      />
      <circle cx={last[0]} cy={last[1]} r="2.25" fill={color} />
    </svg>
  );
}
