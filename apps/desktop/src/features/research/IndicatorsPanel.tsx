/**
 * Indicadores fundamentalistas do ativo (`research.indicators`), em cards por grupo
 * (mercado, valuation, rentabilidade, endividamento, crescimento, por ação, dividendos,
 * FII, bancos). Cada indicador mostra fórmula, período e fonte ao passar o mouse ou ao
 * expandir; indisponíveis mostram o motivo.
 */
import { useEffect, useId, useRef, useState } from 'react';

import type { Indicator } from '@/api/generated/Indicator';
import type { SubjectArgs } from '@/api/generated/SubjectArgs';
import { useB3Refresh, useIndicators } from '@/api/hooks/research';
import { InstrumentCard, KeyValue, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { LoadingBlock, Notice } from '../portfolio/components/States';
import { safeDate, sourceName } from '../portfolio/labels';
import { SubjectError } from './common';
import { formatIndicatorValue } from './format';
import { GROUP_LABEL, PANEL_GROUPS } from './labels';

/** Situação da conferência dos dados de entrada, por extenso. */
function checkLabel(c: Indicator['check']): string | null {
  switch (c) {
    case 'verified':
      return t('pesquisa-conferencia-conferido');
    case 'corrected':
      return t('pesquisa-conferencia-corrigido');
    case 'warning':
      return t('pesquisa-conferencia-aviso');
    case 'blocked':
      return t('pesquisa-conferencia-bloqueado');
    default:
      return null;
  }
}

function IndicatorDetail({ ind }: { ind: Indicator }) {
  const check = checkLabel(ind.check);
  return (
    <KeyValue
      layout="stacked"
      items={[
        { label: t('pesquisa-formula'), value: ind.formula },
        ...(ind.unavailable_reason
          ? [{ label: t('pesquisa-indisponivel'), value: ind.unavailable_reason }]
          : [
              { label: t('pesquisa-periodo'), value: ind.period ?? '—' },
              { label: t('pesquisa-fonte'), value: ind.source ?? '—' },
            ]),
        ...(ind.estimated ? [{ label: t('pesquisa-status'), value: t('pesquisa-estimado') }] : []),
        ...(check
          ? [
              {
                label: t('pesquisa-conferencia'),
                value: ind.check_notes.length ? `${check}: ${ind.check_notes.join(' · ')}` : check,
              },
            ]
          : []),
      ]}
    />
  );
}

export function IndicatorRow({ ind }: { ind: Indicator }) {
  const [open, setOpen] = useState(false);
  const detailId = useId();
  const missing = ind.value == null;
  const value = formatIndicatorValue(ind.value, ind.unit, ind.currency, ind.exact);
  return (
    <li className="border-b border-line/60 last:border-0" data-indicator={ind.key}>
      <div className="flex items-baseline justify-between gap-3 py-1.5">
        <Tooltip
          side="left"
          content={
            <div className="flex max-w-72 flex-col gap-1.5 py-0.5">
              <span className="label-caps text-fg">{ind.label}</span>
              <IndicatorDetail ind={ind} />
            </div>
          }
        >
          <button
            type="button"
            aria-expanded={open}
            aria-controls={detailId}
            onClick={() => setOpen((o) => !o)}
            className="min-w-0 text-left text-[13px] text-fg underline decoration-line-strong decoration-dotted underline-offset-4 hover:text-accent"
          >
            {ind.label}
          </button>
        </Tooltip>
        <span
          className={cx(
            'num shrink-0 text-right text-[13px]',
            missing ? 'text-dim' : 'text-fg',
            ind.estimated && 'italic',
          )}
          aria-label={
            missing
              ? t('pesquisa-indicador-indisponivel', { indicador: ind.label })
              : `${ind.label}: ${value}`
          }
        >
          {ind.estimated && !missing ? '≈ ' : ''}
          {value}
          {ind.check === 'corrected' || ind.check === 'warning' ? (
            <span
              className={cx(
                'ml-1 text-[11px]',
                ind.check === 'warning' ? 'text-warning' : 'text-accent',
              )}
              role="img"
              aria-label={t(
                ind.check === 'warning'
                  ? 'pesquisa-conferencia-marca-aviso'
                  : 'pesquisa-conferencia-marca-corrigido',
                { indicador: ind.label },
              )}
            >
              {ind.check === 'warning' ? '!' : '*'}
            </span>
          ) : null}
        </span>
      </div>
      {missing && ind.unavailable_reason ? (
        <p className="-mt-1 pb-1.5 text-[11.5px] leading-snug text-dim">
          {t('pesquisa-indisponivel-motivo', { motivo: ind.unavailable_reason })}
        </p>
      ) : null}
      {open ? (
        <div id={detailId} className="mb-2 rounded-control bg-surface-2 px-2.5 py-2 text-[12px]">
          <IndicatorDetail ind={ind} />
        </div>
      ) : null}
    </li>
  );
}

export function IndicatorsPanel({ subject }: { subject: SubjectArgs }) {
  const q = useIndicators(subject);
  // Ação sem proventos oficiais da B3 (dividend yield estimado): consulta a B3 uma vez.
  const refresh = useB3Refresh();
  const asked = useRef<string | null>(null);
  const ticker = q.data?.subject.ticker ?? null;
  const estimated =
    q.data?.subject.class === 'stock_br' &&
    q.data.indicators.some((i) => i.key === 'dy' && i.estimated && i.value != null);
  useEffect(() => {
    if (!estimated || !ticker || asked.current === ticker) return;
    asked.current = ticker;
    refresh.mutate({ ticker });
  }, [estimated, ticker, refresh]);
  if (q.isPending) return <LoadingBlock label={t('pesquisa-calculando-indicadores')} />;
  if (q.isError) return <SubjectError error={q.error} onRetry={() => void q.refetch()} />;
  const v = q.data;
  const groups = PANEL_GROUPS.map((g) => ({
    group: g,
    items: v.indicators.filter((i) => i.group === g),
  })).filter((g) => g.items.length > 0);
  return (
    <div className="flex flex-col gap-4">
      <KeyValue
        items={[
          {
            label: t('pesquisa-cotacao'),
            value: v.price
              ? v.price.stale
                ? t('pesquisa-cotacao-anterior', {
                    cotacao: `${safeDate(v.price.date)} · ${sourceName(v.price.source)}`,
                  })
                : `${safeDate(v.price.date)} · ${sourceName(v.price.source)}`
              : t('pesquisa-sem-cotacao'),
          },
          {
            label: t('pesquisa-fluxos'),
            value: v.reference_date
              ? t('pesquisa-fluxos-12-meses', { data: safeDate(v.reference_date) })
              : v.subject.class === 'fii'
                ? t('pesquisa-informe-mensal')
                : '—',
          },
          { label: t('pesquisa-calculado-em'), value: safeDate(v.as_of) },
        ]}
      />
      {v.notes.length ? (
        <Notice title={t('pesquisa-notas-calculo')}>
          <ul className="list-inside list-disc">
            {v.notes.map((n) => (
              <li key={n}>{n}</li>
            ))}
          </ul>
        </Notice>
      ) : null}
      <div className="grid gap-4 @min-[530px]:grid-cols-2 @min-[1040px]:grid-cols-3">
        {groups.map((g, i) => (
          <InstrumentCard
            key={g.group}
            index={String(i + 1).padStart(2, '0')}
            title={GROUP_LABEL[g.group]}
            headingLevel={3}
          >
            <ul aria-label={t('pesquisa-indicadores-grupo', { grupo: GROUP_LABEL[g.group] })}>
              {g.items.map((ind) => (
                <IndicatorRow key={ind.key} ind={ind} />
              ))}
            </ul>
          </InstrumentCard>
        ))}
      </div>
      <p className="text-[12px] text-dim">{t('pesquisa-indicadores-dica')}</p>
    </div>
  );
}
