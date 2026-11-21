/** Previsto × realizado do mês: resumo com ritmo, tabela editável, copiar e sugerir. */
import { useMemo, useState, type ReactNode } from 'react';

import type { Basis } from '@/api/generated/Basis';
import type { BudgetStatus } from '@/api/generated/BudgetStatus';
import { isApiError } from '@/api/client';
import { useBudgetSuggestion, useCopyBudget, useSetBudgetLines } from '@/api/hooks/budget';
import { useCategories } from '@/api/hooks/reports';
import { BulletBar, BulletStatusLabel } from '@/charts/BulletBar';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Icon,
  InstrumentCard,
  Money,
  Private,
  Select,
  toast,
  useDataColumns,
  useFitWidth,
  type DataColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { capitalize, Segmented } from '@/features/reports/controls';
import { Explainable } from '@/features/reports/explain/ExplainButton';
import type { EntityColors } from '@/features/reports/model/colors';
import { categoryDrill } from '@/features/reports/model/drill';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import {
  baseCurrency,
  formatMoney,
  formatMonthLong,
  formatPercent,
  minorUnits,
} from '@/lib/format';
import { monthBounds, shiftMonth } from '@/lib/months';
import { txnLink } from '@/lib/txnLink';

import { BudgetTable } from './BudgetTable';
import { SpendGroupsCard } from './SpendGroupsCard';
import {
  addableCategories,
  budgetSummary,
  budgetTree,
  emptyLine,
  suggestionRows,
  unbudgetedWithSpending,
  type SuggestionRow,
} from './model';

/** Dez unidades da moeda base, sem os centavos zerados (`R$ 10`, `$10`, `10 €`). */
function tenUnits(): string {
  const currency = baseCurrency();
  const decimals = minorUnits(currency);
  const text = formatMoney(10 * 10 ** decimals, currency);
  return decimals > 0 ? text.replace(new RegExp(`[.,]0{${decimals}}(?=\\D*$)`), '') : text;
}

function errorText(e: unknown): string {
  return isApiError(e) || e instanceof Error ? e.message : t('orcamento-erro-gravar');
}

// ── Resumo ───────────────────────────────────────────────────────

function SummaryValue({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  const { box, content } = useFitWidth(children);
  return (
    <dl className="flex min-w-0 flex-col gap-1.5">
      <dt className="label-caps">{label}</dt>
      <dd
        ref={box}
        className="num text-[length:calc(22px*var(--fit,1))] leading-tight font-medium whitespace-nowrap text-fg"
      >
        <span ref={content} className="inline-block">
          {children}
        </span>
      </dd>
      {hint ? <dd className="text-[12px] text-dim">{hint}</dd> : null}
    </dl>
  );
}

function Summary({ status, basis }: { status: BudgetStatus; basis: Basis }) {
  const s = budgetSummary(status);
  const monthName = formatMonthLong(status.month);
  const period = monthBounds(status.month);
  return (
    <InstrumentCard
      index="A"
      title={t('orcamento-resumo-de', { mes: monthName })}
      footer={[
        {
          label: t('orcamento-base'),
          value: basis === 'cash' ? t('orcamento-base-caixa') : t('orcamento-base-competencia'),
        },
        { label: t('orcamento-mes-decorrido'), value: formatPercent(s.elapsed, { decimals: 0 }) },
        ...(s.uncategorized
          ? [{ label: t('controles-sem-categoria'), value: <Money cents={s.uncategorized} /> }]
          : []),
      ]}
    >
      <div className="grid gap-5 @min-[530px]:grid-cols-4 @min-[1040px]:grid-cols-[repeat(4,minmax(0,1fr))_minmax(0,1.6fr)]">
        <SummaryValue label={t('orcamento-previsto')}>
          <Money cents={s.planned} />
        </SummaryValue>
        <SummaryValue label={t('orcamento-realizado')}>
          <Explainable
            target={{
              title: t('orcamento-despesas-de', { mes: monthName }),
              args: { metric: 'spending', params: { ...period, basis } },
              link: txnLink(categoryDrill({ period, basis }, [])),
            }}
          >
            <Money cents={s.actual} estimated={status.estimated} />
          </Explainable>
        </SummaryValue>
        <SummaryValue label={t('orcamento-restante')}>
          <Money cents={s.remaining} tone={s.remaining < 0 ? 'negative' : 'neutral'} />
        </SummaryValue>
        <SummaryValue label={t('orcamento-comprometido')} hint={t('orcamento-comprometido-dica')}>
          <Money cents={s.committed} />
        </SummaryValue>
        <div className="flex flex-col justify-end gap-2 @min-[530px]:col-span-4 @min-[1040px]:col-span-1">
          <div className="flex items-baseline justify-between gap-3">
            <span className="label-caps">{t('orcamento-ritmo')}</span>
            <BulletStatusLabel status={s.status} />
          </div>
          <BulletBar
            label={t('orcamento-de', { mes: monthName })}
            planned={s.planned}
            actual={s.actual}
            committed={s.committed}
            elapsed={s.elapsed}
          />
          <p className="num text-[12px] text-dim">
            {tRich('orcamento-ritmo-texto', {
              pct: (
                <Private>{s.ratio != null ? formatPercent(s.ratio, { decimals: 0 }) : '—'}</Private>
              ),
              decorrido: formatPercent(s.elapsed, { decimals: 0 }),
              projecao: <Money cents={s.projected} />,
            })}
          </p>
        </div>
      </div>
    </InstrumentCard>
  );
}

// ── Copiar do mês anterior ───────────────────────────────────────

function CopyDialog({ month }: { month: string }) {
  const [open, setOpen] = useState(false);
  const [overwrite, setOverwrite] = useState(false);
  const copy = useCopyBudget();
  const prev = shiftMonth(month, -1);
  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        setOpen(o);
        if (!o) copy.reset();
      }}
      title={t('orcamento-copiar-titulo')}
      description={t('orcamento-copiar-descricao', {
        de: formatMonthLong(prev),
        para: formatMonthLong(month),
      })}
      trigger={
        <Button size="sm" icon={<Icon name="refresh" size={13} />}>
          {t('orcamento-copiar-anterior')}
        </Button>
      }
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="primary"
            loading={copy.isPending}
            onClick={() =>
              copy.mutate(
                { from_month: prev, to_month: month, overwrite },
                {
                  onSuccess: () => {
                    setOpen(false);
                    toast({
                      tone: 'success',
                      title: t('orcamento-copiado'),
                      description: `${capitalize(formatMonthLong(prev))} → ${formatMonthLong(month)}.`,
                    });
                  },
                },
              )
            }
          >
            {t('orcamento-copiar')}
          </Button>
        </>
      }
    >
      <label className="flex items-center gap-2 text-[13px] text-fg">
        <input
          type="checkbox"
          checked={overwrite}
          onChange={(e) => setOverwrite(e.target.checked)}
        />
        {t('orcamento-substituir', { mes: formatMonthLong(month) })}
      </label>
      {copy.error ? (
        <p role="alert" className="text-[12.5px] text-negative">
          {errorText(copy.error)}
        </p>
      ) : null}
    </Dialog>
  );
}

// ── Sugerir pela média ───────────────────────────────────────────

function suggestionColumns(): DataColumn<SuggestionRow>[] {
  return [
    {
      key: 'apply',
      header: <span className="sr-only">{t('orcamento-aplicar')}</span>,
      sortable: false,
    },
    { key: 'name', header: t('controles-categoria'), hideable: false },
    { key: 'current', header: t('orcamento-atual'), numeric: true },
    { key: 'suggested', header: t('orcamento-media'), numeric: true },
    { key: 'delta', header: t('orcamento-diferenca'), numeric: true },
  ];
}

function SuggestionTable({
  rows,
  skipped,
  onSkip,
}: {
  rows: readonly SuggestionRow[];
  skipped: ReadonlySet<string>;
  onSkip: (categoryId: string, skip: boolean) => void;
}) {
  const label = t('orcamento-sugestao');
  const columns = useMemo(() => suggestionColumns(), []);
  // As preferências da tabela ficam no nome em pt-BR, para valerem em qualquer idioma.
  const cols = useDataColumns({ label, prefsId: 'Sugestão pela média', columns, rows });
  const cell = (key: string, r: SuggestionRow): ReactNode => {
    switch (key) {
      case 'apply':
        return (
          <input
            type="checkbox"
            aria-label={t('orcamento-aplicar-a', { categoria: r.name })}
            checked={!skipped.has(r.category_id)}
            onChange={(e) => onSkip(r.category_id, !e.target.checked)}
          />
        );
      case 'name':
        return <span className="text-fg">{r.name}</span>;
      case 'current':
        return r.current ? <Money cents={r.current} /> : '—';
      case 'suggested':
        return (
          <span className="text-fg">
            <Money cents={r.suggested} />
          </span>
        );
      case 'delta':
        return <Money cents={r.delta} tone="auto" />;
      default:
        return null;
    }
  };
  return (
    <>
      <table
        className={cx('text-[13px]', !cols.fixed && 'w-full')}
        style={cols.tableStyle}
        aria-label={label}
      >
        {cols.colgroup}
        <thead>
          <tr>
            {cols.shown.map((c) =>
              cols.headerCell(c, {
                className: cx('label-caps py-1.5 font-medium', c.key === 'apply' && 'w-8'),
              }),
            )}
            {cols.pickerCell('py-1.5')}
          </tr>
        </thead>
        <tbody>
          {cols.sortRows(rows).map((r) => (
            <tr key={r.category_id} className="border-t border-line/70">
              {cols.shown.map((c) => (
                <td
                  key={c.key}
                  className={cx(
                    'py-1.5',
                    c.numeric && 'num text-right',
                    c.key === 'current' && 'text-dim',
                    cols.fixed && 'overflow-hidden',
                  )}
                >
                  {cell(c.key, r)}
                </td>
              ))}
              {cols.fillerCell()}
            </tr>
          ))}
        </tbody>
      </table>
      {cols.menus}
    </>
  );
}

function SuggestDialog({ status, basis }: { status: BudgetStatus; basis: Basis }) {
  const [open, setOpen] = useState(false);
  const [monthsBack, setMonthsBack] = useState<'3' | '6' | '12'>('3');
  const suggestion = useBudgetSuggestion(
    { month: status.month, months_back: Number(monthsBack), basis },
    open,
  );
  const categories = useCategories({ kind: 'expense' });
  const save = useSetBudgetLines();
  const rows = useMemo(
    () => suggestionRows(status, suggestion.data ?? [], categories.data ?? []),
    [status, suggestion.data, categories.data],
  );
  const [skipped, setSkipped] = useState<Set<string>>(() => new Set());
  const chosen = rows.filter((r) => !skipped.has(r.category_id));
  return (
    <Dialog
      open={open}
      onOpenChange={setOpen}
      size="lg"
      title={t('orcamento-sugerir')}
      description={t('orcamento-sugerir-descricao', { valor: tenUnits() })}
      trigger={
        <Button size="sm" icon={<Icon name="reticle" size={13} />}>
          {t('orcamento-sugerir')}
        </Button>
      }
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="primary"
            disabled={chosen.length === 0}
            loading={save.isPending}
            onClick={() =>
              save.mutate(
                {
                  month: status.month,
                  lines: chosen.map((r) => ({ category_id: r.category_id, amount: r.suggested })),
                },
                {
                  onSuccess: () => {
                    setOpen(false);
                    toast({
                      tone: 'success',
                      title: t('orcamento-atualizado'),
                      description: t('orcamento-atualizado-descricao', { n: chosen.length }),
                    });
                  },
                },
              )
            }
          >
            {t('orcamento-aplicar-n', { n: chosen.length })}
          </Button>
        </>
      }
    >
      <Segmented
        label={t('orcamento-meses-media')}
        value={monthsBack}
        onChange={setMonthsBack}
        options={[
          { value: '3', label: t('orcamento-n-meses', { n: 3 }) },
          { value: '6', label: t('orcamento-n-meses', { n: 6 }) },
          { value: '12', label: t('orcamento-n-meses', { n: 12 }) },
        ]}
      />
      {suggestion.isPending ? (
        <p className="text-[13px] text-dim">{t('orcamento-calculando')}</p>
      ) : rows.length === 0 ? (
        <p className="text-[13px] text-dim">{t('orcamento-igual-media')}</p>
      ) : (
        <SuggestionTable
          rows={rows}
          skipped={skipped}
          onSkip={(id, skip) =>
            setSkipped((s) => {
              const n = new Set(s);
              if (skip) n.add(id);
              else n.delete(id);
              return n;
            })
          }
        />
      )}
      {save.error ? (
        <p role="alert" className="text-[12.5px] text-negative">
          {errorText(save.error)}
        </p>
      ) : null}
    </Dialog>
  );
}

// ── Painel ───────────────────────────────────────────────────────

export function BudgetPanel({
  status,
  basis,
  colors,
}: {
  status: BudgetStatus;
  basis: Basis;
  colors: EntityColors;
}) {
  const [editing, setEditing] = useState<string | null>(null);
  const [savingId, setSavingId] = useState<string | null>(null);
  const [added, setAdded] = useState<string[]>([]);
  const categories = useCategories({});
  const save = useSetBudgetLines();
  const byId = useMemo(
    () => new Map((categories.data ?? []).map((c) => [c.id, c])),
    [categories.data],
  );
  const extra = added
    .map((id) => byId.get(id))
    .filter((c) => c !== undefined)
    .map(emptyLine);
  const expenseTree = budgetTree(status, 'expense', extra);
  const incomeTree = budgetTree(status, 'income', extra);
  const missing = unbudgetedWithSpending(status);
  const addable = addableCategories(categories.data ?? [], status, 'expense').filter(
    (c) => !added.includes(c.id),
  );

  const onSave = async (categoryId: string, cents: number) => {
    setSavingId(categoryId);
    try {
      await save.mutateAsync({
        month: status.month,
        lines: [{ category_id: categoryId, amount: cents }],
      });
      setAdded((a) => a.filter((x) => x !== categoryId));
    } catch (e) {
      toast({
        title: t('orcamento-gravar-erro'),
        description: errorText(e),
        tone: 'error',
      });
    } finally {
      setSavingId(null);
    }
  };

  const startEditing = (id: string) => {
    setEditing(id);
    requestAnimationFrame(() =>
      document
        .querySelector(`[data-category="${CSS.escape(id)}"]`)
        ?.scrollIntoView({ block: 'center' }),
    );
  };

  return (
    <div className="flex flex-col gap-5">
      <Summary status={status} basis={basis} />
      <SpendGroupsCard month={status.month} basis={basis} index="B" />
      <div className="flex flex-wrap items-center gap-2">
        <CopyDialog month={status.month} />
        <SuggestDialog status={status} basis={basis} />
        <div className="ml-auto w-64">
          <Select
            aria-label={t('orcamento-adicionar-aria')}
            placeholder={t('orcamento-adicionar')}
            value={undefined}
            onValueChange={(id) => {
              setAdded((a) => [...a, id]);
              startEditing(id);
            }}
            options={addable.map((c) => ({ value: c.id, label: c.path }))}
          />
        </div>
      </div>
      {missing.length > 0 ? (
        <section
          aria-label={t('orcamento-sem-orcamento-aria')}
          className="flex flex-col gap-2 rounded-card border border-warning/40 bg-warning-soft px-4 py-3"
        >
          <span className="label-caps flex items-center gap-1.5 text-warning">
            <Icon name="alert" size={13} /> {t('orcamento-n-sem-orcamento', { n: missing.length })}
          </span>
          <ul className="flex flex-wrap gap-2">
            {missing.map((l) => (
              <li key={l.category_id}>
                <button
                  type="button"
                  onClick={() => startEditing(l.category_id)}
                  className="inline-flex items-center gap-2 rounded-control border border-line-strong bg-surface px-2.5 py-1 text-[12.5px] text-fg hover:border-accent focus-visible:outline-2 focus-visible:outline-accent"
                >
                  {l.category_path}
                  <Money cents={l.actual} className="text-dim" />
                  <span className="label-caps text-accent">{t('orcamento-definir')}</span>
                </button>
              </li>
            ))}
          </ul>
        </section>
      ) : null}
      <InstrumentCard index="C" title={t('orcamento-despesas-categoria')} padded={false}>
        {expenseTree.length === 0 ? (
          <p className="p-4 text-[13px] text-dim">
            {t('orcamento-vazio', { mes: formatMonthLong(status.month) })}
          </p>
        ) : (
          <BudgetTable
            label={t('orcamento-tabela-despesas')}
            kind="expense"
            rows={expenseTree}
            month={status.month}
            basis={basis}
            elapsed={status.month_elapsed}
            colors={colors}
            editing={editing}
            onEditingChange={setEditing}
            onSave={onSave}
            savingId={savingId}
          />
        )}
      </InstrumentCard>
      {incomeTree.length > 0 ? (
        <InstrumentCard index="D" title={t('orcamento-receitas-previstas')} padded={false}>
          <BudgetTable
            label={t('orcamento-tabela-receitas')}
            kind="income"
            rows={incomeTree}
            month={status.month}
            basis={basis}
            elapsed={status.month_elapsed}
            colors={colors}
            editing={editing}
            onEditingChange={setEditing}
            onSave={onSave}
            savingId={savingId}
          />
        </InstrumentCard>
      ) : null}
      {status.missing_fx.length > 0 ? (
        <Field label={t('orcamento-cambio')}>
          <p className="text-[12.5px] text-warning">
            {t('orcamento-sem-cambio', { moedas: status.missing_fx.join(', ') })}
          </p>
        </Field>
      ) : null}
    </div>
  );
}
