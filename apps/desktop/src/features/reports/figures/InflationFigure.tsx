/**
 * Figura 07 · Inflação: gastamos mais ou só ficou mais caro? O gasto de cada mês em valor
 * nominal e trazido a dinheiro do último mês pelo índice de inflação do país (IPCA no Brasil),
 * com fonte e data do índice, meses ainda não divulgados marcados com "≈" e a comparação de
 * cada categoria com a inflação do período.
 */
import { useMemo } from 'react';

import type { InflationReport } from '@/api/generated/InflationReport';
import { useInflation } from '@/api/hooks/reports';
import { ChartFrame } from '@/charts/ChartFrame';
import { InflationChart } from '@/charts/InflationChart';
import type { InflationChartInput } from '@/charts/inflationOption';
import { cssColor } from '@/charts/colorMath';
import { Icon, Money, OriginBadge, Private, TechTable, type TechColumn } from '@/design/components';
import { sourceName } from '@/features/portfolio/labels';
import { t } from '@/i18n';
import { formatMonthLong, formatMonthShort, formatNumber, formatPercent } from '@/lib/format';
import { monthBounds } from '@/lib/months';

import { QueryView } from '../controls';
import { othersLabel, OTHERS_KEY } from '../model/colors';
import { categoryDrill } from '../model/drill';
import { trendWindow } from '../model/period';
import { useCategoryColors } from '../useEntityColors';
import {
  basisLabel,
  FilterNote,
  Question,
  Stat,
  Stats,
  useDrill,
  type FigureProps,
} from './common';

/** Média de três meses (início ou fim da janela). */
const avg3 = (v: readonly number[]) => v.reduce((s, x) => s + x, 0) / Math.max(1, v.length);

/** Diferença em pontos percentuais: `+3,2 p.p.`. */
function points(diff: number): string {
  const v = Math.round(diff * 1000) / 10;
  const body = formatNumber(Math.abs(v), 1);
  return t('relatorios-pontos', { valor: `${v > 0 ? '+' : v < 0 ? '−' : ''}${body}` });
}

/** Variação do gasto (subir é ruim): ▲ em vermelho, ▼ em verde. */
function Change({ value, approx }: { value: number | null; approx?: boolean }) {
  if (value === null) return <>—</>;
  return (
    <span className={value > 0 ? 'text-negative' : value < 0 ? 'text-positive' : undefined}>
      {value > 0 ? '▲ ' : value < 0 ? '▼ ' : ''}
      <Private>
        {approx ? '≈ ' : ''}
        {formatPercent(value, { signed: true })}
      </Private>
    </span>
  );
}

/** Meses além do último dado oficial do índice. */
function unpublishedMonths(data: InflationReport): boolean[] {
  const last = data.index?.last_date?.slice(0, 7);
  return data.months.map((m) => (last ? m > last : true));
}

export function InflationFigure({ period, basis, memberId, accountId }: FigureProps) {
  const win = trendWindow(period);
  const query = useInflation({
    months: win.months,
    end_month: win.endMonth,
    basis: basis ?? undefined,
  });
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.nominal.every((v) => v === 0)}
      loadingLabel={t('inflacao-carregando')}
    >
      {(data) => (
        <InflationBody
          data={data}
          endMonth={win.endMonth}
          memberId={memberId}
          accountId={accountId}
        />
      )}
    </QueryView>
  );
}

function InflationBody({
  data,
  endMonth,
  memberId,
  accountId,
}: {
  data: InflationReport;
  endMonth: string;
  memberId: string | null;
  accountId: string | null;
}) {
  const colors = useCategoryColors();
  const drill = useDrill();
  const n = data.months.length;
  const unpublished = useMemo(() => unpublishedMonths(data), [data]);
  const input = useMemo<InflationChartInput>(
    () => ({
      months: data.months,
      nominal: data.nominal,
      real: data.real,
      estimated: data.estimated,
      unpublished,
      currency: data.currency,
    }),
    [data, unpublished],
  );
  const indexName = data.index?.name ?? t('inflacao-indice-generico');
  const hasIndex = data.real.slice(0, -1).some((v) => v !== null);
  const today = formatMonthLong(endMonth);

  // Gasto do começo × fim da janela, nominal e em dinheiro de hoje. O começo é o primeiro
  // mês com gasto (como na comparação por categoria do núcleo).
  const start = Math.max(
    0,
    data.nominal.findIndex((v) => v > 0),
  );
  const k = Math.min(3, Math.floor((n - start) / 2));
  const firstN = k > 0 ? avg3(data.nominal.slice(start, start + k)) : 0;
  const lastN = avg3(data.nominal.slice(n - k));
  const nominalChange = firstN > 0 ? lastN / firstN - 1 : null;
  const realFirst = data.real.slice(start, start + k);
  const realLast = data.real.slice(n - k);
  const realChange =
    k > 0 && hasIndex && realFirst.every((v) => v !== null) && realLast.every((v) => v !== null)
      ? (() => {
          const a = avg3(realFirst);
          const b = avg3(realLast);
          return a > 0 ? b / a - 1 : null;
        })()
      : null;
  const correction = data.real.reduce<number>(
    (s, v, i) => (v === null ? s : s + v - (data.nominal[i] ?? 0)),
    0,
  );
  const anyEstimated = data.estimated.some(Boolean) || data.inflation_estimated;

  type Row = {
    month: string;
    nominal: number;
    real: number | null;
    estimated: boolean;
    unpublished: boolean;
  };
  const rows: Row[] = data.months.map((m, i) => ({
    month: m,
    nominal: data.nominal[i] ?? 0,
    real: data.real[i] ?? null,
    estimated: data.estimated[i] ?? false,
    unpublished: unpublished[i] ?? false,
  }));
  const columns: TechColumn<Row>[] = [
    {
      key: 'month',
      header: t('relatorios-mes'),
      numeric: true,
      cell: (r) => `${formatMonthShort(r.month)}${r.unpublished ? ' ≈' : ''}`,
    },
    {
      key: 'nominal',
      header: t('inflacao-serie-nominal'),
      numeric: true,
      cell: (r) => <Money cents={r.nominal} />,
    },
    {
      key: 'real',
      header: t('inflacao-col-corrigido', { mes: formatMonthShort(endMonth) }),
      numeric: true,
      cell: (r) => (r.real === null ? '—' : <Money cents={r.real} estimated={r.estimated} />),
    },
    {
      key: 'correction',
      header: t('inflacao-col-correcao'),
      numeric: true,
      cell: (r) =>
        r.real === null || r.nominal <= 0 ? (
          '—'
        ) : (
          <Private>
            {r.estimated ? '≈ ' : ''}
            {formatPercent(r.real / r.nominal - 1, { signed: true })}
          </Private>
        ),
    },
    {
      key: 'status',
      header: t('inflacao-col-indice'),
      cell: (r) =>
        r.unpublished
          ? t('inflacao-status-nao-divulgado')
          : r.estimated
            ? t('inflacao-status-estimado')
            : t('inflacao-status-oficial'),
    },
  ];

  type CatRow = InflationReport['categories'][number] & { id: string; name: string };
  const catRows: CatRow[] = data.categories.map((c) => ({
    ...c,
    id: c.key ?? '__sem_categoria__',
    name:
      c.key === OTHERS_KEY
        ? othersLabel()
        : c.key === null
          ? t('relatorios-sem-categoria')
          : c.label,
  }));
  const catColumns: TechColumn<CatRow>[] = [
    {
      key: 'name',
      header: t('relatorios-categoria'),
      cell: (r) => (
        <span className="inline-flex items-center gap-2 font-sans">
          <span
            aria-hidden
            className="inline-block size-2 shrink-0 rounded-full"
            style={{
              background: cssColor((r.key && colors.get(r.key)) ?? r.color ?? 'var(--text-dim)'),
            }}
          />
          {r.name}
        </span>
      ),
    },
    {
      key: 'first',
      header: t('inflacao-col-inicio'),
      numeric: true,
      cell: (r) => <Money cents={r.first} />,
    },
    {
      key: 'last',
      header: t('inflacao-col-fim'),
      numeric: true,
      cell: (r) => <Money cents={r.last} />,
    },
    {
      key: 'change',
      header: t('inflacao-col-variacao'),
      numeric: true,
      cell: (r) => <Change value={r.change} />,
    },
    {
      key: 'inflation',
      header: indexName,
      numeric: true,
      cell: (r) =>
        r.inflation === null ? (
          '—'
        ) : (
          <Private>
            {data.inflation_estimated ? '≈ ' : ''}
            {formatPercent(r.inflation, { signed: true })}
          </Private>
        ),
    },
    {
      key: 'above',
      header: t('inflacao-col-acima'),
      numeric: true,
      cell: (r) =>
        r.above_inflation === null ? (
          '—'
        ) : (
          <span
            className={
              r.above_inflation > 0.005
                ? 'text-negative'
                : r.above_inflation < -0.005
                  ? 'text-positive'
                  : 'text-dim'
            }
          >
            <Private>{points(r.above_inflation)}</Private>
          </span>
        ),
    },
  ];
  const above = catRows.filter((r) => (r.above_inflation ?? 0) > 0.005).length;

  const source = (
    <span className="inline-flex flex-wrap items-center gap-1.5">
      {t('inflacao-fonte', { base: basisLabel(data.basis), mes: today })}
      {data.index ? (
        <span>
          ·{' '}
          {t('inflacao-fonte-indice', {
            indice: data.index.name,
            fonte: sourceName(data.index.source),
            data: data.index.last_date ? formatMonthShort(data.index.last_date.slice(0, 7)) : '—',
          })}
        </span>
      ) : null}
      {data.index ? (
        <OriginBadge
          source={`${data.index.name} · ${sourceName(data.index.source)}`}
          date={data.index.last_date ?? undefined}
          status={anyEstimated ? 'estimado' : 'oficial'}
          note={anyEstimated ? t('inflacao-nota-estimado') : undefined}
        />
      ) : null}
    </span>
  );

  return (
    <div className="flex flex-col gap-4">
      <Question>{t('inflacao-pergunta', { mes: today })}</Question>
      <Stats>
        <Stat
          label={t('inflacao-acumulada', { n })}
          hint={data.index ? data.index.name : t('inflacao-sem-indice-curto')}
        >
          {data.inflation === null ? (
            '—'
          ) : (
            <Private>
              {data.inflation_estimated ? '≈ ' : ''}
              {formatPercent(data.inflation, { signed: true })}
            </Private>
          )}
        </Stat>
        <Stat label={t('inflacao-gasto-nominal')} hint={t('inflacao-tres-meses')}>
          <Change value={nominalChange} />
        </Stat>
        <Stat label={t('inflacao-gasto-real')} hint={t('inflacao-tres-meses')}>
          <Change value={realChange} approx={anyEstimated} />
        </Stat>
        <Stat label={t('inflacao-correcao-total')} hint={t('inflacao-correcao-total-dica')}>
          {hasIndex ? <Money cents={correction} estimated={anyEstimated} /> : '—'}
        </Stat>
      </Stats>
      <FilterNote memberId={memberId} accountId={accountId} />
      {!hasIndex ? (
        <p role="note" className="flex items-center gap-2 text-[12px] text-warning">
          <Icon name="alert" size={13} className="shrink-0" />
          {t('inflacao-sem-indice', { indice: indexName })}
        </p>
      ) : null}
      {data.missing_fx.length ? (
        <p role="note" className="flex items-center gap-2 text-[12px] text-dim">
          <Icon name="info" size={13} className="shrink-0" />
          {t('inflacao-sem-cambio', { moedas: data.missing_fx.join(', ') })}
        </p>
      ) : null}
      <ChartFrame
        index="07"
        title={t('inflacao-titulo', { indice: indexName, n })}
        source={source}
        chart={
          <InflationChart
            input={input}
            ariaLabel={t('inflacao-aria', { indice: indexName, n, mes: today })}
          />
        }
        table={
          <TechTable
            label={t('inflacao-tabela')}
            prefsId="Gasto nominal e corrigido"
            columns={columns}
            rows={rows}
            getRowId={(r) => r.month}
            dense
          />
        }
      />
      {catRows.length ? (
        <section className="flex flex-col gap-2" aria-labelledby="inflacao-categorias">
          <h3 id="inflacao-categorias" className="label-caps text-fg">
            {t('inflacao-categorias-titulo')}
          </h3>
          <p className="text-[12.5px] text-dim">
            {t('inflacao-categorias-dica', {
              inicio: formatMonthShort(data.compare_from ?? data.months[0] ?? endMonth),
              fim: formatMonthShort(data.months[n - 1] ?? endMonth),
              acima: above,
              total: catRows.length,
            })}
          </p>
          <TechTable
            label={t('inflacao-categorias-tabela')}
            prefsId="Categorias contra a inflação"
            columns={catColumns}
            rows={catRows}
            getRowId={(r) => r.id}
            onActivate={(r) => {
              if (r.key === OTHERS_KEY) return;
              const from = monthBounds(data.months[Math.max(0, n - 3)] ?? endMonth).from;
              const period = { from, to: monthBounds(endMonth).to };
              drill(categoryDrill({ period, basis: data.basis }, [r.key]));
            }}
            dense
          />
        </section>
      ) : null}
    </div>
  );
}
