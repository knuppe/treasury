import type { ReactNode } from 'react';

import { Money, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { formatPercent } from '@/lib/format';
import { usePrivacy } from '@/state/ui';
import { t } from '@/i18n';

import { BULLET_STATUS_LABEL, bulletGeometry, type BulletInput, type BulletStatus } from './bullet';

const STATUS_COLOR: Record<BulletStatus, string> = {
  none: 'var(--line-strong)',
  ok: 'var(--accent)',
  pace: 'var(--accent)',
  risk: 'var(--warning)',
  over: 'var(--negative)',
};

/** Cor do rótulo de estado (texto): só os alertas usam cor de status. */
const LABEL_COLOR: Record<BulletStatus, string> = {
  none: 'var(--text-dim)',
  ok: 'var(--text-dim)',
  pace: 'var(--text-dim)',
  risk: 'var(--warning)',
  over: 'var(--negative)',
};

const STATUS_ICON: Record<BulletStatus, string> = {
  none: '·',
  ok: '●',
  pace: '↗',
  risk: '▲',
  over: '▲',
};

export interface BulletBarProps extends BulletInput {
  /** Nome da categoria (para o tooltip e o nome acessível). */
  label: string;
  currency?: string;
  className?: string;
  /** Conteúdo extra no fim do tooltip (ex.: "Clique para ver os lançamentos"). */
  hint?: ReactNode;
}

/**
 * Bullet previsto × realizado em HTML: trilha recessiva, barra do realizado (cor do estado,
 * com rótulo de estado no tooltip e no texto ao lado), segmento claro do comprometido,
 * marca do previsto e marca fina do ritmo do mês.
 */
export function BulletBar({ label, currency, className, hint, ...input }: BulletBarProps) {
  const g = bulletGeometry(input);
  const privacy = usePrivacy();
  const color = STATUS_COLOR[g.status];
  const spoken = `${label}: ${BULLET_STATUS_LABEL[g.status]}${
    g.ratio != null && !privacy
      ? `, ${t('graficos-pct-do-previsto', { pct: formatPercent(g.ratio, { decimals: 0 }) })}`
      : ''
  }`;
  return (
    <Tooltip
      side="top"
      content={
        <div className="flex min-w-52 flex-col gap-1.5">
          <span className="label-caps text-fg">{label}</span>
          <dl className="grid grid-cols-[1fr_auto] gap-x-6 gap-y-0.5 text-[12px]">
            <dt className="text-dim">{t('graficos-realizado')}</dt>
            <dd className="text-right font-medium">
              <Money cents={input.actual} currency={currency} />
            </dd>
            <dt className="text-dim">{t('graficos-previsto')}</dt>
            <dd className="text-right">
              {input.planned > 0 ? <Money cents={input.planned} currency={currency} /> : '—'}
            </dd>
            <dt className="text-dim">{t('graficos-comprometido')}</dt>
            <dd className="text-right">
              <Money cents={input.committed} currency={currency} />
            </dd>
            {g.pace != null ? (
              <>
                <dt className="text-dim">{t('graficos-ritmo-mes')}</dt>
                <dd className="num text-right">{formatPercent(input.elapsed, { decimals: 0 })}</dd>
              </>
            ) : null}
          </dl>
          <span className="label-caps" style={{ color: LABEL_COLOR[g.status] }}>
            {STATUS_ICON[g.status]} {BULLET_STATUS_LABEL[g.status]}
            {g.ratio != null && !privacy ? ` · ${formatPercent(g.ratio, { decimals: 0 })}` : ''}
          </span>
          {hint ? <span className="text-[11.5px] text-dim">{hint}</span> : null}
        </div>
      }
    >
      <span
        role="img"
        aria-label={spoken}
        tabIndex={0}
        data-status={g.status}
        className={cx(
          'relative block h-3.5 w-full min-w-24 rounded-[2px] bg-line/60 focus-visible:outline-2 focus-visible:outline-accent',
          className,
        )}
      >
        <span
          className="absolute inset-y-0.5 left-0 rounded-r-[4px]"
          style={{ width: `${g.actual}%`, background: color }}
        />
        {g.committed > 0 ? (
          <span
            className="absolute inset-y-0.5 rounded-r-[4px]"
            style={{
              left: `calc(${g.actual}% + 2px)`,
              width: `max(0px, calc(${g.committed}% - 2px))`,
              background: `color-mix(in srgb, ${color} 32%, transparent)`,
            }}
          />
        ) : null}
        {g.pace != null ? (
          <span
            aria-hidden
            className="absolute -inset-y-0.5 w-px bg-dim"
            style={{ left: `${g.pace}%` }}
          />
        ) : null}
        {g.planned != null ? (
          <span
            aria-hidden
            className="absolute -inset-y-1 w-0.5 rounded-full bg-fg"
            style={{ left: `calc(${g.planned}% - 1px)` }}
          />
        ) : null}
      </span>
    </Tooltip>
  );
}

/** Estado do bullet em texto (ícone + rótulo), para colunas de tabela. */
export function BulletStatusLabel({ status }: { status: BulletStatus }) {
  return (
    <span
      className="label-caps inline-flex items-center gap-1"
      style={{ color: STATUS_COLOR[status] }}
    >
      <span aria-hidden>{STATUS_ICON[status]}</span>
      {BULLET_STATUS_LABEL[status]}
    </span>
  );
}
