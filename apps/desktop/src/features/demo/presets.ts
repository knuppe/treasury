/**
 * Perfis prontos do cofre de demonstração (docs/demo.md): cinco famílias com rendas, gastos,
 * contas, investimentos e dívidas típicos, para testar como o app se comporta em cada caso,
 * inclusive com pouco dinheiro e com dívidas. Os valores estão em reais e são convertidos para a
 * moeda do país escolhido por um fator de renda típica (não pelo câmbio): o que importa é a
 * proporção entre renda e gastos.
 */
import type { DemoHousing } from '@/api/generated/DemoHousing';
import type { DemoKind } from '@/api/generated/DemoKind';
import type { DemoTransport } from '@/api/generated/DemoTransport';
import type { MemberSex } from '@/api/generated/MemberSex';
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import { t } from '@/i18n';

export type PresetKind = Exclude<DemoKind, 'custom'>;

export interface PresetAdult {
  name: string;
  income: number;
  variable?: boolean;
  retired?: boolean;
  /** Idade hoje: a data de nascimento sai dela, coerente com o cenário. */
  age: number;
  sex: MemberSex;
}

export interface PresetChild {
  name: string;
  age: number;
  sex: MemberSex;
}

export interface Preset {
  adults: PresetAdult[];
  children: PresetChild[];
  otherIncome?: number;
  benefit?: number;
  housing: DemoHousing;
  housingAmount?: number;
  condo?: number;
  utilities: number;
  groceries: number;
  transport: DemoTransport;
  transportAmount?: number;
  school?: number;
  health?: number;
  leisure?: number;
  subscriptions?: number;
  shopping?: number;
  cardLimit?: number;
  reserve?: number;
  reserveMonthly?: number;
  stocks?: number;
  fiis?: number;
  fixedIncome?: number;
  investMonthly?: number;
  pension?: number;
  pensionMonthly?: number;
  carInstallment?: number;
  carLeft?: number;
  personalLoan?: number;
  personalLeft?: number;
  overdraftLimit?: number;
  lateBills?: boolean;
  cardRevolving?: boolean;
  level: ProfileLevel;
}

/** Valores em centavos de real. */
export const PRESETS: Record<PresetKind, Preset> = {
  comfortable: {
    adults: [
      { name: 'Ana', income: 1_200_000, age: 41, sex: 'female' },
      { name: 'Bruno', income: 850_000, age: 43, sex: 'male' },
    ],
    children: [{ name: 'Clara', age: 9, sex: 'female' }],
    housing: 'mortgage',
    housingAmount: 420_000,
    condo: 90_000,
    utilities: 90_000,
    groceries: 220_000,
    transport: 'car',
    transportAmount: 130_000,
    school: 240_000,
    health: 180_000,
    leisure: 130_000,
    subscriptions: 15_000,
    shopping: 70_000,
    cardLimit: 2_500_000,
    reserve: 6_000_000,
    reserveMonthly: 100_000,
    stocks: 8_000_000,
    fiis: 4_000_000,
    fixedIncome: 5_000_000,
    investMonthly: 150_000,
    pension: 3_000_000,
    pensionMonthly: 50_000,
    level: 'intermediario',
  },
  low_income: {
    adults: [
      { name: 'Maria', income: 190_000, age: 34, sex: 'female' },
      { name: 'João', income: 160_000, variable: true, age: 36, sex: 'male' },
    ],
    children: [
      { name: 'Davi', age: 11, sex: 'male' },
      { name: 'Elisa', age: 7, sex: 'female' },
    ],
    benefit: 60_000,
    housing: 'rent',
    housingAmount: 90_000,
    utilities: 42_000,
    groceries: 145_000,
    transport: 'public',
    transportAmount: 38_000,
    leisure: 15_000,
    subscriptions: 4_000,
    shopping: 28_000,
    cardLimit: 120_000,
    reserve: 150_000,
    reserveMonthly: 3_000,
    // Crediário de um eletrodoméstico, como empréstimo.
    personalLoan: 15_000,
    personalLeft: 10,
    level: 'iniciante',
  },
  struggling: {
    adults: [
      { name: 'Rosa', income: 230_000, age: 38, sex: 'female' },
      { name: 'Carlos', income: 90_000, variable: true, age: 41, sex: 'male' },
    ],
    children: [
      { name: 'Davi', age: 13, sex: 'male' },
      { name: 'Elisa', age: 9, sex: 'female' },
    ],
    housing: 'rent',
    housingAmount: 110_000,
    utilities: 38_000,
    groceries: 130_000,
    transport: 'public',
    transportAmount: 30_000,
    leisure: 6_000,
    subscriptions: 5_000,
    shopping: 20_000,
    cardLimit: 180_000,
    personalLoan: 45_000,
    personalLeft: 18,
    overdraftLimit: 150_000,
    lateBills: true,
    cardRevolving: true,
    level: 'iniciante',
  },
  single: {
    adults: [{ name: 'Lucas', income: 550_000, age: 28, sex: 'male' }],
    children: [],
    housing: 'rent',
    housingAmount: 150_000,
    condo: 35_000,
    utilities: 28_000,
    groceries: 70_000,
    transport: 'app',
    transportAmount: 45_000,
    leisure: 60_000,
    subscriptions: 9_000,
    shopping: 30_000,
    cardLimit: 500_000,
    reserve: 300_000,
    reserveMonthly: 30_000,
    fiis: 200_000,
    fixedIncome: 500_000,
    investMonthly: 30_000,
    level: 'intermediario',
  },
  retiree: {
    adults: [{ name: 'Helena', income: 380_000, retired: true, age: 68, sex: 'female' }],
    children: [],
    otherIncome: 150_000,
    housing: 'own',
    condo: 60_000,
    utilities: 42_000,
    groceries: 110_000,
    transport: 'none',
    health: 130_000,
    leisure: 40_000,
    subscriptions: 6_000,
    shopping: 20_000,
    cardLimit: 400_000,
    reserve: 4_000_000,
    reserveMonthly: 20_000,
    fixedIncome: 8_000_000,
    level: 'iniciante',
  },
};

export const PRESET_KINDS: readonly PresetKind[] = [
  'comfortable',
  'low_income',
  'struggling',
  'single',
  'retiree',
];

export function presetLabel(k: PresetKind): string {
  return {
    comfortable: t('demo-perfil-confortavel'),
    low_income: t('demo-perfil-baixa-renda'),
    struggling: t('demo-perfil-dificuldade'),
    single: t('demo-perfil-sozinho'),
    retiree: t('demo-perfil-aposentado'),
  }[k];
}

export function presetHint(k: PresetKind): string {
  return {
    comfortable: t('demo-perfil-confortavel-dica'),
    low_income: t('demo-perfil-baixa-renda-dica'),
    struggling: t('demo-perfil-dificuldade-dica'),
    single: t('demo-perfil-sozinho-dica'),
    retiree: t('demo-perfil-aposentado-dica'),
  }[k];
}

/**
 * Quanto vale R$ 1 de renda típica na moeda do país (renda líquida de classe média comparada,
 * não o câmbio). Moeda sem fator: 0,3, para moedas fortes como o dólar e o euro.
 */
const INCOME_FACTOR: Record<string, number> = {
  BRL: 1,
  USD: 0.45,
  EUR: 0.25,
  GBP: 0.22,
  CAD: 0.55,
  AUD: 0.6,
  CHF: 0.4,
  MXN: 3.5,
  ARS: 250,
  CLP: 300,
  COP: 1_400,
  PEN: 1.2,
  UYU: 12,
  JPY: 55,
};

export function incomeFactor(currency: string): number {
  return INCOME_FACTOR[currency] ?? 0.3;
}

/** Banco da conta principal sugerido pelo país. */
export function defaultBank(country: string): string {
  return (
    (
      {
        BR: 'Itaú',
        US: 'Chase',
        PT: 'Caixa Geral',
        ES: 'Santander',
        MX: 'BBVA',
        AR: 'Banco Nación',
        GB: 'Barclays',
      } as Record<string, string>
    )[country] ?? 'Bank'
  );
}

/** País sugerido pelo idioma do app, quando ainda não há família (primeira vez). */
export function localeCountry(locale: string): string {
  return (
    ({ 'pt-BR': 'BR', 'pt-PT': 'PT', en: 'US', es: 'ES' } as Record<string, string>)[locale] ?? 'BR'
  );
}
