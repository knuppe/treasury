/**
 * Mock de `country.*` e `app.countries` (docs/countries.md): o país da família, o que o app
 * sabe dele e a troca, que liga e desliga as fontes como o núcleo (`country::apply_sources`).
 */
import type { CountryChange } from '@/api/generated/CountryChange';
import type { CountryChoice } from '@/api/generated/CountryChoice';
import type { CountryFeatures } from '@/api/generated/CountryFeatures';
import type { CountryInfo } from '@/api/generated/CountryInfo';
import type { CountryState } from '@/api/generated/CountryState';
import type { CountrySupport } from '@/api/generated/CountrySupport';
import type { PayrollTerms } from '@/api/generated/PayrollTerms';
import type { SourceId } from '@/api/generated/SourceId';

import { onMockReset, registerMockHandlers, takeMockCreatedCountry } from '../mockRegistry';
import { COUNTRIES } from './countryList';
import { guarded } from './ledger/boot';
import { nowUtc } from './ledger/dates';
import { household as currentHousehold } from './ledger/domain';
import { setFixtureBase } from './ledger/fixture';
import { invalid, str } from './ledger/errors';
import { patch, rows, write } from './ledger/store';
import { setSourceEnabled, sourceEnabled, sourceIds } from './sources';

const FEATURED = ['BR', 'US', 'PT', 'ES', 'GB', 'CA', 'FR', 'DE', 'AR', 'CL'];
/** Adaptados na rodada 2 (como `country::ADAPTED_2`). */
const ADAPTED_2 = ['GB', 'CA', 'FR', 'DE', 'AR', 'CL'];
const EU = new Set(
  'AT BE BG CY CZ DE DK EE GR ES FI FR HR HU IE IT LT LU LV MT NL PL PT RO SE SI SK'.split(' '),
);
const GLOBAL: SourceId[] = [
  'yahoo',
  'sec',
  'ecb',
  'oecd',
  'world_bank',
  'coingecko',
  'defillama',
  'coin_metrics',
  'mempool',
  'open_food_facts',
];

const support = (code: string): CountrySupport =>
  code === 'BR' ? 'full' : FEATURED.includes(code) ? 'adapted' : 'generic';

export function choices(): CountryChoice[] {
  const row = (c: (typeof COUNTRIES)[number]): CountryChoice => ({
    code: c[0],
    name: c[1],
    currency: c[2],
    support: support(c[0]),
  });
  const featured = FEATURED.map((f) => COUNTRIES.find((c) => c[0] === f)!).map(row);
  return [...featured, ...COUNTRIES.filter((c) => !FEATURED.includes(c[0])).map(row)];
}

function relevant(code: string): SourceId[] {
  if (code === 'BR') return sourceIds();
  const out = new Set<SourceId>(GLOBAL);
  const extra: Record<string, SourceId[]> = {
    US: ['fred', 'ny_fed'],
    CA: ['boc'],
    GB: ['ons', 'boe'],
  };
  for (const id of extra[code] ?? []) out.add(id);
  if (EU.has(code)) out.add('eurostat');
  return sourceIds().filter((s) => out.has(s));
}

const KINDS: Record<string, Record<string, string>> = {
  BR: { clt: 'CLT', pj: 'PJ', benefit: 'Benefício social' },
  US: {
    clt: 'Empregado (W-2)',
    retirement: 'Aposentadoria (Social Security, pensão)',
    pj: 'Contractor (1099)',
    self_employed: 'Autônomo (self-employed)',
    informal: 'Informal (gig, bicos)',
    benefit: 'Benefício (SNAP, desemprego)',
  },
  PT: {
    clt: 'Trabalho por conta de outrem',
    public_servant: 'Função pública',
    retirement: 'Reforma',
    pj: 'Trabalhador independente (recibos verdes)',
    self_employed: 'Atividade por conta própria',
    rental: 'Rendas de imóveis',
    benefit: 'Prestação social (abono, RSI, desemprego)',
  },
  ES: {
    clt: 'Trabalho por conta alheia',
    public_servant: 'Funcionário público',
    retirement: 'Pensão de jubilación',
    pj: 'Autónomo (faturas)',
    self_employed: 'Negócio próprio',
    benefit: 'Prestação social (paro, Ingreso Mínimo Vital)',
  },
  XX: { clt: 'Empregado', pj: 'Prestador de serviços' },
};
const BASE_KINDS: Record<string, string> = {
  clt: 'Empregado',
  public_servant: 'Servidor público',
  retirement: 'Aposentadoria',
  pension: 'Pensão',
  pj: 'Prestador de serviços',
  self_employed: 'Autônomo',
  informal: 'Informal',
  rental: 'Aluguel',
  benefit: 'Benefício social',
  other: 'Outra',
};

export function info(code: string): CountryInfo {
  const c = COUNTRIES.find((x) => x[0] === code);
  if (!c) invalid(`país fora da lista: ${code}`);
  const eu = EU.has(code);
  const none: CountryFeatures = {
    br_market: false,
    br_income_tax: false,
    br_credit_rates: false,
    fipe: false,
    fgts: false,
    tax_burden: eu,
    payroll: false,
    receipts: false,
    iptu: false,
  };
  const payroll = (p: Partial<PayrollTerms>): PayrollTerms => ({
    contribution: 'Previdência',
    income_tax: 'Imposto retido',
    extra_pay: null,
    payslip: 'recibo de salário',
    payslips: 'recibos de salário',
    employer_id: null,
    ...p,
    kinds: { ...BASE_KINDS, ...(KINDS[FEATURED.includes(code) ? code : 'XX'] ?? {}) },
  });
  const base = {
    code,
    name: c[1],
    currency: c[2],
    support: support(code),
    sources: relevant(code),
    emergency_months: 6,
    tax_dataset: code === 'BR' || code === 'US' ? code : eu ? 'EU' : null,
    tax_region: eu ? (code === 'GR' ? 'EL' : code) : null,
    // Como `country::pension_of`.
    pension:
      code === 'BR'
        ? { name: 'INSS', rules: 'br_inss' }
        : code === 'US'
          ? { name: 'Social Security', rules: 'us_ssa' }
          : code === 'PT'
            ? { name: 'Segurança Social', rules: 'pt_ss' }
            : code === 'ES'
              ? { name: 'Seguridad Social', rules: null }
              : { name: 'previdência pública', rules: null },
  };
  if (code === 'BR')
    return {
      ...base,
      features: {
        br_market: true,
        br_income_tax: true,
        br_credit_rates: true,
        fipe: true,
        fgts: true,
        tax_burden: true,
        payroll: true,
        receipts: true,
        iptu: true,
      },
      inflation_label: 'IPCA',
      rate_label: 'CDI',
      payroll: payroll({
        contribution: 'INSS',
        income_tax: 'IRRF',
        extra_pay: '13º salário e terço de férias',
        payslip: 'holerite',
        payslips: 'holerites',
        employer_id: 'CNPJ',
      }),
      tips: [
        'Reserva de emergência de 6 meses de gastos essenciais, num lugar com liquidez diária.',
      ],
      limits: [],
    };
  if (code === 'US')
    return {
      ...base,
      features: { ...none, tax_burden: true, payroll: true },
      inflation_label: 'CPI-U',
      rate_label: 'Fed Funds',
      payroll: payroll({
        contribution: 'Social Security e Medicare (FICA)',
        income_tax: 'Imposto federal retido',
        payslip: 'paystub (contracheque)',
        payslips: 'paystubs (contracheques)',
        employer_id: 'EIN',
      }),
      tips: [
        'Emergency fund: de 3 a 6 meses de gastos essenciais numa conta de alto rendimento.',
        '401(k): contribua pelo menos o bastante para ganhar todo o match do empregador.',
      ],
      limits: [
        'O imposto estadual e o local não entram nos descontos da folha (só o federal e o FICA).',
      ],
    };
  if (code === 'PT')
    return {
      ...base,
      features: { ...none, tax_burden: true, payroll: true },
      inflation_label: 'IHPC',
      rate_label: '€STR',
      payroll: payroll({
        contribution: 'Segurança Social',
        income_tax: 'IRS retido',
        extra_pay: 'Subsídios de férias e de Natal',
        payslip: 'recibo de vencimento',
        payslips: 'recibos de vencimento',
        employer_id: 'NIF',
      }),
      tips: [
        'Fundo de emergência de 6 meses de despesas essenciais, num depósito a prazo mobilizável ou em Certificados de Aforro.',
        'PPR: 20% do que se aplica no ano abate ao IRS, com limite pela idade.',
      ],
      limits: [
        'A retenção de IRS é estimada pelo imposto anual (escalões do continente), não pelas tabelas mensais de retenção.',
      ],
    };
  if (code === 'ES')
    return {
      ...base,
      features: { ...none, tax_burden: true, payroll: true },
      inflation_label: 'IHPC',
      rate_label: '€STR',
      payroll: payroll({
        contribution: 'Seguridad Social',
        income_tax: 'IRPF retido',
        extra_pay: 'Pagas extra de junho e dezembro',
        payslip: 'nómina',
        payslips: 'nóminas',
        employer_id: 'CIF',
      }),
      tips: [
        'Fundo de emergência de 6 meses de despesas essenciais, numa conta remunerada ou em Letras del Tesoro.',
        'Plan de pensiones: até 1.500 € por ano saem da base do IRPF.',
      ],
      limits: [
        'O IRPF é estimado pela escala geral (estatal + autonómica comum), não pela escala da sua comunidade.',
      ],
    };
  if (ADAPTED_2.includes(code))
    return {
      ...base,
      features: { ...none, payroll: true },
      inflation_label: 'Inflação',
      rate_label: 'Juro de referência',
      payroll: payroll({ extra_pay: code === 'AR' ? 'Aguinaldo (SAC) em junho e dezembro' : null }),
      tips: ['Fundo de emergência de 3 a 6 meses de despesas essenciais.'],
      limits: ['Uma aproximação: sem as deduções e créditos pessoais além dos básicos.'],
    };
  return {
    ...base,
    features: none,
    inflation_label: 'Inflação',
    rate_label: 'Juro de referência',
    payroll: payroll({}),
    tips: [
      'Reserva de emergência de 3 a 6 meses de gastos essenciais, num lugar seguro e com resgate rápido.',
    ],
    limits: [
      'Os descontos do salário (impostos e previdência) não são calculados: informe o líquido ou importe o recibo.',
    ],
  };
}

let offByCountry = new Set<SourceId>();
onMockReset(() => {
  offByCountry = new Set();
});

/** País da família; o escolhido na criação do cofre vale na primeira consulta (o banco do
 * mock nasce depois do `vault.create`, já na moeda do país; aqui entram as fontes dele). Como
 * no núcleo, o cofre novo nasce na moeda do país. */
export function familyCountry(): string {
  const created = takeMockCreatedCountry();
  const h = currentHousehold();
  if (created) {
    if (h.country !== created) {
      patch('household', h.id, { country: created, base_currency: info(created).currency });
      setFixtureBase(info(created).currency);
    }
    applySources(created);
  }
  return currentHousehold().country ?? 'BR';
}

function applySources(code: string) {
  const keep = relevant(code);
  const disabled: SourceId[] = [];
  const enabled: SourceId[] = [];
  for (const id of sourceIds()) {
    if (keep.includes(id)) {
      if (offByCountry.has(id)) {
        offByCountry.delete(id);
        if (!sourceEnabled(id)) {
          setSourceEnabled(id, true);
          enabled.push(id);
        }
      }
    } else if (sourceEnabled(id)) {
      setSourceEnabled(id, false);
      offByCountry.add(id);
      disabled.push(id);
    }
  }
  return { disabled, enabled };
}

function state(): CountryState {
  return {
    info: info(familyCountry()),
    choices: choices(),
    base_currency: currentHousehold().base_currency,
    has_data: rows('account').length > 0 || rows('txn').length > 0,
  };
}

registerMockHandlers({ 'app.countries': () => choices() });
registerMockHandlers(
  guarded({
    'country.get': () => state(),
    'country.set': (args): CountryChange => {
      const code = str(args, 'country').trim().toUpperCase();
      const target = info(code);
      return write(`País da família: ${target.name}`, () => {
        const h = currentHousehold();
        const st = state();
        const change =
          args.change_currency === undefined ? !st.has_data : Boolean(args.change_currency);
        const currencyChanged = change && h.base_currency !== target.currency;
        patch('household', h.id, {
          country: code,
          ...(currencyChanged ? { base_currency: target.currency } : {}),
          updated_at: nowUtc(),
        });
        const { disabled, enabled } = applySources(code);
        return { state: state(), disabled, enabled, currency_changed: currencyChanged };
      }).value;
    },
  }),
);
