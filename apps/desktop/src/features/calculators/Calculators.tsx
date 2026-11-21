/**
 * As quatro calculadoras livres (docs/calculators.md): juros compostos, viver de renda, à vista
 * × parcelado e alugar × comprar. Contas em `math.ts`, na hora, sem gravar nada. As taxas vêm
 * da pessoa; as referências oficiais (com a data) só preenchem o campo quando ela pede.
 */
import { useMemo, useState, type ReactNode } from 'react';
import { Link } from 'react-router';

import type { RateReferences } from '@/api/generated/RateReferences';
import { InstrumentCard, KeyValue, Measure, TechTable, type TechColumn } from '@/design/components';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';
import { slot, withSlots } from '@/features/transactions/ui/slots';

import { Check } from '../portfolio/components/Fields';
import { Segmented } from '../portfolio/components/Segmented';
import {
  compound,
  installments,
  liveOnIncome,
  rentVsBuy,
  type AmortizationSystem,
  type CompoundYear,
} from './math';
import { Amount, MoneyField, NumberField, UseReference, pctLabel } from './shared';

const pct = (v: number) => v / 100;

function Layout({ inputs, result }: { inputs: ReactNode; result: ReactNode }) {
  return (
    <div className="grid gap-5 @min-[1000px]:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
      <InstrumentCard index="01" title={t('calc-entradas')}>
        <div className="@container">
          <div className="grid grid-cols-[minmax(0,1fr)] gap-4 @min-[720px]:grid-cols-2">
            {inputs}
          </div>
        </div>
      </InstrumentCard>
      <InstrumentCard index="02" title={t('calc-resultado')}>
        <div className="flex flex-col gap-5">{result}</div>
      </InstrumentCard>
    </div>
  );
}

// --- Juros compostos -----------------------------------------------------------------------

export function CompoundCalculator({ refs }: { refs: RateReferences | undefined }) {
  const [initial, setInitial] = useState(10_000);
  const [contribution, setContribution] = useState(500);
  const [rate, setRate] = useState(10);
  const [years, setYears] = useState(10);
  const [inflation, setInflation] = useState(4);
  const [fee, setFee] = useState(0);
  const r = useMemo(
    () =>
      compound({
        initial,
        contribution,
        rate: pct(rate),
        months: Math.max(0, Math.round(years * 12)),
        inflation: pct(inflation),
        fee: pct(fee),
      }),
    [initial, contribution, rate, years, inflation, fee],
  );
  const columns: TechColumn<CompoundYear>[] = [
    {
      key: 'year',
      header: t('calc-juros-col-ano'),
      cell: (y) => <span className="num">{y.year}</span>,
    },
    {
      key: 'invested',
      header: t('calc-juros-col-investido'),
      numeric: true,
      cell: (y) => <Amount value={y.invested} />,
    },
    {
      key: 'balance',
      header: t('calc-juros-col-saldo'),
      numeric: true,
      cell: (y) => <Amount value={y.balance} />,
    },
    {
      key: 'real',
      header: t('calc-juros-col-hoje'),
      numeric: true,
      cell: (y) => <Amount value={y.real} />,
    },
  ];
  return (
    <Layout
      inputs={
        <>
          <MoneyField label={t('calc-juros-inicial')} value={initial} onChange={setInitial} />
          <MoneyField
            label={t('calc-juros-aporte')}
            value={contribution}
            onChange={setContribution}
          />
          <NumberField
            label={t('calc-taxa-aa')}
            value={rate}
            onChange={setRate}
            hint={
              <UseReference
                refs={[
                  { kind: 'cdi', ref: refs?.cdi ?? null },
                  { kind: 'policy', ref: refs?.policy_rate ?? null },
                  { kind: 'savings', ref: refs?.savings ?? null },
                ]}
                onUse={setRate}
              />
            }
          />
          <NumberField label={t('calc-anos')} value={years} onChange={setYears} maxDecimals={1} />
          <NumberField
            label={t('calc-inflacao-aa')}
            value={inflation}
            onChange={setInflation}
            hint={
              <UseReference
                refs={[{ kind: 'inflation', ref: refs?.inflation_12m ?? null }]}
                onUse={setInflation}
              />
            }
          />
          <NumberField label={t('calc-juros-taxa-adm')} value={fee} onChange={setFee} />
        </>
      }
      result={
        <>
          <div className="grid gap-5 @min-[520px]:grid-cols-2">
            <Measure size="lg" label={t('calc-juros-saldo')} value={<Amount value={r.balance} />} />
            <Measure size="lg" label={t('calc-juros-hoje')} value={<Amount value={r.real} />} />
          </div>
          <KeyValue
            items={[
              { label: t('calc-juros-investido'), value: <Amount value={r.invested} /> },
              { label: t('calc-juros-juros'), value: <Amount value={r.interest} tone="auto" /> },
              ...(fee > 0
                ? [
                    {
                      label: t('calc-juros-custo-adm'),
                      value: <Amount value={-r.feeCost} tone="auto" />,
                    },
                  ]
                : []),
            ]}
          />
          {r.years.length > 0 ? (
            <TechTable
              label={t('calc-juros-tabela')}
              prefsId="calc-juros"
              dense
              rows={r.years}
              getRowId={(y) => String(y.year)}
              columns={columns}
            />
          ) : null}
        </>
      }
    />
  );
}

// --- Viver de renda --------------------------------------------------------------------------

function duration(months: number): string {
  const total = Math.floor(months);
  return t('calc-renda-dura-anos', { anos: Math.floor(total / 12), meses: total % 12 });
}

export function IncomeCalculator({ refs }: { refs: RateReferences | undefined }) {
  const [spending, setSpending] = useState(8_000);
  const [realRate, setRealRate] = useState(4);
  const [wealth, setWealth] = useState(0);
  const r = useMemo(
    () => liveOnIncome({ spending, realRate: pct(realRate), wealth }),
    [spending, realRate, wealth],
  );
  // Sugestão da taxa real a partir das referências: (1 + CDI) / (1 + inflação) − 1.
  const nominal = refs?.cdi?.rate ?? refs?.policy_rate?.rate ?? null;
  const inflation = refs?.inflation_12m?.rate ?? null;
  const suggested =
    nominal != null && inflation != null ? (1 + nominal) / (1 + inflation) - 1 : null;
  return (
    <Layout
      inputs={
        <>
          <MoneyField label={t('calc-renda-gasto')} value={spending} onChange={setSpending} />
          <MoneyField label={t('calc-renda-patrimonio')} value={wealth} onChange={setWealth} />
          <NumberField
            label={t('calc-renda-taxa-real')}
            value={realRate}
            onChange={setRealRate}
            hint={
              suggested != null
                ? t('calc-renda-taxa-real-dica', {
                    nominal: pctLabel(nominal!),
                    inflacao: pctLabel(inflation!),
                    real: pctLabel(suggested),
                  })
                : undefined
            }
          />
        </>
      }
      result={
        <>
          <Measure
            size="lg"
            label={t('calc-renda-necessario')}
            value={<Amount value={r.needed} />}
          />
          <KeyValue
            items={[
              {
                label: t('calc-renda-cobertura'),
                value: t('calc-renda-cobertura-valor', {
                  pct: formatPercent(Math.min(r.coverage, 9.99), { decimals: 0 }),
                }),
              },
              {
                label: t('calc-renda-dura'),
                value:
                  r.lastsMonths == null ? t('calc-renda-dura-sempre') : duration(r.lastsMonths),
              },
            ]}
          />
          <p className="text-[12.5px] text-dim">
            {t('calc-renda-simples')}{' '}
            <Link to="/orcamento?aba=aposentadoria" className="text-accent hover:underline">
              {t('calc-renda-plano')}
            </Link>
          </p>
        </>
      }
    />
  );
}

// --- À vista × parcelado ---------------------------------------------------------------------

export function InstallmentsCalculator({ refs }: { refs: RateReferences | undefined }) {
  const [cash, setCash] = useState(2_700);
  const [installment, setInstallment] = useState(300);
  const [count, setCount] = useState(10);
  const [firstToday, setFirstToday] = useState(false);
  const [opportunity, setOpportunity] = useState(10);
  const r = useMemo(
    () =>
      installments({
        cash,
        installment,
        count: Math.max(0, Math.round(count)),
        firstToday,
        opportunity: pct(opportunity),
      }),
    [cash, installment, count, firstToday, opportunity],
  );
  const diff = Math.abs(r.advantageCash);
  const verdict =
    diff < 0.5
      ? t('calc-parc-empate')
      : withSlots(
          r.advantageCash > 0
            ? t('calc-parc-vista-melhor', { valor: slot(0) })
            : t('calc-parc-parcelado-melhor', { valor: slot(0) }),
          [<Amount key="v" value={diff} />],
        );
  return (
    <Layout
      inputs={
        <>
          <MoneyField label={t('calc-parc-vista')} value={cash} onChange={setCash} />
          <MoneyField
            label={t('calc-parc-parcela')}
            value={installment}
            onChange={setInstallment}
          />
          <NumberField label={t('calc-parc-n')} value={count} onChange={setCount} maxDecimals={0} />
          <NumberField
            label={t('calc-parc-rendimento')}
            value={opportunity}
            onChange={setOpportunity}
            hint={
              <UseReference
                refs={[
                  { kind: 'cdi', ref: refs?.cdi ?? null },
                  { kind: 'savings', ref: refs?.savings ?? null },
                ]}
                onUse={setOpportunity}
              />
            }
          />
          <div className="@min-[720px]:col-span-2">
            <Check
              label={t('calc-parc-primeira-hoje')}
              checked={firstToday}
              onChange={setFirstToday}
            />
          </div>
          <p className="text-[12px] text-dim @min-[720px]:col-span-2">
            {t('calc-parc-rendimento-dica')}
          </p>
        </>
      }
      result={
        <>
          <Measure
            size="lg"
            label={t('calc-parc-juros')}
            value={
              r.hiddenMonthly == null
                ? t('calc-parc-sem-juros')
                : t('calc-parc-juros-valor', {
                    mes: pctLabel(r.hiddenMonthly),
                    ano: pctLabel(r.hiddenAnnual!),
                  })
            }
          />
          <KeyValue
            items={[
              { label: t('calc-parc-vista'), value: <Amount value={cash} /> },
              { label: t('calc-parc-total'), value: <Amount value={r.total} /> },
              { label: t('calc-parc-hoje'), value: <Amount value={r.presentValue} /> },
            ]}
          />
          <p className="text-[13px] text-fg" role="status">
            {verdict}
          </p>
        </>
      }
    />
  );
}

// --- Alugar × comprar ------------------------------------------------------------------------

export function RentBuyCalculator({ refs }: { refs: RateReferences | undefined }) {
  const [price, setPrice] = useState(500_000);
  const [down, setDown] = useState(100_000);
  const [costs, setCosts] = useState(4);
  const [loanRate, setLoanRate] = useState(11);
  const [loanYears, setLoanYears] = useState(30);
  const [system, setSystem] = useState<AmortizationSystem>('sac');
  const [rent, setRent] = useState(2_500);
  const [rentGrowth, setRentGrowth] = useState(4);
  const [appreciation, setAppreciation] = useState(4);
  const [ownerCosts, setOwnerCosts] = useState(1);
  const [investReturn, setInvestReturn] = useState(10);
  const [inflation, setInflation] = useState(4);
  const [horizon, setHorizon] = useState(30);
  const r = useMemo(
    () =>
      rentVsBuy({
        price,
        downPayment: down,
        buyCosts: pct(costs),
        loanRate: pct(loanRate),
        loanMonths: Math.max(1, Math.round(loanYears * 12)),
        system,
        rent,
        rentGrowth: pct(rentGrowth),
        appreciation: pct(appreciation),
        ownerCosts: pct(ownerCosts),
        investReturn: pct(investReturn),
        inflation: pct(inflation),
        months: Math.max(1, Math.round(horizon * 12)),
      }),
    [
      price,
      down,
      costs,
      loanRate,
      loanYears,
      system,
      rent,
      rentGrowth,
      appreciation,
      ownerCosts,
      investReturn,
      inflation,
      horizon,
    ],
  );
  const inflationRef = refs?.inflation_12m ?? null;
  const diff = r.buyWealth - r.rentWealth;
  const verdict = withSlots(
    diff >= 0
      ? t('calc-alug-diferenca-compra', { valor: slot(0) })
      : t('calc-alug-diferenca-aluga', { valor: slot(0) }),
    [<Amount key="v" value={Math.abs(diff)} />],
  );
  return (
    <Layout
      inputs={
        <>
          <MoneyField label={t('calc-alug-preco')} value={price} onChange={setPrice} />
          <MoneyField label={t('calc-alug-entrada')} value={down} onChange={setDown} />
          <NumberField
            label={t('calc-alug-custos')}
            value={costs}
            onChange={setCosts}
            hint={t('calc-alug-custos-dica')}
          />
          <NumberField label={t('calc-alug-juros')} value={loanRate} onChange={setLoanRate} />
          <NumberField
            label={t('calc-alug-prazo')}
            value={loanYears}
            onChange={setLoanYears}
            maxDecimals={0}
          />
          <div className="flex flex-col gap-1.5">
            <span className="label-caps">{t('calc-alug-sistema')}</span>
            <Segmented<AmortizationSystem>
              label={t('calc-alug-sistema')}
              value={system}
              onChange={setSystem}
              options={[
                { value: 'sac', label: t('calc-alug-sac') },
                { value: 'price', label: t('calc-alug-price') },
              ]}
            />
          </div>
          <MoneyField label={t('calc-alug-aluguel')} value={rent} onChange={setRent} />
          <NumberField
            label={t('calc-alug-reajuste')}
            value={rentGrowth}
            onChange={setRentGrowth}
            hint={
              <UseReference
                refs={[{ kind: 'inflation', ref: inflationRef }]}
                onUse={setRentGrowth}
              />
            }
          />
          <NumberField
            label={t('calc-alug-valorizacao')}
            value={appreciation}
            onChange={setAppreciation}
            hint={
              <UseReference
                refs={[{ kind: 'inflation', ref: inflationRef }]}
                onUse={setAppreciation}
              />
            }
          />
          <NumberField label={t('calc-alug-dono')} value={ownerCosts} onChange={setOwnerCosts} />
          <NumberField
            label={t('calc-alug-rendimento')}
            value={investReturn}
            onChange={setInvestReturn}
            hint={
              <UseReference
                refs={[
                  { kind: 'cdi', ref: refs?.cdi ?? null },
                  { kind: 'policy', ref: refs?.policy_rate ?? null },
                ]}
                onUse={setInvestReturn}
              />
            }
          />
          <NumberField
            label={t('calc-inflacao-aa')}
            value={inflation}
            onChange={setInflation}
            hint={
              <UseReference
                refs={[{ kind: 'inflation', ref: inflationRef }]}
                onUse={setInflation}
              />
            }
          />
          <NumberField
            label={t('calc-alug-horizonte')}
            value={horizon}
            onChange={setHorizon}
            maxDecimals={0}
          />
        </>
      }
      result={
        <>
          <div className="grid gap-5 @min-[520px]:grid-cols-2">
            <section aria-label={t('calc-alug-comprar')} className="flex flex-col gap-3">
              <Measure
                size="lg"
                label={`${t('calc-alug-comprar')} · ${t('calc-alug-patrimonio')}`}
                value={<Amount value={r.buyWealth} />}
                hint={withSlots(t('calc-alug-patrimonio-hoje', { valor: slot(0) }), [
                  <Amount key="v" value={r.buyWealthReal} />,
                ])}
              />
              <KeyValue
                items={[
                  { label: t('calc-alug-imovel'), value: <Amount value={r.propertyValue} /> },
                  { label: t('calc-alug-saldo'), value: <Amount value={r.loanBalance} /> },
                  { label: t('calc-alug-parcela'), value: <Amount value={r.firstInstallment} /> },
                  { label: t('calc-alug-juros-pagos'), value: <Amount value={r.interestPaid} /> },
                ]}
              />
            </section>
            <section aria-label={t('calc-alug-alugar')} className="flex flex-col gap-3">
              <Measure
                size="lg"
                label={`${t('calc-alug-alugar')} · ${t('calc-alug-patrimonio')}`}
                value={<Amount value={r.rentWealth} />}
                hint={withSlots(t('calc-alug-patrimonio-hoje', { valor: slot(0) }), [
                  <Amount key="v" value={r.rentWealthReal} />,
                ])}
              />
            </section>
          </div>
          <p className="text-[13px] text-fg" role="status">
            {verdict}
          </p>
          <p className="text-[12px] text-dim">{t('calc-alug-como')}</p>
          <p className="text-[12px] text-dim">{t('calc-alug-premissas')}</p>
        </>
      }
    />
  );
}
