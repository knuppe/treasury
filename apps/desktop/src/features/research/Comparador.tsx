/**
 * Comparador (`research.compare`): 2 a 6 papéis lado a lado, melhor valor de cada linha
 * destacado (pela ordem natural do indicador, com marca além da cor) e a cotação
 * normalizada em base 100 no mesmo dia para todos.
 */
import { Fragment, useCallback, useMemo, useRef, useState, type FormEvent } from 'react';

import type { CompareRow } from '@/api/generated/CompareRow';
import type { CompareView } from '@/api/generated/CompareView';
import { useCompare } from '@/api/hooks/research';
import { ChartFrame, EChart, type ChartTheme } from '@/charts';
import {
  Button,
  EmptyState,
  Icon,
  IconButton,
  Input,
  TechTable,
  useDataColumns,
  type DataColumn,
  useStickyHeader,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatNumber } from '@/lib/format';

import { ErrorBlock, LoadingBlock, Notice } from '../portfolio/components/States';
import { CLASS_LABEL, safeDate } from '../portfolio/labels';
import { linesOption } from './charts';
import { bestOfRow, groupRows, rebaseCommon } from './compare';
import { SubjectError, TickerLink, useUniverseList } from './common';
import { formatIndicatorValue } from './format';
import { GROUP_LABEL } from './labels';

export const MAX_COMPARE = 6;

function CompareTable({ view }: { view: CompareView }) {
  const label = t('pesquisa-cmp-tabela');
  // Uma coluna por papel: ordenar reordena os indicadores dentro de cada grupo.
  const columns = useMemo<DataColumn<CompareRow>[]>(
    () => [
      {
        key: 'indicator',
        header: t('pesquisa-cmp-indicador'),
        hideable: false,
        sortValue: (r) => r.label,
      },
      ...view.subjects.map((s, i) => ({
        key: s.ticker ?? s.name,
        title: s.ticker ?? s.name,
        numeric: true,
        header: (
          <span className="flex flex-col items-end leading-tight">
            <TickerLink assetId={s.asset_id} ticker={s.ticker ?? s.name} />
            <span className="font-sans text-[11px] font-normal text-dim">
              {CLASS_LABEL[s.class]}
            </span>
          </span>
        ),
        sortValue: (r: CompareRow) => r.values[i],
      })),
    ],
    [view.subjects],
  );
  const cols = useDataColumns({
    label,
    prefsId: 'Comparação de indicadores',
    columns,
    rows: view.rows,
  });
  const boxRef = useRef<HTMLDivElement>(null);
  useStickyHeader(boxRef);
  const index = new Map(view.subjects.map((s, i) => [s.ticker ?? s.name, i]));
  const groups = groupRows(view.rows);
  return (
    <div ref={boxRef} className="overflow-x-auto rounded-card border border-line bg-surface">
      <table
        className={cx('border-collapse text-[13px]', !cols.fixed && 'w-full')}
        style={cols.tableStyle}
        aria-label={label}
      >
        {cols.colgroup}
        <thead>
          <tr className="border-b border-line">
            {cols.shown.map((c) =>
              c.key === 'indicator'
                ? cols.headerCell(c, { className: 'label-caps px-3 py-2' })
                : cols.headerCell(c, { className: 'px-3 py-2', sortIconOnly: true }),
            )}
            {cols.pickerCell('px-1 py-2')}
          </tr>
        </thead>
        <tbody>
          {groups.map((g) => (
            <Fragment key={g.group}>
              <tr className="bg-surface-2/60">
                <th
                  colSpan={cols.colSpan}
                  scope="colgroup"
                  className="label-caps px-3 py-1.5 text-left text-fg"
                >
                  {GROUP_LABEL[g.group]}
                </th>
              </tr>
              {cols.sortRows(g.rows).map((r) => {
                const best = bestOfRow(r);
                return (
                  <tr key={r.key} className="border-b border-line/60 last:border-0">
                    {cols.shown.map((c) => {
                      if (c.key === 'indicator')
                        return (
                          <th
                            key={c.key}
                            scope="row"
                            className="overflow-hidden px-3 py-1.5 text-left font-sans font-normal text-ellipsis text-fg"
                          >
                            {r.label}
                          </th>
                        );
                      const i = index.get(c.key) ?? -1;
                      const v = r.values[i] ?? null;
                      const isBest = best.has(i);
                      const cur = view.subjects[i]?.currency;
                      return (
                        <td
                          key={c.key}
                          data-best={isBest || undefined}
                          className={cx(
                            'num px-3 py-1.5 text-right whitespace-nowrap',
                            cols.fixed && 'overflow-hidden text-ellipsis',
                            v == null ? 'text-dim' : 'text-fg',
                            isBest && 'bg-[var(--accent-faint)] font-semibold text-accent',
                          )}
                        >
                          {isBest ? (
                            <span aria-hidden className="mr-1 text-[10px]">
                              {'◆'}
                            </span>
                          ) : null}
                          {formatIndicatorValue(v, r.unit, cur)}
                          {isBest ? (
                            <span className="sr-only"> {t('pesquisa-cmp-melhor')}</span>
                          ) : null}
                        </td>
                      );
                    })}
                    {cols.fillerCell()}
                  </tr>
                );
              })}
            </Fragment>
          ))}
        </tbody>
      </table>
      {cols.menus}
    </div>
  );
}

function PriceIndexChart({ view }: { view: CompareView }) {
  const data = useMemo(() => rebaseCommon(view.price_index), [view.price_index]);
  const keys = useMemo(() => view.subjects.map((s) => s.ticker ?? s.name), [view.subjects]);
  const labels = useMemo(() => data.dates.map((d) => safeDate(d).slice(0, 5)), [data.dates]);
  const series = useMemo(
    () => data.series.map((s) => ({ key: s.ticker, name: s.ticker, values: s.values })),
    [data.series],
  );
  const option = useCallback(
    (theme: ChartTheme) =>
      linesOption(theme, {
        labels,
        series,
        colorKeys: keys,
        format: (v) => formatNumber(v, 0),
        baseline: 100,
      }),
    [labels, series, keys],
  );
  type Row = { date: string; i: number };
  // Tabela: um ponto por mês (o último pregão de cada mês) para ficar legível.
  const rows: Row[] = data.dates
    .map((date, i) => ({ date, i }))
    .filter((r, idx, all) => all[idx + 1]?.date.slice(0, 7) !== r.date.slice(0, 7));
  if (!data.base)
    return (
      <Notice title={t('pesquisa-cmp-sem-historico')}>
        {t('pesquisa-cmp-sem-historico-texto')}
      </Notice>
    );
  const source = t('pesquisa-cmp-fonte', { data: safeDate(data.base) });
  return (
    <ChartFrame
      index="01"
      title={t('pesquisa-cmp-cotacao-normalizada')}
      source={
        data.missing.length
          ? t('pesquisa-cmp-fonte-faltando', { fonte: source, tickers: data.missing.join(', ') })
          : source
      }
      chart={
        <EChart
          option={option}
          height={300}
          ariaLabel={t('pesquisa-cmp-grafico-rotulo', {
            tickers: series.map((s) => s.name).join(', '),
            data: safeDate(data.base),
          })}
        />
      }
      table={
        <TechTable
          label={t('pesquisa-cmp-tabela-mensal')}
          prefsId="Cotação normalizada por mês"
          dense
          rows={rows}
          getRowId={(r) => r.date}
          columns={[
            { key: 'date', header: t('pesquisa-cmp-data'), cell: (r) => safeDate(r.date) },
            ...series.map((s) => ({
              key: s.key,
              header: s.name,
              numeric: true,
              cell: (r: Row) => {
                const v = s.values[r.i];
                return v == null ? '—' : formatNumber(v, 1);
              },
            })),
          ]}
        />
      }
    />
  );
}

export function Comparador({
  tickers,
  onChange,
}: {
  tickers: string[];
  onChange: (tickers: string[]) => void;
}) {
  const universe = useUniverseList();
  const [text, setText] = useState('');
  const [error, setError] = useState<string | null>(null);
  const q = useCompare(tickers.length >= 2 ? { tickers } : null);
  const items = universe.data?.items ?? [];

  const add = (e: FormEvent) => {
    e.preventDefault();
    const ticker = text.trim().toUpperCase();
    if (!ticker) return;
    if (tickers.includes(ticker)) return setError(t('pesquisa-cmp-ja-esta', { ticker }));
    if (tickers.length >= MAX_COMPARE)
      return setError(t('pesquisa-cmp-maximo', { n: MAX_COMPARE }));
    if (items.length && !items.some((i) => i.ticker === ticker))
      return setError(t('pesquisa-cmp-fora-universo', { ticker }));
    setError(null);
    setText('');
    onChange([...tickers, ticker]);
  };

  return (
    <div className="flex flex-col gap-4">
      <form
        onSubmit={add}
        className="flex flex-wrap items-center gap-2"
        aria-label={t('pesquisa-cmp-adicionar-rotulo')}
      >
        <div className="w-56">
          <Input
            aria-label={t('pesquisa-cmp-ticker')}
            placeholder={t('pesquisa-cmp-ticker-exemplo')}
            list="compare-universe"
            value={text}
            onChange={(e) => setText(e.target.value)}
            prefix={<Icon name="plus" size={12} />}
          />
          <datalist id="compare-universe">
            {items.map((i) => (
              <option key={i.ticker} value={i.ticker}>
                {i.name}
              </option>
            ))}
          </datalist>
        </div>
        <Button type="submit" disabled={tickers.length >= MAX_COMPARE}>
          {t('pesquisa-cmp-adicionar')}
        </Button>
        <ul className="flex flex-wrap items-center gap-1.5" aria-label={t('pesquisa-cmp-papeis')}>
          {tickers.map((ticker) => (
            <li
              key={ticker}
              className="num flex items-center gap-1 rounded-control border border-line-strong/70 bg-surface-2 py-0.5 pr-0.5 pl-2 text-[12px] text-fg"
            >
              {ticker}
              <IconButton
                size="sm"
                label={t('pesquisa-cmp-tirar', { ticker })}
                icon={<Icon name="x" size={11} />}
                onClick={() => onChange(tickers.filter((x) => x !== ticker))}
              />
            </li>
          ))}
        </ul>
        <span className="text-[12px] text-dim">
          {tickers.length}/{MAX_COMPARE}
        </span>
      </form>
      {error ? (
        <p role="alert" className="text-[12px] text-negative">
          {error}
        </p>
      ) : null}
      {tickers.length < 2 ? (
        <EmptyState
          title={t('pesquisa-cmp-escolha')}
          description={t('pesquisa-cmp-escolha-texto')}
          actions={
            <Button size="sm" variant="ghost" onClick={() => onChange(['PETR4', 'VALE3', 'WEGE3'])}>
              {t('pesquisa-cmp-exemplo')}
            </Button>
          }
        />
      ) : q.isPending ? (
        <LoadingBlock label={t('pesquisa-cmp-comparando')} />
      ) : q.isError ? (
        <SubjectError error={q.error} onRetry={() => void q.refetch()} />
      ) : (
        <>
          <PriceIndexChart view={q.data} />
          <CompareTable view={q.data} />
          <p className="text-[12px] text-dim">
            {t('pesquisa-cmp-rodape', { data: safeDate(q.data.as_of) })}
          </p>
        </>
      )}
      {universe.isError ? <ErrorBlock error={universe.error} /> : null}
    </div>
  );
}
