/**
 * Mock de `sources.*` (fontes externas com liga/desliga) e de `research.fundamentus`
 * (indicadores e proventos de um papel, com o cache local de um dia).
 */
import type { FundamentusSnapshot } from '@/api/generated/FundamentusSnapshot';
import type { SourceId } from '@/api/generated/SourceId';
import type { SourceInfo } from '@/api/generated/SourceInfo';

import { t } from '@/i18n';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { ready } from './ledger/boot';
import { invalid, str } from './ledger/errors';

const SOURCES: [SourceId, string, string, string][] = [
  [
    'yahoo',
    'Yahoo Finance',
    'https://finance.yahoo.com',
    'Cotações diárias e intradiárias, proventos, desdobramentos e busca de tickers.',
  ],
  [
    'fundamentus',
    'Fundamentus',
    'https://fundamentus.com.br',
    'Indicadores (P/L, P/VP, DY, ROE…), proventos e dados técnicos de ações e FIIs, com cache local.',
  ],
  [
    'tesouro',
    'Tesouro Direto',
    'https://www.tesourotransparente.gov.br',
    'Preços e taxas dos títulos públicos.',
  ],
  [
    'cvm',
    'CVM Dados Abertos',
    'https://dados.cvm.gov.br',
    'Cotas de fundos (informe diário), demonstrações de companhias, informes de FIIs e documentos.',
  ],
  [
    'b3',
    'B3 (COTAHIST e empresas listadas)',
    'https://www.b3.com.br',
    'Fechamento e volume do mercado à vista e proventos e eventos em ações oficiais (dividendos, JCP, rendimentos de FII, desdobramentos) para o universo da análise de ativos.',
  ],
  [
    'bcb',
    'Banco Central do Brasil',
    'https://dadosabertos.bcb.gov.br',
    'Índices (CDI, Selic, IPCA, TR…), câmbio PTAX, Focus e a lista oficial de bancos.',
  ],
  ['sec', 'SEC EDGAR', 'https://www.sec.gov/edgar', 'Fundamentos de empresas americanas.'],
  [
    'fred',
    'FRED (St. Louis Fed)',
    'https://fred.stlouisfed.org',
    'Índices dos Estados Unidos (inflação, juros).',
  ],
  [
    'ny_fed',
    'Federal Reserve Bank of New York',
    'https://markets.newyorkfed.org',
    'Fed Funds efetiva (EFFR), diária e sem chave: o juro de referência das famílias que moram nos Estados Unidos.',
  ],
  ['ecb', 'Banco Central Europeu', 'https://data.ecb.europa.eu', 'Câmbio e juros da zona do euro.'],
  ['eurostat', 'Eurostat', 'https://ec.europa.eu/eurostat', 'Inflação dos países europeus.'],
  ['oecd', 'OCDE', 'https://data.oecd.org', 'Inflação e juros de outros países do catálogo.'],
  ['boc', 'Bank of Canada', 'https://www.bankofcanada.ca', 'Índices do Canadá.'],
  ['ons', 'ONS (Reino Unido)', 'https://www.ons.gov.uk', 'Inflação do Reino Unido.'],
  ['boe', 'Bank of England', 'https://www.bankofengland.co.uk', 'Juros do Reino Unido.'],
  [
    'world_bank',
    'Banco Mundial',
    'https://data.worldbank.org',
    'Inflação anual de países sem fonte mensal.',
  ],
  [
    'fipe',
    'Tabela FIPE',
    'https://veiculos.fipe.org.br',
    'Avaliação de veículos do patrimônio físico.',
  ],
  [
    'coingecko',
    'CoinGecko',
    'https://www.coingecko.com',
    'Valor de mercado, oferta, volume e desenvolvimento dos criptoativos cadastrados (API pública, sem chave).',
  ],
  [
    'defillama',
    'DefiLlama',
    'https://defillama.com',
    'TVL das redes, lastro e paridade das stablecoins, preços em dólar dos criptoativos cadastrados e a lista de ataques às redes.',
  ],
  [
    'coin_metrics',
    'Coin Metrics (Community)',
    'https://coinmetrics.io/community-network-data/',
    'Endereços ativos por dia das redes cobertas pela API aberta (CC BY-NC 4.0), para o score de cripto.',
  ],
  [
    'mempool',
    'mempool.space',
    'https://mempool.space',
    'Blocos da semana por pool de mineração do bitcoin, para o coeficiente de Nakamoto do score de cripto.',
  ],
  [
    'tiingo',
    'Tiingo',
    'https://www.tiingo.com',
    'Opcional, com a sua chave gratuita: fechamento diário, dividendos e desdobramentos de ações, ETFs e REITs dos Estados Unidos, no lugar do Yahoo.',
  ],
  [
    'eodhd',
    'EODHD',
    'https://eodhd.com',
    'Opcional, com a sua chave: fechamento diário, dividendos e desdobramentos de bolsas do mundo todo, no lugar do Yahoo. O plano gratuito da EODHD é só de teste.',
  ],
  [
    'brapi',
    'brapi.dev',
    'https://brapi.dev',
    'Opcional, com token: segunda fonte de indicadores e proventos de ações e FIIs da B3, para conferir os dados oficiais (só os ativos cadastrados).',
  ],
  [
    'ibge',
    'IBGE (SIDRA)',
    'https://sidra.ibge.gov.br',
    'IPCA por grupo (alimentação, habitação, transportes…), para a inflação pessoal da família.',
  ],
  [
    'sefaz',
    'SEFAZ (consulta da NFC-e)',
    'https://www.nfe.fazenda.gov.br',
    'Itens, valores e tributos aproximados do cupom fiscal, pela consulta pública da secretaria da fazenda do estado (o link do QR do cupom).',
  ],
  [
    'siconfi',
    'Tesouro Nacional (Siconfi)',
    'https://siconfi.tesouro.gov.br',
    'Arrecadação de tributos do município da família (IPTU, ISS, ITBI e taxas) e a população, pelo relatório que a prefeitura entrega ao Tesouro; só o código do município é enviado.',
  ],
  [
    'mail_autoconfig',
    'Autoconfiguração de e-mail (Thunderbird)',
    'https://autoconfig.thunderbird.net',
    'Servidor IMAP de um domínio fora do catálogo, no assistente do e-mail de ingestão: o autoconfig do próprio domínio e a base do Thunderbird (só o domínio é enviado, nunca o endereço).',
  ],
  [
    'gleif',
    'GLEIF',
    'https://www.gleif.org',
    'LEI (identificador global de entidade legal) dos emissores brasileiros, pelo CNPJ, nos dados dos ativos.',
  ],
  [
    'brasilapi',
    'BrasilAPI (Receita Federal)',
    'https://brasilapi.com.br',
    'Razão social e situação cadastral do CNPJ dos emissores e fundos, pelo espelho público do cadastro da Receita; só o CNPJ é enviado.',
  ],
  [
    'open_food_facts',
    'Open Food Facts',
    'https://world.openfoodfacts.org',
    'Nome, marca, informação nutricional, grupo NOVA e alérgenos dos produtos pelo código de barras; só o código vai, uma vez por produto. Dados abertos (ODbL), indicativos, sem uso médico.',
  ],
  [
    'news_feeds',
    'Portais de notícias (RSS)',
    'https://www.infomoney.com.br/feed/',
    'Manchetes de InfoMoney, Money Times, Seu Dinheiro, E-Investidor e Suno para a aba Notícias do ativo. O pedido é o mesmo para todos: o filtro pelo ticker é feito no computador, então nada da carteira sai. Só título, portal, data e link.',
  ],
];

/** Fontes que só a pessoa liga (desligadas por padrão, como `sources::OPT_IN` do núcleo). */
const OPT_IN: SourceId[] = ['open_food_facts'];

/** País dos dados de cada fonte (o mesmo `SourceId::country` do núcleo). */
const COUNTRY: Partial<Record<SourceId, string>> = {
  fundamentus: 'BR',
  tesouro: 'BR',
  cvm: 'BR',
  b3: 'BR',
  bcb: 'BR',
  fipe: 'BR',
  brapi: 'BR',
  tiingo: 'US',
  ibge: 'BR',
  brasilapi: 'BR',
  sefaz: 'BR',
  siconfi: 'BR',
  news_feeds: 'BR',
  sec: 'US',
  fred: 'US',
  ny_fed: 'US',
  ecb: 'EU',
  eurostat: 'EU',
  boc: 'CA',
  ons: 'GB',
  boe: 'GB',
};

const disabled = new Set<SourceId>(OPT_IN);
const cache = new Map<string, FundamentusSnapshot>();
onMockReset(() => {
  disabled.clear();
  for (const id of OPT_IN) disabled.add(id);
  cache.clear();
});

/** Para o mock de `country.*`: ligar e desligar sem passar pela tela. */
export const sourceIds = (): SourceId[] => SOURCES.map(([id]) => id);
export const sourceEnabled = (id: SourceId) => !disabled.has(id);
export function setSourceEnabled(id: SourceId, on: boolean) {
  if (on) disabled.delete(id);
  else disabled.add(id);
}

function list(): SourceInfo[] {
  return SOURCES.map(([id, name, homepage, used_for]) => ({
    id,
    name,
    homepage,
    used_for,
    terms: t(`nucleo-fontes-${id}-termos`),
    country: COUNTRY[id] ?? 'INT',
    enabled: !disabled.has(id),
    health: null,
  }));
}

const item = (label: string, text: string, hint: string | null = null) => {
  const raw = text.replace('%', '').trim();
  const n = raw === '-' ? NaN : Number(raw.replace(/\./g, '').replace(',', '.'));
  return { label, text, value: Number.isFinite(n) ? n : null, hint };
};

/** Retrato sintético e estável por ticker (FII quando termina em 11). */
function snapshot(ticker: string): FundamentusSnapshot {
  const fii = ticker.endsWith('11');
  const seed = [...ticker].reduce((s, c) => s + c.charCodeAt(0), 0);
  const price = (10 + (seed % 90) + (seed % 7) / 10).toFixed(2).replace('.', ',');
  const today = new Date().toISOString().slice(0, 10);
  const indicators = fii
    ? [
        item(
          'Div. Yield',
          `${(7 + (seed % 5)).toFixed(1).replace('.', ',')}%`,
          'Dividendos dos últimos 12 meses sobre a cotação',
        ),
        item(
          'P/VP',
          (0.8 + (seed % 4) / 10).toFixed(2).replace('.', ','),
          'Cotação dividida pelo valor patrimonial por cota',
        ),
        item('FFO Yield', '7,11%'),
        item('Vacância Média', '2,4%'),
        item('Cap Rate', '8,6%'),
        item('Qtd imóveis', '37'),
      ]
    : [
        item(
          'P/L',
          (6 + (seed % 12)).toFixed(2).replace('.', ','),
          'Preço da ação dividido pelo lucro por ação',
        ),
        item(
          'P/VP',
          (1 + (seed % 9) / 10).toFixed(2).replace('.', ','),
          'Preço da ação dividido pelo valor patrimonial por ação',
        ),
        item(
          'Div. Yield',
          `${(3 + (seed % 6)).toFixed(1).replace('.', ',')}%`,
          'Dividendos dos últimos 12 meses sobre a cotação',
        ),
        item(
          'ROE',
          `${(10 + (seed % 15)).toFixed(1).replace('.', ',')}%`,
          'Lucro líquido sobre o patrimônio líquido',
        ),
        item('Marg. Líquida', '18,4%'),
        item('Dív Líq / Patrim', '0,42'),
        item('Liquidez Corr', '1,65'),
        item('Valor de mercado', '42.500.000.000'),
        item('Min 52 sem', '9,85'),
        item('Max 52 sem', '16,54'),
        item('Lucro Líquido (12m)', '5.244.000.000'),
      ];
  return {
    ticker,
    fii,
    found: true,
    name: fii ? `${ticker} FUNDO DE INVESTIMENTO IMOBILIÁRIO` : `${ticker.slice(0, 4)} S.A.`,
    sector: fii ? 'Logística' : 'Energia Elétrica',
    subsector: fii ? 'Ativa' : 'Energia Elétrica',
    price: Number(price.replace(',', '.')),
    price_date: today,
    indicators,
    oscillations: [item('Dia', '-1,08%'), item('Mês', '3,20%'), item('12 meses', '18,7%')],
    dividends: [
      {
        date_com: '2026-09-30',
        value: fii ? 1.1 : 0.42,
        kind: fii ? 'Rendimento' : 'DIVIDENDO',
        pay_date: '2026-10-15',
        per_shares: 1,
      },
      {
        date_com: '2026-06-30',
        value: fii ? 1.05 : 0.18,
        kind: fii ? 'Rendimento' : 'JRS CAP PROPRIO',
        pay_date: '2026-07-15',
        per_shares: 1,
      },
    ],
    fetched_at: new Date().toISOString(),
    from_cache: false,
    source_url: `https://fundamentus.com.br/detalhes.php?papel=${ticker}`,
  };
}

registerMockHandlers({
  'sources.list': (): SourceInfo[] => {
    ready();
    return list();
  },
  'sources.set': (args): SourceInfo[] => {
    ready();
    const id = str(args, 'id') as SourceId;
    if (!SOURCES.some(([s]) => s === id))
      invalid(`argumentos inválidos: unknown variant \`${id}\``);
    if (args.enabled === false) disabled.add(id);
    else disabled.delete(id);
    return list();
  },
  'portfolio.sync_fundamentus_income': (args) => {
    ready();
    if (disabled.has('fundamentus') && !args.asset_id)
      return { assets: 0, events_added: 0, errors: [] };
    if (disabled.has('fundamentus'))
      invalid('a fonte Fundamentus está desligada em Ajustes › Fontes de dados');
    return { assets: 1, events_added: 2, errors: [] };
  },
  'research.b3_refresh': () => ({ fetched: false, rows: 0 }),
  'research.fundamentus': (args): FundamentusSnapshot => {
    ready();
    const ticker = str(args, 'ticker').trim().toUpperCase().replace(/\.SA$/, '');
    if (!/^[A-Z0-9]{4,8}$/.test(ticker)) invalid(`ticker inválido para o Fundamentus: ${ticker}`);
    const hit = cache.get(ticker);
    if (hit && args.refresh !== true) return { ...hit, from_cache: true };
    if (disabled.has('fundamentus')) {
      if (hit) return { ...hit, from_cache: true };
      invalid('a fonte Fundamentus está desligada em Ajustes › Fontes de dados');
    }
    const s = snapshot(ticker);
    cache.set(ticker, s);
    return s;
  },
});
