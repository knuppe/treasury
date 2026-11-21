/**
 * Alocação: donut por classe, desvio do alvo, por setor, alocação-alvo editável e
 * rebalanceamento com novo aporte (`portfolio.rebalance`), sempre com o aviso de
 * que é uma análise dos seus dados, não recomendação.
 */
import { useEffect, useMemo, useState } from 'react';

import {
  usePortfolioSummary,
  useRebalance,
  useSetTargets,
  useTargets,
} from '@/api/hooks/portfolio';
import type { AssetClass } from '@/api/generated/AssetClass';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import type { RebalanceConcentrationLine } from '@/api/generated/RebalanceConcentrationLine';
import type { RebalanceLine } from '@/api/generated/RebalanceLine';
import type { Target } from '@/api/generated/Target';
import {
  Button,
  Field,
  Icon,
  InstrumentCard,
  Money,
  MoneyInput,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { FormError } from '@/vault/FormError';

import { AllocationDonut, DeviationChart } from './charts/AllocationCharts';
import { DecimalInput } from './components/DecimalInput';
import { ObjectiveAllocation } from './ObjectiveAllocation';
import { SectorAllocation } from './SectorAllocation';
import { Pct } from './components/Pct';
import { ErrorBlock, LoadingBlock, Notice } from './components/States';
import { addDecimal, compareDecimal, formatDecimal, subtractDecimal } from './decimal';
import { CLASS_LABEL, CLASS_ORDER, classLabel } from './labels';
import { usePortfolioUi } from './state';

interface TargetRow {
  key: AssetClass;
  target: string | null;
  tolerance: string | null;
}

function rowsFrom(targets: readonly Target[], summary: PortfolioSummary | undefined): TargetRow[] {
  const keys = new Set<AssetClass>();
  for (const c of summary?.by_class ?? []) keys.add(c.class);
  for (const target of targets) if (target.scope === 'class') keys.add(target.key as AssetClass);
  return CLASS_ORDER.filter((c) => keys.has(c)).map((c) => {
    const target = targets.find((x) => x.scope === 'class' && x.key === c);
    return {
      key: c,
      target: target?.target_pct ?? null,
      tolerance: target?.tolerance_pct ?? null,
    };
  });
}

/** Soma exata dos alvos (decimais em texto). */
export function targetsTotal(rows: readonly { target: string | null }[]): string {
  return rows.reduce((acc, r) => (r.target ? addDecimal(acc, r.target) : acc), '0');
}

/** Colunas da tabela de alvos: o cabeçalho e as linhas usam as mesmas, para alinhar. */
const TARGET_COLS = 'grid grid-cols-[minmax(0,1fr)_60px_84px_84px] gap-2';

export function TargetsEditor({ summary }: { summary: PortfolioSummary | undefined }) {
  const targetsQ = useTargets();
  const save = useSetTargets();
  const [rows, setRows] = useState<TargetRow[] | null>(null);
  const base = useMemo(() => rowsFrom(targetsQ.data ?? [], summary), [targetsQ.data, summary]);
  const current = rows ?? base;
  const total = targetsTotal(current);
  const over = compareDecimal(total, '100') > 0;
  const exact = compareDecimal(total, '100') === 0;
  const dirty = rows != null;
  const missing = CLASS_ORDER.filter((c) => !current.some((r) => r.key === c));
  const weight = (c: AssetClass) => summary?.by_class.find((x) => x.class === c)?.weight ?? 0;

  const setRow = (key: AssetClass, patch: Partial<TargetRow>) =>
    setRows(current.map((r) => (r.key === key ? { ...r, ...patch } : r)));

  const submit = () => {
    const targets = current
      .filter((r) => r.target != null)
      .map((r) => ({
        scope: 'class' as const,
        key: r.key,
        target_pct: r.target!,
        ...(r.tolerance ? { tolerance_pct: r.tolerance } : {}),
      }));
    save.mutate(
      { scope: 'class', targets },
      {
        onSuccess: () => {
          setRows(null);
          toast({ tone: 'success', title: t('carteira-alvos-salvos') });
        },
      },
    );
  };

  return (
    <InstrumentCard
      index="04"
      title={t('carteira-alvos-titulo')}
      footer={
        <div className="flex flex-wrap items-center gap-3">
          <span
            className={cx(
              'label-caps',
              over ? 'text-negative' : exact ? 'text-positive' : 'text-warning',
            )}
          >
            {over
              ? t('carteira-alvos-soma-passa', { total: formatDecimal(total, { maxDecimals: 2 }) })
              : exact
                ? t('carteira-alvos-soma-completa', {
                    total: formatDecimal(total, { maxDecimals: 2 }),
                  })
                : t('carteira-alvos-soma-faltam', {
                    total: formatDecimal(total, { maxDecimals: 2 }),
                    falta: formatDecimal(subtractDecimal('100', total), { maxDecimals: 2 }),
                  })}
          </span>
          <span className="ml-auto flex gap-2">
            {dirty ? (
              <Button size="sm" variant="ghost" onClick={() => setRows(null)}>
                {t('carteira-descartar')}
              </Button>
            ) : null}
            <Button
              size="sm"
              variant="primary"
              disabled={!dirty || over}
              loading={save.isPending}
              onClick={submit}
            >
              {t('carteira-salvar-alvos')}
            </Button>
          </span>
        </div>
      }
    >
      {targetsQ.isPending ? (
        <LoadingBlock />
      ) : targetsQ.isError ? (
        <ErrorBlock error={targetsQ.error} onRetry={() => void targetsQ.refetch()} />
      ) : (
        <div className="flex flex-col gap-3">
          <div role="table" aria-label={t('carteira-alvos-tabela')} className="flex flex-col">
            <div
              role="row"
              className={cx(
                TARGET_COLS,
                'label-caps border-b border-line pb-1.5 whitespace-nowrap',
              )}
            >
              <span role="columnheader">{t('carteira-col-classe')}</span>
              <span role="columnheader" className="text-right">
                {t('carteira-col-atual')}
              </span>
              {/* Os números dos campos ficam recuados pelo preenchimento do campo. */}
              <span role="columnheader" className="pr-3 text-right">
                {t('carteira-col-alvo-pct')}
              </span>
              <span role="columnheader" className="pr-3 text-right">
                {t('carteira-col-tolerancia')}
              </span>
            </div>
            {current.map((r) => (
              <div
                role="row"
                key={r.key}
                className={cx(TARGET_COLS, 'items-center border-b border-line/60 py-1.5')}
              >
                <span
                  role="cell"
                  className="truncate text-[13px] text-fg"
                  title={CLASS_LABEL[r.key]}
                >
                  {CLASS_LABEL[r.key]}
                </span>
                <span role="cell" className="text-right text-[12.5px]">
                  <Pct value={weight(r.key)} className="text-dim" />
                </span>
                <span role="cell">
                  <DecimalInput
                    aria-label={t('carteira-alvo-de', { classe: CLASS_LABEL[r.key] })}
                    value={r.target}
                    maxDecimals={2}
                    onValueChange={(v) => setRow(r.key, { target: v })}
                    placeholder="—"
                    wrapperClassName="h-8"
                  />
                </span>
                <span role="cell">
                  <DecimalInput
                    aria-label={t('carteira-tolerancia-de', { classe: CLASS_LABEL[r.key] })}
                    value={r.tolerance}
                    maxDecimals={2}
                    onValueChange={(v) => setRow(r.key, { tolerance: v })}
                    placeholder="5"
                    wrapperClassName="h-8"
                  />
                </span>
              </div>
            ))}
          </div>
          {missing.length ? (
            <div className="flex items-center gap-2">
              <span className="label-caps">{t('carteira-adicionar-classe')}</span>
              <div className="w-56">
                <Select
                  aria-label={t('carteira-adicionar-classe-aria')}
                  value={undefined}
                  placeholder={t('carteira-escolha-classe')}
                  onValueChange={(v) =>
                    setRows(
                      [...current, { key: v as AssetClass, target: null, tolerance: null }].sort(
                        (a, b) => CLASS_ORDER.indexOf(a.key) - CLASS_ORDER.indexOf(b.key),
                      ),
                    )
                  }
                  options={missing.map((c) => ({ value: c, label: CLASS_LABEL[c] }))}
                />
              </div>
            </div>
          ) : null}
          <p className="text-[12px] text-dim">{t('carteira-tolerancia-texto')}</p>
          <FormError error={save.error} />
        </div>
      )}
    </InstrumentCard>
  );
}

const rebalanceColumns = (): TechColumn<RebalanceLine>[] => [
  {
    key: 'label',
    header: t('carteira-col-classe'),
    cell: (l) => (
      <span className="flex items-center gap-1.5 font-sans whitespace-nowrap">
        {classLabel(l.class, l.label)}
        {l.out_of_band ? (
          <span
            className="inline-flex items-center text-warning"
            title={t('carteira-fora-tolerancia-alvo')}
          >
            <Icon name="alert" size={12} />
            <span className="sr-only">{t('carteira-fora-tolerancia-alvo-curto')}</span>
          </span>
        ) : null}
      </span>
    ),
  },
  {
    key: 'current',
    header: t('carteira-col-atual'),
    numeric: true,
    cell: (l) => <Money cents={l.current} />,
  },
  {
    key: 'current_pct',
    header: t('carteira-col-atual-pct'),
    numeric: true,
    cell: (l) => <Pct value={l.current_pct / 100} />,
  },
  {
    key: 'target_pct',
    header: t('carteira-col-alvo-pct'),
    numeric: true,
    cell: (l) => <Pct value={l.target_pct / 100} />,
  },
  {
    key: 'gap',
    header: t('carteira-col-falta-sobra'),
    numeric: true,
    cell: (l) => <Money cents={l.gap} tone="auto" />,
    sortValue: (l) => l.gap,
  },
  {
    key: 'suggested',
    header: t('carteira-col-do-aporte'),
    numeric: true,
    cell: (l) =>
      l.suggested > 0 ? (
        <Money cents={l.suggested} tone="positive" />
      ) : (
        <span className="text-dim">—</span>
      ),
    sortValue: (l) => l.suggested,
  },
];

/** Sem alvo de classe: o aporte por ativo dentro dos limites de concentração (nunca venda). */
const concentrationColumns = (): TechColumn<RebalanceConcentrationLine>[] => [
  {
    key: 'asset',
    header: t('carteira-col-ativo'),
    cell: (l) => <span className="font-sans">{l.asset.ticker ?? l.asset.name}</span>,
    sortValue: (l) => l.asset.ticker ?? l.asset.name,
  },
  {
    key: 'current',
    header: t('carteira-col-atual'),
    numeric: true,
    cell: (l) => <Money cents={l.current} />,
    sortValue: (l) => l.current,
  },
  {
    key: 'limit',
    header: t('carteira-conc-col-limite'),
    numeric: true,
    cell: (l) => <Pct value={l.limit} />,
    sortValue: (l) => l.limit,
  },
  {
    key: 'room',
    header: t('carteira-rebal-conc-espaco'),
    numeric: true,
    cell: (l) => <Money cents={l.room} />,
    sortValue: (l) => l.room,
  },
  {
    key: 'suggested',
    header: t('carteira-col-do-aporte'),
    numeric: true,
    cell: (l) =>
      l.over_limit ? (
        <span className="font-sans text-[12px] text-warning">{t('carteira-rebal-conc-acima')}</span>
      ) : l.suggested > 0 ? (
        <Money cents={l.suggested} />
      ) : (
        '—'
      ),
    sortValue: (l) => (l.over_limit ? -1 : l.suggested),
  },
];

export function RebalancePanel() {
  const [input, setInput] = useState<number | null>(null);
  const [contribution, setContribution] = useState(0);
  // Espera a pessoa parar de digitar para recalcular.
  useEffect(() => {
    const id = setTimeout(() => setContribution(Math.max(0, input ?? 0)), 300);
    return () => clearTimeout(id);
  }, [input]);
  const q = useRebalance(contribution > 0 ? { contribution } : {});
  const columns = useMemo(() => rebalanceColumns(), []);
  const concCols = useMemo(() => concentrationColumns(), []);
  const r = q.data;
  const totalSuggested = r?.lines.reduce((s, l) => s + l.suggested, 0) ?? 0;
  return (
    <InstrumentCard index="05" title={t('carteira-rebal-titulo')} padded={false}>
      <div className="flex flex-col gap-3 p-4">
        <div className="flex flex-wrap items-end gap-4">
          <Field
            label={t('carteira-novo-aporte')}
            className="w-72"
            hint={t('carteira-novo-aporte-dica')}
          >
            <MoneyInput value={input} onValueChange={setInput} />
          </Field>
          {r ? (
            <dl className="flex flex-wrap gap-x-6 gap-y-1 pb-5 text-[12.5px]">
              <div className="flex items-baseline gap-2">
                <dt className="label-caps">{t('carteira-carteira-hoje')}</dt>
                <dd>
                  <Money cents={r.total} />
                </dd>
              </div>
              <div className="flex items-baseline gap-2">
                <dt className="label-caps">{t('carteira-depois-aporte')}</dt>
                <dd>
                  <Money cents={r.total + r.contribution} />
                </dd>
              </div>
            </dl>
          ) : null}
        </div>
        {r?.targets_incomplete ? (
          <Notice tone="warning" title={t('carteira-alvos-incompletos')}>
            {t('carteira-alvos-incompletos-texto')}
          </Notice>
        ) : null}
        {r && r.unallocated > 0 ? (
          <Notice tone="warning" title={t('carteira-aporte-sem-destino')}>
            <Money cents={r.unallocated} /> {t('carteira-aporte-sem-destino-texto')}
          </Notice>
        ) : null}
      </div>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <div className="p-4">
          <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
        </div>
      ) : r && r.lines.length === 0 ? (
        <>
          <p className="px-4 pb-4 text-[13px] text-dim">{t('carteira-defina-alvos')}</p>
          {r.concentration.length ? (
            <>
              <p className="px-4 pb-2 text-[12.5px] text-dim">{t('carteira-rebal-conc-texto')}</p>
              <TechTable
                label={t('carteira-rebal-conc-tabela')}
                prefsId="Aporte pelos limites de concentração"
                rows={r.concentration}
                columns={concCols}
                getRowId={(l) => l.asset.id}
                defaultSort={{ key: 'suggested', direction: 'desc' }}
              />
            </>
          ) : null}
        </>
      ) : r ? (
        <TechTable
          label={t('carteira-rebal-tabela')}
          prefsId="Rebalanceamento por classe"
          rows={r.lines}
          columns={columns}
          getRowId={(l) => l.class}
          defaultSort={{ key: 'gap', direction: 'desc' }}
          footer={{
            label: t('carteira-total'),
            current: <Money cents={r.total} />,
            suggested: totalSuggested > 0 ? <Money cents={totalSuggested} /> : undefined,
          }}
        />
      ) : null}
      <div className="p-4">
        {/* O aviso do núcleo (`disclaimer`) é este mesmo texto fixo; vem do catálogo no idioma do app. */}
        <Notice title={t('carteira-nao-recomendacao')}>
          {t('carteira-rebal-aviso')} {t('carteira-rebal-aviso-sugestao')}
        </Notice>
      </div>
    </InstrumentCard>
  );
}

export function AllocationPanel() {
  const deflator = usePortfolioUi((s) => s.deflator);
  const q = usePortfolioSummary({ deflator });
  if (q.isPending) return <LoadingBlock label={t('carteira-calculando-alocacao')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const s = q.data;
  return (
    <div className="flex flex-col gap-5">
      <div className="grid gap-5 @min-[1040px]:grid-cols-2">
        <AllocationDonut index="01" byClass={s.by_class} total={s.market_value} />
        <DeviationChart index="02" byClass={s.by_class} />
      </div>
      <SectorAllocation index="03" summary={s} />
      <ObjectiveAllocation index="04" />
      <div className="grid gap-5 @min-[1040px]:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
        <TargetsEditor summary={s} />
        <RebalancePanel />
      </div>
    </div>
  );
}
