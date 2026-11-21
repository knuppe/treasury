import { cx } from '@/design/cx';
import { Private } from '@/design/components';
import { formatPercent } from '@/lib/format';

/**
 * Percentual (razão → `12,3%`), escondido no modo privacidade. Com `tone="auto"`,
 * ganho e perda ganham cor, mas o sinal `+`/`−` sempre aparece.
 */
export function Pct({
  value,
  signed = false,
  decimals = 1,
  tone = 'neutral',
  estimated = false,
  className,
}: {
  value: number | null | undefined;
  signed?: boolean;
  decimals?: number;
  tone?: 'auto' | 'neutral';
  estimated?: boolean;
  className?: string;
}) {
  if (value == null || !Number.isFinite(value))
    return <span className={cx('num text-dim', className)}>—</span>;
  const color =
    tone === 'auto'
      ? value > 0
        ? 'text-positive'
        : value < 0
          ? 'text-negative'
          : undefined
      : undefined;
  return (
    <Private className={className}>
      <span className={cx('num whitespace-nowrap', color, estimated && 'italic', className)}>
        {estimated ? <span aria-hidden>≈&#8239;</span> : null}
        {formatPercent(value, { signed: signed || tone === 'auto', decimals })}
      </span>
    </Private>
  );
}
