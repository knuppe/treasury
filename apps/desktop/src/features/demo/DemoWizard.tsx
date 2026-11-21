/**
 * Assistente do cofre de demonstração (docs/demo.md): a pessoa conta, em cinco passos, como
 * é a família (renda, despesas, contas, investimentos e dívidas) e o app cria um cofre novo,
 * separado e marcado como demonstração, com um histórico de exemplo (`vaults.new_demo`). O
 * cofre atual fecha com backup e não muda.
 */
import { useState, type ReactNode } from 'react';

import type { DemoHousing } from '@/api/generated/DemoHousing';
import type { DemoKind } from '@/api/generated/DemoKind';
import type { DemoProfile } from '@/api/generated/DemoProfile';
import type { DemoTransport } from '@/api/generated/DemoTransport';
import type { MemberSex } from '@/api/generated/MemberSex';
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import { useCountryChoices, useCountryInfo } from '@/api/hooks/country';
import { useNewDemoVault } from '@/api/hooks/vaults';
import { LEVEL_LABEL } from '@/app/profile';
import {
  Button,
  Dialog,
  Field,
  Icon,
  Input,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { CountryPicker } from '@/features/country/CountryPicker';
import { sexOptions } from '@/features/longevity/sex';
import { Check } from '@/features/portfolio/components/Fields';
import { tRich } from '@/features/settings/richText';
import { InlineError, Segmented } from '@/features/transactions/ui/controls';
import { t, useLocale } from '@/i18n';
import { formatDate, minorUnits } from '@/lib/format';

import {
  defaultBank,
  incomeFactor,
  localeCountry,
  PRESET_KINDS,
  PRESETS,
  presetHint,
  presetLabel,
  type PresetKind,
} from './presets';

const BANKS = ['Itaú', 'Nubank', 'Bradesco', 'Banco do Brasil', 'Caixa', 'Santander', 'Inter'];
const CHILD_NAMES = ['Clara', 'Davi', 'Elisa', 'Felipe'];
/** Sexo dos nomes que o assistente sugere (quem troca o nome escolhe o sexo ao lado). */
const NAME_SEX: Record<string, MemberSex> = {
  Ana: 'female',
  Bruno: 'male',
  Clara: 'female',
  Davi: 'male',
  Elisa: 'female',
  Felipe: 'male',
};

/** Idade e sexo de uma pessoa no rascunho; idade vazia = o app escolhe pelo perfil. */
export interface DraftPerson {
  age: string;
  sex: MemberSex | '';
}

const person = (name: string, age?: number): DraftPerson => ({
  age: age == null ? '' : String(age),
  sex: NAME_SEX[name] ?? '',
});
const FALLBACK_COUNTRIES = [
  { code: 'BR', name: 'Brasil', currency: 'BRL', support: 'full' as const },
];

export interface Draft {
  kind: DemoKind;
  country: string;
  name: string;
  household: string;
  months: number;
  adults: ({
    name: string;
    income: number | null;
    variable: boolean;
    retired: boolean;
  } & DraftPerson)[];
  children: string[];
  /** Idade e sexo de cada filho, na ordem de `children`. */
  kids: DraftPerson[];
  otherIncome: number | null;
  housing: DemoHousing;
  housingAmount: number | null;
  condo: number | null;
  utilities: number | null;
  groceries: number | null;
  transport: DemoTransport;
  transportAmount: number | null;
  school: number | null;
  health: number | null;
  leisure: number | null;
  subscriptions: number | null;
  shopping: number | null;
  bank: string;
  card: boolean;
  cardLimit: number | null;
  reserve: number | null;
  reserveMonthly: number | null;
  stocks: number | null;
  fiis: number | null;
  fixedIncome: number | null;
  investMonthly: number | null;
  pension: number | null;
  pensionMonthly: number | null;
  carLoan: boolean;
  carInstallment: number | null;
  carLeft: string;
  benefit: number | null;
  personalLoan: boolean;
  personalInstallment: number | null;
  personalLeft: string;
  overdraft: boolean;
  overdraftLimit: number | null;
  lateBills: boolean;
  cardRevolving: boolean;
  level: ProfileLevel;
}

const STEPS = [
  'demo-passo-perfil',
  'demo-passo-familia',
  'demo-passo-renda',
  'demo-passo-moradia',
  'demo-passo-contas',
  'demo-passo-dividas',
] as const;

/** R$ arredondado a dezenas. */
const r10 = (cents: number) => Math.round(cents / 1_000) * 1_000;

function totalIncome(d: Draft): number {
  return (
    d.adults.reduce((s, a) => s + (a.income ?? 0), 0) + (d.otherIncome ?? 0) + (d.benefit ?? 0)
  );
}

/** Valores típicos pela renda (proporções de um orçamento comum). */
function suggest(d: Draft): Draft {
  const t = Math.max(totalIncome(d), 300_000);
  const pct = (p: number) => r10((t * p) / 100);
  return {
    ...d,
    housingAmount: d.housing === 'own' ? 0 : pct(d.housing === 'mortgage' ? 20 : 22),
    condo: pct(6),
    utilities: pct(5),
    groceries: pct(11),
    transportAmount: d.transport === 'none' ? 0 : pct(d.transport === 'car' ? 7 : 4),
    school: d.children.length ? pct(9) : 0,
    health: pct(7),
    leisure: pct(6),
    subscriptions: r10((t * 1.2) / 100),
    shopping: pct(3),
    cardLimit: r10(t * 1.2),
    reserve: r10(t * 3),
    reserveMonthly: pct(4),
    stocks: r10(t * 2),
    fiis: r10(t * 1.5),
    fixedIncome: r10(t),
    investMonthly: pct(6),
    pension: r10(t),
    pensionMonthly: pct(3),
    carInstallment: d.carInstallment ?? pct(9),
  };
}

export function initialDraft(level: ProfileLevel): Draft {
  return suggest({
    kind: 'custom',
    country: 'BR',
    name: t('demo-padrao-nome'),
    household: t('demo-padrao-familia'),
    months: 12,
    adults: [
      { name: 'Ana', income: 900_000, variable: false, retired: false, ...person('Ana', 38) },
      { name: 'Bruno', income: 600_000, variable: false, retired: false, ...person('Bruno', 40) },
    ],
    children: ['Clara'],
    kids: [person('Clara', 8)],
    otherIncome: 0,
    housing: 'rent',
    housingAmount: null,
    condo: null,
    utilities: null,
    groceries: null,
    transport: 'car',
    transportAmount: null,
    school: null,
    health: null,
    leisure: null,
    subscriptions: null,
    shopping: null,
    bank: 'Itaú',
    card: true,
    cardLimit: null,
    reserve: null,
    reserveMonthly: null,
    stocks: null,
    fiis: null,
    fixedIncome: null,
    investMonthly: null,
    pension: null,
    pensionMonthly: null,
    carLoan: false,
    carInstallment: null,
    carLeft: '24',
    benefit: 0,
    personalLoan: false,
    personalInstallment: null,
    personalLeft: '12',
    overdraft: false,
    overdraftLimit: null,
    lateBills: false,
    cardRevolving: false,
    level,
  });
}

/** Rascunho a partir de um perfil pronto, com os valores na moeda do país. */
export function fromPreset(kind: PresetKind, country: string, currency: string, d: Draft): Draft {
  const p = PRESETS[kind];
  // Os perfis estão em centavos de real; a moeda do país pode não ter centavos (iene, peso
  // chileno) ou ter três casas, e o valor vai ao núcleo na menor unidade dela.
  const f = incomeFactor(currency) * 10 ** (minorUnits(currency) - 2);
  const m = (v: number | undefined) => r10(Math.round((v ?? 0) * f));
  return {
    ...d,
    kind,
    country,
    name: t('demo-perfil-cofre', { perfil: presetLabel(kind) }),
    adults: p.adults.map((a) => ({
      name: a.name,
      income: m(a.income),
      variable: Boolean(a.variable),
      retired: Boolean(a.retired),
      age: String(a.age),
      sex: a.sex,
    })),
    children: p.children.map((c) => c.name),
    kids: p.children.map((c) => ({ age: String(c.age), sex: c.sex })),
    otherIncome: m(p.otherIncome),
    benefit: m(p.benefit),
    housing: p.housing,
    housingAmount: m(p.housingAmount),
    condo: m(p.condo),
    utilities: m(p.utilities),
    groceries: m(p.groceries),
    transport: p.transport,
    transportAmount: m(p.transportAmount),
    school: m(p.school),
    health: m(p.health),
    leisure: m(p.leisure),
    subscriptions: m(p.subscriptions),
    shopping: m(p.shopping),
    bank: defaultBank(country),
    card: Boolean(p.cardLimit),
    cardLimit: m(p.cardLimit),
    reserve: m(p.reserve),
    reserveMonthly: m(p.reserveMonthly),
    stocks: m(p.stocks),
    fiis: m(p.fiis),
    fixedIncome: m(p.fixedIncome),
    investMonthly: m(p.investMonthly),
    pension: m(p.pension),
    pensionMonthly: m(p.pensionMonthly),
    carLoan: Boolean(p.carInstallment),
    carInstallment: m(p.carInstallment),
    carLeft: String(p.carLeft ?? 24),
    personalLoan: Boolean(p.personalLoan),
    personalInstallment: m(p.personalLoan),
    personalLeft: String(p.personalLeft ?? 12),
    overdraft: Boolean(p.overdraftLimit),
    overdraftLimit: m(p.overdraftLimit),
    lateBills: Boolean(p.lateBills),
    cardRevolving: Boolean(p.cardRevolving),
    level: p.level,
  };
}

export function toProfile(d: Draft): DemoProfile {
  const n = (v: number | null) => v ?? 0;
  return {
    months: d.months,
    kind: d.kind,
    country: d.country,
    adults: d.adults.map((a) => ({
      name: a.name.trim(),
      income: n(a.income),
      variable: a.variable,
      ...(a.retired ? { retired: true } : {}),
      ...(a.age.trim() ? { age: Number(a.age) } : {}),
      ...(a.sex ? { sex: a.sex } : {}),
    })),
    children: d.children.map((c) => c.trim()),
    child_details: d.children.map((_, i) => {
      const k = d.kids[i];
      return {
        ...(k?.age.trim() ? { age: Number(k.age) } : {}),
        ...(k?.sex ? { sex: k.sex } : {}),
      };
    }),
    bank: d.bank.trim(),
    ...(d.card && n(d.cardLimit) > 0 ? { card_limit: n(d.cardLimit) } : {}),
    other_income: n(d.otherIncome),
    housing: d.housing,
    housing_amount: d.housing === 'own' ? 0 : n(d.housingAmount),
    condo: n(d.condo),
    utilities: n(d.utilities),
    groceries: n(d.groceries),
    transport: d.transport,
    transport_amount: d.transport === 'none' ? 0 : n(d.transportAmount),
    school_per_child: d.children.length ? n(d.school) : 0,
    health: n(d.health),
    leisure: n(d.leisure),
    subscriptions: n(d.subscriptions),
    shopping: n(d.shopping),
    reserve: n(d.reserve),
    reserve_monthly: n(d.reserveMonthly),
    stocks: n(d.stocks),
    fiis: n(d.fiis),
    fixed_income: n(d.fixedIncome),
    invest_monthly: n(d.investMonthly),
    pension: n(d.pension),
    pension_monthly: n(d.pensionMonthly),
    car_installment: d.carLoan ? n(d.carInstallment) : 0,
    car_installments_left: d.carLoan ? Number(d.carLeft) || 0 : 0,
    ...(n(d.benefit) > 0 ? { benefit: n(d.benefit) } : {}),
    ...(d.personalLoan && n(d.personalInstallment) > 0
      ? {
          personal_loan: n(d.personalInstallment),
          personal_loan_left: Number(d.personalLeft) || 0,
        }
      : {}),
    ...(d.overdraft && n(d.overdraftLimit) > 0 ? { overdraft_limit: n(d.overdraftLimit) } : {}),
    ...(d.lateBills ? { late_bills: true } : {}),
    ...(d.card && d.cardRevolving ? { card_revolving: true } : {}),
    level: d.level,
  };
}

/** Gasto mensal informado (sem investimentos). */
function monthlySpend(d: Draft): number {
  const p = toProfile(d);
  return (
    p.housing_amount +
    p.condo +
    p.utilities +
    p.groceries +
    p.transport_amount +
    p.school_per_child * p.children.length +
    p.health +
    p.leisure +
    p.subscriptions +
    p.shopping +
    p.car_installment +
    (p.personal_loan ?? 0)
  );
}

/** Nome, idade e sexo de uma pessoa da família de exemplo. */
function PersonRow({
  label,
  name,
  onName,
  person: p,
  onPerson,
}: {
  label: string;
  name: string;
  onName: (name: string) => void;
  person: DraftPerson;
  onPerson: (patch: Partial<DraftPerson>) => void;
}) {
  const who = name.trim() || label;
  return (
    <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_7rem_12rem]">
      <Field label={label}>
        <Input value={name} onChange={(e) => onName(e.target.value)} />
      </Field>
      <Field label={t('demo-idade')}>
        <Input
          inputMode="numeric"
          aria-label={t('demo-idade-de', { nome: who })}
          placeholder={t('demo-idade-auto')}
          value={p.age}
          onChange={(e) => onPerson({ age: e.target.value.replace(/\D/g, '').slice(0, 3) })}
        />
      </Field>
      <Field label={t('familia-sexo')}>
        <Select
          aria-label={t('demo-sexo-de', { nome: who })}
          value={p.sex || undefined}
          onValueChange={(v) => onPerson({ sex: v as MemberSex })}
          options={sexOptions()}
          placeholder={t('familia-sexo-nao-informado')}
        />
      </Field>
    </div>
  );
}

function Grid({ children }: { children: ReactNode }) {
  return <div className="grid gap-4 sm:grid-cols-2">{children}</div>;
}

function Amount({
  label,
  value,
  onChange,
  hint,
}: {
  label: string;
  value: number | null;
  onChange: (v: number | null) => void;
  hint?: string;
}) {
  return (
    <Field label={label} hint={hint}>
      <MoneyInput value={value} onValueChange={onChange} />
    </Field>
  );
}

export function DemoWizard({
  open,
  onOpenChange,
  level = 'avancado',
  initial,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Nível da interface sugerido para o cofre novo (o de quem está usando). */
  level?: ProfileLevel;
  /** Perfil e país já escolhidos (vindos da criação rápida). */
  initial?: { kind: DemoKind; country: string };
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      index="★"
      title={t('demo-titulo')}
      description={t('demo-descricao')}
    >
      {open ? <Wizard level={level} initial={initial} onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}

function Wizard({
  level,
  initial,
  onDone,
}: {
  level: ProfileLevel;
  initial?: { kind: DemoKind; country: string };
  onDone: () => void;
}) {
  const create = useNewDemoVault();
  const [step, setStep] = useState(0);
  // O país da família do cofre aberto vem marcado.
  const locale = useLocale();
  const home = useCountryInfo()?.code ?? localeCountry(locale);
  const countries = useCountryChoices().data ?? [];
  const [d, setD] = useState<Draft>(() => {
    const base = { ...initialDraft(level), country: home, bank: defaultBank(home) };
    if (!initial) return base;
    if (initial.kind === 'custom') return { ...base, country: initial.country };
    const cur = countries.find((c) => c.code === initial.country)?.currency ?? 'BRL';
    return fromPreset(initial.kind, initial.country, cur, base);
  });
  // Renda pela qual os valores das despesas foram sugeridos: mudou, sugere de novo.
  const [suggestedFor, setSuggestedFor] = useState(() => totalIncome(d));
  const [error, setError] = useState<string | null>(null);
  const currency = countries.find((c) => c.code === d.country)?.currency ?? 'BRL';
  const pick = (kind: DemoKind) => {
    if (kind === 'custom') {
      setD((x) => ({ ...x, kind }));
      return;
    }
    const next = fromPreset(kind, d.country, currency, d);
    setD(next);
    setSuggestedFor(totalIncome(next));
  };
  const setCountry = (country: string) => {
    const cur = countries.find((c) => c.code === country)?.currency ?? 'BRL';
    setD((x) =>
      x.kind === 'custom'
        ? { ...x, country, bank: defaultBank(country) }
        : fromPreset(x.kind, country, cur, x),
    );
  };
  const set = <K extends keyof Draft>(k: K, v: Draft[K]) => setD((x) => ({ ...x, [k]: v }));
  const setAdult = (i: number, patch: Partial<Draft['adults'][number]>) =>
    setD((x) => ({ ...x, adults: x.adults.map((a, j) => (j === i ? { ...a, ...patch } : a)) }));
  const income = totalIncome(d);
  const spend = monthlySpend(d);
  const saved = (d.reserveMonthly ?? 0) + (d.investMonthly ?? 0) + (d.pensionMonthly ?? 0);

  /** O que falta para seguir de cada passo. */
  const problem = (): string | null => {
    if (step === 1) {
      if (!d.name.trim()) return t('demo-erro-nome');
      if (!d.household.trim()) return t('demo-erro-familia');
      const names = [...d.adults.map((a) => a.name), ...d.children].map((n) => n.trim());
      if (names.some((n) => !n)) return t('demo-erro-pessoa');
      if (new Set(names.map((n) => n.toLowerCase())).size !== names.length)
        return t('demo-erro-nomes-iguais');
      const badAge = (age: string, min: number, max: number) => {
        if (!age.trim()) return false;
        const n = Number(age);
        return !Number.isInteger(n) || n < min || n > max;
      };
      if (d.adults.some((a) => badAge(a.age, 18, 100)) || d.kids.some((k) => badAge(k.age, 0, 17)))
        return t('demo-erro-idade');
    }
    if (step === 2 && income <= 0) return t('demo-erro-renda');
    if (step === 3 && d.housing === 'mortgage' && !d.housingAmount)
      return t('demo-erro-financiamento');
    if (step === 4 && !d.bank.trim()) return t('demo-erro-banco');
    if (step === 5) {
      if (d.carLoan && (!d.carInstallment || !(Number(d.carLeft) >= 1)))
        return t('demo-erro-carro');
      if (d.personalLoan && (!d.personalInstallment || !(Number(d.personalLeft) >= 1)))
        return t('demo-erro-emprestimo');
    }
    return null;
  };

  const next = () => {
    const p = problem();
    setError(p);
    if (p) return;
    if (step < STEPS.length - 1) {
      if (step === 2 && income !== suggestedFor) {
        setD((x) => suggest(x));
        setSuggestedFor(income);
      }
      setStep(step + 1);
      return;
    }
    create.mutate(
      {
        name: d.name.trim(),
        household_name: d.household.trim(),
        profile: toProfile(d),
      },
      {
        onSuccess: (r) => {
          toast({
            tone: 'success',
            title: t('demo-criado', { nome: r.status.vault_name }),
            description: t('demo-criado-dica', {
              n: r.summary.transactions,
              data: formatDate(r.summary.from),
            }),
          });
          onDone();
        },
      },
    );
  };

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-1.5">
        <span className="text-[12.5px] text-dim">
          {t('demo-passo', { n: step + 1, total: STEPS.length, passo: t(STEPS[step]!) })}
        </span>
        <div
          role="progressbar"
          aria-label={t('demo-progresso')}
          aria-valuemin={0}
          aria-valuemax={STEPS.length}
          aria-valuenow={step + 1}
          className="h-1.5 w-full overflow-hidden rounded-full bg-surface-2"
        >
          <div
            className="h-full rounded-full bg-accent transition-[width]"
            style={{ width: `${((step + 1) / STEPS.length) * 100}%` }}
          />
        </div>
      </div>

      <section aria-label={t(STEPS[step]!)} className="flex flex-col gap-4">
        {step === 0 ? (
          <>
            <Field label={t('demo-pais')} hint={t('demo-pais-dica')}>
              <CountryPicker
                choices={countries.length ? countries : FALLBACK_COUNTRIES}
                value={d.country}
                onChange={setCountry}
              />
            </Field>
            <ProfileCards value={d.kind} onChange={pick} />
            <p className="text-[12.5px] text-dim">{t('demo-perfil-nota')}</p>
          </>
        ) : null}

        {step === 1 ? (
          <>
            <Grid>
              <Field label={t('demo-nome-cofre')} hint={t('demo-nome-cofre-dica')}>
                <Input value={d.name} onChange={(e) => set('name', e.target.value)} />
              </Field>
              <Field label={t('demo-nome-familia')}>
                <Input value={d.household} onChange={(e) => set('household', e.target.value)} />
              </Field>
            </Grid>
            <Field label={t('demo-adultos')}>
              <Segmented
                label={t('demo-adultos')}
                value={String(d.adults.length) as '1' | '2'}
                onChange={(v) =>
                  setD((x) => ({
                    ...x,
                    adults:
                      v === '1'
                        ? x.adults.slice(0, 1)
                        : [
                            ...x.adults,
                            {
                              name: 'Bruno',
                              income: 600_000,
                              variable: false,
                              retired: false,
                              ...person('Bruno'),
                            },
                          ],
                  }))
                }
                options={[
                  { value: '1', label: t('demo-um') },
                  { value: '2', label: t('demo-dois') },
                ]}
              />
            </Field>
            <div className="flex flex-col gap-3">
              {d.adults.map((a, i) => (
                <PersonRow
                  key={i}
                  label={t('demo-nome-adulto', { n: i + 1 })}
                  name={a.name}
                  onName={(name) => setAdult(i, { name })}
                  person={a}
                  onPerson={(patch) => setAdult(i, patch)}
                />
              ))}
            </div>
            <Field label={t('demo-filhos')}>
              <Segmented
                label={t('demo-filhos')}
                value={String(d.children.length) as '0' | '1' | '2' | '3'}
                onChange={(v) =>
                  setD((x) => ({
                    ...x,
                    children: CHILD_NAMES.slice(0, Number(v)).map((n, i) => x.children[i] ?? n),
                    kids: CHILD_NAMES.slice(0, Number(v)).map((n, i) => x.kids[i] ?? person(n)),
                  }))
                }
                options={[
                  { value: '0', label: t('demo-nenhum') },
                  { value: '1', label: '1' },
                  { value: '2', label: '2' },
                  { value: '3', label: '3' },
                ]}
              />
            </Field>
            {d.children.length ? (
              <div className="flex flex-col gap-3">
                {d.children.map((c, i) => (
                  <PersonRow
                    key={i}
                    label={t('demo-nome-filho', { n: i + 1 })}
                    name={c}
                    onName={(name) =>
                      set(
                        'children',
                        d.children.map((x, j) => (j === i ? name : x)),
                      )
                    }
                    person={d.kids[i] ?? person(c)}
                    onPerson={(patch) =>
                      set(
                        'kids',
                        d.children.map((x, j) => {
                          const cur = d.kids[j] ?? person(x);
                          return j === i ? { ...cur, ...patch } : cur;
                        }),
                      )
                    }
                  />
                ))}
              </div>
            ) : null}
            <Field label={t('demo-historico')} hint={t('demo-historico-dica')}>
              <Segmented
                label={t('demo-meses-historico')}
                value={String(d.months) as '6' | '12' | '24'}
                onChange={(v) => set('months', Number(v))}
                options={[
                  { value: '6', label: t('demo-meses', { n: 6 }) },
                  { value: '12', label: t('demo-meses', { n: 12 }) },
                  { value: '24', label: t('demo-meses', { n: 24 }) },
                ]}
              />
            </Field>
          </>
        ) : null}

        {step === 2 ? (
          <>
            {d.adults.map((a, i) => (
              <div key={i} className="grid items-end gap-4 sm:grid-cols-2">
                <Amount
                  label={t('demo-renda-de', {
                    nome: a.name.trim() || t('demo-adulto-n', { n: i + 1 }),
                  })}
                  value={a.income}
                  onChange={(v) => setAdult(i, { income: v })}
                  hint={t('demo-renda-dica')}
                />
                <div className="flex flex-col gap-2">
                  <Check
                    checked={a.variable}
                    onChange={(v) => setAdult(i, { variable: v, ...(v ? { retired: false } : {}) })}
                    label={t('demo-renda-variavel')}
                    hint={t('demo-renda-variavel-dica')}
                  />
                  <Check
                    checked={a.retired}
                    onChange={(v) => setAdult(i, { retired: v, ...(v ? { variable: false } : {}) })}
                    label={t('demo-aposentado')}
                    hint={t('demo-aposentado-dica')}
                  />
                </div>
              </div>
            ))}
            <Grid>
              <Amount
                label={t('demo-outras-rendas')}
                value={d.otherIncome}
                onChange={(v) => set('otherIncome', v)}
                hint={t('demo-outras-rendas-dica')}
              />
              <Amount
                label={t('demo-beneficio')}
                value={d.benefit}
                onChange={(v) => set('benefit', v)}
                hint={t('demo-beneficio-dica')}
              />
            </Grid>
            <p className="text-[13px] text-dim">
              {tRich('demo-renda-total', { valor: <Money cents={income} /> })}
            </p>
          </>
        ) : null}

        {step === 3 ? (
          <>
            <div className="flex flex-wrap items-end justify-between gap-3">
              <Field label={t('demo-moradia')}>
                <Segmented
                  label={t('demo-moradia')}
                  value={d.housing}
                  onChange={(v) => set('housing', v)}
                  options={[
                    { value: 'rent', label: t('demo-aluguel') },
                    { value: 'mortgage', label: t('demo-financiamento') },
                    { value: 'own', label: t('demo-propria') },
                  ]}
                />
              </Field>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => {
                  setD((x) => suggest(x));
                  setSuggestedFor(income);
                }}
              >
                {t('demo-sugerir')}
              </Button>
            </div>
            <Grid>
              {d.housing !== 'own' ? (
                <Amount
                  label={d.housing === 'rent' ? t('demo-aluguel') : t('demo-parcela-financiamento')}
                  value={d.housingAmount}
                  onChange={(v) => set('housingAmount', v)}
                />
              ) : null}
              <Amount
                label={t('demo-condominio')}
                value={d.condo}
                onChange={(v) => set('condo', v)}
              />
              <Amount
                label={t('demo-contas-casa')}
                value={d.utilities}
                onChange={(v) => set('utilities', v)}
                hint={t('demo-contas-casa-dica')}
              />
              <Amount
                label={t('demo-mercado')}
                value={d.groceries}
                onChange={(v) => set('groceries', v)}
              />
            </Grid>
            <Field label={t('demo-transporte')}>
              <Segmented
                label={t('demo-transporte')}
                value={d.transport}
                onChange={(v) => set('transport', v)}
                options={[
                  { value: 'car', label: t('demo-carro') },
                  { value: 'app', label: t('demo-aplicativo') },
                  { value: 'public', label: t('demo-onibus') },
                  { value: 'none', label: t('demo-nao-tem') },
                ]}
              />
            </Field>
            <Grid>
              {d.transport !== 'none' ? (
                <Amount
                  label={d.transport === 'car' ? t('demo-combustivel') : t('demo-transporte')}
                  value={d.transportAmount}
                  onChange={(v) => set('transportAmount', v)}
                />
              ) : null}
              {d.children.length ? (
                <Amount
                  label={t('demo-escola')}
                  value={d.school}
                  onChange={(v) => set('school', v)}
                />
              ) : null}
              <Amount
                label={t('demo-saude')}
                value={d.health}
                onChange={(v) => set('health', v)}
                hint={t('demo-saude-dica')}
              />
              <Amount
                label={t('demo-lazer')}
                value={d.leisure}
                onChange={(v) => set('leisure', v)}
                hint={t('demo-lazer-dica')}
              />
              <Amount
                label={t('demo-assinaturas')}
                value={d.subscriptions}
                onChange={(v) => set('subscriptions', v)}
                hint={t('demo-assinaturas-dica')}
              />
              <Amount
                label={t('demo-compras')}
                value={d.shopping}
                onChange={(v) => set('shopping', v)}
                hint={t('demo-compras-dica')}
              />
            </Grid>
          </>
        ) : null}

        {step === 4 ? (
          <>
            <Grid>
              <Field label={t('demo-banco')}>
                <Input
                  value={d.bank}
                  onChange={(e) => set('bank', e.target.value)}
                  list="demo-banks"
                />
                <datalist id="demo-banks">
                  {BANKS.map((b) => (
                    <option key={b} value={b} />
                  ))}
                </datalist>
              </Field>
              <div className="flex flex-col gap-2">
                <Check
                  checked={d.card}
                  onChange={(v) => set('card', v)}
                  label={t('demo-cartao')}
                  hint={t('demo-cartao-dica')}
                />
                {d.card ? (
                  <Amount
                    label={t('demo-limite')}
                    value={d.cardLimit}
                    onChange={(v) => set('cardLimit', v)}
                  />
                ) : null}
              </div>
            </Grid>
            <fieldset className="flex flex-col gap-3 rounded-card border border-line p-3">
              <legend className="label-caps px-1 text-fg">{t('demo-investimentos')}</legend>
              <Grid>
                <Amount
                  label={t('demo-reserva')}
                  value={d.reserve}
                  onChange={(v) => set('reserve', v)}
                />
                <Amount
                  label={t('demo-reserva-mes')}
                  value={d.reserveMonthly}
                  onChange={(v) => set('reserveMonthly', v)}
                />
                <Amount
                  label={t('demo-acoes')}
                  value={d.stocks}
                  onChange={(v) => set('stocks', v)}
                />
                <Amount label={t('demo-fiis')} value={d.fiis} onChange={(v) => set('fiis', v)} />
                <Amount
                  label={t('demo-tesouro')}
                  value={d.fixedIncome}
                  onChange={(v) => set('fixedIncome', v)}
                />
                <Amount
                  label={t('demo-aporte')}
                  value={d.investMonthly}
                  onChange={(v) => set('investMonthly', v)}
                />
                <Amount
                  label={t('demo-previdencia')}
                  value={d.pension}
                  onChange={(v) => set('pension', v)}
                />
                <Amount
                  label={t('demo-aporte-previdencia')}
                  value={d.pensionMonthly}
                  onChange={(v) => set('pensionMonthly', v)}
                />
              </Grid>
              <p className="text-[12px] text-dim">{t('demo-cotacoes-ficticias')}</p>
            </fieldset>
          </>
        ) : null}

        {step === 5 ? (
          <>
            <div className="flex flex-col gap-3">
              <Check
                checked={d.carLoan}
                onChange={(v) => set('carLoan', v)}
                label={t('demo-financiamento-carro')}
                hint={t('demo-financiamento-carro-dica')}
              />
              {d.carLoan ? (
                <Grid>
                  <Amount
                    label={t('demo-parcela')}
                    value={d.carInstallment}
                    onChange={(v) => set('carInstallment', v)}
                  />
                  <Field label={t('demo-parcelas-faltam')}>
                    <Input
                      inputMode="numeric"
                      value={d.carLeft}
                      onChange={(e) => set('carLeft', e.target.value.replace(/\D/g, ''))}
                    />
                  </Field>
                </Grid>
              ) : null}
              {d.housing === 'mortgage' ? (
                <p className="text-[12.5px] text-dim">{t('demo-financiamento-moradia')}</p>
              ) : null}
              <Check
                checked={d.personalLoan}
                onChange={(v) => set('personalLoan', v)}
                label={t('demo-emprestimo')}
                hint={t('demo-emprestimo-dica')}
              />
              {d.personalLoan ? (
                <Grid>
                  <Amount
                    label={t('demo-parcela')}
                    value={d.personalInstallment}
                    onChange={(v) => set('personalInstallment', v)}
                  />
                  <Field label={t('demo-parcelas-faltam')}>
                    <Input
                      inputMode="numeric"
                      value={d.personalLeft}
                      onChange={(e) => set('personalLeft', e.target.value.replace(/\D/g, ''))}
                    />
                  </Field>
                </Grid>
              ) : null}
              <Check
                checked={d.overdraft}
                onChange={(v) => set('overdraft', v)}
                label={t('demo-cheque-especial')}
                hint={t('demo-cheque-especial-dica')}
              />
              {d.overdraft ? (
                <Grid>
                  <Amount
                    label={t('demo-limite')}
                    value={d.overdraftLimit}
                    onChange={(v) => set('overdraftLimit', v)}
                  />
                </Grid>
              ) : null}
              <Check
                checked={d.lateBills}
                onChange={(v) => set('lateBills', v)}
                label={t('demo-contas-atrasadas')}
                hint={t('demo-contas-atrasadas-dica')}
              />
              {d.card ? (
                <Check
                  checked={d.cardRevolving}
                  onChange={(v) => set('cardRevolving', v)}
                  label={t('demo-rotativo')}
                  hint={t('demo-rotativo-dica')}
                />
              ) : null}
            </div>
            <dl
              aria-label={t('demo-resumo')}
              className="grid grid-cols-3 gap-3 rounded-card border border-line px-4 py-3 text-[13px]"
            >
              <div>
                <dt className="text-dim">{t('demo-entra')}</dt>
                <dd>
                  <Money cents={income} tone="positive" />
                </dd>
              </div>
              <div>
                <dt className="text-dim">{t('demo-sai')}</dt>
                <dd>
                  <Money cents={spend} tone="negative" />
                </dd>
              </div>
              <div>
                <dt className="text-dim">{t('demo-investe')}</dt>
                <dd>
                  <Money cents={saved} />
                </dd>
              </div>
            </dl>
            {spend + saved > income ? (
              <p className="text-[12.5px] text-warning">{t('demo-vermelho')}</p>
            ) : null}
            <Field label={t('demo-nivel')}>
              <Segmented
                label={t('demo-nivel-rotulo')}
                value={d.level}
                onChange={(v) => set('level', v)}
                options={(['iniciante', 'intermediario', 'avancado'] as const).map((l) => ({
                  value: l,
                  label: LEVEL_LABEL[l],
                }))}
              />
            </Field>
            <p
              role="note"
              className="flex items-start gap-2 rounded-card border border-warning/60 bg-warning/5 px-3 py-2 text-[12.5px] text-warning"
            >
              <Icon name="info" size={14} className="mt-0.5 shrink-0" />
              <span>{t('demo-sem-senha')}</span>
            </p>
            <p className="text-[12.5px] text-dim">{t('demo-troca')}</p>
          </>
        ) : null}
      </section>

      {error ? (
        <p role="alert" className="text-[12.5px] text-negative">
          {error}
        </p>
      ) : null}
      <InlineError error={create.error} />
      <div className="flex justify-between gap-2 border-t border-line pt-3">
        <Button
          variant="ghost"
          disabled={step === 0 || create.isPending}
          onClick={() => {
            setError(null);
            setStep(step - 1);
          }}
        >
          {t('comum-voltar')}
        </Button>
        <Button variant="primary" loading={create.isPending} onClick={next}>
          {step < STEPS.length - 1 ? t('comum-continuar') : t('demo-criar')}
        </Button>
      </div>
    </div>
  );
}

/** Os perfis prontos (e, com `custom`, o de montar do zero), como cartões de escolha. */
export function ProfileCards({
  value,
  onChange,
  custom = true,
}: {
  value: DemoKind;
  onChange: (kind: DemoKind) => void;
  custom?: boolean;
}) {
  const kinds: DemoKind[] = custom ? [...PRESET_KINDS, 'custom'] : [...PRESET_KINDS];
  return (
    <div role="radiogroup" aria-label={t('demo-perfil')} className="grid gap-2 sm:grid-cols-2">
      {kinds.map((k) => {
        const on = value === k;
        return (
          <button
            key={k}
            type="button"
            role="radio"
            aria-checked={on}
            onClick={() => onChange(k)}
            className={cx(
              'flex flex-col gap-1 rounded-card border px-3 py-2.5 text-left transition-colors',
              on ? 'border-accent bg-accent/10' : 'border-line hover:border-line-strong',
            )}
          >
            <span className="text-[13.5px] font-medium text-fg">
              {k === 'custom' ? t('demo-perfil-custom') : presetLabel(k)}
            </span>
            <span className="text-[12px] leading-snug text-dim">
              {k === 'custom' ? t('demo-perfil-custom-dica') : presetHint(k)}
            </span>
          </button>
        );
      })}
    </div>
  );
}
