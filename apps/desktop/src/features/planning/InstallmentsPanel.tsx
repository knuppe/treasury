/**
 * Parcelas em andamento detectadas pela descrição ("Parcela 8/10", "PREST.EMPREST 003/005"):
 * quanto falta e quando termina, por série. Estimativa, sem gravar nada; "Não é parcela" tira
 * a série da lista e da previsão (docs/design/detected-installments.md).
 */
import type { DetectedSeries } from '@/api/generated/DetectedSeries';
import type { SeriesKind } from '@/api/generated/SeriesKind';
import { useDetectedInstallments, useIgnoreInstallmentSeries } from '@/api/hooks/planning';
import {
  Button,
  EmptyState,
  InstrumentCard,
  Money,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatMonthLong } from '@/lib/format';

const KIND: Record<SeriesKind, () => string> = {
  card: () => t('planejamento-parcelas-tipo-card'),
  loan: () => t('planejamento-parcelas-tipo-loan'),
  bill: () => t('planejamento-parcelas-tipo-bill'),
  account: () => t('planejamento-parcelas-tipo-account'),
};

export function InstallmentsPanel() {
  const { data, isLoading, error } = useDetectedInstallments();
  const ignore = useIgnoreInstallmentSeries();
  const columns: TechColumn<DetectedSeries>[] = [
    {
      key: 'description',
      header: t('planejamento-parcelas-descricao'),
      sortValue: (s) => s.description,
      cell: (s) => (
        <span className="flex flex-col">
          <span>{s.description}</span>
          <span className="text-[12px] text-dim">
            {KIND[s.kind]()} · {s.account_name}
            {s.member_name ? ` · ${s.member_name}` : ''}
          </span>
        </span>
      ),
    },
    {
      key: 'progress',
      header: t('planejamento-parcelas-parcela'),
      sortValue: (s) => s.number / s.total,
      cell: (s) => (
        <span className={cx('num', s.missing && 'text-warning')}>
          {s.number}/{s.total}
          {s.missing ? ` · ${t('planejamento-parcelas-nao-apareceu')}` : ''}
        </span>
      ),
    },
    {
      key: 'amount',
      header: t('planejamento-parcelas-valor'),
      numeric: true,
      sortValue: (s) => s.amount,
      cell: (s) => <Money cents={s.amount} currency={s.currency} />,
    },
    {
      key: 'remaining',
      header: t('planejamento-parcelas-falta'),
      numeric: true,
      sortValue: (s) => s.remaining_amount,
      cell: (s) => <Money cents={s.remaining_amount} currency={s.currency} />,
    },
    {
      key: 'end',
      header: t('planejamento-parcelas-termina'),
      sortValue: (s) => s.end_month,
      cell: (s) => formatMonthLong(s.end_month),
    },
    {
      key: 'actions',
      header: '',
      cell: (s) => (
        <Button
          size="sm"
          variant="ghost"
          loading={ignore.isPending && ignore.variables?.key === s.key}
          onClick={() =>
            ignore.mutate(
              { key: s.key },
              {
                onSuccess: () =>
                  toast({ tone: 'success', title: t('planejamento-parcelas-ignorada') }),
              },
            )
          }
        >
          {t('planejamento-parcelas-ignorar')}
        </Button>
      ),
    },
  ];
  return (
    <QueryState loading={isLoading} error={error}>
      {data ? (
        data.series.length === 0 ? (
          <EmptyState
            title={t('planejamento-parcelas-vazio-titulo')}
            description={t('planejamento-parcelas-vazio')}
          />
        ) : (
          <div className="flex flex-col gap-3">
            <InstrumentCard
              index="05.B"
              title={t('planejamento-parcelas-titulo')}
              padded={false}
              footer={[
                {
                  label: t('planejamento-parcelas-total'),
                  value: <Money cents={data.remaining_total} currency={data.currency} />,
                },
                {
                  label: t('planejamento-parcelas-proximo-mes'),
                  value: <Money cents={data.by_month[1]?.amount ?? 0} currency={data.currency} />,
                },
              ]}
            >
              <TechTable
                columns={columns}
                label={t('planejamento-parcelas-titulo')}
                rows={data.series}
                getRowId={(s) => s.key}
                dense
              />
            </InstrumentCard>
            <InlineError error={ignore.error} />
            <p className="max-w-[80ch] text-[12.5px] text-dim">{t('planejamento-parcelas-nota')}</p>
          </div>
        )
      ) : null}
    </QueryState>
  );
}
