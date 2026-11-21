/**
 * Peças comuns da tabela de parcelas e dos diálogos de pagamento: o selo da situação da
 * parcela e a diferença entre o valor pago e o da parcela.
 */
import { Money } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { statusLabel, type InstallmentStatus } from './transform';

const STATUS_TONE: Record<InstallmentStatus, string> = {
  paid: 'border-positive/50 text-positive',
  overdue: 'border-negative/60 text-negative',
  next: 'border-accent/60 text-accent',
  planned: 'border-line-strong border-dashed text-dim',
};

const STATUS_ICON: Record<InstallmentStatus, string> = {
  paid: '✓',
  overdue: '!',
  next: '›',
  planned: '',
};

export function InstallmentBadge({ status }: { status: InstallmentStatus }) {
  return (
    <span
      className={cx(
        'label-caps inline-flex h-5 items-center gap-1 rounded-control border px-1.5 whitespace-nowrap',
        STATUS_TONE[status],
      )}
    >
      {STATUS_ICON[status] ? <span aria-hidden>{STATUS_ICON[status]}</span> : null}
      {statusLabel(status)}
    </span>
  );
}

/** Diferença entre o valor pago e o da parcela (`igual` quando bate). */
export function PaymentDiff({
  paid,
  total,
  currency,
}: {
  paid: number;
  total: number;
  currency?: string;
}) {
  const diff = paid - total;
  if (diff === 0) return <span className="text-dim">{t('dividas-igual')}</span>;
  return (
    <span title={diff > 0 ? t('dividas-pago-a-mais') : t('dividas-pago-a-menos')}>
      <Money cents={diff} currency={currency} signed />
    </span>
  );
}
