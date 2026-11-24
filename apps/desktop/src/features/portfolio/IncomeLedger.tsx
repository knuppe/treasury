/**
 * Income ledger (`portfolio.income_ledger`): what the family was entitled to, distribution by
 * distribution, by the position on the record date and the official notice (Fundos.NET for
 * real estate funds, B3 for companies), against what is recorded. The totals show what the
 * sources say was received since the first purchase and what is still to be paid; the fixes
 * (missing, repeated or with another amount) go through a preview and are applied in one
 * change set (`portfolio.income_rebuild`), which also turns the automatic recording on.
 */
import { useMemo, useState } from 'react';

import type { IncomeLedgerEntry } from '@/api/generated/IncomeLedgerEntry';
import type { IncomeLedgerFix } from '@/api/generated/IncomeLedgerFix';
import type { IncomeLedgerPeriod } from '@/api/generated/IncomeLedgerPeriod';
import type { IncomeLedgerState } from '@/api/generated/IncomeLedgerState';
import {
  useIncomeAuto,
  useIncomeLedger,
  useIncomeRebuild,
  useInvestmentAccounts,
} from '@/api/hooks/portfolio';
import {
  Button,
  Dialog,
  DialogClose,
  Measure,
  Money,
  TechTable,
  toast,
  Tooltip,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { Checkbox } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { FormError } from '@/vault/FormError';

import { ErrorBlock, LoadingBlock, Notice } from './components/States';
import { formatQuantity, formatUnitPrice } from './decimal';
import { INCOME_KIND_LABEL, sourceName } from './labels';

const STATE_TONE: Record<IncomeLedgerState, string> = {
  matches: 'text-positive',
  missing: 'text-warning',
  upcoming: 'text-dim',
  value_diff: 'text-warning',
  tax_diff: 'text-warning',
  duplicate: 'text-negative',
  unexpected: 'text-dim',
};

export function stateLabel(s: IncomeLedgerState): string {
  switch (s) {
    case 'matches':
      return t('carteira-razao-estado-confere');
    case 'missing':
      return t('carteira-razao-estado-faltando');
    case 'upcoming':
      return t('carteira-razao-estado-a-receber');
    case 'value_diff':
      return t('carteira-razao-estado-valor');
    case 'tax_diff':
      return t('carteira-razao-estado-ir');
    case 'duplicate':
      return t('carteira-razao-estado-repetido');
    case 'unexpected':
      return t('carteira-razao-estado-sem-fonte');
  }
}

function fixLabel(f: IncomeLedgerFix): string {
  switch (f) {
    case 'create':
      return t('carteira-razao-acao-lancar');
    case 'update':
      return t('carteira-razao-acao-corrigir');
    case 'delete':
      return t('carteira-razao-acao-apagar');
  }
}

/** What the rebuild may fix: unofficial distributions to create wait in "To confirm". */
export function proposable(e: IncomeLedgerEntry): boolean {
  return e.fix != null && (e.official || e.fix !== 'create');
}

function StateBadge({ e }: { e: IncomeLedgerEntry }) {
  const text = stateLabel(e.state);
  const badge = (
    <span className={cx('text-[12px]', STATE_TONE[e.state])}>
      {text}
      {e.note ? ' *' : ''}
    </span>
  );
  return e.note ? (
    <Tooltip side="left" content={<span className="max-w-72 text-[12px]">{e.note}</span>}>
      <span tabIndex={0} className="cursor-help">
        {badge}
      </span>
    </Tooltip>
  ) : (
    badge
  );
}

const entryNet = (e: IncomeLedgerEntry) => e.expected_net ?? e.recorded?.net ?? 0;

function columns(
  accountName: (id: string) => string,
  showAsset: boolean,
): TechColumn<IncomeLedgerEntry>[] {
  const cols: TechColumn<IncomeLedgerEntry>[] = [
    {
      key: 'ex',
      header: t('carteira-razao-col-data-com'),
      cell: (e) => (e.ex_date ? formatDate(e.ex_date) : '—'),
      sortValue: (e) => e.ex_date ?? '',
    },
    {
      key: 'pay',
      header: t('carteira-pagamento'),
      cell: (e) =>
        e.pay_date ? (
          <span className={cx(e.pay_date_estimated && 'italic text-dim')}>
            {formatDate(e.pay_date)}
          </span>
        ) : (
          '—'
        ),
      sortValue: (e) => e.pay_date ?? e.ex_date ?? '',
    },
  ];
  if (showAsset) {
    cols.push({
      key: 'asset',
      header: t('carteira-col-ativo'),
      cell: (e) => <span className="font-medium">{e.asset.ticker ?? e.asset.name}</span>,
      sortValue: (e) => e.asset.ticker ?? e.asset.name,
    });
  }
  cols.push(
    {
      key: 'kind',
      header: t('carteira-razao-col-tipo'),
      cell: (e) => (
        <span className="flex flex-col leading-tight">
          <span>{INCOME_KIND_LABEL[e.kind]}</span>
          <span className="text-[11px] text-dim">{accountName(e.account_id)}</span>
        </span>
      ),
    },
    {
      key: 'qty',
      header: t('carteira-razao-col-qtd-valor'),
      align: 'right',
      cell: (e) =>
        e.quantity && e.amount_per_share ? (
          <span className="num text-dim">
            {formatQuantity(e.quantity)} × {formatUnitPrice(e.amount_per_share, e.currency)}
          </span>
        ) : (
          '—'
        ),
    },
    {
      key: 'gross',
      header: t('carteira-razao-col-bruto'),
      align: 'right',
      cell: (e) => {
        const g = e.expected_gross ?? e.recorded?.gross;
        return g == null ? '—' : <Money cents={g} currency={e.currency} />;
      },
    },
    {
      key: 'tax',
      header: t('carteira-razao-col-ir'),
      align: 'right',
      cell: (e) => {
        const v = e.expected_tax ?? e.recorded?.tax_withheld ?? 0;
        return v ? <Money cents={v} currency={e.currency} /> : <span className="text-dim">—</span>;
      },
    },
    {
      key: 'net',
      header: t('carteira-razao-col-liquido'),
      align: 'right',
      cell: (e) => (
        <span className="flex flex-col items-end leading-tight">
          <Money cents={entryNet(e)} currency={e.currency} />
          {e.recorded && e.expected_net != null && e.recorded.net !== e.expected_net ? (
            <span className="text-[11px] text-warning">
              {t('carteira-razao-lancado')} <Money cents={e.recorded.net} currency={e.currency} />
            </span>
          ) : null}
        </span>
      ),
      sortValue: entryNet,
    },
    {
      key: 'source',
      header: t('carteira-razao-col-fonte'),
      cell: (e) => (
        <span className="flex flex-col leading-tight">
          <span>{e.source ? sourceName(e.source) : sourceName(e.recorded?.source)}</span>
          {e.period ? <span className="text-[11px] text-dim">{e.period}</span> : null}
        </span>
      ),
    },
    {
      key: 'state',
      header: t('carteira-razao-col-situacao'),
      cell: (e) => <StateBadge e={e} />,
      sortValue: (e) => e.state,
    },
  );
  return cols;
}

function useAccountName() {
  const accounts = useInvestmentAccounts().data ?? [];
  return (id: string) => accounts.find((a) => a.id === id)?.name ?? t('carteira-conta');
}

/** Totals per year: received by the sources, what is recorded, and what is still to be paid. */
function YearsTable({
  years,
  currency,
}: {
  years: readonly IncomeLedgerPeriod[];
  currency: string;
}) {
  const rows = [...years].reverse();
  const differs = rows.some((y) => y.recorded_base !== y.received_base);
  return (
    <TechTable
      label={t('carteira-razao-anos-tabela')}
      prefsId="Proventos por ano"
      dense
      rows={rows}
      getRowId={(y) => y.period}
      columns={[
        { key: 'year', header: t('carteira-razao-col-ano'), cell: (y) => y.period },
        {
          key: 'received',
          header: t('carteira-razao-col-recebido'),
          align: 'right',
          cell: (y) => <Money cents={y.received_base} currency={currency} />,
        },
        ...(differs
          ? [
              {
                key: 'recorded',
                header: t('carteira-razao-col-lancado'),
                align: 'right' as const,
                cell: (y: IncomeLedgerPeriod) => (
                  <Money cents={y.recorded_base} currency={currency} />
                ),
              },
            ]
          : []),
        {
          key: 'upcoming',
          header: t('carteira-razao-col-a-receber'),
          align: 'right',
          cell: (y) =>
            y.upcoming_base ? <Money cents={y.upcoming_base} currency={currency} /> : '—',
        },
      ]}
    />
  );
}

/**
 * Summary of the ledger: totals, years and the fixes to review. With `assetId`, only that
 * asset (its page); without it, the whole portfolio with one line per asset.
 */
export function IncomeLedgerSummary({ assetId }: { assetId?: string }) {
  const q = useIncomeLedger(assetId ? { asset_id: assetId } : { only_problems: true });
  const auto = useIncomeAuto();
  const [open, setOpen] = useState(false);
  if (q.isPending) return <LoadingBlock label={t('carteira-razao-calculando')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const l = q.data;
  if (!l.entries.length && !l.received_base && !l.upcoming_base) return null;
  const cur = l.base_currency;
  const first = assetId ? l.assets[0]?.first_date : null;
  const c = l.counts;
  return (
    <section className="flex flex-col gap-3" aria-label={t('carteira-razao-titulo')}>
      <div className="flex flex-wrap gap-6">
        <Measure
          label={
            first
              ? t('carteira-razao-recebido-desde', { data: formatDate(first) })
              : t('carteira-razao-recebido')
          }
          value={<Money cents={l.received_base} currency={cur} />}
          hint={t('carteira-razao-recebido-dica')}
        />
        {l.upcoming_base ? (
          <Measure
            label={t('carteira-razao-a-receber')}
            value={<Money cents={l.upcoming_base} currency={cur} />}
          />
        ) : null}
        {l.recorded_base !== l.received_base ? (
          <Measure
            label={t('carteira-razao-lancado-hoje')}
            value={<Money cents={l.recorded_base} currency={cur} />}
          />
        ) : null}
      </div>
      {c.fixes > 0 ? (
        <Notice
          tone="warning"
          title={t('carteira-razao-correcoes', { n: c.fixes })}
          actions={
            <Button size="sm" variant="primary" onClick={() => setOpen(true)}>
              {t('carteira-razao-revisar')}
            </Button>
          }
        >
          {t('carteira-razao-correcoes-detalhe', {
            faltando: c.missing - c.unofficial_missing,
            repetidos: c.duplicate,
            valor: c.value_diff + c.tax_diff,
          })}
          {!l.auto ? ` ${t('carteira-razao-automatico-depois')}` : ''}
        </Notice>
      ) : null}
      {l.auto ? (
        <p className="flex flex-wrap items-center gap-2 text-[12px] text-dim">
          {t('carteira-razao-automatico-ligado', {
            data: l.auto_since ? formatDate(l.auto_since) : '',
          })}
          <Button
            size="sm"
            variant="ghost"
            loading={auto.isPending}
            onClick={() => auto.mutate({ enabled: false })}
          >
            {t('carteira-razao-automatico-desligar')}
          </Button>
        </p>
      ) : null}
      {c.unofficial_missing > 0 ? (
        <p className="text-[12px] text-dim">
          {t('carteira-razao-sem-aviso', { n: c.unofficial_missing })}
        </p>
      ) : null}
      {l.years.length ? <YearsTable years={l.years} currency={cur} /> : null}
      {!assetId && l.assets.length > 1 ? (
        <TechTable
          label={t('carteira-razao-ativos-tabela')}
          prefsId="Proventos por ativo desde a compra"
          dense
          rows={l.assets}
          getRowId={(a) => a.asset.id}
          defaultSort={{ key: 'received', direction: 'desc' }}
          columns={[
            {
              key: 'asset',
              header: t('carteira-col-ativo'),
              cell: (a) => <span className="font-medium">{a.asset.ticker ?? a.asset.name}</span>,
              sortValue: (a) => a.asset.ticker ?? a.asset.name,
            },
            {
              key: 'since',
              header: t('carteira-razao-col-desde'),
              cell: (a) => (a.first_date ? formatDate(a.first_date) : '—'),
              sortValue: (a) => a.first_date ?? '',
            },
            {
              key: 'received',
              header: t('carteira-razao-col-recebido'),
              align: 'right',
              cell: (a) => <Money cents={a.received_base} currency={cur} />,
              sortValue: (a) => a.received_base,
            },
            {
              key: 'upcoming',
              header: t('carteira-razao-col-a-receber'),
              align: 'right',
              cell: (a) =>
                a.upcoming_base ? <Money cents={a.upcoming_base} currency={cur} /> : '—',
              sortValue: (a) => a.upcoming_base,
            },
            {
              key: 'fixes',
              header: t('carteira-razao-col-correcoes'),
              align: 'right',
              cell: (a) => (a.fixes ? <span className="text-warning">{a.fixes}</span> : '—'),
              sortValue: (a) => a.fixes,
            },
          ]}
        />
      ) : null}
      <IncomeRebuildDialog open={open} onOpenChange={setOpen} {...(assetId ? { assetId } : {})} />
    </section>
  );
}

/** Every distribution of the asset, newest first. */
export function IncomeLedgerTable({ assetId }: { assetId: string }) {
  const q = useIncomeLedger({ asset_id: assetId });
  const accountName = useAccountName();
  const cols = useMemo(() => columns(accountName, false), [accountName]);
  if (!q.data?.entries.length) return null;
  return (
    <section className="flex flex-col gap-2">
      <h3 className="label-caps text-fg">{t('carteira-razao-tabela-titulo')}</h3>
      <TechTable
        label={t('carteira-razao-tabela')}
        prefsId="Razão de proventos do ativo"
        dense
        rows={q.data.entries}
        getRowId={(e) => e.key}
        columns={cols}
      />
    </section>
  );
}

/** Preview of the fixes, each one checked by default; applying is one change set. */
export function IncomeRebuildDialog({
  open,
  onOpenChange,
  assetId,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  assetId?: string;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={t('carteira-razao-revisar-titulo')}
      description={t('carteira-razao-revisar-texto')}
    >
      {open ? (
        <RebuildBody onDone={() => onOpenChange(false)} {...(assetId ? { assetId } : {})} />
      ) : null}
    </Dialog>
  );
}

function RebuildBody({ assetId, onDone }: { assetId?: string; onDone: () => void }) {
  const q = useIncomeLedger(assetId ? { asset_id: assetId } : { only_problems: true });
  const save = useIncomeRebuild();
  const accountName = useAccountName();
  const [off, setOff] = useState<ReadonlySet<string>>(new Set());
  const rows = (q.data?.entries ?? []).filter(proposable);
  if (q.isPending) return <LoadingBlock label={t('carteira-razao-calculando')} />;
  if (!rows.length) return <p className="text-[13px] text-dim">{t('carteira-razao-nada')}</p>;
  const chosen = rows.filter((e) => !off.has(e.key));
  const toggle = (k: string, on: boolean) =>
    setOff((prev) => {
      const next = new Set(prev);
      if (on) next.delete(k);
      else next.add(k);
      return next;
    });
  const submit = () =>
    save.mutate(
      { keys: chosen.map((e) => e.key), ...(assetId ? { asset_id: assetId } : {}) },
      {
        onSuccess: (r) => {
          toast({
            tone: 'success',
            title: t('carteira-razao-feito', {
              lancados: r.created,
              corrigidos: r.updated,
              apagados: r.deleted,
            }),
          });
          onDone();
        },
      },
    );
  const allOn = off.size === 0;
  return (
    <div className="flex flex-col gap-3">
      <table
        className="w-full border-collapse text-[13px]"
        aria-label={t('carteira-razao-revisar-tabela')}
      >
        <thead>
          <tr className="label-caps text-left text-dim">
            <th className="w-8 pb-1.5">
              <Checkbox
                label={t('carteira-pend-marcar-todos')}
                hideLabel
                checked={allOn}
                indeterminate={!allOn && off.size < rows.length}
                onChange={(on) => setOff(on ? new Set() : new Set(rows.map((e) => e.key)))}
              />
            </th>
            <th className="pb-1.5 pr-2 font-normal">{t('carteira-razao-col-acao')}</th>
            <th className="pb-1.5 pr-2 font-normal">{t('carteira-col-ativo')}</th>
            <th className="pb-1.5 pr-2 font-normal">{t('carteira-razao-col-data-com')}</th>
            <th className="pb-1.5 pr-2 text-right font-normal">
              {t('carteira-razao-col-lancado')}
            </th>
            <th className="pb-1.5 text-right font-normal">{t('carteira-razao-col-fica')}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((e) => {
            const on = !off.has(e.key);
            const label = e.asset.ticker ?? e.asset.name;
            return (
              <tr key={e.key} className={cx('border-t border-line align-top', !on && 'opacity-50')}>
                <td className="py-1.5">
                  <Checkbox
                    label={t('carteira-razao-marcar', { ativo: label, acao: fixLabel(e.fix!) })}
                    hideLabel
                    checked={on}
                    onChange={(v) => toggle(e.key, v)}
                  />
                </td>
                <td className="py-1.5 pr-2">
                  <span className="flex flex-col leading-tight">
                    <span className="font-medium text-fg">{fixLabel(e.fix!)}</span>
                    <span className={cx('text-[11.5px]', STATE_TONE[e.state])}>
                      {stateLabel(e.state)}
                    </span>
                  </span>
                </td>
                <td className="py-1.5 pr-2">
                  <span className="flex flex-col leading-tight">
                    <span className="font-medium text-fg">{label}</span>
                    <span className="text-[11.5px] text-dim">
                      {INCOME_KIND_LABEL[e.kind]} · {accountName(e.account_id)}
                    </span>
                    {e.note ? <span className="text-[11.5px] text-dim">{e.note}</span> : null}
                  </span>
                </td>
                <td className="num py-1.5 pr-2">
                  {e.ex_date ? formatDate(e.ex_date) : '—'}
                  {e.pay_date ? (
                    <span className="block text-[11px] text-dim">
                      {t('carteira-razao-pago-em', { data: formatDate(e.pay_date) })}
                    </span>
                  ) : null}
                </td>
                <td className="num py-1.5 pr-2 text-right">
                  {e.recorded ? <Money cents={e.recorded.net} currency={e.currency} /> : '—'}
                </td>
                <td className="num py-1.5 text-right">
                  {e.fix === 'delete' ? (
                    '—'
                  ) : (
                    <span className="flex flex-col items-end leading-tight">
                      <Money cents={e.expected_net ?? 0} currency={e.currency} />
                      {e.quantity && e.amount_per_share ? (
                        <span className="text-[11px] text-dim">
                          {formatQuantity(e.quantity)} ×{' '}
                          {formatUnitPrice(e.amount_per_share, e.currency)}
                        </span>
                      ) : null}
                    </span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <FormError error={save.error} />
      <div className="flex flex-wrap items-center justify-end gap-3 border-t border-line pt-3">
        <span role="status" className="mr-auto text-[12px] text-dim">
          {t('carteira-pend-marcados', { n: chosen.length, total: rows.length })}
        </span>
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button
          variant="primary"
          disabled={!chosen.length}
          loading={save.isPending}
          onClick={submit}
        >
          {t('carteira-razao-aplicar', { n: chosen.length })}
        </Button>
      </div>
    </div>
  );
}
