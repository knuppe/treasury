/**
 * Total effective cost of a loan on screen (docs/design/loan-cet.md): the name the family's
 * country uses (CET, TAEG, TAE, APR…), the number to show for a contract (the one typed from
 * the contract, else ours) and the card that explains our calculation step by step, next to
 * the number the bank informed.
 */
import { useCountryInfo } from '@/api/hooks/country';
import type { CetReport } from '@/api/generated/CetReport';
import type { LoanView } from '@/api/generated/LoanView';
import { InstrumentCard, KeyValue, Money } from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { rateText } from './transform';

/** The family's country (Brazil while it loads, like the other screens). */
export function useCountryCode(): string {
  return useCountryInfo()?.code ?? 'BR';
}

/** Name of the all-in rate in the family's country: CET, TAEG, TAE, APR… */
export function useCetName(): string {
  return t('dividas-cet-sigla', { pais: useCountryCode() });
}

/** Above this gap (percentage points) the typed and the computed rate disagree. */
const GAP_PP = 0.1;

/** Our rate in % a year as text: the US APR (nominal) in the US, the effective rate elsewhere. */
export function computedPct(cet: CetReport | null | undefined, country: string): string | null {
  if (!cet) return null;
  if (country === 'US' && cet.us_apr != null) return (cet.us_apr * 100).toFixed(2);
  return cet.annual_pct;
}

/** The rate a list shows: the one typed from the contract, else ours. */
export function headlinePct(
  loan: LoanView,
  country: string,
): { pct: string | null; computed: boolean } {
  if (loan.cet_annual) return { pct: loan.cet_annual, computed: false };
  return { pct: computedPct(loan.cet, country), computed: true };
}

/** Lines for the "rate and cost" card: typed, ours and, when they disagree, why. */
export function cetItems(loan: LoanView, country: string, sigla: string) {
  const ours = computedPct(loan.cet, country);
  return [
    {
      label: t('dividas-cet-contratado', { sigla }),
      value: loan.cet_annual
        ? t('dividas-taxa-aa', { taxa: rateText(loan.cet_annual) })
        : t('dividas-cet-nao-informado'),
    },
    {
      label: t('dividas-cet-calculado', { sigla }),
      value: ours ? t('dividas-taxa-aa', { taxa: rateText(ours) }) : '—',
    },
  ];
}

/** The typed rate minus ours, when it is worth a warning. */
function gap(loan: LoanView, country: string): number | null {
  const ours = computedPct(loan.cet, country);
  if (!loan.cet_annual || !ours) return null;
  const typed = Number(loan.cet_annual.replace(',', '.'));
  const d = typed - Number(ours);
  return Number.isFinite(d) && Math.abs(d) > GAP_PP ? d : null;
}

function share(part: number, total: number): string {
  return total > 0 ? rateText(String((part / total) * 100), 1) : '—';
}

/** Card with our calculation, step by step, next to what the contract says. */
export function CetExplanation({ loan, index }: { loan: LoanView; index: string }) {
  const country = useCountryCode();
  const sigla = t('dividas-cet-sigla', { pais: country });
  const cet = loan.cet;
  if (!cet) return null;
  const cur = loan.currency;
  const money = (cents: number) => <Money cents={cents} currency={cur} />;
  const total = cet.total_paid;
  const d = gap(loan, country);
  const noCosts = cet.upfront.iof + cet.upfront.fees + cet.upfront.insurance === 0;
  const parts: [string, number][] = [
    [t('dividas-cet-amortizacao'), cet.amortization],
    [t('dividas-cet-juros'), cet.interest],
    [t('dividas-cet-seguros'), cet.insurance],
    [t('dividas-cet-taxas'), cet.fees],
    [t('dividas-cet-iof'), cet.upfront.iof],
    [t('dividas-cet-tarifas'), cet.upfront.fees],
    [t('dividas-cet-seguro-vista'), cet.upfront.insurance],
  ];
  return (
    <InstrumentCard index={index} title={t('dividas-cet-explicacao-titulo', { sigla })}>
      <div className="flex flex-col gap-4 text-[13px]">
        <p className="text-dim">{t('dividas-cet-explicacao-intro')}</p>
        <figure className="flex flex-col gap-1">
          <code className="num overflow-x-auto rounded bg-surface-2 px-3 py-2 text-[12.5px] text-fg">
            {t('dividas-cet-formula', { sigla })}
          </code>
          <figcaption className="text-[12px] text-dim">
            {t('dividas-cet-formula-legenda')}
          </figcaption>
        </figure>

        <div className="grid gap-6 @min-[790px]:grid-cols-3">
          <section className="flex flex-col gap-2">
            <h4 className="label-caps">{t('dividas-cet-passo-liberado')}</h4>
            <KeyValue
              layout="stacked"
              items={[
                { label: t('dividas-cet-valor-contrato'), value: money(cet.principal) },
                ...(
                  [
                    [t('dividas-cet-iof'), cet.upfront.iof],
                    [t('dividas-cet-tarifas'), cet.upfront.fees],
                    [t('dividas-cet-seguro-vista'), cet.upfront.insurance],
                  ] as [string, number][]
                )
                  .filter(([, v]) => v > 0)
                  .map(([label, v]) => ({ label, value: money(-v) })),
                { label: t('dividas-cet-liberado'), value: money(cet.released) },
              ]}
            />
          </section>
          <section className="flex flex-col gap-2">
            <h4 className="label-caps">{t('dividas-cet-passo-pagamentos')}</h4>
            <p className="text-[12px] text-dim">
              {t('dividas-cet-parcelas', {
                n: cet.installments,
                de: formatDate(cet.release_date),
                ate: formatDate(cet.last_date),
                dias: cet.days,
              })}
            </p>
            <KeyValue
              layout="stacked"
              items={[
                ...parts
                  .filter(([, v]) => v > 0)
                  .map(([label, v]) => ({
                    label,
                    value: (
                      <span>
                        {money(v)}{' '}
                        <span className="text-dim">
                          {t('dividas-cet-do-total', { pct: share(v, total) })}
                        </span>
                      </span>
                    ),
                  })),
                { label: t('dividas-cet-total-pago'), value: money(total) },
              ]}
            />
          </section>
          <section className="flex flex-col gap-2">
            <h4 className="label-caps">{t('dividas-cet-passo-resultado')}</h4>
            <KeyValue
              layout="stacked"
              items={[
                {
                  label: t('dividas-cet-calculado', { sigla }),
                  value: t('dividas-cet-resultado-valor', {
                    aa: rateText(cet.annual_pct),
                    am: rateText(cet.monthly_pct),
                  }),
                },
                ...(country === 'US' && cet.us_apr != null
                  ? [
                      {
                        label: t('dividas-cet-apr-nominal'),
                        value: t('dividas-taxa-aa', {
                          taxa: rateText((cet.us_apr * 100).toFixed(2)),
                        }),
                      },
                    ]
                  : []),
                {
                  label: t('dividas-cet-contratado', { sigla }),
                  value: loan.cet_annual
                    ? t('dividas-taxa-aa', { taxa: rateText(loan.cet_annual) })
                    : t('dividas-cet-nao-informado'),
                },
              ]}
            />
          </section>
        </div>

        {d != null ? (
          <Notice tone="warning" title={t('dividas-cet-diferenca-titulo', { sigla })}>
            {t('dividas-cet-diferenca', {
              informado: rateText(loan.cet_annual),
              calculado: rateText(computedPct(cet, country)),
              diferenca: rateText(String(d)).replace('%', ''),
            })}{' '}
            {d > 0 && noCosts
              ? t('dividas-cet-diferenca-custos')
              : t('dividas-cet-diferenca-confira')}
          </Notice>
        ) : null}
        <ul className="flex list-disc flex-col gap-1 pl-5 text-[12px] text-dim">
          {cet.index_excluded ? (
            <li>{t('dividas-cet-indice-fora', { indice: cet.index_excluded })}</li>
          ) : null}
          {noCosts ? <li>{t('dividas-cet-sem-custos', { sigla })}</li> : null}
          <li>
            {country === 'BR'
              ? t('dividas-cet-fonte-br')
              : country === 'US'
                ? t('dividas-cet-fonte-eua')
                : t('dividas-cet-fonte-outros')}
          </li>
        </ul>
      </div>
    </InstrumentCard>
  );
}

/** Balance drops the realized rate counted as paid, so the user can confirm them. */
export function RealizedDrops({ loan }: { loan: LoanView }) {
  const drops = loan.realized_drops;
  if (!drops.length) return null;
  const sum = drops.reduce((s, x) => s + x.amount, 0);
  return (
    <p className="mt-2 text-[12px] text-dim">
      {withSlots(
        t('dividas-cet-quedas', { n: drops.length, parcela: drops[0]!.number, valor: slot(0) }),
        [<Money key="v" cents={sum} currency={loan.currency} />],
      )}
    </p>
  );
}
