import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';
import { usePrivacy } from '@/state/ui';

export interface InflationMeterProps {
  /** Retorno nominal (razão: 0,12 = 12%). */
  nominal: number | null;
  /** Retorno real, descontado pelo deflator (razão). */
  real: number | null;
  /** |valor| que ocupa a barra inteira (escala comum da tabela). */
  scale: number;
  /** Nome do deflator (IPCA, US CPI…), para o texto acessível. */
  deflator: string;
  /** Mostra os números ao lado das barras. */
  showValues?: boolean;
  className?: string;
}

function Bar({
  value,
  scale,
  tone,
}: {
  value: number | null;
  scale: number;
  tone: 'nominal' | 'real';
}) {
  if (value == null || !Number.isFinite(value)) {
    return <div className="h-[3px] w-full rounded-full bg-line/60" />;
  }
  const width = Math.min(1, Math.abs(value) / scale) * 50;
  const negative = value < 0;
  return (
    <div className="relative h-[5px] w-full">
      <span aria-hidden className="absolute top-[-2px] left-1/2 h-[9px] w-px bg-line-strong" />
      <span
        aria-hidden
        className={cx(
          'absolute top-0 h-[5px]',
          tone === 'nominal' ? 'bg-accent' : 'bg-warning',
          negative ? 'rounded-l-[2px]' : 'rounded-r-[2px]',
        )}
        style={
          negative ? { right: '50%', width: `${width}%` } : { left: '50%', width: `${width}%` }
        }
      />
    </div>
  );
}

/**
 * Medidor de inflação (componente-assinatura): barra dupla nominal × real a partir de
 * um zero central. A distância entre as barras é o que a inflação comeu.
 */
export function InflationMeter({
  nominal,
  real,
  scale,
  deflator,
  showValues = true,
  className,
}: InflationMeterProps) {
  const privacy = usePrivacy();
  const text = (v: number | null) =>
    privacy ? '•••••' : v == null ? '—' : formatPercent(v, { signed: true });
  const spoken = privacy
    ? t('carteira-medidor-oculto', { deflator })
    : t('carteira-medidor', { nominal: text(nominal), deflator, real: text(real) });
  return (
    <div
      role="img"
      aria-label={spoken}
      className={cx('flex min-w-[96px] items-center gap-1.5', className)}
    >
      <div className="flex min-w-10 flex-1 flex-col gap-[3px]">
        <Bar value={nominal} scale={scale} tone="nominal" />
        <Bar value={real} scale={scale} tone="real" />
      </div>
      {showValues ? (
        <div aria-hidden className="num flex flex-col text-right text-[10.5px] leading-[12px]">
          <span className="text-fg">{text(nominal)}</span>
          <span className={cx(real == null ? 'text-dim' : 'text-warning')}>{text(real)}</span>
        </div>
      ) : null}
    </div>
  );
}
