/**
 * Taxas de juros do mercado (`debt.market_rates`, Banco Central): para cada contrato ativo,
 * se a taxa está abaixo, na média ou acima do que os bancos cobram, e a tabela das
 * modalidades (mediana e os mais baratos). Sem IA: números oficiais, comparação simples.
 */
import { useState } from 'react';

import type { LoanRateCheck } from '@/api/generated/LoanRateCheck';
import { useMarketRates } from '@/api/hooks/debt';
import { Button, InstrumentCard } from '@/design/components';
import { cx } from '@/design/cx';
import { QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { decimalText } from './transform';

const pct = (v: number) => `${decimalText(v, 2, 2)}%`;

/** Leitura da taxa do contrato frente ao mercado (rótulo no idioma do app e tom). */
function verdict(v: string): { label: string; tone: string } {
  switch (v) {
    case 'below':
      return { label: t('dividas-mercado-abaixo'), tone: 'text-positive' };
    case 'above':
      return { label: t('dividas-mercado-acima'), tone: 'text-warning' };
    case 'far_above':
      return { label: t('dividas-mercado-bem-acima'), tone: 'text-negative' };
    default:
      return { label: t('dividas-mercado-media'), tone: 'text-fg' };
  }
}

function LoanLine({ c }: { c: LoanRateCheck }) {
  const v = verdict(c.verdict);
  return (
    <li className="flex flex-col gap-0.5 border-b border-line/60 py-2 last:border-b-0">
      <span className="flex flex-wrap items-baseline gap-2 text-[13px]">
        <span className="font-medium">{c.loan_name}</span>
        <span className={cx('label-caps', v.tone)}>{v.label}</span>
      </span>
      <span className="text-[12.5px] text-dim">{c.summary}</span>
    </li>
  );
}

export function MarketRatesPanel() {
  const [refresh, setRefresh] = useState(false);
  const q = useMarketRates(refresh);
  const r = q.data;
  return (
    <InstrumentCard index="06.9" title={t('dividas-mercado-titulo')}>
      <QueryState loading={q.isPending} error={q.error}>
        {r ? (
          <div className="flex flex-col gap-4">
            <p className="max-w-[80ch] text-[13px] text-dim">{t('dividas-mercado-intro')}</p>
            {r.loans.length ? (
              <ul aria-label={t('dividas-mercado-seus-contratos')} className="flex flex-col">
                {r.loans.map((c) => (
                  <LoanLine key={c.loan_id} c={c} />
                ))}
              </ul>
            ) : null}
            {r.modalities.length ? (
              <table
                className="w-full border-collapse text-[12.5px]"
                aria-label={t('dividas-mercado-taxas')}
              >
                <thead>
                  <tr className="label-caps text-left text-dim">
                    <th className="pb-1 font-normal">{t('dividas-mercado-credito')}</th>
                    <th className="pb-1 text-right font-normal">
                      {t('dividas-mercado-mais-barato')}
                    </th>
                    <th className="pb-1 text-right font-normal">{t('dividas-mercado-mediana')}</th>
                    <th className="pb-1 text-right font-normal">
                      {t('dividas-mercado-mais-caro')}
                    </th>
                    <th className="pb-1 pl-3 font-normal">{t('dividas-mercado-bancos')}</th>
                  </tr>
                </thead>
                <tbody>
                  {r.modalities.map((m) => (
                    <tr key={m.modality} className="border-t border-line align-top">
                      <td className="py-1.5 pr-2">{m.label}</td>
                      <td className="num py-1.5 text-right">{pct(m.min)}</td>
                      <td className="num py-1.5 text-right">
                        {pct(m.median)}
                        <span className="block text-[11px] text-dim">
                          {t('dividas-mercado-ao-ano', { taxa: pct(m.median_year) })}
                        </span>
                      </td>
                      <td className="num py-1.5 text-right">{pct(m.max)}</td>
                      <td className="py-1.5 pl-3 text-dim">
                        {m.best
                          .slice(0, 3)
                          .map((b) => `${b.institution} (${pct(b.rate_month)})`)
                          .join(' · ')}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : null}
            <div className="flex flex-wrap items-center gap-3 text-[12px] text-dim">
              <span>
                {r.fetched_at
                  ? `${t('dividas-mercado-atualizado', { data: formatDateTime(r.fetched_at) })} `
                  : ''}
                {r.warning ?? ''}
              </span>
              <Button
                size="sm"
                variant="ghost"
                loading={q.isFetching && refresh}
                onClick={() => setRefresh(true)}
              >
                {t('dividas-mercado-atualizar')}
              </Button>
            </div>
            <p className="text-[11.5px] text-dim">{r.source}</p>
          </div>
        ) : null}
      </QueryState>
    </InstrumentCard>
  );
}
