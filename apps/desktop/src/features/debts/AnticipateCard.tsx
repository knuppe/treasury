/**
 * Antecipação de parcelas com desconto dos juros: escolhe quantas, a ordem (das últimas para
 * as primeiras, que encurta o contrato, ou das próximas em diante) e a data; o núcleo traz
 * cada parcela a valor presente pela taxa do contrato (dias corridos ÷ 30) e mostra quanto
 * pagar e o desconto. Registrar marca as parcelas como pagas e lança a saída na conta.
 */
import { useDeferredValue, useState } from 'react';

import { useAnticipate, useAnticipationSimulation } from '@/api/hooks/debt';
import { useUndoToast } from '@/api/hooks/undoToast';
import type { AnticipateOrder } from '@/api/generated/AnticipateOrder';
import type { LoanView } from '@/api/generated/LoanView';
import { Button, DateInput, Field, Input, KeyValue, Money } from '@/design/components';
import { InlineError, Segmented } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatMoney, todayIso } from '@/lib/format';

const orderHint = (order: AnticipateOrder) =>
  order === 'last' ? t('dividas-antecipar-dica-ultimas') : t('dividas-antecipar-dica-proximas');

export function AnticipateCard({ loan }: { loan: LoanView }) {
  const [count, setCount] = useState('1');
  const [order, setOrder] = useState<AnticipateOrder>('last');
  const [date, setDate] = useState<string | null>(todayIso());
  const n = Number.parseInt(count, 10);
  const deferred = useDeferredValue(n);
  const valid = Number.isFinite(deferred) && deferred > 0 && date != null;
  const args = valid ? { loan_id: loan.id, date: date, count: deferred, order } : null;
  const sim = useAnticipationSimulation(args);
  const s = args ? sim.data : undefined;
  const apply = useAnticipate();
  const undoToast = useUndoToast();

  return (
    <div className="flex flex-col gap-4">
      <div className="grid items-start gap-3 @min-[530px]:grid-cols-3">
        <Field
          label={t('dividas-parcelas')}
          hint={t('dividas-antecipar-em-aberto', { n: loan.remaining_count })}
        >
          <Input
            numeric
            inputMode="numeric"
            value={count}
            onChange={(e) => setCount(e.target.value.replace(/\D/g, ''))}
          />
        </Field>
        <Field label={t('dividas-antecipar-ordem')} hint={orderHint(order)}>
          <Segmented<AnticipateOrder>
            label={t('dividas-antecipar-ordem-aria')}
            fill
            value={order}
            onChange={setOrder}
            options={[
              { value: 'last', label: t('dividas-antecipar-ultimas') },
              { value: 'next', label: t('dividas-antecipar-proximas') },
            ]}
          />
        </Field>
        <Field label={t('dividas-data-pagamento')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
      </div>

      {!args ? (
        <p className="text-[13px] text-dim">{t('dividas-antecipar-informe')}</p>
      ) : sim.error ? (
        <InlineError error={sim.error} />
      ) : !s ? (
        <p className="text-[13px] text-dim">{t('dividas-calculando')}</p>
      ) : (
        <div className="flex flex-col gap-3">
          <p role="status" className="text-[13.5px] text-positive">
            {withSlots(
              t(
                s.charges_dropped > 0
                  ? 'dividas-antecipar-resumo-encargos'
                  : 'dividas-antecipar-resumo',
                {
                  valor: slot(0),
                  data: date ? formatDate(date) : '',
                  desconto: slot(1),
                  encargos: slot(2),
                },
              ),
              [
                <Money key="v" cents={s.to_pay} />,
                <Money key="d" cents={s.discount} />,
                <Money key="e" cents={s.charges_dropped} />,
              ],
            )}
          </p>
          <KeyValue
            items={[
              {
                label: t('dividas-parcelas'),
                value: s.installments.map((i) => `${i.number}`).join(', '),
              },
              {
                label: t('dividas-antecipar-valor-cronograma'),
                value: <Money cents={s.nominal_total} />,
              },
              { label: t('dividas-antecipar-a-pagar'), value: <Money cents={s.to_pay} /> },
              { label: t('dividas-antecipar-ficam'), value: `${s.remaining_after}` },
            ]}
          />
          <div className="flex items-center gap-3">
            <Button
              variant="primary"
              loading={apply.isPending}
              onClick={() =>
                args &&
                apply.mutate(args, {
                  onSuccess: (r) =>
                    undoToast({
                      tone: 'success',
                      title: t('dividas-antecipadas'),
                      description: t('dividas-antecipadas-descricao', {
                        n: r.installments.length,
                        valor: formatMoney(r.to_pay),
                      }),
                    }),
                })
              }
            >
              {t('dividas-antecipar-registrar')}
            </Button>
            <span className="text-[12px] text-dim">
              {loan.debit_account_id
                ? t('dividas-antecipar-com-conta')
                : t('dividas-antecipar-sem-conta')}
            </span>
          </div>
          <InlineError error={apply.error} />
        </div>
      )}
    </div>
  );
}
