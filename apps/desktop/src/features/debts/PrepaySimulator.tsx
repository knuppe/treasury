/**
 * Simulador de amortização extraordinária: valor e data, e as duas escolhas lado a lado
 * (reduzir prazo × reduzir parcela) com juros economizados, prazo restante, quitação e a
 * próxima parcela. A escolha marcada desenha o saldo simulado na figura do saldo e pode
 * virar a amortização registrada.
 */
import { useDeferredValue, useEffect, useState } from 'react';

import { usePrepaySimulation } from '@/api/hooks/debt';
import type { Installment } from '@/api/generated/Installment';
import type { LoanView } from '@/api/generated/LoanView';
import type { PrepayMode } from '@/api/generated/PrepayMode';
import { Button, DateInput, Field, Money, MoneyInput } from '@/design/components';
import { cx } from '@/design/cx';
import { InlineError } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';

import { MODE_HINT, MODE_LABEL } from './labels';
import { simulationSummary, termText } from './transform';

const MODES: PrepayMode[] = ['reduce_term', 'reduce_installment'];

export interface PrepayChoice {
  amount: number;
  date: string;
  mode: PrepayMode;
}

export function PrepaySimulator({
  loan,
  onSimulated,
  onRegister,
}: {
  loan: LoanView;
  /** Cronograma simulado da escolha marcada (ou `null` sem simulação). */
  onSimulated: (schedule: readonly Installment[] | null) => void;
  onRegister: (choice: PrepayChoice) => void;
}) {
  const [amount, setAmount] = useState<number | null>(null);
  const [date, setDate] = useState<string | null>(todayIso());
  const [mode, setMode] = useState<PrepayMode>('reduce_term');
  const deferred = useDeferredValue(amount);
  const ready = deferred != null && deferred > 0 && date != null;
  const args = (m: PrepayMode) =>
    ready ? { loan_id: loan.id, date: date, amount: deferred, mode: m } : null;
  const term = usePrepaySimulation(args('reduce_term'));
  const inst = usePrepaySimulation(args('reduce_installment'));
  const sims = { reduce_term: term, reduce_installment: inst } as const;
  const chosen = ready ? sims[mode].data : undefined;
  const tooBig = amount != null && amount >= loan.balance && loan.balance > 0;

  useEffect(() => {
    onSimulated(chosen ? chosen.schedule : null);
  }, [chosen, onSimulated]);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-start gap-3">
        <Field
          label={t('dividas-simulador-valor')}
          className="w-56"
          hint={
            <span>
              {withSlots(
                t(tooBig ? 'dividas-simulador-acima-saldo' : 'dividas-simulador-saldo-hoje', {
                  saldo: slot(0),
                }),
                [<Money key="s" cents={loan.balance} />],
              )}
            </span>
          }
        >
          <MoneyInput value={amount} onValueChange={setAmount} />
        </Field>
        <Field label={t('dividas-data')} className="w-44">
          <DateInput value={date} onValueChange={setDate} />
        </Field>
      </div>

      {!ready ? (
        <p className="text-[13px] text-dim">{t('dividas-simulador-informe')}</p>
      ) : (
        <div
          role="radiogroup"
          aria-label={t('dividas-simulador-forma')}
          className="grid gap-3 @min-[530px]:grid-cols-2"
        >
          {MODES.map((m) => {
            const q = sims[m];
            const s = q.data ? simulationSummary(q.data, date) : null;
            const selected = m === mode;
            return (
              <div
                key={m}
                role="radio"
                aria-checked={selected}
                aria-label={MODE_LABEL[m]}
                tabIndex={selected ? 0 : -1}
                onClick={() => setMode(m)}
                onKeyDown={(e) => {
                  if (['ArrowRight', 'ArrowLeft', 'ArrowUp', 'ArrowDown', ' '].includes(e.key)) {
                    e.preventDefault();
                    setMode(
                      e.key === ' '
                        ? m
                        : m === 'reduce_term'
                          ? 'reduce_installment'
                          : 'reduce_term',
                    );
                  }
                }}
                className={cx(
                  'flex cursor-pointer flex-col gap-3 rounded-card border p-3.5 outline-none transition-colors focus-visible:shadow-[0_0_0_1px_var(--accent)]',
                  selected
                    ? 'border-accent bg-[var(--accent-faint)]'
                    : 'border-line hover:border-line-strong',
                )}
              >
                <div className="flex items-baseline justify-between gap-3">
                  <span className="label-caps text-fg">
                    <span aria-hidden className={selected ? 'text-accent' : 'text-dim'}>
                      {selected ? '● ' : '○ '}
                    </span>
                    {MODE_LABEL[m]}
                  </span>
                </div>
                <p className="text-[12.5px] leading-snug text-dim">{MODE_HINT[m]}</p>
                {q.error ? (
                  <InlineError error={q.error} />
                ) : !s ? (
                  <p className="text-[12.5px] text-dim">{t('dividas-calculando')}</p>
                ) : (
                  <dl className="grid grid-cols-2 gap-x-4 gap-y-2.5 text-[13px]">
                    <dt className="label-caps self-center">
                      {t('dividas-simulador-juros-economizados')}
                    </dt>
                    <dd className="text-right">
                      <Money
                        cents={s.interestSaved}
                        tone="positive"
                        className="text-[15px] font-medium"
                      />
                    </dd>
                    <dt className="label-caps self-center">
                      {t('dividas-simulador-parcelas-restantes')}
                    </dt>
                    <dd className="num text-right">
                      {s.remainingBefore} → <strong className="text-fg">{s.remainingAfter}</strong>
                      {s.monthsSaved > 0 ? (
                        <span className="block text-[11.5px] text-dim">
                          {t('dividas-simulador-a-menos', { prazo: termText(s.monthsSaved) })}
                        </span>
                      ) : null}
                    </dd>
                    <dt className="label-caps self-center">{t('dividas-proxima-parcela')}</dt>
                    <dd className="text-right">
                      <Money cents={s.nextBefore} className="text-dim" /> →{' '}
                      <Money cents={s.nextAfter} className="font-medium" />
                    </dd>
                    <dt className="label-caps self-center">{t('dividas-simulador-quitacao')}</dt>
                    <dd className="num text-right">
                      {s.payoffDate ? formatDate(s.payoffDate) : '—'}
                    </dd>
                  </dl>
                )}
              </div>
            );
          })}
        </div>
      )}

      {ready && chosen ? (
        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-line pt-3">
          <p className="text-[12.5px] text-dim">
            {loan.indexer !== 'none'
              ? t('dividas-simulador-nota-indice')
              : t('dividas-simulador-nota')}
          </p>
          <Button
            variant="primary"
            onClick={() => onRegister({ amount: deferred, date: date, mode })}
          >
            {t('dividas-simulador-registrar')}
          </Button>
        </div>
      ) : null}
    </div>
  );
}
