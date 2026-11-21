/**
 * Cadastro e edição de ativo. Primeiro o tipo; depois só os campos daquele tipo
 * (`assetClassForm.ts`): a bolsa para o que é negociado em bolsa, o CNPJ para fundos e
 * previdência, o título para o Tesouro, os termos de renda fixa (tipo, indexador, taxa,
 * emissão, vencimento, carência, liquidez, isenção e emissor) e o plano de previdência
 * (VGBL/PGBL, regime, seguradora e taxas), que grava junto com o ativo. Fonte e símbolo da
 * cotação, ISIN, moeda e país das bolsas ficam em "Cotação e identificadores", recolhido.
 *
 * No cadastro, a busca de tickers (ativos locais, universo da análise e Yahoo) escolhe o tipo
 * e preenche ticker, bolsa, moeda, país e nome. A bolsa vem do catálogo (`market.exchanges`)
 * e dá a moeda e o país padrão. Na edição, mercado, moeda, país, ISIN, CNPJ e código do
 * Tesouro ficam só leitura (salvo o CNPJ do banco emissor da renda fixa, que o score usa).
 */
import { useState, type FormEvent } from 'react';

import { useCreateAsset, useUpdateAsset } from '@/api/hooks/assets';
import { useCustodyAccounts } from '@/api/hooks/portfolio';
import { ACCOUNT_KIND } from '@/features/transactions/ui/labels';
import { useExchanges } from '@/api/hooks/market';
import { usePension, useSetPensionPlan } from '@/api/hooks/pension';
import type { TickerSuggestion } from '@/api/generated/TickerSuggestion';
import type { Asset } from '@/api/generated/Asset';
import type { AssetClass } from '@/api/generated/AssetClass';
import type { AssetPatch } from '@/api/generated/AssetPatch';
import type { FixedIncomeKind } from '@/api/generated/FixedIncomeKind';
import type { FixedIncomeTerms } from '@/api/generated/FixedIncomeTerms';
import type { Indexer } from '@/api/generated/Indexer';
import type { Liquidity } from '@/api/generated/Liquidity';
import type { PensionPlan } from '@/api/generated/PensionPlan';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Input,
  Select,
  toast,
} from '@/design/components';
import { t } from '@/i18n';

/** Valor do seletor de conta sem conta (o Radix não aceita item vazio). */
const NO_ACCOUNT = '__sem_conta__';
import { FormError } from '@/vault/FormError';

import { describeSuggestion } from '../market/AssetPicker';
import { TickerSearch } from '../market/TickerSearch';
import { Check } from '../portfolio/components/Fields';
import { DecimalInput } from '../portfolio/components/DecimalInput';
import {
  CLASS_LABEL,
  defaultCurrency,
  defaultIrExempt,
  defaultMarket,
  FIXED_INCOME_KIND_LABEL,
  INDEXER_LABEL,
  isExchangeListed,
  LIQUIDITY_LABEL,
  needsExchange,
  rateHint,
} from '../portfolio/labels';
import { classForm, providerOptions, typeGroups } from './assetClassForm';
import { PensionPlanFields, planDraft, planTerms, type PlanDraft } from './PensionPlanFields';

const DEFAULT_MARKET = '__padrao';

interface Terms {
  kind: FixedIncomeKind;
  indexer: Indexer;
  rate: string | null;
  issueDate: string | null;
  maturityDate: string | null;
  graceUntil: string | null;
  liquidity: Liquidity;
  irExempt: boolean;
  issuer: string;
}

function termsFrom(fi: FixedIncomeTerms | null | undefined, cls: AssetClass): Terms {
  if (fi) {
    return {
      kind: fi.kind,
      indexer: fi.indexer,
      rate: fi.rate,
      issueDate: fi.issue_date,
      maturityDate: fi.maturity_date,
      graceUntil: fi.grace_until,
      liquidity: fi.liquidity,
      irExempt: fi.ir_exempt,
      issuer: fi.issuer ?? '',
    };
  }
  const treasury = cls === 'treasury_br';
  return {
    kind: treasury ? 'tesouro_ipca' : 'cdb',
    indexer: treasury ? 'ipca' : 'cdi',
    rate: null,
    issueDate: null,
    maturityDate: null,
    graceUntil: null,
    liquidity: treasury ? 'daily' : 'at_maturity',
    irExempt: false,
    issuer: treasury ? 'Tesouro Nacional' : '',
  };
}

function toTerms(d: Terms): FixedIncomeTerms {
  return {
    kind: d.kind,
    indexer: d.indexer,
    rate: d.rate ?? '0',
    issue_date: d.issueDate,
    maturity_date: d.maturityDate,
    grace_until: d.graceUntil,
    liquidity: d.liquidity,
    ir_exempt: d.irExempt,
    issuer: d.issuer.trim() || null,
  };
}

export function AssetFormDialog({
  open,
  onOpenChange,
  asset,
  onSaved,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset?: Asset | null;
  onSaved?: (asset: Asset) => void;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      index="07"
      title={
        asset
          ? t('ativos-form-editar', { nome: asset.ticker ?? asset.name })
          : t('ativos-cadastrar')
      }
      description={asset ? t('ativos-form-editar-descricao') : t('ativos-form-novo-descricao')}
    >
      {open ? (
        <AssetFormLoader
          asset={asset ?? null}
          onDone={(a) => {
            onOpenChange(false);
            onSaved?.(a);
          }}
        />
      ) : null}
    </Dialog>
  );
}

/** Na edição de uma previdência, espera o plano para preencher os campos dele. */
function AssetFormLoader({ asset, onDone }: { asset: Asset | null; onDone: (a: Asset) => void }) {
  const pension = asset?.class === 'pension';
  const q = usePension(asset?.id ?? '', pension);
  if (pension && q.isLoading) return null;
  return <AssetForm asset={asset} plan={q.data?.plan ?? null} onDone={onDone} />;
}

/** Escolha do tipo de ativo, agrupada (rádios). */
function TypePicker({
  value,
  onChange,
}: {
  value: AssetClass | null;
  onChange: (c: AssetClass) => void;
}) {
  return (
    <div role="radiogroup" aria-label={t('ativos-form-tipo')} className="flex flex-col gap-2">
      <span className="label-caps">{t('ativos-form-tipo')}</span>
      {typeGroups().map((g) => (
        <div key={g.label} className="flex items-start gap-1.5">
          <span className="w-36 shrink-0 pt-1 text-[12px] text-dim">{g.label}</span>
          <div className="flex flex-wrap gap-1.5">
            {g.classes.map((c) => {
              const checked = c === value;
              return (
                <button
                  key={c}
                  type="button"
                  role="radio"
                  aria-checked={checked}
                  onClick={() => onChange(c)}
                  className="chip"
                >
                  {CLASS_LABEL[c]}
                </button>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}

function AssetForm({
  asset,
  plan: savedPlan,
  onDone,
}: {
  asset: Asset | null;
  plan: PensionPlan | null;
  onDone: (a: Asset) => void;
}) {
  const create = useCreateAsset();
  const update = useUpdateAsset();
  const setPlan = useSetPensionPlan();
  const [cls, setCls] = useState<AssetClass | null>(asset?.class ?? null);
  const [ticker, setTicker] = useState(asset?.ticker ?? '');
  const [name, setName] = useState(asset?.name ?? '');
  const [officialName, setOfficialName] = useState(asset?.official_name ?? '');
  // Conta onde o ativo fica (corretora, banco do CDB/LCI/LCA, conta no exterior).
  const custodyQ = useCustodyAccounts();
  const [custody, setCustody] = useState<string>(asset?.account_id ?? NO_ACCOUNT);
  const [market, setMarket] = useState(asset?.market ?? '');
  const [currency, setCurrency] = useState(asset?.currency ?? '');
  const [country, setCountry] = useState(asset?.country ?? '');
  const [isin, setIsin] = useState(asset?.isin ?? '');
  const [cnpj, setCnpj] = useState(asset?.cnpj ?? '');
  const [treasuryCode, setTreasuryCode] = useState(asset?.treasury_code ?? '');
  const [sector, setSector] = useState(asset?.sector ?? '');
  const [segment, setSegment] = useState(asset?.segment ?? '');
  const [provider, setProvider] = useState<string>(asset?.price_provider ?? 'auto');
  const [symbol, setSymbol] = useState(asset?.provider_symbol ?? '');
  const [notes, setNotes] = useState(asset?.notes ?? '');
  const [withTerms, setWithTerms] = useState(Boolean(asset?.fixed_income));
  const [terms, setTerms] = useState<Terms>(() =>
    termsFrom(asset?.fixed_income, asset?.class ?? 'stock_br'),
  );
  const [plan, setPlanDraft] = useState<PlanDraft>(() => planDraft(savedPlan));
  const [submitted, setSubmitted] = useState(false);
  const [picked, setPicked] = useState<TickerSuggestion | null>(null);
  const exchangesQ = useExchanges();
  const setT = <K extends keyof Terms>(k: K, v: Terms[K]) =>
    setTerms((prev) => ({ ...prev, [k]: v }));
  const editing = asset != null;
  const form = cls ? classForm(cls) : null;
  const listed = cls ? isExchangeListed(cls) : false;
  const exchange = (exchangesQ.data ?? []).find(
    (e) => e.code === market.trim().toUpperCase() || e.aliases.includes(market.trim()),
  );
  const ptaxMissing = exchange && !exchange.fx_automatic && currency.trim() === '';
  const pending = create.isPending || update.isPending || setPlan.isPending;
  const error = create.error ?? update.error ?? setPlan.error;

  const changeClass = (c: AssetClass) => {
    setCls(c);
    if (!asset?.fixed_income) setTerms(termsFrom(null, c));
    // A bolsa, a moeda e o país de outro tipo não valem para este.
    if (!editing && c !== cls) {
      setMarket('');
      setCurrency('');
      setCountry('');
    }
    if (!providerOptions(c, null).some((p) => p.value === provider)) setProvider('auto');
  };

  const fill = (sg: TickerSuggestion) => {
    setPicked(sg);
    if (sg.class) changeClass(sg.class);
    setTicker(sg.ticker.toUpperCase());
    setName(sg.name);
    setMarket(sg.exchange && sg.exchange !== 'CRYPTO' ? sg.exchange : '');
    setCurrency(sg.currency ?? '');
    setCountry(sg.country ?? '');
    if (sg.sector) setSector(sg.sector);
  };

  const showTerms = form?.terms === 'required' || (form?.terms === 'optional' && withTerms);
  const errors = {
    cls: !cls ? t('ativos-form-erro-tipo') : null,
    name: !name.trim() ? t('ativos-form-erro-nome') : null,
    currency:
      form?.currency && currency.trim() && !/^[A-Za-z]{3}$/.test(currency.trim())
        ? t('ativos-form-erro-moeda')
        : null,
    rate: showTerms && !terms.rate ? t('ativos-form-erro-taxa') : null,
    market: !editing && cls && needsExchange(cls) && !exchange ? t('ativos-form-erro-bolsa') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? e : null) ?? undefined;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid || !cls || !form) return;
    // Só o que o tipo usa vai para o núcleo (o que ficou de outro tipo, não).
    const opt = (v: string, used: unknown = true) => (used && v.trim() ? v.trim() : undefined);
    const fixed = showTerms ? toTerms(terms) : undefined;
    const success = (a: Asset) => {
      toast({
        tone: 'success',
        title: editing ? t('ativos-form-atualizado') : t('ativos-form-cadastrado'),
        description: a.ticker ?? a.name,
      });
      onDone(a);
    };
    if (asset) {
      const patch: AssetPatch = { id: asset.id };
      if (name.trim() !== asset.name) patch.name = name.trim();
      if (cls !== asset.class) patch.class = cls;
      const nullable = (v: string, cur: string | null) => {
        const next = v.trim() || null;
        return next === cur ? undefined : next;
      };
      const on = nullable(officialName, asset.official_name);
      if (on !== undefined) patch.official_name = on;
      const acc = custody === NO_ACCOUNT ? null : custody;
      if (acc !== asset.account_id) patch.account_id = acc;
      const tk = nullable(ticker.toUpperCase(), asset.ticker);
      if (tk !== undefined) patch.ticker = tk;
      const se = nullable(sector, asset.sector);
      if (se !== undefined) patch.sector = se;
      const sg = nullable(segment, asset.segment);
      if (sg !== undefined) patch.segment = sg;
      const pv = nullable(provider === 'auto' ? '' : provider, asset.price_provider);
      if (pv !== undefined) patch.price_provider = pv;
      const sy = nullable(symbol, asset.provider_symbol);
      if (sy !== undefined) patch.provider_symbol = sy;
      const no = nullable(notes, asset.notes);
      if (no !== undefined) patch.notes = no;
      if (fixed) patch.fixed_income = fixed;
      // CNPJ do banco emissor da renda fixa: editável (o score do emissor depende dele).
      if (cls === 'fixed_income') {
        const cn = nullable(cnpj, asset.cnpj);
        if (cn !== undefined) patch.cnpj = cn;
      }
      const planChanged =
        form.pension &&
        JSON.stringify(planTerms(plan)) !== JSON.stringify(planTerms(planDraft(savedPlan)));
      const savePlan = (a: Asset) =>
        planChanged
          ? setPlan.mutate({ asset_id: a.id, ...planTerms(plan) }, { onSuccess: () => success(a) })
          : success(a);
      if (Object.keys(patch).length > 1) update.mutate(patch, { onSuccess: savePlan });
      else savePlan(asset);
      return;
    }
    create.mutate(
      {
        name: name.trim(),
        ...(officialName.trim() ? { official_name: officialName.trim() } : {}),
        class: cls,
        ticker: opt(ticker.toUpperCase(), form.ticker),
        market: opt(market, form.exchange || form.market),
        currency: opt(currency.toUpperCase(), form.currency),
        country: opt(country.toUpperCase(), form.country),
        isin: opt(isin.toUpperCase(), form.isin),
        cnpj: opt(cnpj, form.cnpj),
        treasury_code: opt(treasuryCode, form.treasuryCode),
        sector: opt(sector, form.sector),
        segment: opt(segment, form.segment),
        price_provider: provider === 'auto' ? undefined : provider,
        provider_symbol: opt(symbol),
        notes: opt(notes),
        fixed_income: fixed,
        pension: form.pension ? planTerms(plan) : undefined,
        ...(custody !== NO_ACCOUNT ? { account_id: custody } : {}),
      },
      { onSuccess: success },
    );
  };

  const exchangeField = cls ? (
    editing ? (
      <Field label={t('ativos-form-bolsa')} hint={t('ativos-form-so-leitura')}>
        <Input value={market} disabled />
      </Field>
    ) : (
      <Field
        label={t('ativos-form-bolsa')}
        error={show(errors.market)}
        hint={
          exchange
            ? `${exchange.country} · ${exchange.currency} · ${exchange.open_time}–${exchange.close_time} (${exchange.timezone})`
            : needsExchange(cls)
              ? t('ativos-form-bolsa-obrigatoria')
              : t('ativos-form-padrao', {
                  padrao: defaultMarket(cls) ?? t('ativos-form-nenhuma'),
                })
        }
      >
        <Select
          value={exchange?.code ?? DEFAULT_MARKET}
          onValueChange={(v) => {
            setMarket(v === DEFAULT_MARKET ? '' : v);
            setCurrency('');
            setCountry('');
          }}
          options={[
            {
              value: DEFAULT_MARKET,
              label: needsExchange(cls)
                ? t('ativos-form-escolha-bolsa')
                : t('ativos-form-padrao-opcao', {
                    padrao: defaultMarket(cls) ?? t('ativos-form-nenhuma'),
                  }),
            },
            ...(exchangesQ.data ?? []).map((e) => ({
              value: e.code,
              label: `${e.code} · ${e.name}`,
              hint: `${e.country} ${e.currency}`,
            })),
          ]}
        />
      </Field>
    )
  ) : null;

  const currencyField = cls ? (
    <Field
      label={cls === 'crypto' ? t('ativos-form-moeda-cotacao') : t('ativos-col-moeda')}
      error={show(errors.currency)}
      hint={
        editing
          ? t('ativos-form-so-leitura')
          : ptaxMissing
            ? t('ativos-form-padrao-sem-ptax', { moeda: exchange.currency })
            : t('ativos-form-padrao', { padrao: defaultCurrency(cls, exchange?.currency) })
      }
    >
      <Input
        value={currency}
        onChange={(e) => setCurrency(e.target.value.toUpperCase())}
        placeholder={defaultCurrency(cls, exchange?.currency)}
        maxLength={3}
        className="num uppercase"
        disabled={editing}
      />
    </Field>
  ) : null;

  const countryField = (
    <Field
      label={t('ativos-form-pais')}
      hint={
        exchange && listed
          ? t('ativos-form-padrao', { padrao: exchange.country })
          : t('ativos-form-pais-dica')
      }
    >
      <Input
        value={country}
        onChange={(e) => setCountry(e.target.value.toUpperCase())}
        maxLength={2}
        className="num uppercase"
        disabled={editing}
      />
    </Field>
  );

  const isinField = (
    <Field label="ISIN" hint={t('ativos-opcional-ponto')}>
      <Input
        value={isin}
        onChange={(e) => setIsin(e.target.value)}
        className="num uppercase"
        disabled={editing}
      />
    </Field>
  );

  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-4"
      aria-label={t('ativos-form-rotulo')}
    >
      <TypePicker value={cls} onChange={changeClass} />
      {form ? (
        <p className="-mt-1 text-[12.5px] text-dim">{form.about}</p>
      ) : (
        <p role={submitted ? 'alert' : undefined} className="text-[12.5px] text-dim">
          {submitted && errors.cls ? errors.cls : t('ativos-form-ou-busque')}
        </p>
      )}

      {!editing && (!form || form.search) ? (
        <Field
          label={t('ativos-form-buscar')}
          hint={
            picked
              ? picked.asset_id
                ? t('ativos-form-preenchido-cadastrado', { ativo: describeSuggestion(picked) })
                : t('ativos-form-preenchido', { ativo: describeSuggestion(picked) })
              : t('ativos-form-buscar-dica')
          }
        >
          <TickerSearch onPick={fill} autoFocus />
        </Field>
      ) : null}

      {cls && form ? (
        <>
          <div className="grid gap-4 sm:grid-cols-3">
            {form.ticker ? (
              <Field label={form.ticker.label} hint={form.ticker.hint}>
                <Input
                  value={ticker}
                  onChange={(e) => setTicker(e.target.value.toUpperCase())}
                  autoCapitalize="characters"
                  placeholder={form.ticker.placeholder}
                  className="num uppercase"
                />
              </Field>
            ) : null}
            <Field
              label={form.name.label}
              error={show(errors.name)}
              className={form.ticker ? undefined : 'sm:col-span-2'}
            >
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={form.name.placeholder}
              />
            </Field>
            {/* O nome do registro (CVM, B3…) fica guardado; a tela mostra o nome acima. */}
            <Field
              label={t('ativos-nome-oficial')}
              hint={t('ativos-nome-oficial-dica')}
              className="sm:col-span-2"
            >
              <Input value={officialName} onChange={(e) => setOfficialName(e.target.value)} />
            </Field>
            <Field
              label={t('ativos-conta-custodia')}
              hint={t('ativos-conta-custodia-dica')}
              className="sm:col-span-2"
            >
              <Select
                aria-label={t('ativos-conta-custodia')}
                value={custody}
                onValueChange={setCustody}
                options={[
                  { value: NO_ACCOUNT, label: t('ativos-conta-custodia-nenhuma') },
                  ...(custodyQ.data ?? []).map((a) => ({
                    value: a.id,
                    label: a.name,
                    hint: ACCOUNT_KIND[a.kind],
                  })),
                ]}
              />
            </Field>
            {form.exchange ? exchangeField : null}
            {form.market ? (
              <Field
                label={t('ativos-col-mercado')}
                hint={editing ? t('ativos-form-so-leitura') : t('ativos-opcional-ponto')}
              >
                <Input
                  value={market}
                  onChange={(e) => setMarket(e.target.value)}
                  placeholder="NASDAQ, NYSE…"
                  disabled={editing}
                />
              </Field>
            ) : null}
            {form.currency === 'main' ? currencyField : null}
            {form.country === 'main' ? countryField : null}
            {form.cnpj ? (
              <Field label={form.cnpj.label} hint={form.cnpj.hint}>
                <Input
                  value={cnpj}
                  onChange={(e) => setCnpj(e.target.value)}
                  placeholder="00.000.000/0001-00"
                  className="num"
                  disabled={editing && cls !== 'fixed_income'}
                />
              </Field>
            ) : null}
            {form.treasuryCode ? (
              <Field
                label={t('ativos-form-titulo-tesouro')}
                hint={t('ativos-form-titulo-tesouro-dica')}
              >
                <Input
                  value={treasuryCode}
                  onChange={(e) => setTreasuryCode(e.target.value)}
                  disabled={editing}
                />
              </Field>
            ) : null}
            {form.sector ? (
              <Field label={form.sector.label}>
                <Input
                  value={sector}
                  onChange={(e) => setSector(e.target.value)}
                  placeholder={form.sector.placeholder}
                />
              </Field>
            ) : null}
            {form.segment ? (
              <Field label={form.segment.label}>
                <Input
                  value={segment}
                  onChange={(e) => setSegment(e.target.value)}
                  placeholder={form.segment.placeholder}
                />
              </Field>
            ) : null}
          </div>

          {form.terms === 'optional' ? (
            <Check
              checked={withTerms}
              onChange={setWithTerms}
              label={t('ativos-form-informar-termos')}
              hint={t('ativos-form-informar-termos-dica')}
            />
          ) : null}

          {showTerms ? (
            <fieldset className="flex flex-col gap-3 rounded-card border border-line p-3">
              <legend className="label-caps px-1 text-fg">{t('ativos-termos-titulo')}</legend>
              <div className="grid gap-4 sm:grid-cols-3">
                <Field label={t('ativos-termos-tipo')}>
                  <Select
                    value={terms.kind}
                    onValueChange={(v) =>
                      setTerms((prev) => ({
                        ...prev,
                        kind: v as FixedIncomeKind,
                        irExempt: defaultIrExempt(v as FixedIncomeKind),
                      }))
                    }
                    options={Object.entries(FIXED_INCOME_KIND_LABEL).map(([value, label]) => ({
                      value,
                      label,
                    }))}
                  />
                </Field>
                <Field label={t('ativos-termos-indexador')}>
                  <Select
                    value={terms.indexer}
                    onValueChange={(v) => setT('indexer', v as Indexer)}
                    options={Object.entries(INDEXER_LABEL).map(([value, label]) => ({
                      value,
                      label,
                    }))}
                  />
                </Field>
                <Field
                  label={t('ativos-termos-taxa')}
                  error={show(errors.rate)}
                  hint={rateHint(terms.indexer)}
                >
                  <DecimalInput
                    value={terms.rate}
                    onValueChange={(v) => setT('rate', v)}
                    maxDecimals={4}
                    placeholder={terms.indexer === 'cdi' ? '110' : '6,2'}
                  />
                </Field>
                <Field label={t('ativos-termos-emissao')}>
                  <DateInput value={terms.issueDate} onValueChange={(v) => setT('issueDate', v)} />
                </Field>
                <Field label={t('ativos-termos-vencimento')}>
                  <DateInput
                    value={terms.maturityDate}
                    onValueChange={(v) => setT('maturityDate', v)}
                  />
                </Field>
                <Field label={t('ativos-form-carencia-ate')} hint={t('ativos-opcional-ponto')}>
                  <DateInput
                    value={terms.graceUntil}
                    onValueChange={(v) => setT('graceUntil', v)}
                  />
                </Field>
                <Field label={t('ativos-termos-liquidez')}>
                  <Select
                    value={terms.liquidity}
                    onValueChange={(v) => setT('liquidity', v as Liquidity)}
                    options={Object.entries(LIQUIDITY_LABEL).map(([value, label]) => ({
                      value,
                      label,
                    }))}
                  />
                </Field>
                <Field label={t('ativos-termos-emissor')}>
                  <Input value={terms.issuer} onChange={(e) => setT('issuer', e.target.value)} />
                </Field>
                <div className="flex items-end pb-2">
                  <Check
                    checked={terms.irExempt}
                    onChange={(v) => setT('irExempt', v)}
                    label={t('ativos-form-isento-ir')}
                    hint={t('ativos-form-isento-ir-dica')}
                  />
                </div>
              </div>
            </fieldset>
          ) : null}

          {form.pension ? (
            <fieldset
              aria-label={t('ativos-form-plano')}
              className="@container flex flex-col gap-3 rounded-card border border-line p-3"
            >
              <legend className="label-caps px-1 text-fg">{t('ativos-form-plano')}</legend>
              <PensionPlanFields value={plan} onChange={setPlanDraft} />
              <p className="text-[12px] text-dim">{t('ativos-form-plano-dica')}</p>
            </fieldset>
          ) : null}

          <details className="group rounded-card border border-line px-3 py-2">
            <summary className="cursor-pointer text-[13px] text-fg">
              {t('ativos-form-cotacao-identificadores')}
              <span className="ml-2 text-[12px] text-dim">
                {providerOptions(cls, provider).find((p) => p.value === provider)?.label}
              </span>
            </summary>
            <div className="mt-3 grid gap-4 sm:grid-cols-3">
              <Field label={t('ativos-form-fonte')}>
                <Select
                  value={provider}
                  onValueChange={setProvider}
                  options={providerOptions(cls, asset?.price_provider).map((p) => ({
                    value: p.value,
                    label: p.label,
                  }))}
                />
              </Field>
              <Field
                label={t('ativos-form-simbolo')}
                hint={
                  asset?.quote_symbol && !symbol
                    ? t('ativos-form-simbolo-automatico', { simbolo: asset.quote_symbol })
                    : t('ativos-form-simbolo-dica')
                }
              >
                <Input
                  value={symbol}
                  onChange={(e) => setSymbol(e.target.value)}
                  className="num"
                  placeholder={
                    exchange && ticker
                      ? `${ticker}${exchange.yahoo_suffix}`
                      : (asset?.quote_symbol ?? undefined)
                  }
                />
              </Field>
              {form.isin ? isinField : null}
              {form.currency === 'advanced' ? currencyField : null}
              {form.country === 'advanced' ? countryField : null}
            </div>
          </details>

          <Field label={t('ativos-form-observacoes')}>
            <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
          </Field>
        </>
      ) : null}

      <FormError error={error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button type="submit" variant="primary" loading={pending}>
          {editing ? t('ativos-form-salvar') : t('ativos-cadastrar')}
        </Button>
      </div>
    </form>
  );
}
