/**
 * Carnê-leão do mês (`tax.carne_leao`): rendimentos recebidos de pessoas físicas ou do
 * exterior (aluguel de inquilino pessoa física, consultas particulares, trabalho de fora),
 * com previdência oficial, dependentes, pensão e livro-caixa, pela tabela mensal e a redução
 * da Lei 15.270/2025; DARF 0190 até o último dia útil do mês seguinte. Fica fechado até a
 * família abrir.
 */
import { useState } from 'react';

import { useCarneLeao } from '@/api/hooks/tax';
import { KeyValue, Money, MoneyInput } from '@/design/components';
import { InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

export function CarneLeaoCard({ month }: { month: string }) {
  const [income, setIncome] = useState<number | null>(null);
  const [inss, setInss] = useState<number | null>(0);
  const [dependents, setDependents] = useState(0);
  const [alimony, setAlimony] = useState<number | null>(0);
  const [livroCaixa, setLivroCaixa] = useState<number | null>(0);
  const q = useCarneLeao(
    income && income > 0
      ? {
          month,
          income,
          social_security: inss ?? 0,
          dependents,
          alimony: alimony ?? 0,
          livro_caixa: livroCaixa ?? 0,
          foreign_tax: 0,
        }
      : null,
  );
  const money = (label: string, value: number | null, set: (v: number | null) => void) => (
    <label className="flex flex-col gap-1 text-[12.5px]">
      <span className="text-dim">{label}</span>
      <MoneyInput value={value} onValueChange={set} currency="BRL" aria-label={label} />
    </label>
  );
  return (
    <details className="group rounded-card border border-line px-3.5 py-2.5">
      <summary className="cursor-pointer select-none text-[13px] font-medium text-fg">
        {t('ir-carne-titulo')}
      </summary>
      <div className="mt-3 flex flex-col gap-3">
        <p className="text-[12.5px] text-dim">{t('ir-carne-dica')}</p>
        <div className="grid gap-2 @min-[530px]:grid-cols-3">
          {money(t('ir-carne-recebido'), income, setIncome)}
          {money(t('ir-carne-inss'), inss, setInss)}
          <label className="flex flex-col gap-1 text-[12.5px]">
            <span className="text-dim">{t('ir-anual-campo-dependentes')}</span>
            <input
              type="number"
              min={0}
              max={20}
              aria-label={t('ir-anual-campo-dependentes')}
              className="h-9 rounded-control border border-line bg-surface px-2.5 text-[13px] text-fg"
              value={dependents}
              onChange={(e) =>
                setDependents(Math.max(0, Math.min(20, Number(e.target.value) || 0)))
              }
            />
          </label>
          {money(t('ir-carne-pensao'), alimony, setAlimony)}
          {money(t('ir-carne-livro-caixa'), livroCaixa, setLivroCaixa)}
        </div>
        <InlineError error={q.error} />
        {q.data ? (
          <KeyValue
            layout="stacked"
            items={[
              {
                label: t('ir-carne-imposto'),
                value: <Money cents={q.data.tax} className="font-medium" />,
              },
              {
                label: 'DARF',
                value: q.data.below_minimum
                  ? t('ir-carne-abaixo-minimo')
                  : t('ir-carne-darf', {
                      codigo: q.data.revenue_code,
                      vencimento: formatDate(q.data.due_date),
                    }),
              },
            ]}
          />
        ) : null}
      </div>
    </details>
  );
}
