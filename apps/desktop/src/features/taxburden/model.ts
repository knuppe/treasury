/**
 * Textos (pelo catálogo `impostos`) e cálculos de exibição da estimativa de impostos (Impostos): cenários e o
 * parâmetro da URL, rótulos dos grupos de imposto direto e fora do consumo (os mesmos de
 * `tax_burden::mapping`), motivos de "fora da estimativa", alíquotas e fontes.
 */
import type { ConsumptionGroup } from '@/api/generated/ConsumptionGroup';
import type { GroupKind } from '@/api/generated/GroupKind';
import type { Scenario } from '@/api/generated/Scenario';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import type { TaxPartKind } from '@/api/generated/TaxPartKind';
import type { TaxSource } from '@/api/generated/TaxSource';
import { t } from '@/i18n';
import { formatLocale } from '@/lib/format';

export interface ScenarioDef {
  value: Scenario;
  label: string;
  /** Valor em `?cenario=`. */
  param: string;
  hint: string;
}

export const SCENARIOS: readonly ScenarioDef[] = [
  {
    value: 'conservative',
    param: 'conservador',
    get label() {
      return t('impostos-cenario-conservador');
    },
    get hint() {
      return t('impostos-cenario-conservador-dica');
    },
  },
  {
    value: 'probable',
    param: 'provavel',
    get label() {
      return t('impostos-cenario-provavel');
    },
    get hint() {
      return t('impostos-cenario-provavel-dica');
    },
  },
  {
    value: 'high',
    param: 'elevado',
    get label() {
      return t('impostos-cenario-elevado');
    },
    get hint() {
      return t('impostos-cenario-elevado-dica');
    },
  },
];

export const DEFAULT_SCENARIO: Scenario = 'probable';

/** Aviso obrigatório, enquanto os dados do país não chegam (o de cada país vem no dataset). */
export function defaultDisclaimer(): string {
  return t('impostos-aviso-padrao');
}

export function scenarioFromParam(param: string | null): Scenario {
  return SCENARIOS.find((s) => s.param === param)?.value ?? DEFAULT_SCENARIO;
}

export function scenarioDef(s: Scenario): ScenarioDef {
  return SCENARIOS.find((d) => d.value === s)!;
}

/** O valor de um cenário num trio (`AmountTriple`, `RateTriple`). */
export function pick<T>(triple: Record<Scenario, T>, s: Scenario): T {
  return triple[s];
}

/** Rótulo curto de cada parte (gráfico e tabela dos cenários). */
export const PART_SHORT: Record<TaxPartKind, string> = {
  get consumption() {
    return t('impostos-parte-consumo');
  },
  get income_tax() {
    return t('impostos-parte-ir');
  },
  get social_security() {
    return t('impostos-parte-previdencia');
  },
  get property() {
    return t('impostos-parte-patrimonio');
  },
  get investments() {
    return t('impostos-parte-proventos');
  },
  get other() {
    return t('impostos-parte-outros');
  },
};

/** Grupos de imposto direto: o lançado na categoria é imposto pago (`mapping::DIRECT`). */
export function directGroups(): readonly (readonly [string, string])[] {
  return [
    ['tax_income', t('impostos-grupo-ir-pago')],
    ['tax_iof', t('impostos-grupo-iof-pago')],
    ['tax_property_iptu', t('impostos-grupo-iptu')],
    ['tax_property_ipva', t('impostos-grupo-ipva')],
    ['tax_other', t('impostos-grupo-outros-pagos')],
  ];
}

/** Grupos fora do consumo (`mapping::NOT_CONSUMPTION`). */
export function notConsumptionGroups(): readonly (readonly [string, string])[] {
  return [
    ['financial', t('impostos-grupo-financeiro')],
    ['transfers', t('impostos-grupo-transferencias')],
  ];
}

export function groupKind(key: string): GroupKind {
  if (directGroups().some(([k]) => k === key)) return 'direct_tax';
  if (notConsumptionGroups().some(([k]) => k === key)) return 'not_consumption';
  return 'consumption';
}

export function groupLabel(key: string | null, ds: TaxDataset | undefined): string {
  if (!key) return t('impostos-sem-grupo');
  const own =
    directGroups().find(([k]) => k === key) ?? notConsumptionGroups().find(([k]) => k === key);
  if (own) return own[1];
  return ds?.consumption.find((g) => g.key === key)?.label ?? key;
}

export function consumptionGroup(
  ds: TaxDataset | undefined,
  key: string,
): ConsumptionGroup | undefined {
  return ds?.consumption.find((g) => g.key === key);
}

/** O grupo tem alíquota nos três cenários (senão o gasto fica "sem dado confiável"). */
export function groupRated(g: ConsumptionGroup): boolean {
  return g.low != null && g.central != null && g.high != null;
}

export const ORIGIN_LABEL: Record<string, string> = {
  get user() {
    return t('impostos-origem-usuario');
  },
  get default() {
    return t('impostos-origem-padrao');
  },
  get parent() {
    return t('impostos-origem-pai');
  },
  get receipt() {
    return t('impostos-origem-cupons');
  },
};

export interface ReasonDef {
  title: string;
  explanation: string;
}

/** Por que um gasto ficou fora da estimativa de consumo. */
export const REASONS: Record<string, ReasonDef> = {
  uncategorized: {
    get title() {
      return t('impostos-motivo-sem-categoria');
    },
    get explanation() {
      return t('impostos-motivo-sem-categoria-dica');
    },
  },
  unmapped: {
    get title() {
      return t('impostos-motivo-sem-grupo');
    },
    get explanation() {
      return t('impostos-motivo-sem-grupo-dica');
    },
  },
  not_consumption: {
    get title() {
      return t('impostos-motivo-fora-consumo');
    },
    get explanation() {
      return t('impostos-motivo-fora-consumo-dica');
    },
  },
  no_rate: {
    get title() {
      return t('impostos-motivo-sem-dado');
    },
    get explanation() {
      return t('impostos-motivo-sem-dado-dica');
    },
  },
};

export const REASON_ORDER = ['unmapped', 'no_rate', 'uncategorized', 'not_consumption'];

/** Código do `Intl` no idioma da formatação (o pseudo-idioma usa o pt-BR). */
export function intlLocale(): string {
  const code = formatLocale();
  return code === 'xx' ? 'pt-BR' : code;
}

/** Alíquota em % do preço: `28,45%`, `0%`. */
export function formatRate(pct: number, decimals = 2): string {
  if (!Number.isFinite(pct)) return '—';
  return `${new Intl.NumberFormat(intlLocale(), { maximumFractionDigits: decimals }).format(pct)}%`;
}

/** Ponto percentual com sinal: `+1 p.p.`, `−0,5 p.p.`. */
export function formatPoints(pp: number): string {
  const v = new Intl.NumberFormat(intlLocale(), { maximumFractionDigits: 2 }).format(Math.abs(pp));
  return `${pp < 0 ? '−' : '+'}${t('impostos-pontos', { valor: v })}`;
}

/** Alíquota guardada (número) no campo de texto, com a vírgula ou o ponto do idioma. */
export function rateInputText(rate: number | null | undefined): string {
  if (rate == null) return '';
  return formatLocale() === 'en' ? String(rate) : String(rate).replace('.', ',');
}

/** Texto da base de uma alíquota do país (`RateSpec`): número em % ou base do estado. */
export function rateSpecText(spec: number | string | null | undefined): string {
  if (spec == null) return t('impostos-sem-dado');
  if (typeof spec === 'number') return formatRate(spec);
  const BASIS: Record<string, string> = {
    exempt: t('impostos-base-isento'),
    vat_standard: t('impostos-base-iva-padrao'),
    vat_reduced: t('impostos-base-iva-reduzido'),
    vat_super_reduced: t('impostos-base-iva-super-reduzido'),
    sales_tax: t('impostos-base-vendas'),
  };
  return BASIS[spec] ?? spec;
}

export function regionLabel(country: string): string {
  return country === 'BR'
    ? t('impostos-regiao-uf')
    : country === 'EU'
      ? t('impostos-regiao-pais-membro')
      : t('impostos-regiao-estado');
}

export const isDraft = (coverage: string) => coverage !== 'completo';

export function sourcesOf(ds: TaxDataset | undefined, ids: readonly string[]): TaxSource[] {
  if (!ds) return [];
  return ids
    .map((id) => ds.sources.find((s) => s.id === id))
    .filter((s): s is TaxSource => s !== undefined);
}

/** Anos do seletor: o atual e os anteriores até o primeiro com dados (no máximo 10). */
export function yearOptions(today: string, firstYear: number | null, extra: number[]): number[] {
  const current = Number(today.slice(0, 4));
  const first = Math.max(Math.min(firstYear ?? current, ...extra, current), current - 9);
  const years: number[] = [];
  for (let y = current; y >= first; y--) years.push(y);
  return years;
}
