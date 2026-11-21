/**
 * Histórico do ativo (`research.history`): receita e lucro, margens e ROE, e dividendos por
 * ação de cada exercício (10 anos); FIIs com o informe mensal (rendimento e valor
 * patrimonial por cota, 24 meses). Dinheiro do núcleo em centavos. Uma unidade por figura.
 */
import { useCallback, useMemo } from 'react';

import type { HistoryView } from '@/api/generated/HistoryView';
import type { SubjectArgs } from '@/api/generated/SubjectArgs';
import { useHistory } from '@/api/hooks/research';
import { ChartFrame, EChart, type ChartTheme } from '@/charts';
import { EmptyState, TechTable, type TechColumn } from '@/design/components';
import { t } from '@/i18n';
import { formatMonthShort, formatPercent } from '@/lib/format';

import { LoadingBlock } from '../portfolio/components/States';
import { decimalToNumber } from '../portfolio/decimal';
import { barsOption, linesOption, type ChartSeries } from './charts';
import { SubjectError } from './common';
import { compactAmount, unitAmount } from './format';

const fromCents = (v: number | null) => (v == null ? null : v / 100);

export interface HistoryCharts {
  years: string[];
  money: ChartSeries[];
  ratios: ChartSeries[];
  dps: ChartSeries[];
  months: string[];
  income: ChartSeries[];
  nav: ChartSeries[];
}

/** Séries dos gráficos a partir do histórico (centavos → unidades da moeda). */
export function historyCharts(h: HistoryView): HistoryCharts {
  const y = h.years;
  const ebitdaMargin = y.map((r) => (r.ebitda != null && r.revenue ? r.ebitda / r.revenue : null));
  const hasRevenue = y.some((r) => r.revenue != null);
  return {
    years: y.map((r) => String(r.year)),
    money: [
      ...(hasRevenue
        ? [
            {
              key: 'receita',
              name: t('pesquisa-hist-receita-liquida'),
              values: y.map((r) => fromCents(r.revenue)),
            },
          ]
        : []),
      {
        key: 'lucro',
        name: t('pesquisa-hist-lucro-liquido'),
        values: y.map((r) => fromCents(r.net_income)),
      },
    ],
    ratios: [
      ...(ebitdaMargin.some((v) => v != null)
        ? [{ key: 'ebitda', name: t('pesquisa-hist-margem-ebitda'), values: ebitdaMargin }]
        : []),
      ...(y.some((r) => r.net_margin != null)
        ? [
            {
              key: 'margem',
              name: t('pesquisa-hist-margem-liquida'),
              values: y.map((r) => r.net_margin),
            },
          ]
        : []),
      ...(y.some((r) => r.roe != null)
        ? [{ key: 'roe', name: t('pesquisa-hist-roe'), values: y.map((r) => r.roe) }]
        : []),
    ],
    dps: [
      {
        key: 'dps',
        name: t('pesquisa-hist-dividendos-acao'),
        values: y.map((r) => decimalToNumber(r.dps)),
      },
    ],
    months: h.months.map((m) => formatMonthShort(m.month)),
    income: [
      {
        key: 'rendimento',
        name: t('pesquisa-hist-rendimento-cota'),
        values: h.months.map((m) => decimalToNumber(m.income_per_share)),
      },
    ],
    nav: [
      {
        key: 'vp',
        name: t('pesquisa-hist-vp-cota'),
        values: h.months.map((m) => decimalToNumber(m.nav_per_share)),
      },
    ],
  };
}

const MONEY_KEYS = ['receita', 'lucro'];
const RATIO_KEYS = ['ebitda', 'margem', 'roe'];

function Figure({
  index,
  title,
  source,
  labels,
  series,
  colorKeys,
  format,
  kind,
  ariaLabel,
  tableLabel,
  tablePrefsId,
}: {
  index: string;
  title: string;
  source: string;
  labels: string[];
  series: ChartSeries[];
  colorKeys: string[];
  format: (v: number) => string;
  kind: 'bars' | 'lines';
  ariaLabel: string;
  tableLabel: string;
  /** Preferências da tabela pelo título antigo em pt-BR (o rótulo muda com o idioma). */
  tablePrefsId: string;
}) {
  const option = useCallback(
    (theme: ChartTheme) =>
      kind === 'bars'
        ? barsOption(theme, { labels, series, colorKeys, format })
        : linesOption(theme, { labels, series, colorKeys, format }),
    [kind, labels, series, colorKeys, format],
  );
  type Row = { label: string; i: number };
  const rows: Row[] = labels.map((label, i) => ({ label, i }));
  const columns: TechColumn<Row>[] = [
    { key: 'label', header: t('pesquisa-periodo'), cell: (r) => r.label, width: '90px' },
    ...series.map((s): TechColumn<Row> => ({
      key: s.key,
      header: s.name,
      numeric: true,
      cell: (r) => {
        const v = s.values[r.i];
        return v == null ? '—' : format(v);
      },
    })),
  ];
  return (
    <ChartFrame
      index={index}
      title={title}
      source={source}
      chart={<EChart option={option} height={240} ariaLabel={ariaLabel} />}
      table={
        <TechTable
          label={tableLabel}
          prefsId={tablePrefsId}
          dense
          rows={rows}
          getRowId={(r) => r.label}
          columns={columns}
        />
      }
    />
  );
}

export function HistoryPanel({ subject }: { subject: SubjectArgs }) {
  const q = useHistory({ ...subject, years: 10 });
  const data = useMemo(() => (q.data ? historyCharts(q.data) : null), [q.data]);
  if (q.isPending) return <LoadingBlock label={t('pesquisa-hist-carregando')} />;
  if (q.isError) return <SubjectError error={q.error} onRetry={() => void q.refetch()} />;
  const h = q.data;
  const c = data!;
  const cur = h.currency;
  const name = h.subject.ticker ?? h.subject.name;
  const notes = h.notes;
  const money = (v: number) => compactAmount(v, cur);
  const perShare = (v: number) => unitAmount(v, cur);
  const pct = (v: number) => formatPercent(v, { decimals: 1 });
  const fii = h.subject.class === 'fii';
  if (h.years.length === 0 && h.months.length === 0) {
    return (
      <EmptyState title={t('pesquisa-hist-vazio')} description={t('pesquisa-hist-vazio-texto')} />
    );
  }
  const first = c.years[0];
  const last = c.years[c.years.length - 1];
  const span = first && last ? `${first}–${last}` : '';
  return (
    <div className="flex flex-col gap-5">
      <div className="grid gap-5 @min-[1040px]:grid-cols-2">
        {c.years.length ? (
          <Figure
            index="01"
            title={
              fii ? t('pesquisa-hist-resultado-distribuivel') : t('pesquisa-hist-receita-lucro')
            }
            source={
              fii
                ? t('pesquisa-hist-fonte-fii', { moeda: cur })
                : t('pesquisa-hist-fonte-dfp', { moeda: cur })
            }
            labels={c.years}
            series={c.money}
            colorKeys={MONEY_KEYS}
            format={money}
            kind="bars"
            ariaLabel={t('pesquisa-hist-receita-lucro-rotulo', { nome: name, periodo: span })}
            tableLabel={t('pesquisa-hist-receita-lucro')}
            tablePrefsId="Receita e lucro por exercício"
          />
        ) : null}
        {c.ratios.length ? (
          <Figure
            index="02"
            title={t('pesquisa-hist-margens')}
            source={t('pesquisa-hist-margens-fonte')}
            labels={c.years}
            series={c.ratios}
            colorKeys={RATIO_KEYS}
            format={pct}
            kind="lines"
            ariaLabel={t('pesquisa-hist-margens-rotulo', { nome: name, periodo: span })}
            tableLabel={t('pesquisa-hist-margens-tabela')}
            tablePrefsId="Margens e ROE por exercício"
          />
        ) : null}
        {c.years.length ? (
          <Figure
            index={c.ratios.length ? '03' : '02'}
            title={t('pesquisa-hist-dividendos')}
            source={t('pesquisa-hist-dividendos-fonte')}
            labels={c.years}
            series={c.dps}
            colorKeys={['dps']}
            format={perShare}
            kind="bars"
            ariaLabel={t('pesquisa-hist-dividendos-rotulo', { nome: name, periodo: span })}
            tableLabel={t('pesquisa-hist-dividendos')}
            tablePrefsId="Dividendos por ação por ano"
          />
        ) : null}
        {c.months.length ? (
          <>
            <Figure
              index="04"
              title={t('pesquisa-hist-rendimento')}
              source={t('pesquisa-hist-fonte-informe-fii')}
              labels={c.months}
              series={c.income}
              colorKeys={['rendimento']}
              format={perShare}
              kind="bars"
              ariaLabel={t('pesquisa-hist-rendimento-rotulo', { nome: name })}
              tableLabel={t('pesquisa-hist-rendimento-tabela')}
              tablePrefsId="Rendimento por cota por mês"
            />
            <Figure
              index="05"
              title={t('pesquisa-hist-vp')}
              source={t('pesquisa-hist-fonte-informe-fii')}
              labels={c.months}
              series={c.nav}
              colorKeys={['vp']}
              format={perShare}
              kind="lines"
              ariaLabel={t('pesquisa-hist-vp-rotulo', { nome: name })}
              tableLabel={t('pesquisa-hist-vp-tabela')}
              tablePrefsId="Valor patrimonial por cota por mês"
            />
          </>
        ) : null}
      </div>
      {notes.length ? <p className="text-[12px] text-dim">{notes.join(' ')}</p> : null}
    </div>
  );
}
