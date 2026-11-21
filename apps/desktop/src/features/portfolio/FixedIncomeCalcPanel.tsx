/**
 * Calcular renda fixa (só no Brasil), sem gravar nada no cofre:
 *
 * - Comparar ofertas (`portfolio.compare_fixed_income`): as ofertas que a pessoa digita
 *   (CDB, LCI, Tesouro, poupança…) lado a lado, **na ordem digitada**: bruto, IOF, IR,
 *   custódia, líquido, rendimento ao ano, acima da inflação e a equivalência em % do CDI.
 *   Sem "melhor", sem destaque e sem sugerir produto ou emissor (CVM Res. 20/2021): a
 *   pessoa pode ordenar as colunas, a tela não ordena sozinha.
 * - Vender antes do vencimento (`portfolio.simulate_fixed_income_sale`): um prefixado ou
 *   IPCA+ pela taxa de mercado do dia (marcação a mercado) ao lado de levar até o fim, e a
 *   taxa que um título novo precisa pagar para a troca empatar.
 */
import { useMemo, useState } from 'react';

import type { CompareFixedIncomeArgs } from '@/api/generated/CompareFixedIncomeArgs';
import type { FixedIncomeKind } from '@/api/generated/FixedIncomeKind';
import type { FixedIncomeOffer } from '@/api/generated/FixedIncomeOffer';
import type { FixedIncomeSaleArgs } from '@/api/generated/FixedIncomeSaleArgs';
import type { OfferIndex } from '@/api/generated/OfferIndex';
import type { OfferResult } from '@/api/generated/OfferResult';
import type { RateAssumption } from '@/api/generated/RateAssumption';
import type { SaleIndex } from '@/api/generated/SaleIndex';
import type { SaleLeg } from '@/api/generated/SaleLeg';
import { useAssets } from '@/api/hooks/assets';
import {
  useCompareFixedIncome,
  useFixedIncomeSale,
  usePortfolioSummary,
} from '@/api/hooks/portfolio';
import {
  Button,
  DateInput,
  Field,
  Icon,
  IconButton,
  Input,
  InstrumentCard,
  Money,
  MoneyInput,
  Select,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { Checkbox } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatPercent, todayIso } from '@/lib/format';

import { numText } from './charts/common';
import { DecimalInput } from './components/DecimalInput';
import { Pct } from './components/Pct';
import { Segmented } from './components/Segmented';
import { ErrorBlock, LoadingBlock, Notice } from './components/States';
import { FIXED_INCOME_KIND_LABEL } from './labels';
import { usePortfolioUi } from './state';

/** Teto de ofertas do núcleo (`compare::MAX_OFFERS`). */
const MAX_OFFERS = 12;

/** Tipos no seletor (o Renda+ e o "outro" ficam de fora: o fluxo deles não cabe aqui). */
const KINDS: readonly FixedIncomeKind[] = [
  'cdb',
  'rdb',
  'lc',
  'lci',
  'lca',
  'cri',
  'cra',
  'debenture',
  'debenture_incentivada',
  'tesouro_selic',
  'tesouro_prefixado',
  'tesouro_ipca',
  'poupanca',
];

const INDEX_LABEL: Record<Exclude<OfferIndex, 'savings'>, () => string> = {
  cdi_pct: () => t('carteira-rf-indice-cdi-pct'),
  cdi_plus: () => t('carteira-rf-indice-cdi-mais'),
  selic_plus: () => t('carteira-rf-indice-selic-mais'),
  prefixed: () => t('carteira-rf-indice-pre'),
  ipca_plus: () => t('carteira-rf-indice-ipca-mais'),
};

/** Rendimento que o tipo pede ao ser escolhido (a pessoa ainda pode trocar, menos na poupança). */
const INDEX_OF_KIND: Partial<Record<FixedIncomeKind, OfferIndex>> = {
  tesouro_selic: 'selic_plus',
  tesouro_prefixado: 'prefixed',
  tesouro_ipca: 'ipca_plus',
  poupanca: 'savings',
};

interface OfferDraft {
  id: number;
  kind: FixedIncomeKind;
  index: OfferIndex;
  /** Decimal canônico (`"110"`, `"12.5"`) ou `null`. */
  rate: string | null;
  months: string;
  end: string | null;
  label: string;
}

/** Dois exemplos para a tela já mostrar números ao abrir. */
const EXAMPLES: readonly OfferDraft[] = [
  { id: 1, kind: 'cdb', index: 'cdi_pct', rate: '110', months: '24', end: null, label: '' },
  { id: 2, kind: 'lci', index: 'cdi_pct', rate: '90', months: '24', end: null, label: '' },
];

/** Percentual digitado (`"14.9"`) → fração (`0.149`). */
const frac = (s: string | null) => {
  const n = s == null ? NaN : Number(s);
  return Number.isFinite(n) ? n / 100 : undefined;
};

const rateOf = (s: string | null) => {
  const n = s == null ? NaN : Number(s);
  return Number.isFinite(n) ? n : null;
};

function sourceText(a: RateAssumption): string {
  switch (a.source) {
    case 'user':
      return t('carteira-rf-fonte-user');
    case 'vault':
      return a.date
        ? t('carteira-rf-fonte-vault-data', { data: formatDate(a.date) })
        : t('carteira-rf-fonte-vault');
    case 'projection':
      return t('carteira-rf-fonte-projection');
    default:
      return t('carteira-rf-fonte-default');
  }
}

function Notes({ notes }: { notes: readonly string[] }) {
  if (!notes.length) return null;
  return (
    <ul className="flex flex-col gap-0.5 text-[12px] text-dim">
      {notes.map((n) => (
        <li key={n}>{n}</li>
      ))}
    </ul>
  );
}

// ---------------------------------------------------------------------------
// Comparar ofertas

function OfferEditor({
  offer,
  n,
  onChange,
  onRemove,
}: {
  offer: OfferDraft;
  n: number;
  onChange: (patch: Partial<OfferDraft>) => void;
  onRemove?: () => void;
}) {
  const savings = offer.kind === 'poupanca';
  const setKind = (kind: FixedIncomeKind) => {
    const forced = INDEX_OF_KIND[kind];
    onChange({
      kind,
      index: forced ?? (offer.index === 'savings' ? 'cdi_pct' : offer.index),
    });
  };
  return (
    <div
      role="group"
      aria-label={t('carteira-rf-oferta-n', { n })}
      className="grid items-end gap-2 border-t border-line pt-3 sm:grid-cols-[repeat(6,minmax(0,1fr))_auto]"
    >
      <Field label={t('carteira-rf-tipo')}>
        <Select
          value={offer.kind}
          onValueChange={(v) => setKind(v as FixedIncomeKind)}
          options={KINDS.map((k) => ({ value: k, label: FIXED_INCOME_KIND_LABEL[k] }))}
        />
      </Field>
      {savings ? (
        // Poupança: a regra da lei, sem taxa para digitar.
        <div className="sm:col-span-2" />
      ) : (
        <>
          <Field label={t('carteira-rf-indice')}>
            <Select
              value={offer.index}
              onValueChange={(v) => onChange({ index: v as OfferIndex })}
              options={(Object.keys(INDEX_LABEL) as (keyof typeof INDEX_LABEL)[]).map((i) => ({
                value: i,
                label: INDEX_LABEL[i](),
              }))}
            />
          </Field>
          <Field label={t('carteira-rf-taxa')}>
            <DecimalInput
              value={offer.rate}
              onValueChange={(rate) => onChange({ rate })}
              maxDecimals={4}
            />
          </Field>
        </>
      )}
      <Field label={t('carteira-rf-meses')}>
        <Input
          numeric
          inputMode="numeric"
          value={offer.months}
          onChange={(e) => onChange({ months: e.target.value.replace(/\D/g, '').slice(0, 3) })}
        />
      </Field>
      <Field label={t('carteira-rf-resgate')}>
        <DateInput value={offer.end} onValueChange={(end) => onChange({ end })} />
      </Field>
      <Field label={t('carteira-rf-nome')}>
        <Input
          value={offer.label}
          maxLength={60}
          onChange={(e) => onChange({ label: e.target.value })}
        />
      </Field>
      <div className="flex h-9 items-center">
        {onRemove ? (
          <IconButton
            label={t('carteira-rf-remover', { n })}
            icon={<Icon name="x" size={12} />}
            onClick={onRemove}
          />
        ) : null}
      </div>
    </div>
  );
}

function equivalenceText(r: OfferResult): string {
  // Isento: quanto um CDB precisaria pagar; tributável: quanto uma LCI/LCA precisaria pagar.
  const pct = r.ir_exempt ? r.taxable_equivalent_pct_cdi : r.exempt_equivalent_pct_cdi;
  if (pct == null) return '—';
  return t(r.ir_exempt ? 'carteira-rf-equiv-cdb' : 'carteira-rf-equiv-lci', {
    pct: numText(pct, 1),
  });
}

type ResultRow = OfferResult & { id: string };

function resultColumns(): TechColumn<ResultRow>[] {
  type MoneyKey = 'gross' | 'iof' | 'custody' | 'net';
  const money = (key: MoneyKey, header: string, strong = false) => ({
    key,
    header,
    numeric: true,
    sortValue: (r: ResultRow) => r[key],
    cell: (r: ResultRow) => (
      <span className={strong ? 'font-medium' : undefined}>
        <Money cents={r[key]} />
      </span>
    ),
  });
  return [
    {
      key: 'label',
      header: t('carteira-rf-col-oferta'),
      hideable: false,
      width: '200px',
      cell: (r) => (
        <span className="flex flex-col leading-tight">
          <span className="font-medium">{r.label}</span>
          <span className="text-[11.5px] text-dim">
            {t('carteira-rf-ate', { data: formatDate(r.end) })}
          </span>
        </span>
      ),
    },
    {
      key: 'calendar_days',
      header: t('carteira-rf-col-prazo'),
      numeric: true,
      sortValue: (r) => r.calendar_days,
      cell: (r) => t('carteira-rf-dias', { n: r.calendar_days }),
    },
    money('gross', t('carteira-rf-col-bruto')),
    money('iof', t('carteira-rf-col-iof')),
    {
      key: 'ir',
      header: t('carteira-rf-col-ir'),
      numeric: true,
      sortValue: (r) => r.ir,
      cell: (r) =>
        r.ir_exempt ? (
          <span className="text-dim">{t('carteira-rf-isento')}</span>
        ) : (
          <span className="flex flex-col items-end leading-tight">
            <Money cents={r.ir} />
            <span className="text-[11px] text-dim">
              {formatPercent(r.ir_rate, { decimals: 1 })}
            </span>
          </span>
        ),
    },
    money('custody', t('carteira-rf-col-custodia')),
    money('net', t('carteira-rf-col-liquido'), true),
    {
      key: 'net_annual',
      header: t('carteira-rf-col-ano'),
      numeric: true,
      sortValue: (r) => r.net_annual,
      cell: (r) => <Pct value={r.net_annual} decimals={2} />,
    },
    {
      key: 'real_annual',
      header: t('carteira-rf-col-real'),
      numeric: true,
      sortValue: (r) => r.real_annual,
      cell: (r) => <Pct value={r.real_annual} decimals={2} signed />,
    },
    {
      key: 'equivalence',
      header: t('carteira-rf-col-equivalencia'),
      width: '190px',
      sortValue: (r) =>
        (r.ir_exempt ? r.taxable_equivalent_pct_cdi : r.exempt_equivalent_pct_cdi) ?? null,
      cell: (r) => <span className="text-[12px] whitespace-normal">{equivalenceText(r)}</span>,
    },
    {
      key: 'fgc',
      header: t('carteira-rf-col-fgc'),
      align: 'center',
      sortValue: (r) => (r.fgc == null ? null : r.fgc ? 1 : 0),
      cell: (r) => (r.fgc == null ? '—' : r.fgc ? t('carteira-rf-sim') : t('carteira-rf-nao')),
    },
    {
      key: 'notes',
      header: t('carteira-rf-col-avisos'),
      width: '220px',
      sortable: false,
      cell: (r) => (
        <span className="text-[11.5px] whitespace-normal text-dim">{r.notes.join(' ')}</span>
      ),
    },
  ];
}

function CompareSection() {
  const [amount, setAmount] = useState<number | null>(1_000_000);
  const [start, setStart] = useState<string | null>(null);
  const [offers, setOffers] = useState<OfferDraft[]>(() => EXAMPLES.map((o) => ({ ...o })));
  const [nextId, setNextId] = useState(EXAMPLES.length + 1);
  const [cdi, setCdi] = useState<string | null>(null);
  const [selic, setSelic] = useState<string | null>(null);
  const [ipca, setIpca] = useState<string | null>(null);
  const [tr, setTr] = useState<string | null>(null);

  const args = useMemo<CompareFixedIncomeArgs | null>(() => {
    if (!amount || amount <= 0) return null;
    const list: FixedIncomeOffer[] = [];
    for (const o of offers) {
      const savings = o.kind === 'poupanca';
      const rate = savings ? 0 : rateOf(o.rate);
      const months = Number(o.months);
      if (rate == null || (!o.end && !(months > 0))) return null;
      list.push({
        kind: o.kind,
        index: savings ? 'savings' : o.index,
        rate,
        ...(o.end ? { end: o.end } : { months }),
        ...(o.label.trim() ? { label: o.label.trim() } : {}),
      });
    }
    return {
      amount,
      offers: list,
      start: start ?? undefined,
      cdi: frac(cdi),
      selic: frac(selic),
      ipca: frac(ipca),
      tr: frac(tr),
    };
  }, [amount, start, offers, cdi, selic, ipca, tr]);
  const q = useCompareFixedIncome(args);
  const columns = useMemo(() => resultColumns(), []);
  const rows = useMemo<ResultRow[]>(
    () => (q.data?.offers ?? []).map((r, i) => ({ ...r, id: String(i) })),
    [q.data],
  );

  const patch = (id: number, p: Partial<OfferDraft>) =>
    setOffers((list) => list.map((o) => (o.id === id ? { ...o, ...p } : o)));
  const add = () => {
    setOffers((list) => [
      ...list,
      { id: nextId, kind: 'cdb', index: 'cdi_pct', rate: null, months: '', end: null, label: '' },
    ]);
    setNextId((n) => n + 1);
  };

  const a = q.data?.assumptions;
  const assumption = (
    label: string,
    value: string | null,
    set: (v: string | null) => void,
    used: RateAssumption | undefined,
    digits: number,
  ) => (
    <Field label={label} hint={used ? sourceText(used) : undefined}>
      <DecimalInput
        value={value}
        onValueChange={set}
        maxDecimals={digits + 2}
        placeholder={used ? numText(used.value * 100, digits) : undefined}
      />
    </Field>
  );

  return (
    <InstrumentCard index="07.10" title={t('carteira-rf-comparar-titulo')}>
      <div className="flex flex-col gap-4">
        <p className="text-[13px] text-dim">{t('carteira-rf-comparar-texto')}</p>
        <div className="grid gap-3 sm:grid-cols-3">
          <Field label={t('carteira-rf-valor')}>
            <MoneyInput value={amount} onValueChange={setAmount} currency="BRL" />
          </Field>
          <Field label={t('carteira-rf-inicio')} hint={t('carteira-rf-inicio-dica')}>
            <DateInput value={start} onValueChange={setStart} />
          </Field>
        </div>

        <section aria-label={t('carteira-rf-ofertas')} className="flex flex-col gap-3">
          <h3 className="label-caps text-fg">{t('carteira-rf-ofertas')}</h3>
          {offers.map((o, i) => (
            <OfferEditor
              key={o.id}
              offer={o}
              n={i + 1}
              onChange={(p) => patch(o.id, p)}
              onRemove={
                offers.length > 1
                  ? () => setOffers((list) => list.filter((x) => x.id !== o.id))
                  : undefined
              }
            />
          ))}
          <div className="flex items-center gap-3">
            <Button
              icon={<Icon name="plus" size={12} />}
              onClick={add}
              disabled={offers.length >= MAX_OFFERS}
            >
              {t('carteira-rf-adicionar')}
            </Button>
            {offers.length >= MAX_OFFERS ? (
              <span className="text-[12px] text-dim">
                {t('carteira-rf-limite', { max: MAX_OFFERS })}
              </span>
            ) : null}
          </div>
        </section>

        <section aria-label={t('carteira-rf-premissas')} className="flex flex-col gap-2">
          <h3 className="label-caps text-fg">{t('carteira-rf-premissas')}</h3>
          <p className="text-[12px] text-dim">{t('carteira-rf-premissas-texto')}</p>
          <div className="grid gap-3 sm:grid-cols-4">
            {assumption(t('carteira-rf-cdi'), cdi, setCdi, a?.cdi, 2)}
            {assumption(t('carteira-rf-selic'), selic, setSelic, a?.selic, 2)}
            {assumption(t('carteira-rf-ipca'), ipca, setIpca, a?.ipca, 2)}
            {assumption(t('carteira-rf-tr'), tr, setTr, a?.tr, 4)}
          </div>
        </section>

        {!args ? (
          <Notice tone="info" title={t('carteira-rf-incompleto')}>
            {t('carteira-rf-incompleto-texto')}
          </Notice>
        ) : q.isPending ? (
          <LoadingBlock />
        ) : q.isError ? (
          <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
        ) : (
          <>
            {/* Sem ordenação padrão: as linhas seguem a ordem digitada. */}
            <TechTable
              label={t('carteira-rf-tabela')}
              prefsId="carteira-rf-ofertas"
              minWidth="1250px"
              rows={rows}
              columns={columns}
              getRowId={(r) => r.id}
            />
            <Notes notes={q.data.notes} />
          </>
        )}
      </div>
    </InstrumentCard>
  );
}

// ---------------------------------------------------------------------------
// Vender antes do vencimento

/** Prefixados e IPCA+ da carteira com vencimento, para preencher o formulário. */
function useHeldBonds() {
  const deflator = usePortfolioUi((s) => s.deflator);
  const assets = useAssets({ classes: ['fixed_income', 'treasury_br'], position: 'open' });
  const summary = usePortfolioSummary({ deflator });
  return useMemo(() => {
    const positions = summary.data?.positions ?? [];
    return (assets.data ?? []).flatMap((asset) => {
      const fi = asset.fixed_income;
      if (!fi?.maturity_date || (fi.indexer !== 'prefixado' && fi.indexer !== 'ipca')) return [];
      const held = positions.filter((p) => p.asset.id === asset.id && Number(p.quantity) > 0);
      if (!held.length) return [];
      const first = held
        .map((p) => p.first_date)
        .filter((d): d is string => Boolean(d))
        .sort()[0];
      const index: SaleIndex = fi.indexer === 'ipca' ? 'ipca_plus' : 'prefixed';
      return [
        {
          id: asset.id,
          name: asset.name,
          index,
          rate: fi.rate,
          start: fi.issue_date ?? first ?? null,
          maturity: fi.maturity_date,
          exempt: fi.ir_exempt,
          cost: held.reduce((s, p) => s + p.cost, 0),
        },
      ];
    });
  }, [assets.data, summary.data]);
}

function LegCells({ leg }: { leg: SaleLeg }) {
  return (
    <>
      <td className="num py-1.5 pl-3 text-right">
        <Money cents={leg.gross} />
      </td>
      <td className="num py-1.5 pl-3 text-right">
        <Money cents={leg.ir} />
      </td>
      <td className="num py-1.5 pl-3 text-right">
        <Money cents={leg.iof} />
      </td>
      <td className="num py-1.5 pl-3 text-right font-medium">
        <Money cents={leg.net} />
      </td>
    </>
  );
}

function SaleSection() {
  const bonds = useHeldBonds();
  const [picked, setPicked] = useState<string | undefined>(undefined);
  const [index, setIndex] = useState<SaleIndex>('prefixed');
  const [amount, setAmount] = useState<number | null>(null);
  const [start, setStart] = useState<string | null>(null);
  const [maturity, setMaturity] = useState<string | null>(null);
  const [saleDate, setSaleDate] = useState<string | null>(() => todayIso());
  const [contracted, setContracted] = useState<string | null>(null);
  const [market, setMarket] = useState<string | null>(null);
  const [exempt, setExempt] = useState(false);
  const [ipca, setIpca] = useState<string | null>(null);

  const fill = (id: string) => {
    setPicked(id);
    const b = bonds.find((x) => x.id === id);
    if (!b) return;
    setIndex(b.index);
    setAmount(b.cost > 0 ? b.cost : null);
    setStart(b.start);
    setMaturity(b.maturity);
    setContracted(b.rate);
    setExempt(b.exempt);
  };

  const args = useMemo<FixedIncomeSaleArgs | null>(() => {
    const rc = rateOf(contracted);
    const rm = rateOf(market);
    if (!amount || amount <= 0 || !start || !maturity || !saleDate) return null;
    if (rc == null || rm == null || !(start < saleDate && saleDate < maturity)) return null;
    return {
      amount,
      start,
      maturity,
      sale_date: saleDate,
      index,
      contracted_rate: rc,
      market_rate: rm,
      ir_exempt: exempt,
      ipca: index === 'ipca_plus' ? frac(ipca) : undefined,
    };
  }, [amount, start, maturity, saleDate, index, contracted, market, exempt, ipca]);
  const q = useFixedIncomeSale(args);
  const s = q.data;
  const rateHint = index === 'ipca_plus' ? t('carteira-rf-venda-taxa-real') : undefined;

  return (
    <InstrumentCard index="07.11" title={t('carteira-rf-venda-titulo')}>
      <div className="flex flex-col gap-4">
        <p className="text-[13px] text-dim">{t('carteira-rf-venda-texto')}</p>
        {bonds.length ? (
          <Field label={t('carteira-rf-venda-carteira')} className="sm:max-w-sm">
            <Select
              value={picked}
              onValueChange={fill}
              placeholder={t('carteira-rf-venda-carteira-escolha')}
              options={bonds.map((b) => ({ value: b.id, label: b.name }))}
            />
          </Field>
        ) : null}
        <Segmented
          label={t('carteira-rf-venda-indice')}
          value={index}
          onChange={setIndex}
          options={[
            { value: 'prefixed', label: t('carteira-rf-venda-pre') },
            { value: 'ipca_plus', label: t('carteira-rf-venda-ipca') },
          ]}
        />
        <div className="grid gap-3 sm:grid-cols-4">
          <Field label={t('carteira-rf-valor')}>
            <MoneyInput value={amount} onValueChange={setAmount} currency="BRL" />
          </Field>
          <Field label={t('carteira-rf-venda-aplicacao')}>
            <DateInput value={start} onValueChange={setStart} />
          </Field>
          <Field label={t('carteira-rf-venda-vencimento')}>
            <DateInput value={maturity} onValueChange={setMaturity} />
          </Field>
          <Field label={t('carteira-rf-venda-data')}>
            <DateInput value={saleDate} onValueChange={setSaleDate} />
          </Field>
          <Field label={t('carteira-rf-venda-contratada')} hint={rateHint}>
            <DecimalInput value={contracted} onValueChange={setContracted} maxDecimals={4} />
          </Field>
          <Field label={t('carteira-rf-venda-mercado')} hint={rateHint}>
            <DecimalInput value={market} onValueChange={setMarket} maxDecimals={4} />
          </Field>
          {index === 'ipca_plus' ? (
            <Field
              label={t('carteira-rf-venda-ipca-premissa')}
              hint={s ? sourceText(s.ipca) : undefined}
            >
              <DecimalInput
                value={ipca}
                onValueChange={setIpca}
                maxDecimals={4}
                placeholder={s ? numText(s.ipca.value * 100, 2) : undefined}
              />
            </Field>
          ) : null}
          <div className="flex h-9 items-center self-end">
            <Checkbox checked={exempt} onChange={setExempt} label={t('carteira-rf-venda-isento')} />
          </div>
        </div>

        {!args ? (
          <Notice tone="info" title={t('carteira-rf-venda-incompleto')}>
            {t('carteira-rf-venda-incompleto-texto')}
          </Notice>
        ) : q.isPending ? (
          <LoadingBlock />
        ) : q.isError ? (
          <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
        ) : s ? (
          <>
            <dl className="grid gap-3 sm:grid-cols-3">
              <div className="flex flex-col gap-0.5">
                <dt className="label-caps">{t('carteira-rf-venda-curva')}</dt>
                <dd className="num text-[18px]">
                  <Money cents={s.curve_value} />
                </dd>
              </div>
              <div className="flex flex-col gap-0.5">
                <dt className="label-caps">{t('carteira-rf-venda-preco')}</dt>
                <dd className="num text-[18px]">
                  <Money cents={s.market_value} />
                </dd>
              </div>
              <div className="flex flex-col gap-0.5">
                <dt className="label-caps">{t('carteira-rf-venda-diferenca')}</dt>
                <dd className="num text-[18px]" aria-label={t('carteira-rf-venda-diferenca')}>
                  <Money cents={s.mark_to_market} tone="auto" />
                </dd>
                <dd className="text-[11.5px] text-dim">{t('carteira-rf-venda-diferenca-dica')}</dd>
              </div>
            </dl>
            <table
              className="w-full border-collapse text-[13px]"
              aria-label={t('carteira-rf-venda-tabela')}
            >
              <thead>
                <tr className="label-caps text-left text-dim">
                  <th className="pb-1.5 font-normal" />
                  <th className="pb-1.5 pl-3 text-right font-normal">
                    {t('carteira-rf-venda-bruto')}
                  </th>
                  <th className="pb-1.5 pl-3 text-right font-normal">
                    {t('carteira-rf-venda-ir')}
                  </th>
                  <th className="pb-1.5 pl-3 text-right font-normal">
                    {t('carteira-rf-venda-iof')}
                  </th>
                  <th className="pb-1.5 pl-3 text-right font-normal">
                    {t('carteira-rf-venda-liquido')}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr className="border-t border-line">
                  <th scope="row" className="py-1.5 text-left font-normal">
                    {t('carteira-rf-venda-hoje')}
                  </th>
                  <LegCells leg={s.sale} />
                </tr>
                <tr className="border-t border-line">
                  <th scope="row" className="py-1.5 text-left font-normal">
                    {t('carteira-rf-venda-levar')}
                  </th>
                  <LegCells leg={s.hold} />
                </tr>
              </tbody>
            </table>
            <div className="flex flex-col gap-0.5">
              <span className="label-caps">{t('carteira-rf-venda-empate')}</span>
              <p className="text-[13px]">
                {s.breakeven_rate == null || !maturity
                  ? t('carteira-rf-venda-empate-nenhuma')
                  : t(
                      index === 'ipca_plus'
                        ? 'carteira-rf-venda-empate-ipca'
                        : 'carteira-rf-venda-empate-pre',
                      { data: formatDate(maturity), taxa: numText(s.breakeven_rate, 2) },
                    )}
              </p>
            </div>
            <Notes notes={s.notes} />
          </>
        ) : null}
      </div>
    </InstrumentCard>
  );
}

export function FixedIncomeCalcPanel() {
  return (
    <div className="flex flex-col gap-6">
      <CompareSection />
      <SaleSection />
    </div>
  );
}
