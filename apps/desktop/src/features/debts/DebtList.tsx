/**
 * Resumo das dívidas (saldo total, compromisso mensal, juros a pagar) e a lista de
 * contratos com saldo, taxa, sistema, indexador, progresso, próxima parcela, CET e custo
 * real. Selecionar uma linha abre o contrato.
 */
import type { DebtSummary } from '@/api/generated/DebtSummary';
import type { LoanView } from '@/api/generated/LoanView';
import {
  InstrumentCard,
  Measure,
  Money,
  TechTable,
  type ContextMenuEntry,
} from '@/design/components';
import { Pct } from '@/features/portfolio/components/Pct';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { INDEXER_LABEL, isPostIndexer, KIND_LABEL, SYSTEM_LABEL } from './labels';
import { loanRateLabel, progress, rateText } from './transform';

export function DebtSummaryCards({ summary }: { summary: DebtSummary }) {
  const overdue = summary.loans.reduce((s, l) => s + l.overdue_count, 0);
  return (
    <div className="grid gap-4 @min-[530px]:grid-cols-3">
      <InstrumentCard index="01" title={t('dividas-resumo-saldo-total')} headingLevel={2}>
        <Measure
          label={t('dividas-resumo-contratos-ativos', { n: summary.loans.length })}
          value={<Money cents={summary.total_balance} />}
        />
      </InstrumentCard>
      <InstrumentCard index="02" title={t('dividas-resumo-compromisso')} headingLevel={2}>
        <Measure
          label={t('dividas-resumo-proximas-somadas')}
          value={<Money cents={summary.monthly_commitment} />}
          hint={
            overdue > 0 ? (
              <span className="text-negative">{t('dividas-resumo-em-atraso', { n: overdue })}</span>
            ) : undefined
          }
        />
      </InstrumentCard>
      <InstrumentCard index="03" title={t('dividas-juros-a-pagar')} headingLevel={2}>
        <Measure
          label={t('dividas-resumo-ate-o-fim')}
          value={<Money cents={summary.interest_remaining} />}
          hint={t('dividas-resumo-pelo-cronograma')}
        />
      </InstrumentCard>
    </div>
  );
}

export function LoanTable({
  loans,
  selectedId,
  onSelect,
  rowMenu,
}: {
  loans: readonly LoanView[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  rowMenu?: (loan: LoanView) => readonly ContextMenuEntry[];
}) {
  return (
    <TechTable<LoanView>
      label={t('dividas-contratos')}
      prefsId="Contratos"
      rowMenu={rowMenu}
      rowMenuLabel={(l) => t('dividas-acoes-de', { nome: l.name })}
      minWidth="980px"
      rows={loans}
      getRowId={(l) => l.id}
      selectedId={selectedId}
      onSelectedChange={(id) => id && onSelect(id)}
      onActivate={(l) => onSelect(l.id)}
      columns={[
        {
          key: 'name',
          header: t('dividas-contrato'),
          sortValue: (l) => l.name,
          cell: (l) => (
            <span className="flex flex-col">
              {/* Long names wrap: with nowrap the table outgrew the default window (Linux fonts). */}
              <span className="font-medium break-words">{l.name}</span>
              <span className="text-[11.5px] text-dim">
                {KIND_LABEL[l.kind]}
                {l.contract_number ? (
                  <>
                    {' · '}
                    <span className="whitespace-nowrap">
                      {t('dividas-numero-contrato', { numero: l.contract_number })}
                    </span>
                  </>
                ) : null}
                {l.archived ? ` · ${t('dividas-arquivado')}` : ''}
              </span>
            </span>
          ),
        },
        {
          key: 'system',
          header: t('dividas-sistema'),
          cell: (l) => (
            <span className="num text-[12.5px] whitespace-nowrap">
              {SYSTEM_LABEL[l.system]} ·{' '}
              {l.indexer === 'none'
                ? t('dividas-pre')
                : isPostIndexer(l.indexer)
                  ? t('dividas-pos')
                  : INDEXER_LABEL[l.indexer]}
            </span>
          ),
        },
        {
          key: 'rate',
          header: t('dividas-col-taxa-aa'),
          numeric: true,
          sortValue: (l) => Number(l.annual_rate),
          cell: (l) => loanRateLabel(l, { perYear: false }),
        },
        {
          key: 'balance',
          header: t('dividas-saldo-devedor'),
          numeric: true,
          sortValue: (l) => l.balance,
          cell: (l) => <Money cents={l.balance} />,
        },
        {
          key: 'progress',
          header: t('dividas-col-progresso'),
          width: '120px',
          sortValue: (l) => progress(l),
          cell: (l) => {
            const p = progress(l);
            return (
              <span className="flex items-center gap-2">
                <span aria-hidden className="h-1.5 w-16 overflow-hidden rounded-full bg-surface-2">
                  <span
                    className="block h-full bg-accent"
                    style={{ width: `${Math.round(p * 100)}%` }}
                  />
                </span>
                <span className="num text-[12px] text-dim">
                  {l.paid_count}/{l.paid_count + l.remaining_count}
                </span>
              </span>
            );
          },
        },
        {
          key: 'next',
          header: t('dividas-proxima-parcela'),
          numeric: true,
          sortValue: (l) => l.next_installment?.due_date,
          cell: (l) =>
            l.next_installment ? (
              <span className="flex flex-col items-end">
                <Money cents={l.next_installment.total} estimated={l.next_installment.estimated} />
                <span
                  className={
                    l.next_installment.overdue
                      ? 'text-[11.5px] text-negative'
                      : 'text-[11.5px] text-dim'
                  }
                >
                  {l.next_installment.overdue
                    ? t('dividas-atrasada-em', { data: formatDate(l.next_installment.due_date) })
                    : formatDate(l.next_installment.due_date)}
                </span>
              </span>
            ) : (
              <span className="text-dim">{t('dividas-quitado-min')}</span>
            ),
        },
        {
          key: 'cet',
          header: 'CET',
          numeric: true,
          sortValue: (l) => (l.cet_annual ? Number(l.cet_annual) : null),
          cell: (l) => (l.cet_annual ? rateText(l.cet_annual) : '—'),
        },
        {
          key: 'real',
          header: t('dividas-col-custo-real'),
          numeric: true,
          sortValue: (l) => l.real_cost,
          cell: (l) => <Pct value={l.real_cost} decimals={1} />,
        },
      ]}
    />
  );
}
