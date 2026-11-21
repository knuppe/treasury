/**
 * Formulários de dívidas: cadastrar/editar contrato e registrar amortização
 * extraordinária. O núcleo valida e calcula o cronograma; aqui só a prévia da primeira
 * parcela (sem correção) e as validações de forma.
 */
import { useState, type FormEvent } from 'react';
import { useUndoToast } from '@/api/hooks/undoToast';

import { useAccounts } from '@/api/hooks/accounts';
import { useCountryFeatures } from '@/api/hooks/country';
import { useFgtsAccounts, useFgtsHousing } from '@/api/hooks/fgts';
import { useAddExtraPayment, useCreateLoan, useUpdateLoan, type LoanPatch } from '@/api/hooks/debt';
import { useInstitutions, useMembers } from '@/api/hooks/household';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import type { AmortSystem } from '@/api/generated/AmortSystem';
import type { LoanIndexer } from '@/api/generated/LoanIndexer';
import type { LoanInput } from '@/api/generated/LoanInput';
import type { LoanKind } from '@/api/generated/LoanKind';
import type { LoanView } from '@/api/generated/LoanView';
import type { PrepayMode } from '@/api/generated/PrepayMode';
import type { SpreadMode } from '@/api/generated/SpreadMode';
import type { CorrectionReference } from '@/api/generated/CorrectionReference';
import type { InterestBasis } from '@/api/generated/InterestBasis';
import type { Prorata } from '@/api/generated/Prorata';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Input,
  KeyValue,
  Money,
  MoneyInput,
  Select,
} from '@/design/components';
import { DecimalInput } from '@/features/portfolio/components/DecimalInput';
import { Segmented, Switch } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { currencyPrefix, formatDate, formatMoney, todayIso } from '@/lib/format';
import { rateString, solveLoanRate } from '@/lib/loanRate';
import { addMonthsToDate } from '@/lib/months';
import { useBaseCurrency } from '@/state/currency';
import { FormError } from '@/vault/FormError';

import {
  BASIS_LABEL,
  INDEXER_LABEL,
  PRORATA_LABEL,
  REFERENCE_LABEL,
  INDEXERS,
  KIND_LABEL,
  KINDS,
  MODE_HINT,
  MODE_LABEL,
  isPostIndexer,
  POST_INDEXERS,
  SPREAD_MODE_LABEL,
  SYSTEM_HINT,
  SYSTEM_HINT_POST,
  SYSTEM_LABEL,
} from './labels';
import { decimalString, decimalText, firstPaymentPreview, rateText } from './transform';

const NONE = '__none__';

/** Contas de onde sai o pagamento: corrente, poupança e dinheiro. */
function usePaymentAccounts() {
  const q = useAccounts();
  return (q.data ?? []).filter(
    (a) => !a.archived && (a.kind === 'checking' || a.kind === 'savings' || a.kind === 'cash'),
  );
}

/** Como a taxa entra: calculada pela parcela (padrão no cadastro) ou digitada. */
type RateMode = 'installment' | 'rate';
/** Pré-fixada (com ou sem correção do saldo) ou pós-fixada (juros pelo CDI/Selic). */
type RateKind = 'pre' | 'post';
type PostIndex = (typeof POST_INDEXERS)[number];
type InsuranceMode = 'rate' | 'amount';

/** Percentual (texto "0,0247") equivalente a um valor sobre uma base, ou o inverso. */
const pctOf = (cents: number, base: number) =>
  base > 0 ? decimalText(Math.round((cents / base) * 100 * 1e6) / 1e6, 6) : null;
const amountOf = (rate: string | null, base: number | null) =>
  rate && base ? Math.round((Number(rate.replace(',', '.')) / 100) * base) : null;

interface LoanForm {
  name: string;
  contractNumber: string;
  rateMode: RateMode;
  rateKind: RateKind;
  postIndex: PostIndex;
  indexPct: string | null;
  spreadMode: SpreadMode;
  lag: string;
  correctionLag: string;
  prorata: Prorata;
  floor: boolean;
  basis: InterestBasis;
  /** `null`: padrão do índice (aniversário na TR e na poupança). */
  reference: CorrectionReference | null;
  /** Contrato em andamento: marcar como pagas as parcelas já vencidas. */
  markPastPaid: boolean;
  installment: number | null;
  /** A data do contrato acompanha o 1º vencimento até a pessoa mexer nela. */
  contractTouched: boolean;
  kind: LoanKind;
  institutionId: string;
  memberId: string;
  principal: number | null;
  annualRate: string | null;
  system: AmortSystem;
  indexer: LoanIndexer;
  contractDate: string | null;
  firstDueDate: string | null;
  term: string;
  grace: string;
  insurance: number | null;
  propertyValue: number | null;
  mipRate: string | null;
  dfiRate: string | null;
  /** Seguro como no contrato: % a.m. ou valor fixo por mês (na moeda do contrato). */
  mipMode: InsuranceMode;
  mipAmount: number | null;
  dfiMode: InsuranceMode;
  dfiAmount: number | null;
  fees: number | null;
  cet: string | null;
  debitAccountId: string;
  categoryId: string | null;
  notes: string;
}

function initial(loan: LoanView | null | undefined): LoanForm {
  const today = todayIso();
  return {
    name: loan?.name ?? '',
    contractNumber: loan?.contract_number ?? '',
    rateMode: loan ? 'rate' : 'installment',
    rateKind: loan && isPostIndexer(loan.indexer) ? 'post' : 'pre',
    postIndex: loan?.indexer === 'selic' ? 'selic' : 'cdi',
    indexPct: loan?.index_pct ?? '100',
    spreadMode: loan?.spread_mode ?? 'exp252',
    lag: loan ? String(loan.index_lag_days) : '0',
    correctionLag: loan ? String(loan.correction_lag_months) : '1',
    prorata: loan?.correction_prorata ?? 'none',
    floor: loan?.deflation_floor ?? false,
    basis: loan?.interest_basis ?? 'monthly',
    reference: loan?.correction_reference ?? null,
    markPastPaid: !loan,
    installment: null,
    contractTouched: Boolean(loan),
    kind: loan?.kind ?? 'personal',
    institutionId: loan?.institution_id ?? NONE,
    memberId: loan?.member_id ?? NONE,
    principal: loan?.principal ?? null,
    annualRate: loan?.annual_rate ?? null,
    system: loan?.system ?? 'price',
    indexer: loan && !isPostIndexer(loan.indexer) ? loan.indexer : 'none',
    contractDate: loan?.contract_date ?? today,
    firstDueDate: loan?.first_due_date ?? addMonthsToDate(today, 1),
    term: loan ? String(loan.term_months) : '',
    grace: loan ? String(loan.grace_months) : '0',
    insurance: loan?.insurance_monthly ?? 0,
    propertyValue: loan?.property_value ?? null,
    mipRate: loan?.mip_rate ?? null,
    dfiRate: loan?.dfi_rate ?? null,
    mipMode: loan?.mip_amount != null ? 'amount' : 'rate',
    mipAmount: loan?.mip_amount ?? null,
    dfiMode: loan?.dfi_amount != null ? 'amount' : 'rate',
    dfiAmount: loan?.dfi_amount ?? null,
    fees: loan?.fees_monthly ?? 0,
    cet: loan?.cet_annual ?? null,
    debitAccountId: loan?.debit_account_id ?? NONE,
    categoryId: loan?.category_id ?? null,
    notes: loan?.notes ?? '',
  };
}

/** TR e poupança corrigem pelo período que começa no aniversário; os demais, pelo mês. */
const defaultReference = (i: LoanIndexer): CorrectionReference =>
  i === 'tr' || i === 'poupanca' ? 'anniversary' : 'month';

/** Taxa nominal anual (mensal = anual ÷ 12) → efetiva anual, e o inverso (texto decimal). */
const effectiveFromNominal = (n: string) =>
  String(Math.round(((1 + Number(n) / 1200) ** 12 - 1) * 100 * 1e6) / 1e6);
const nominalFromEffective = (e: string) =>
  String(Math.round(((1 + Number(e) / 100) ** (1 / 12) - 1) * 1200 * 1e6) / 1e6);

const intOf = (s: string) => (/^\d+$/.test(s.trim()) ? Number(s.trim()) : null);

export function LoanFormDialog({
  open,
  onOpenChange,
  loan,
  onSaved,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Editar este contrato (todos os campos; as parcelas em aberto são recalculadas). */
  loan?: LoanView | null;
  onSaved?: (loan: LoanView) => void;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      index="05"
      title={loan ? t('dividas-form-editar') : t('dividas-novo-contrato')}
      description={loan ? t('dividas-form-editar-descricao') : t('dividas-form-novo-descricao')}
    >
      {open ? (
        <LoanFormBody
          loan={loan}
          onDone={(v) => {
            // Quem passa `onSaved` fecha o diálogo (as duas mudanças de URL juntas).
            if (onSaved) onSaved(v);
            else onOpenChange(false);
          }}
        />
      ) : null}
    </Dialog>
  );
}

function LoanFormBody({
  loan,
  onDone,
}: {
  loan?: LoanView | null;
  onDone: (loan: LoanView) => void;
}) {
  const editing = Boolean(loan);
  // Índices, consignado e base de 252 dias são do Brasil.
  const br = useCountryFeatures().br_credit_rates;
  // Valores do formulário na moeda base da família (como os campos de valor).
  const symbol = currencyPrefix(useBaseCurrency());
  const [f, setF] = useState<LoanForm>(() => initial(loan));
  const [submitted, setSubmitted] = useState(false);
  const set = <K extends keyof LoanForm>(k: K, v: LoanForm[K]) => setF((s) => ({ ...s, [k]: v }));
  const accounts = usePaymentAccounts();
  const members = useMembers().data ?? [];
  const institutions = useInstitutions().data ?? [];
  const create = useCreateLoan();
  const update = useUpdateLoan();
  const undoToast = useUndoToast();
  const mutation = editing ? update : create;

  const [nominal, setNominal] = useState(false);
  const [nominalRate, setNominalRate] = useState<string | null>(null);
  const term = intOf(f.term);
  // Base do MIP para converter valor ↔ percentual: o saldo de hoje (edição) ou o financiado.
  const mipBase = loan ? loan.balance || f.principal : f.principal;
  const grace = intOf(f.grace || '0');
  const charges = (f.insurance ?? 0) + (f.fees ?? 0);
  const post = f.rateKind === 'post';
  const byInstallment = !post && f.rateMode === 'installment';
  const solved =
    byInstallment && f.principal && f.installment && term
      ? solveLoanRate({
          principal: f.principal,
          installment: f.installment,
          term,
          grace: grace ?? 0,
          system: f.system,
          charges,
        })
      : null;
  const rate = post
    ? Number(f.annualRate ?? '0')
    : f.rateMode === 'installment'
      ? (solved?.annualPct ?? null)
      : f.annualRate != null
        ? Number(f.annualRate)
        : null;
  const errors = {
    name: !f.name.trim() ? t('dividas-form-erro-nome') : null,
    principal: !(f.principal != null && f.principal > 0) ? t('dividas-form-erro-principal') : null,
    installment: !byInstallment
      ? null
      : !(f.installment != null && f.installment > 0)
        ? t('dividas-form-erro-parcela')
        : f.principal && term && !solved
          ? t('dividas-form-erro-parcela-nao-fecha')
          : null,
    rate:
      post || f.rateMode !== 'rate'
        ? null
        : rate == null
          ? t('dividas-form-erro-taxa')
          : rate > 1000
            ? t('dividas-form-erro-taxa-intervalo')
            : null,
    contract: !f.contractDate ? t('dividas-form-erro-data-contrato') : null,
    firstDue: !f.firstDueDate
      ? t('dividas-form-erro-primeiro-vencimento')
      : f.contractDate && f.firstDueDate < f.contractDate
        ? t('dividas-form-erro-vencimento-antes')
        : null,
    term: !(term != null && term > 0) ? t('dividas-form-erro-prazo') : null,
    grace:
      grace == null
        ? t('dividas-form-erro-carencia')
        : term != null && grace >= term
          ? t('dividas-form-erro-carencia-prazo')
          : null,
    indexPct:
      post && !(Number(f.indexPct) > 0 && Number(f.indexPct) <= 1000)
        ? t('dividas-form-erro-percentual')
        : null,
    lag:
      post && !(intOf(f.lag || '0') != null && intOf(f.lag || '0')! <= 10)
        ? t('dividas-form-erro-defasagem')
        : null,
    bulletGrace: f.system === 'bullet' && (grace ?? 0) > 0 ? t('dividas-form-erro-bullet') : null,
    insurance: f.insurance == null ? t('dividas-form-erro-valor') : null,
    fees: f.fees == null ? t('dividas-form-erro-valor') : null,
  };
  const relevant = Object.values(errors);
  const invalid = relevant.some(Boolean);
  const show = (e: string | null) => (submitted ? (e ?? undefined) : undefined);
  const preview =
    !post && f.principal && rate != null && term
      ? firstPaymentPreview({
          principal: f.principal,
          annualRatePct: rate,
          system: f.system,
          term,
          grace: grace ?? 0,
          charges,
        })
      : null;

  // Parcelas do cronograma que já venceram (contrato cadastrado com atraso).
  const today = todayIso();
  let pastDue = 0;
  if (f.firstDueDate && term) {
    while (pastDue < term && addMonthsToDate(f.firstDueDate, pastDue) < today) pastDue += 1;
  }
  // Na edição, só conta o que ainda está em aberto (as pagas ficam como estão).
  if (loan) pastDue = Math.max(0, pastDue - loan.paid_count);

  const setFirstDue = (v: string | null) =>
    setF((s) => ({
      ...s,
      firstDueDate: v,
      // Contrato antigo: sem data do contrato informada, assume um mês antes do 1º vencimento.
      contractDate: !s.contractTouched && v ? addMonthsToDate(v, -1) : s.contractDate,
    }));

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid) return;
    const opt = (v: string) => (v === NONE ? undefined : v);
    const input: LoanInput = {
      name: f.name.trim(),
      ...(f.contractNumber.trim() ? { contract_number: f.contractNumber.trim() } : {}),
      ...(pastDue > 0 && f.markPastPaid ? { mark_past_paid: true } : {}),
      kind: f.kind,
      principal: f.principal!,
      // Pela parcela, o núcleo recalcula a taxa no mesmo modelo do cronograma. Nos pós, a
      // taxa é o spread sobre o índice.
      ...(post
        ? {
            annual_rate: f.annualRate ?? '0',
            index_pct: f.indexPct ?? '100',
            spread_mode: f.spreadMode,
            index_lag_days: intOf(f.lag || '0') ?? 0,
          }
        : {
            ...(f.rateMode === 'installment'
              ? { annual_rate: '', installment_amount: f.installment! }
              : { annual_rate: f.annualRate! }),
            ...(f.indexer !== 'none'
              ? {
                  correction_lag_months: Number(f.correctionLag),
                  deflation_floor: f.floor,
                  ...(f.reference ? { correction_reference: f.reference } : {}),
                }
              : {}),
            correction_prorata: f.prorata,
            interest_basis: f.basis,
          }),
      system: f.system,
      indexer: post ? f.postIndex : f.indexer,
      contract_date: f.contractDate!,
      first_due_date: f.firstDueDate!,
      term_months: term!,
      ...(grace ? { grace_months: grace } : {}),
      ...(f.insurance ? { insurance_monthly: f.insurance } : {}),
      ...(f.propertyValue ? { property_value: f.propertyValue } : {}),
      ...(f.mipMode === 'amount'
        ? f.mipAmount
          ? { mip_amount: f.mipAmount }
          : {}
        : f.mipRate
          ? { mip_rate: f.mipRate }
          : {}),
      ...(f.dfiMode === 'amount'
        ? f.dfiAmount
          ? { dfi_amount: f.dfiAmount }
          : {}
        : f.dfiRate
          ? { dfi_rate: f.dfiRate }
          : {}),
      ...(f.fees ? { fees_monthly: f.fees } : {}),
      ...(f.cet ? { cet_annual: f.cet } : {}),
      ...(opt(f.institutionId) ? { institution_id: opt(f.institutionId) } : {}),
      ...(opt(f.memberId) ? { member_id: opt(f.memberId) } : {}),
      ...(opt(f.debitAccountId) ? { debit_account_id: opt(f.debitAccountId) } : {}),
      ...(f.categoryId ? { category_id: f.categoryId } : {}),
      ...(f.notes.trim() ? { notes: f.notes.trim() } : {}),
    };
    if (loan) {
      // Todos os termos: o núcleo recalcula as parcelas em aberto e mantém as pagas, as
      // amortizações extraordinárias e as antecipações.
      const patch: LoanPatch = {
        id: loan.id,
        terms: input,
      };
      update.mutate(patch, {
        onSuccess: (v) => {
          undoToast({ tone: 'success', title: t('dividas-form-alterado'), description: v.name });
          onDone(v);
        },
      });
      return;
    }
    create.mutate(input, {
      onSuccess: (v) => {
        undoToast({
          tone: 'success',
          title: t('dividas-form-cadastrado'),
          description: t('dividas-form-cadastrado-descricao', {
            nome: v.name,
            n: v.remaining_count,
          }),
        });
        onDone(v);
      },
    });
  };

  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-5"
      aria-label={t('dividas-contrato')}
    >
      <fieldset className="flex flex-col gap-4">
        <legend className="label-caps mb-3 text-fg">{t('dividas-form-dados')}</legend>
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label={t('dividas-form-nome')} error={show(errors.name)}>
            <Input
              value={f.name}
              onChange={(e) => set('name', e.target.value)}
              placeholder={t('dividas-form-nome-exemplo')}
            />
          </Field>
          <Field label={t('dividas-form-numero')} hint={t('dividas-form-numero-dica')}>
            <Input
              value={f.contractNumber}
              onChange={(e) => set('contractNumber', e.target.value)}
              placeholder="0001234-56"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              data-selectable
            />
          </Field>
          <Field label={t('dividas-form-principal')} error={show(errors.principal)}>
            <MoneyInput value={f.principal} onValueChange={(v) => set('principal', v)} />
          </Field>
          <Field label={t('dividas-prazo-meses')} error={show(errors.term)}>
            <Input
              inputMode="numeric"
              numeric
              value={f.term}
              onChange={(e) => set('term', e.target.value.replace(/\D/g, ''))}
              placeholder="48"
            />
          </Field>
          <Field label={t('dividas-form-primeiro-vencimento')} error={show(errors.firstDue)}>
            <DateInput value={f.firstDueDate} onValueChange={setFirstDue} />
          </Field>
          <Field
            label={t('dividas-form-sistema')}
            hint={(post ? SYSTEM_HINT_POST : SYSTEM_HINT)[f.system]}
            error={show(errors.bulletGrace)}
          >
            <Segmented<AmortSystem>
              label={t('dividas-form-sistema')}
              fill
              value={f.system}
              onChange={(v) => set('system', v)}
              options={(['price', 'sac', 'bullet'] as const).map((s) => ({
                value: s,
                label: SYSTEM_LABEL[s],
              }))}
            />
          </Field>
          {/* Pós-fixado é CDI ou Selic: só no Brasil (docs/international.md). */}
          {br || post ? (
            <Field label={t('dividas-form-tipo-taxa')} className="sm:col-span-2">
              <Segmented<RateKind>
                label={t('dividas-form-tipo-taxa')}
                fill
                value={f.rateKind}
                onChange={(v) => set('rateKind', v)}
                options={[
                  { value: 'pre', label: t('dividas-form-pre') },
                  { value: 'post', label: t('dividas-form-pos') },
                ]}
              />
            </Field>
          ) : null}
          {post ? (
            <>
              <Field label={t('dividas-form-indice')} hint={t('dividas-form-indice-dica')}>
                <Select
                  value={f.postIndex}
                  onValueChange={(v) => v && set('postIndex', v as PostIndex)}
                  options={POST_INDEXERS.map((k) => ({ value: k, label: INDEXER_LABEL[k] }))}
                />
              </Field>
              <Field
                label={t('dividas-form-percentual')}
                error={show(errors.indexPct)}
                hint={t('dividas-form-percentual-dica')}
              >
                <DecimalInput
                  value={f.indexPct}
                  onValueChange={(v) => set('indexPct', v)}
                  maxDecimals={4}
                  placeholder="100"
                  suffix="%"
                />
              </Field>
              <Field
                label={t('dividas-form-spread')}
                hint={t('dividas-form-spread-dica', { indice: INDEXER_LABEL[f.postIndex] })}
              >
                <DecimalInput
                  value={f.annualRate}
                  onValueChange={(v) => set('annualRate', v)}
                  maxDecimals={6}
                  placeholder="0"
                  suffix="%"
                />
              </Field>
            </>
          ) : (
            <>
              <Field label={t('dividas-form-taxa-juros')} className="sm:col-span-2">
                <Segmented<RateMode>
                  label={t('dividas-form-taxa-juros')}
                  fill
                  value={f.rateMode}
                  onChange={(v) => set('rateMode', v)}
                  options={[
                    { value: 'installment', label: t('dividas-form-pela-parcela') },
                    { value: 'rate', label: t('dividas-form-informar-taxa') },
                  ]}
                />
              </Field>
              {f.rateMode === 'installment' ? (
                <>
                  <Field
                    label={
                      f.system === 'sac'
                        ? t('dividas-form-primeira-parcela')
                        : f.system === 'bullet'
                          ? t('dividas-form-parcela-juros')
                          : t('dividas-form-valor-parcela')
                    }
                    error={show(errors.installment)}
                    hint={
                      charges > 0
                        ? t('dividas-form-parcela-dica-encargos')
                        : t('dividas-form-parcela-dica')
                    }
                  >
                    <MoneyInput
                      value={f.installment}
                      onValueChange={(v) => set('installment', v)}
                    />
                  </Field>
                  <Field label={t('dividas-form-taxa-calculada')}>
                    <p
                      className="num flex h-9 items-center text-[14px] text-fg"
                      aria-live="polite"
                      data-testid="taxa-calculada"
                    >
                      {solved
                        ? `${t('dividas-taxa-aa', { taxa: rateText(rateString(solved.annualPct)) })} · ${t('dividas-taxa-am', { taxa: rateText(rateString(solved.monthlyPct), 4) })}`
                        : '—'}
                    </p>
                  </Field>
                </>
              ) : (
                <Field
                  label={nominal ? t('dividas-form-taxa-nominal') : t('dividas-form-taxa-efetiva')}
                  error={show(errors.rate)}
                  aside={
                    <Segmented<'effective' | 'nominal'>
                      label={t('dividas-form-como-taxa')}
                      value={nominal ? 'nominal' : 'effective'}
                      onChange={(v) => {
                        setNominal(v === 'nominal');
                        setNominalRate(
                          v === 'nominal' && f.annualRate
                            ? nominalFromEffective(f.annualRate)
                            : null,
                        );
                      }}
                      options={[
                        { value: 'effective', label: t('dividas-form-efetiva') },
                        { value: 'nominal', label: t('dividas-form-nominal') },
                      ]}
                    />
                  }
                  hint={
                    nominal
                      ? f.annualRate
                        ? t('dividas-form-nominal-dica-equivale', {
                            taxa: decimalString(f.annualRate),
                          })
                        : t('dividas-form-nominal-dica')
                      : t('dividas-form-efetiva-dica')
                  }
                >
                  <DecimalInput
                    value={nominal ? nominalRate : f.annualRate}
                    onValueChange={(v) => {
                      if (!nominal) return set('annualRate', v);
                      setNominalRate(v);
                      set('annualRate', v ? effectiveFromNominal(v) : null);
                    }}
                    maxDecimals={6}
                    placeholder={decimalText(12.5, 1)}
                    suffix="%"
                  />
                </Field>
              )}
            </>
          )}
        </div>
      </fieldset>

      <fieldset className="flex flex-col gap-4 border-t border-line pt-4">
        <legend className="label-caps mb-3 pt-4 text-fg">{t('dividas-form-detalhes')}</legend>
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label={t('dividas-form-tipo')}>
            <Select
              value={f.kind}
              onValueChange={(v) => v && set('kind', v as LoanKind)}
              options={KINDS.filter((k) => br || k !== 'payroll' || f.kind === k).map((k) => ({
                value: k,
                label: KIND_LABEL[k],
              }))}
            />
          </Field>
          <Field label={t('dividas-form-instituicao')} hint={t('dividas-opcional')}>
            <Select
              value={f.institutionId}
              onValueChange={(v) => v && set('institutionId', v)}
              options={[
                { value: NONE, label: t('dividas-form-nao-informar') },
                ...institutions.map((i) => ({ value: i.id, label: i.name })),
              ]}
            />
          </Field>
          <Field label={t('dividas-form-membro')} hint={t('dividas-opcional')}>
            <Select
              value={f.memberId}
              onValueChange={(v) => v && set('memberId', v)}
              options={[
                { value: NONE, label: t('dividas-form-familia') },
                ...members.map((m) => ({ value: m.id, label: m.name })),
              ]}
            />
          </Field>
          <Field
            label={t('dividas-form-data-contrato')}
            error={show(errors.contract)}
            hint={!f.contractTouched ? t('dividas-form-data-contrato-dica') : undefined}
          >
            <DateInput
              value={f.contractDate}
              onValueChange={(v) => setF((s) => ({ ...s, contractDate: v, contractTouched: true }))}
            />
          </Field>
          {/* Índices de correção (TR, IPCA, poupança…) e base de 252 dias: só no Brasil. */}
          {br || post || f.indexer !== 'none' ? (
            <>
              {post ? (
                <>
                  <Field
                    label={t('dividas-form-forma-spread')}
                    hint={t('dividas-form-forma-spread-dica')}
                  >
                    <Select
                      value={f.spreadMode}
                      onValueChange={(v) => v && set('spreadMode', v as SpreadMode)}
                      options={(['exp252', 'lin360'] as const).map((k) => ({
                        value: k,
                        label: SPREAD_MODE_LABEL[k],
                      }))}
                    />
                  </Field>
                  <Field
                    label={t('dividas-form-defasagem')}
                    error={show(errors.lag)}
                    hint={t('dividas-form-defasagem-dica')}
                  >
                    <Input
                      inputMode="numeric"
                      numeric
                      value={f.lag}
                      onChange={(e) => set('lag', e.target.value.replace(/\D/g, ''))}
                    />
                  </Field>
                </>
              ) : (
                <>
                  <Field label={t('dividas-correcao-saldo')} hint={t('dividas-form-correcao-dica')}>
                    <Select
                      value={f.indexer}
                      onValueChange={(v) => v && set('indexer', v as LoanIndexer)}
                      options={INDEXERS.map((k) => ({ value: k, label: INDEXER_LABEL[k] }))}
                    />
                  </Field>
                  <Field label={t('dividas-juros')} hint={t('dividas-form-juros-dica')}>
                    <Select
                      value={f.basis}
                      onValueChange={(v) => v && set('basis', v as InterestBasis)}
                      options={(['monthly', 'bd252', 'dc365'] as const).map((k) => ({
                        value: k,
                        label: BASIS_LABEL[k],
                      }))}
                    />
                  </Field>
                  {f.indexer !== 'none' ? (
                    <>
                      <Field
                        label={t('dividas-form-indice-usado')}
                        hint={
                          (f.reference ?? defaultReference(f.indexer)) === 'anniversary'
                            ? t('dividas-form-indice-aniversario-dica')
                            : t('dividas-form-indice-mes-dica')
                        }
                      >
                        <Select
                          value={f.reference ?? defaultReference(f.indexer)}
                          onValueChange={(v) => v && set('reference', v as CorrectionReference)}
                          options={(['month', 'anniversary'] as const).map((k) => ({
                            value: k,
                            label: REFERENCE_LABEL[k],
                          }))}
                        />
                      </Field>
                      {(f.reference ?? defaultReference(f.indexer)) === 'month' ? (
                        <Field
                          label={t('dividas-form-mes-indice')}
                          hint={t('dividas-form-mes-indice-dica')}
                        >
                          <Select
                            value={f.correctionLag}
                            onValueChange={(v) => v && set('correctionLag', v)}
                            options={['0', '1', '2', '3'].map((n) => ({
                              value: n,
                              label: n === '0' ? t('dividas-form-mesmo-mes') : `M − ${n}`,
                            }))}
                          />
                        </Field>
                      ) : null}
                      <Switch
                        checked={f.floor}
                        onChange={(v) => set('floor', v)}
                        label={t('dividas-form-deflacao')}
                        description={t('dividas-form-deflacao-dica')}
                      />
                    </>
                  ) : null}
                </>
              )}
            </>
          ) : null}
          <Field label={t('dividas-form-prorata')} hint={t('dividas-form-prorata-dica')}>
            <Select
              value={f.prorata}
              disabled={post}
              onValueChange={(v) => v && set('prorata', v as Prorata)}
              options={(['none', 'calendar', 'business'] as const).map((k) => ({
                value: k,
                label: post ? t('dividas-form-prorata-pos') : PRORATA_LABEL[k],
              }))}
            />
          </Field>
          <Field
            label={t('dividas-form-carencia')}
            error={show(errors.grace)}
            hint={t('dividas-form-carencia-dica')}
          >
            <Input
              inputMode="numeric"
              numeric
              value={f.grace}
              onChange={(e) => set('grace', e.target.value.replace(/\D/g, ''))}
            />
          </Field>
          {f.kind === 'mortgage' ? (
            <>
              <Field
                label={t('dividas-form-valor-imovel')}
                hint={t('dividas-form-valor-imovel-dica')}
              >
                <MoneyInput
                  value={f.propertyValue}
                  onValueChange={(v) => set('propertyValue', v)}
                />
              </Field>
              <Field
                label={
                  f.mipMode === 'rate'
                    ? t('dividas-form-mip-taxa')
                    : t('dividas-form-mip-valor', { moeda: symbol })
                }
                aside={
                  <Segmented<InsuranceMode>
                    label={t('dividas-form-como-mip')}
                    value={f.mipMode}
                    onChange={(v) => set('mipMode', v)}
                    options={[
                      { value: 'rate', label: '%' },
                      { value: 'amount', label: symbol },
                    ]}
                  />
                }
                hint={
                  f.mipMode === 'rate'
                    ? amountOf(f.mipRate, mipBase)
                      ? t('dividas-form-mip-taxa-dica-hoje', {
                          valor: formatMoney(amountOf(f.mipRate, mipBase)!),
                        })
                      : t('dividas-form-mip-taxa-dica')
                    : f.mipAmount && pctOf(f.mipAmount, mipBase ?? 0)
                      ? t('dividas-form-mip-valor-dica-equivale', {
                          pct: pctOf(f.mipAmount, mipBase!)!,
                          saldo: formatMoney(mipBase!),
                        })
                      : t('dividas-form-mip-valor-dica')
                }
              >
                {f.mipMode === 'rate' ? (
                  <DecimalInput
                    value={f.mipRate}
                    onValueChange={(v) => set('mipRate', v)}
                    maxDecimals={6}
                    suffix="%"
                  />
                ) : (
                  <MoneyInput value={f.mipAmount} onValueChange={(v) => set('mipAmount', v)} />
                )}
              </Field>
              <Field
                label={
                  f.dfiMode === 'rate'
                    ? t('dividas-form-dfi-taxa')
                    : t('dividas-form-dfi-valor', { moeda: symbol })
                }
                aside={
                  <Segmented<InsuranceMode>
                    label={t('dividas-form-como-dfi')}
                    value={f.dfiMode}
                    onChange={(v) => set('dfiMode', v)}
                    options={[
                      { value: 'rate', label: '%' },
                      { value: 'amount', label: symbol },
                    ]}
                  />
                }
                hint={
                  f.dfiMode === 'rate'
                    ? amountOf(f.dfiRate, f.propertyValue)
                      ? t('dividas-form-dfi-taxa-dica-mes', {
                          valor: formatMoney(amountOf(f.dfiRate, f.propertyValue)!),
                        })
                      : t('dividas-form-dfi-taxa-dica')
                    : f.dfiAmount && f.propertyValue && pctOf(f.dfiAmount, f.propertyValue)
                      ? t('dividas-form-dfi-valor-dica-equivale', {
                          pct: pctOf(f.dfiAmount, f.propertyValue)!,
                        })
                      : t('dividas-form-dfi-valor-dica')
                }
              >
                {f.dfiMode === 'rate' ? (
                  <DecimalInput
                    value={f.dfiRate}
                    onValueChange={(v) => set('dfiRate', v)}
                    maxDecimals={6}
                    suffix="%"
                  />
                ) : (
                  <MoneyInput value={f.dfiAmount} onValueChange={(v) => set('dfiAmount', v)} />
                )}
              </Field>
            </>
          ) : null}
          <Field
            label={
              f.kind === 'mortgage' ? t('dividas-form-outros-seguros') : t('dividas-form-seguros')
            }
            error={show(errors.insurance)}
            hint={
              f.kind === 'mortgage'
                ? t('dividas-form-outros-seguros-dica')
                : t('dividas-form-seguros-dica')
            }
          >
            <MoneyInput value={f.insurance} onValueChange={(v) => set('insurance', v)} />
          </Field>
          <Field
            label={t('dividas-form-taxa-adm')}
            error={show(errors.fees)}
            hint={t('dividas-form-taxa-adm-dica')}
          >
            <MoneyInput value={f.fees} onValueChange={(v) => set('fees', v)} />
          </Field>
          <Field label={t('dividas-form-cet')} hint={t('dividas-form-cet-dica')}>
            <DecimalInput
              value={f.cet}
              onValueChange={(v) => set('cet', v)}
              maxDecimals={6}
              suffix="%"
            />
          </Field>
          <Field label={t('dividas-conta-de-debito')} hint={t('dividas-form-conta-dica')}>
            <Select
              value={f.debitAccountId}
              onValueChange={(v) => v && set('debitAccountId', v)}
              options={[
                { value: NONE, label: t('dividas-nenhuma') },
                ...accounts.map((a) => ({ value: a.id, label: a.name, hint: a.currency })),
              ]}
            />
          </Field>
          <Field label={t('dividas-categoria-pagamentos')} hint={t('dividas-form-categoria-dica')}>
            <CategoryPicker
              label={t('dividas-categoria-pagamentos')}
              kind="expense"
              value={f.categoryId}
              onChange={(v) => set('categoryId', v)}
              noneLabel={t('dividas-nenhuma')}
            />
          </Field>
          <Field
            label={t('dividas-form-observacoes')}
            hint={t('dividas-opcional')}
            className="sm:col-span-2"
          >
            <Input value={f.notes} onChange={(e) => set('notes', e.target.value)} />
          </Field>
        </div>
      </fieldset>

      {pastDue > 0 ? (
        <div className="rounded-control border border-warning/50 bg-warning/5 px-3 py-2.5">
          <Switch
            checked={f.markPastPaid}
            onChange={(v) => set('markPastPaid', v)}
            label={t('dividas-form-ja-paguei', { n: pastDue })}
            description={t('dividas-form-ja-paguei-dica')}
          />
        </div>
      ) : null}
      {post ? (
        <p className="rounded-control border border-line bg-bg px-3 py-2.5 text-[12.5px] text-dim">
          {t('dividas-form-pos-nota', { indice: INDEXER_LABEL[f.postIndex] })}
        </p>
      ) : null}
      {preview ? (
        <div className="rounded-control border border-line bg-bg px-3 py-2.5" aria-live="polite">
          <KeyValue
            items={[
              ...(preview.grace != null
                ? [
                    {
                      label: t('dividas-form-na-carencia'),
                      value: <Money cents={preview.grace} estimated />,
                    },
                  ]
                : []),
              {
                label:
                  f.system === 'sac' ? t('dividas-form-primeira-parcela') : t('dividas-parcela'),
                value: <Money cents={preview.first} estimated />,
              },
              ...(f.system === 'sac'
                ? [
                    {
                      label: t('dividas-form-ultima-parcela'),
                      value: <Money cents={preview.last} estimated />,
                    },
                  ]
                : []),
              ...(f.indexer !== 'none'
                ? [
                    {
                      label: t('dividas-correcao'),
                      value: t('dividas-form-correcao-valor', { indice: INDEXER_LABEL[f.indexer] }),
                    },
                  ]
                : []),
            ]}
          />
        </div>
      ) : null}

      <FormError error={mutation.error} />

      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button
          type="submit"
          variant="primary"
          loading={mutation.isPending}
          disabled={submitted && invalid}
        >
          {editing ? t('dividas-form-salvar') : t('dividas-form-cadastrar')}
        </Button>
      </div>
    </form>
  );
}

// ── Amortização extraordinária ───────────────────────────────────

export interface ExtraPreset {
  amount?: number;
  date?: string;
  mode?: PrepayMode;
}

export function ExtraPaymentDialog({
  open,
  onOpenChange,
  loan,
  preset,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  loan: LoanView;
  preset?: ExtraPreset | null;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="md"
      index="05"
      title={t('dividas-amortizacao-extraordinaria')}
      description={t('dividas-extra-descricao', { nome: loan.name })}
    >
      {open ? <ExtraForm loan={loan} preset={preset} onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}

function ExtraForm({
  loan,
  preset,
  onDone,
}: {
  loan: LoanView;
  preset?: ExtraPreset | null;
  onDone: () => void;
}) {
  const [amount, setAmount] = useState<number | null>(preset?.amount ?? null);
  const [date, setDate] = useState<string | null>(preset?.date ?? todayIso());
  const [mode, setMode] = useState<PrepayMode>(preset?.mode ?? 'reduce_term');
  const [accountId, setAccountId] = useState<string>(loan.debit_account_id ?? NONE);
  const [submitted, setSubmitted] = useState(false);
  const accounts = usePaymentAccounts();
  const add = useAddExtraPayment();
  const undoToast = useUndoToast();
  // No financiamento imobiliário (Brasil), o FGTS pode amortizar: a saída fica na conta FGTS.
  const features = useCountryFeatures();
  const housing = useFgtsHousing();
  const fgts = (useFgtsAccounts(features.fgts && loan.kind === 'mortgage').data ?? []).filter(
    (a) => !a.archived && a.balance > 0,
  );
  const fromFgts = fgts.some((a) => a.account_id === accountId);
  const errors = {
    amount: !(amount != null && amount > 0) ? t('dividas-extra-erro-valor') : null,
    date: !date ? t('dividas-extra-erro-data') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid) return;
    if (fromFgts) {
      housing.mutate(
        { account_id: accountId, loan_id: loan.id, date: date!, amount: amount!, mode },
        {
          onSuccess: (r) => {
            undoToast({
              tone: r.warnings.length ? 'warning' : 'success',
              title: t('dividas-extra-registrada'),
              description: r.warnings.join(' ') || t('dividas-extra-fgts-descricao'),
            });
            onDone();
          },
        },
      );
      return;
    }
    add.mutate(
      {
        loan_id: loan.id,
        date: date!,
        amount: amount!,
        mode,
        ...(accountId !== NONE ? { from_account_id: accountId } : { without_txn: true }),
      },
      {
        onSuccess: (v) => {
          undoToast({
            tone: 'success',
            title: t('dividas-extra-registrada'),
            description: t('dividas-extra-registrada-descricao', { nome: v.name }),
          });
          onDone();
        },
      },
    );
  };
  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-4"
      aria-label={t('dividas-amortizacao')}
    >
      <div className="grid gap-4 sm:grid-cols-2">
        <Field
          label={t('dividas-valor')}
          error={submitted ? (errors.amount ?? undefined) : undefined}
        >
          <MoneyInput value={amount} onValueChange={setAmount} />
        </Field>
        <Field
          label={t('dividas-data-pagamento')}
          error={submitted ? (errors.date ?? undefined) : undefined}
        >
          <DateInput value={date} onValueChange={setDate} />
        </Field>
      </div>
      <Field label={t('dividas-extra-depois')} hint={MODE_HINT[mode]}>
        <Segmented<PrepayMode>
          label={t('dividas-extra-depois')}
          value={mode}
          onChange={setMode}
          options={(['reduce_term', 'reduce_installment'] as const).map((m) => ({
            value: m,
            label: MODE_LABEL[m],
          }))}
        />
      </Field>
      <Field
        label={t('dividas-extra-sai-da-conta')}
        hint={
          accountId === NONE ? t('dividas-extra-sem-conta-dica') : t('dividas-extra-com-conta-dica')
        }
      >
        <Select
          value={accountId}
          onValueChange={(v) => v && setAccountId(v)}
          options={[
            ...accounts.map((a) => ({ value: a.id, label: a.name, hint: a.currency })),
            ...fgts.map((a) => ({
              value: a.account_id,
              label: a.name,
              hint: t('dividas-extra-fgts-saldo', { saldo: formatMoney(a.balance, 'BRL') }),
            })),
            { value: NONE, label: t('dividas-extra-sem-lancamento') },
          ]}
        />
      </Field>
      {date && date < todayIso() ? (
        <p className="text-[12.5px] text-dim">
          {t('dividas-extra-data-passada', { data: formatDate(date) })}
        </p>
      ) : null}
      <FormError error={add.error ?? housing.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button
          type="submit"
          variant="primary"
          loading={add.isPending || housing.isPending}
          disabled={submitted && invalid}
        >
          {t('dividas-extra-registrar')}
        </Button>
      </div>
    </form>
  );
}
