/**
 * Simulador de portabilidade: o saldo devedor de hoje levado a outro banco com nova taxa,
 * prazo e sistema, mais os custos da troca, comparado com o que falta pagar no contrato
 * atual. Não grava nada.
 */
import { useDeferredValue, useState } from 'react';

import { usePortabilitySimulation } from '@/api/hooks/debt';
import type { AmortSystem } from '@/api/generated/AmortSystem';
import type { LoanView } from '@/api/generated/LoanView';
import { Field, Input, KeyValue, Money, MoneyInput } from '@/design/components';
import { DecimalInput } from '@/features/portfolio/components/DecimalInput';
import { InlineError, Segmented } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

import { SYSTEM_LABEL } from './labels';
import { decimalText, loanRateLabel } from './transform';

const SYSTEMS: AmortSystem[] = ['sac', 'price'];

export function PortabilitySimulator({ loan }: { loan: LoanView }) {
  const [rate, setRate] = useState<string | null>(null);
  const [term, setTerm] = useState<string>(String(loan.remaining_count));
  const [system, setSystem] = useState<AmortSystem>(loan.system);
  const [costs, setCosts] = useState<number | null>(0);
  const deferredRate = useDeferredValue(rate);
  const months = Number.parseInt(term, 10);
  const validTerm = Number.isFinite(months) && months > 0 && months <= 600;
  const args =
    deferredRate && Number(deferredRate) > 0 && validTerm
      ? {
          loan_id: loan.id,
          annual_rate: deferredRate,
          term_months: months,
          system,
          ...(costs ? { costs } : {}),
        }
      : null;
  const sim = usePortabilitySimulation(args);
  const s = args ? sim.data : undefined;

  return (
    <div className="flex flex-col gap-4">
      <div className="grid items-start gap-3 @min-[530px]:grid-cols-2 @min-[1040px]:grid-cols-4">
        <Field
          label={t('dividas-portabilidade-nova-taxa')}
          hint={t('dividas-portabilidade-hoje', { taxa: loanRateLabel(loan) })}
        >
          <DecimalInput
            value={rate}
            onValueChange={setRate}
            maxDecimals={6}
            placeholder={decimalText(10.5, 1)}
            suffix="%"
          />
        </Field>
        <Field
          label={t('dividas-prazo-meses')}
          error={validTerm ? undefined : t('dividas-portabilidade-prazo-erro')}
          hint={t('dividas-portabilidade-restam', { n: loan.remaining_count })}
        >
          <Input
            numeric
            inputMode="numeric"
            value={term}
            invalid={!validTerm}
            onChange={(e) => setTerm(e.target.value.replace(/\D/g, ''))}
          />
        </Field>
        <Field label={t('dividas-sistema')} hint={t('dividas-portabilidade-sistema-dica')}>
          <Segmented<AmortSystem>
            label={t('dividas-portabilidade-sistema-aria')}
            fill
            value={system}
            onChange={setSystem}
            options={SYSTEMS.map((v) => ({ value: v, label: SYSTEM_LABEL[v] }))}
          />
        </Field>
        <Field label={t('dividas-custos-troca')} hint={t('dividas-portabilidade-custos-dica')}>
          <MoneyInput value={costs} onValueChange={setCosts} />
        </Field>
      </div>

      {!args ? (
        <p className="text-[13px] text-dim">{t('dividas-portabilidade-informe')}</p>
      ) : sim.error ? (
        <InlineError error={sim.error} />
      ) : !s ? (
        <p className="text-[13px] text-dim">{t('dividas-calculando')}</p>
      ) : (
        <div className="flex flex-col gap-3">
          <p
            role="status"
            className={
              s.savings > 0 ? 'text-[13.5px] text-positive' : 'text-[13.5px] text-negative'
            }
          >
            {withSlots(
              t(s.savings > 0 ? 'dividas-portabilidade-vale' : 'dividas-portabilidade-nao-vale', {
                valor: slot(0),
              }),
              [<Money key="v" cents={Math.abs(s.savings)} />],
            )}
          </p>
          <KeyValue
            layout="stacked"
            items={[
              {
                label: t('dividas-portabilidade-saldo-levado'),
                value: <Money cents={s.balance} />,
              },
              {
                label: t('dividas-portabilidade-falta-atual'),
                value: <Money cents={s.current_remaining_total} />,
              },
              {
                label: t('dividas-portabilidade-total-nova'),
                value: <Money cents={s.new_total} />,
              },
              { label: t('dividas-custos-troca'), value: <Money cents={s.costs} /> },
              {
                label: t('dividas-parcela'),
                value: (
                  <span>
                    <Money cents={s.current_next_payment} /> → <Money cents={s.new_first_payment} />
                  </span>
                ),
              },
              {
                label: t('dividas-portabilidade-novo-prazo'),
                value: t('dividas-n-meses', { n: s.new_term_months }),
              },
            ]}
          />
          <p className="text-[12px] text-dim">{t('dividas-portabilidade-nota')}</p>
        </div>
      )}
    </div>
  );
}
