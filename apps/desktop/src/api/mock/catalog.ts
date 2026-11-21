/**
 * Mock do catálogo de países e dos provedores de índices (`indices.catalog`,
 * `indices.set_country`, `indices.providers`). Um recorte do catálogo real (migração
 * V84): Brasil, EUA, zona do euro, Portugal, Canadá, Reino Unido, Japão e Argentina.
 */
import type { CatalogSeries } from '../generated/CatalogSeries';
import type { CountryCatalog } from '../generated/CountryCatalog';
import type { CountryProfile } from '../generated/CountryProfile';
import type { DataProvider } from '../generated/DataProvider';
import type { DataProviderInfo } from '../generated/DataProviderInfo';
import type { ProviderStatus } from '../generated/ProviderStatus';
import { mockFail, onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';

let chosen: string | null = null;
onMockReset(() => {
  chosen = null;
});

function ser(
  id: string,
  name: string,
  provider: DataProvider,
  source: string,
  opts: Partial<CatalogSeries> = {},
): CatalogSeries {
  return {
    id,
    name,
    kind: 'level',
    unit: 'índice',
    frequency: 'monthly',
    source,
    provider,
    priority: 0,
    ...opts,
  };
}

const ecbFx = (ccy: string[]) =>
  ccy.map((c, i) =>
    ser(`fx_eur_${c.toLowerCase()}`, `EUR/${c} (BCE)`, 'ecb', `ecb:EXR/D.${c}.EUR.SP00.A`, {
      frequency: 'daily',
      unit: `${c} por EUR`,
      priority: i,
    }),
  );
const wb = (iso: string, name: string, priority: number) =>
  ser(
    `wb_cpi_${iso.toLowerCase()}`,
    `Inflação anual ${name} (Banco Mundial)`,
    'world_bank',
    `worldbank:${iso}`,
    {
      unit: 'índice (estimado)',
      priority,
    },
  );
const estr = ser('estr', '€STR (taxa de curto prazo do euro)', 'ecb', 'ecb:EST/B.EU000A2X2A25.WT', {
  kind: 'annual_rate',
  unit: '% a.a.',
  frequency: 'daily',
});

const COUNTRIES: CountryProfile[] = [
  {
    country: 'BR',
    name: 'Brasil',
    currency: 'BRL',
    fx_provider: 'bcb_ptax',
    notes: 'Comportamento de sempre: IPCA, CDI/Selic e PTAX do Banco Central.',
    inflation: [
      ser('ipca', 'IPCA', 'bcb_sgs', 'bcb_sgs:433', { kind: 'monthly_rate', unit: '% a.m.' }),
    ],
    rates: [
      ser('cdi', 'CDI', 'bcb_sgs', 'bcb_sgs:12', {
        kind: 'daily_rate',
        unit: '% a.d.',
        frequency: 'daily',
      }),
    ],
    fx: [],
  },
  {
    country: 'US',
    name: 'Estados Unidos',
    currency: 'USD',
    fx_provider: 'ecb',
    notes: 'CPI-U da FRED exige a chave gratuita; sem ela, o CPI da OCDE (sem chave) é usado.',
    inflation: [
      ser('us_cpi', 'US CPI-U', 'fred', 'fred:CPIAUCSL'),
      ser('cpi_us_oecd', 'CPI Estados Unidos (OCDE)', 'oecd', 'oecd:…/USA.M.N.CPI.IX._T.N._Z', {
        priority: 1,
      }),
      wb('US', 'Estados Unidos', 2),
    ],
    rates: [
      ser('fed_funds', 'Fed Funds', 'fred', 'fred:FEDFUNDS', {
        kind: 'annual_rate',
        unit: '% a.a.',
      }),
    ],
    fx: ecbFx(['USD', 'BRL']),
  },
  {
    country: 'EA',
    name: 'Zona do euro',
    currency: 'EUR',
    fx_provider: 'ecb',
    notes: null,
    inflation: [ser('hicp_ea', 'HICP zona do euro', 'ecb', 'ecb:HICP/M.U2.N.000000.4D0.INX')],
    rates: [estr],
    fx: ecbFx(['USD', 'GBP', 'BRL']),
  },
  {
    country: 'PT',
    name: 'Portugal',
    currency: 'EUR',
    fx_provider: 'ecb',
    notes: null,
    inflation: [
      ser('hicp_pt', 'HICP Portugal', 'ecb', 'ecb:HICP/M.PT.N.000000.4D0.INX', {
        unit: 'índice (2025=100)',
      }),
      wb('PT', 'Portugal', 1),
    ],
    rates: [estr],
    fx: ecbFx(['USD', 'GBP', 'BRL']),
  },
  {
    country: 'CA',
    name: 'Canadá',
    currency: 'CAD',
    fx_provider: 'boc',
    notes: null,
    inflation: [ser('cpi_ca', 'CPI Canadá', 'boc', 'boc:V41690973'), wb('CA', 'Canadá', 1)],
    rates: [
      ser('corra', 'CORRA (Canadá)', 'boc', 'boc:AVG.INTWO', {
        kind: 'annual_rate',
        unit: '% a.a.',
        frequency: 'daily',
      }),
    ],
    fx: [
      ser('fx_usd_cad_boc', 'USD/CAD (Bank of Canada)', 'boc', 'boc:FXUSDCAD', {
        frequency: 'daily',
        unit: 'CAD por USD',
      }),
    ],
  },
  {
    country: 'GB',
    name: 'Reino Unido',
    currency: 'GBP',
    fx_provider: 'boe',
    notes: null,
    inflation: [
      ser('cpi_gb', 'CPI Reino Unido (ONS)', 'ons', 'ons:mm23/d7bt'),
      ser('cpih_gb', 'CPIH Reino Unido (ONS)', 'ons', 'ons:mm23/l522', { priority: 1 }),
      wb('GB', 'Reino Unido', 2),
    ],
    rates: [
      ser('sonia', 'SONIA (Reino Unido)', 'boe', 'boe:IUDSOIA', {
        kind: 'annual_rate',
        unit: '% a.a.',
        frequency: 'daily',
      }),
    ],
    fx: [
      ser('fx_gbp_usd', 'GBP/USD (Bank of England)', 'boe', 'boe:XUDLUSS', {
        frequency: 'daily',
        unit: 'USD por GBP',
      }),
    ],
  },
  {
    country: 'JP',
    name: 'Japão',
    currency: 'JPY',
    fx_provider: 'ecb',
    notes: null,
    inflation: [
      ser('cpi_jp', 'CPI Japão (OCDE)', 'oecd', 'oecd:…/JPN.M.N.CPI.IX._T.N._Z'),
      wb('JP', 'Japão', 1),
    ],
    rates: [
      ser('rate3m_jp', 'Juros de 3 meses Japão (OCDE)', 'oecd', 'oecd:…/JPN.M.IR3TIB', {
        kind: 'annual_rate',
        unit: '% a.a.',
      }),
    ],
    fx: ecbFx(['USD', 'GBP', 'BRL', 'JPY']),
  },
  {
    country: 'AR',
    name: 'Argentina',
    currency: 'ARS',
    fx_provider: 'ecb',
    notes: 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).',
    inflation: [wb('AR', 'Argentina', 0)],
    rates: [],
    fx: ecbFx(['USD', 'GBP', 'BRL']),
  },
];

export function referenceCountry(): string {
  return chosen ?? 'BR';
}

/** Deflator simulado equivalente à inflação do país de referência (`null` sem série no mock). */
export function countryDeflator(): 'ipca' | 'us_cpi' | null {
  const c = referenceCountry();
  return c === 'BR' ? 'ipca' : c === 'US' ? 'us_cpi' : null;
}

export function referenceProfile(): CountryProfile {
  return COUNTRIES.find((c) => c.country === referenceCountry()) ?? COUNTRIES[0]!;
}

function catalog(): CountryCatalog {
  return {
    country: referenceCountry(),
    automatic: chosen === null,
    base_currency: 'BRL',
    countries: [...COUNTRIES].sort((a, b) => a.name.localeCompare(b.name, 'pt-BR')),
  };
}

const INFO: Record<DataProvider, Omit<DataProviderInfo, 'id'>> = {
  bcb_sgs: {
    name: 'Banco Central do Brasil (SGS, PTAX, Focus)',
    homepage: 'https://dadosabertos.bcb.gov.br',
    terms: 'Dados abertos do BCB, uso livre com citação da fonte.',
    key_required: false,
  },
  fred: {
    name: 'FRED (Federal Reserve Bank of St. Louis)',
    homepage: 'https://fred.stlouisfed.org',
    terms: 'Gratuito com chave pessoal; cada série segue os termos da fonte original.',
    key_required: true,
  },
  yahoo: {
    name: 'Yahoo Finance',
    homepage: 'https://finance.yahoo.com',
    terms: 'Fonte não oficial, sem garantia; usada só para fechamentos diários.',
    key_required: false,
  },
  ecb: {
    name: 'Banco Central Europeu (ECB Data Portal)',
    homepage: 'https://data.ecb.europa.eu',
    terms: 'Reutilização livre com citação da fonte (termos do ECB Data Portal).',
    key_required: false,
  },
  eurostat: {
    name: 'Eurostat',
    homepage: 'https://ec.europa.eu/eurostat',
    terms: 'Reutilização livre com citação (CC BY 4.0).',
    key_required: false,
  },
  oecd: {
    name: 'OCDE (OECD Data Explorer)',
    homepage: 'https://data-explorer.oecd.org',
    terms: 'CC BY 4.0; a API pública limita o número de consultas por hora.',
    key_required: false,
  },
  boc: {
    name: 'Bank of Canada (Valet)',
    homepage: 'https://www.bankofcanada.ca/valet/docs',
    terms: 'Uso livre com citação (termos do Bank of Canada).',
    key_required: false,
  },
  ons: {
    name: 'Office for National Statistics (Reino Unido)',
    homepage: 'https://www.ons.gov.uk',
    terms: 'Open Government Licence v3.0.',
    key_required: false,
  },
  boe: {
    name: 'Bank of England (Database)',
    homepage: 'https://www.bankofengland.co.uk/boeapps/database',
    terms: 'Uso livre com citação (termos do Bank of England).',
    key_required: false,
  },
  world_bank: {
    name: 'Banco Mundial (World Development Indicators)',
    homepage: 'https://data.worldbank.org',
    terms: 'CC BY 4.0. Inflação anual; os meses são interpolados e ficam marcados como estimados.',
    key_required: false,
  },
  ibge_sidra: {
    name: 'IBGE (SIDRA)',
    homepage: 'https://sidra.ibge.gov.br',
    terms: 'Dados públicos do IBGE, uso livre com citação da fonte.',
    key_required: false,
  },
  ny_fed: {
    name: 'Federal Reserve Bank of New York (Markets Data)',
    homepage: 'https://markets.newyorkfed.org',
    terms: 'Dados públicos do Federal Reserve Bank of New York, uso livre com citação da fonte.',
    key_required: false,
  },
};

function providers(): ProviderStatus[] {
  const p = referenceProfile();
  const used = new Set<DataProvider>(['bcb_sgs', 'fred', 'yahoo']);
  for (const s of [...p.inflation, ...p.rates, ...p.fx]) if (s.provider) used.add(s.provider);
  const at = '2026-10-14T06:00:00Z';
  return (Object.keys(INFO) as DataProvider[]).map((id) => {
    const inUse = used.has(id);
    return {
      info: { id, ...INFO[id] },
      in_use: inUse,
      series_total: id === 'world_bank' ? 57 : id === 'ecb' ? 52 : 4,
      series_in_use: inUse ? 2 : 0,
      last_fetched_at: inUse && id !== 'world_bank' ? at : null,
      last_date: inUse && id !== 'world_bank' ? '2026-10-13' : null,
      status: !inUse ? 'idle' : id === 'world_bank' ? 'never' : 'ok',
      error: null,
      error_at: null,
    };
  });
}

registerMockHandlers({
  'indices.catalog': () => {
    requireUnlocked();
    return catalog();
  },
  'indices.providers': () => {
    requireUnlocked();
    return providers();
  },
  'indices.set_country': (a) => {
    requireUnlocked();
    const c = typeof a.country === 'string' ? a.country.trim().toUpperCase() : '';
    if (c && !COUNTRIES.some((p) => p.country === c))
      mockFail('invalid', `país fora do catálogo: ${c}`);
    chosen = c || null;
    return catalog();
  },
});
