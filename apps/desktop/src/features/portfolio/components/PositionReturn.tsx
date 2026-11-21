import type { PositionView } from '@/api/generated/PositionView';
import { PRIVACY_MASK, Tooltip } from '@/design/components';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';
import { usePrivacy } from '@/state/ui';

import { Pct } from './Pct';

/**
 * Retorno de uma posição pela idade (`return_basis` do núcleo): abaixo de 30 dias só o do
 * período (o anualizado de poucos dias exagera: 5 dias de −6% viram −99% a.a.); até um ano o
 * do período em destaque e o anualizado ao lado; depois, o anualizado (XIRR).
 */
export function headlineReturn(p: PositionView, real = false): number | null {
  if (p.return_basis === 'annualized') return real ? p.xirr_real : p.xirr;
  return real ? p.period_return_real : p.period_return;
}

/** "Comprada há N dias" (hoje, 1 dia, N dias). */
export function boughtAgo(days: number | null): string {
  return t('carteira-recente-comprada', { dias: days ?? 0 });
}

const signed = (v: number | null, privacy: boolean) =>
  privacy ? PRIVACY_MASK : v == null ? '—' : formatPercent(v, { signed: true });

/** Dica da posição recente: há quantos dias, o retorno no período e quando vem o anualizado. */
export function recentHint(p: PositionView, real: boolean, privacy: boolean): string {
  return t('carteira-recente-dica', {
    comprada: boughtAgo(p.holding_days),
    retorno: signed(real ? p.period_return_real : p.period_return, privacy),
  });
}

/** Etiqueta "posição recente" com a dica (no lugar do retorno anual, que ainda não vale). */
export function RecentPill({ hint }: { hint: string }) {
  return (
    <Tooltip content={<span className="block max-w-64">{hint}</span>}>
      <button
        type="button"
        className="inline-flex items-center rounded-full border border-line px-1.5 py-px font-sans text-[10px] leading-tight whitespace-nowrap text-dim"
      >
        <span>{t('carteira-recente-pill')}</span>
        <span className="sr-only">{`: ${hint}`}</span>
      </button>
    </Tooltip>
  );
}

/** Célula do retorno anual na tabela de posições (nominal ou, com `real`, o real). */
export function PositionReturnCell({ p, real = false }: { p: PositionView; real?: boolean }) {
  const privacy = usePrivacy();
  const annual = real ? p.xirr_real : p.xirr;
  const period = real ? p.period_return_real : p.period_return;
  if (p.return_basis === 'period') {
    return (
      <div className="flex flex-col items-end gap-0.5 leading-tight">
        <span className="flex items-center gap-1.5">
          <RecentPill hint={recentHint(p, real, privacy)} />
          <span className="num text-dim">—</span>
        </span>
        <span className="flex flex-wrap items-baseline justify-end gap-x-1 text-[11px]">
          <Pct value={period} tone="auto" />
          <span className="font-sans whitespace-normal text-dim">
            {t('carteira-recente-no-periodo')}
          </span>
        </span>
      </div>
    );
  }
  if (p.return_basis === 'period+annualized') {
    return (
      <div className="flex flex-col items-end gap-0.5 leading-tight">
        <Pct value={period} tone="auto" />
        {/* Quebra dentro da célula (a coluna é estreita): nunca invade a coluna ao lado. */}
        <span className="max-w-full text-right font-sans text-[11px] whitespace-normal text-dim">
          {t('carteira-recente-anualizado', {
            retorno: signed(annual, privacy),
            dias: p.holding_days ?? 0,
          })}
        </span>
      </div>
    );
  }
  return <Pct value={annual} signed />;
}

/**
 * Nota sob o retorno na página do ativo: na posição recente, há quantos dias e quando vem o
 * anualizado; até um ano, o anualizado (nominal e, com deflator, o real) em texto discreto.
 */
export function RecentReturnNote({ p, withReal }: { p: PositionView; withReal: boolean }) {
  const privacy = usePrivacy();
  if (p.return_basis === 'period') {
    return (
      <p className="flex flex-wrap items-center gap-2 text-[12px] text-dim">
        <RecentPill hint={recentHint(p, false, privacy)} />
        <span>{t('carteira-recente-nota', { comprada: boughtAgo(p.holding_days) })}</span>
      </p>
    );
  }
  if (p.return_basis === 'period+annualized') {
    const days = p.holding_days ?? 0;
    return (
      <p className="text-[12px] text-dim">
        {t('carteira-recente-anualizado', { retorno: signed(p.xirr, privacy), dias: days })}
        {withReal
          ? ` · ${t('carteira-recente-anualizado-real', { retorno: signed(p.xirr_real, privacy) })}`
          : null}
      </p>
    );
  }
  return null;
}
