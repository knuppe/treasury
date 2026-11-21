/**
 * Custo das taxas no longo prazo (`portfolio.fee_cost`): quanto a administração, a
 * performance e o carregamento dos fundos e planos de previdência tiram do patrimônio em 10,
 * 20 e 30 anos (valores reais de hoje), contra o mesmo dinheiro sem taxas e contra um
 * cenário de taxa menor.
 */
import { useState } from 'react';

import type { FeeHolding } from '@/api/generated/FeeHolding';
import type { FeeProjection } from '@/api/generated/FeeProjection';
import { useFeeCost } from '@/api/hooks/portfolio';
import { Field, InstrumentCard, Money, MoneyInput } from '@/design/components';
import { withSlots, slot } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

import { numText } from './charts/common';
import { ErrorBlock, LoadingBlock, Notice } from './components/States';
import { DecimalInput } from './components/DecimalInput';
import { formatDecimal } from './decimal';

const pct = (v: number | null | undefined, digits = 2) =>
  v == null ? '—' : `${numText(v * 100, digits)}%`;

const toFrac = (s: string | null) => {
  const n = s == null ? NaN : Number(s.replace(',', '.'));
  return Number.isFinite(n) ? n / 100 : undefined;
};

function Cell({ p }: { p: FeeProjection | undefined }) {
  if (!p) return <td />;
  return (
    <td className="py-1.5 pl-3 text-right align-top">
      <span className="flex flex-col leading-tight">
        <span className="text-negative">
          {'−'}
          <Money cents={p.cost} />
        </span>
        <span className="text-[11px] text-positive">
          {withSlots(t('carteira-taxas-economia', { valor: slot(0) }), [
            <Money key="v" cents={p.savings} />,
          ])}
        </span>
      </span>
    </td>
  );
}

export function FeeCostPanel() {
  const [ret, setRet] = useState<string | null>('4');
  const [alt, setAlt] = useState<string | null>('0.5');
  const [monthly, setMonthly] = useState<number | null>(null);
  const q = useFeeCost({
    real_return: toFrac(ret),
    alternative_fee: toFrac(alt),
    ...(monthly ? { monthly_contribution: monthly } : {}),
  });
  return (
    <InstrumentCard index="07.9" title={t('carteira-taxas-titulo')}>
      <div className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <Field label={t('carteira-taxas-retorno')}>
            <DecimalInput value={ret} onValueChange={setRet} placeholder="4" />
          </Field>
          <Field label={t('carteira-taxas-alternativa')}>
            <DecimalInput value={alt} onValueChange={setAlt} placeholder={formatDecimal('0.5')} />
          </Field>
          <Field label={t('carteira-taxas-aporte')}>
            <MoneyInput value={monthly} onValueChange={setMonthly} />
          </Field>
        </div>
        {q.isPending ? (
          <LoadingBlock />
        ) : q.isError ? (
          <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
        ) : !q.data.holdings.length ? (
          <Notice tone="info" title={t('carteira-taxas-nenhum')}>
            {t('carteira-taxas-nenhum-texto')}
          </Notice>
        ) : (
          <table
            className="w-full border-collapse text-[13px]"
            aria-label={t('carteira-taxas-tabela')}
          >
            <thead>
              <tr className="label-caps text-left text-dim">
                <th className="pb-1.5 font-normal">{t('carteira-taxas-fundo')}</th>
                <th className="pb-1.5 text-right font-normal">{t('carteira-taxas-hoje')}</th>
                {q.data.totals.map((total) => (
                  <th key={total.years} className="pb-1.5 pl-3 text-right font-normal">
                    {t('carteira-taxas-em-anos', { anos: total.years })}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {q.data.holdings.map((h: FeeHolding) => (
                <tr key={h.asset.id} className="border-t border-line align-top">
                  <td className="py-1.5 pr-2">
                    <span className="flex flex-col leading-tight">
                      <span className="font-medium">{h.asset.ticker ?? h.asset.name}</span>
                      <span className="text-[11.5px] text-dim">
                        {[
                          t('carteira-taxa-administracao', { pct: pct(h.admin_fee) }),
                          h.performance_fee
                            ? t('carteira-taxa-performance', { pct: pct(h.performance_fee, 0) })
                            : null,
                          h.loading_fee
                            ? t('carteira-taxa-carregamento', { pct: pct(h.loading_fee) })
                            : null,
                          h.source,
                        ]
                          .filter(Boolean)
                          .join(' · ')}
                      </span>
                    </span>
                  </td>
                  <td className="num py-1.5 text-right">
                    <Money cents={h.value} />
                  </td>
                  {h.projections.map((p) => (
                    <Cell key={p.years} p={p} />
                  ))}
                </tr>
              ))}
              <tr className="border-t border-line-strong font-medium">
                <td className="py-1.5">{t('carteira-total')}</td>
                <td />
                {q.data.totals.map((total) => (
                  <Cell key={total.years} p={total} />
                ))}
              </tr>
            </tbody>
          </table>
        )}
        {q.data ? (
          <ul className="flex flex-col gap-0.5 text-[12px] text-dim">
            {q.data.notes.map((n) => (
              <li key={n}>{n}</li>
            ))}
            {q.data.missing.length ? (
              <li>{t('carteira-taxas-sem-taxa', { lista: q.data.missing.join(', ') })}</li>
            ) : null}
          </ul>
        ) : null}
      </div>
    </InstrumentCard>
  );
}
