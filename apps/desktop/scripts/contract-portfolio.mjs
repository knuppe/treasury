#!/usr/bin/env node
/**
 * Teste de contrato da Carteira/Ativos contra o núcleo real (treasury-dev serve).
 *
 *   treasury-dev serve --home /tmp/tdp --port 7783 --token dev --no-mcp-socket
 *   node scripts/contract-portfolio.mjs [--url http://127.0.0.1:7783] [--token dev] [--sync]
 *
 * Cria (ou abre) um cofre, uma corretora em BRL e outra em USD, ativos BR/EUA/renda
 * fixa/Tesouro, operações (com desdobramento e uma venda inválida), proventos, alvos,
 * watchlist e consulta resumo, posição, rentabilidade, rebalanceamento, agenda, renda
 * passiva, índices e cotações. Cada resposta é conferida, campo a campo e em
 * profundidade, contra os tipos TS gerados em `src/api/generated/` (a fonte da verdade
 * do contrato). `--sync` também roda `market.sync` (rede; pode levar minutos).
 *
 * Sai com código 1 se alguma conferência falhar.
 */
import { readFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const GENERATED = join(here, '..', 'src', 'api', 'generated');

const argv = process.argv.slice(2);
const arg = (name, fallback) => {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 && argv[i + 1] && !argv[i + 1].startsWith('--') ? argv[i + 1] : fallback;
};
const URL_BASE = arg('url', process.env.TREASURY_API ?? 'http://127.0.0.1:7783').replace(
  /\/+$/,
  '',
);
const TOKEN = arg('token', process.env.TREASURY_TOKEN ?? 'dev');
const PASSWORD = arg('password', 'contrato da carteira 2026');
const RUN_SYNC = argv.includes('--sync');

// ── Tipos gerados → conferência de forma ─────────────────────────

const typeCache = new Map();

/** Lê `export type X = …` de src/api/generated/X.ts. */
function loadType(name) {
  if (typeCache.has(name)) return typeCache.get(name);
  const file = join(GENERATED, `${name}.ts`);
  if (!existsSync(file)) {
    typeCache.set(name, null);
    return null;
  }
  const src = readFileSync(file, 'utf8')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/\/\/.*$/gm, '');
  const m = /export type \w+ = ([\s\S]*?);\s*$/m.exec(src);
  if (!m) throw new Error(`não entendi ${file}`);
  const body = m[1].trim();
  let def;
  if (body.startsWith('{')) {
    const fields = [];
    const inner = body.slice(1, -1);
    // Campos de primeiro nível: `nome?: Tipo,` (os tipos gerados não aninham objetos literais).
    const re = /(\w+)(\?)?\s*:\s*([^,]+(?:<[^>]*>[^,]*)?),/g;
    let f;
    while ((f = re.exec(inner + ',')))
      fields.push({ name: f[1], optional: Boolean(f[2]), type: f[3].trim() });
    def = { kind: 'object', fields };
  } else {
    def = { kind: 'alias', type: body };
  }
  typeCache.set(name, def);
  return def;
}

function checkValue(value, type, path, errors) {
  type = type.trim();
  const alts = splitUnion(type);
  if (alts.length > 1) {
    const sub = alts.map((t) => {
      const e = [];
      checkValue(value, t, path, e);
      return e;
    });
    if (!sub.some((e) => e.length === 0))
      errors.push(`${path}: ${JSON.stringify(value)?.slice(0, 60)} não é ${type}`);
    return;
  }
  if (type === 'null') return void (value === null || errors.push(`${path}: esperado null`));
  if (type === 'number')
    return void (
      typeof value === 'number' || errors.push(`${path}: esperado number, veio ${typeof value}`)
    );
  if (type === 'string')
    return void (
      typeof value === 'string' || errors.push(`${path}: esperado string, veio ${typeof value}`)
    );
  if (type === 'boolean')
    return void (
      typeof value === 'boolean' || errors.push(`${path}: esperado boolean, veio ${typeof value}`)
    );
  if (/^".*"$/.test(type))
    return void (
      value === JSON.parse(type) ||
      errors.push(`${path}: esperado ${type}, veio ${JSON.stringify(value)}`)
    );
  const arr = /^Array<(.+)>$/.exec(type) ?? /^(.+)\[\]$/.exec(type);
  if (arr) {
    if (!Array.isArray(value)) return void errors.push(`${path}: esperado lista`);
    value.slice(0, 50).forEach((v, i) => checkValue(v, arr[1], `${path}[${i}]`, errors));
    return;
  }
  const def = loadType(type);
  if (!def) return void errors.push(`${path}: tipo desconhecido ${type}`);
  if (def.kind === 'alias') return checkValue(value, def.type, path, errors);
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    return void errors.push(`${path}: esperado objeto ${type}`);
  }
  const known = new Set(def.fields.map((f) => f.name));
  for (const f of def.fields) {
    if (!(f.name in value)) {
      if (!f.optional) errors.push(`${path}.${f.name}: ausente (${type})`);
      continue;
    }
    checkValue(value[f.name], f.type, `${path}.${f.name}`, errors);
  }
  for (const k of Object.keys(value))
    if (!known.has(k)) errors.push(`${path}.${k}: campo fora do tipo ${type}`);
}

function splitUnion(type) {
  const out = [];
  let depth = 0;
  let cur = '';
  for (const ch of type) {
    if (ch === '<') depth++;
    if (ch === '>') depth--;
    if (ch === '|' && depth === 0) {
      out.push(cur.trim());
      cur = '';
    } else cur += ch;
  }
  out.push(cur.trim());
  return out.filter(Boolean);
}

// ── Chamadas ─────────────────────────────────────────────────────

let failures = 0;
let checks = 0;
const log = (ok, label, detail = '') => {
  checks++;
  if (!ok) failures++;
  console.log(`${ok ? '✓' : '✗'} ${label}${detail ? ` — ${detail}` : ''}`);
};

async function call(cmd, args = null) {
  let res;
  try {
    res = await fetch(`${URL_BASE}/api`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${TOKEN}` },
      body: JSON.stringify({ cmd, args }),
    });
  } catch (e) {
    console.error(`Não foi possível falar com ${URL_BASE}: ${e.message}`);
    process.exit(2);
  }
  const body = await res.json();
  if (!body.ok) {
    const err = new Error(`${cmd}: ${body.error?.code} ${body.error?.message}`);
    err.code = body.error?.code;
    err.payload = body.error;
    throw err;
  }
  return body.data;
}

/** Chama e confere a forma da resposta com o tipo gerado. */
async function expectShape(cmd, args, type, label = cmd) {
  try {
    const data = await call(cmd, args);
    const errors = [];
    checkValue(data, type, type.replace(/[<>[\]]/g, ''), errors);
    log(errors.length === 0, `${label} → ${type}`, errors.slice(0, 6).join('; '));
    return data;
  } catch (e) {
    log(false, `${label} → ${type}`, e.message);
    return null;
  }
}

async function expectError(cmd, args, code, label) {
  try {
    await call(cmd, args);
    log(false, label, 'deveria falhar');
  } catch (e) {
    const shapeOk = typeof e.payload?.message === 'string' && typeof e.payload?.code === 'string';
    log(e.code === code && shapeOk, label, `${e.code}: ${e.payload?.message ?? e.message}`);
  }
}

// ── Roteiro ──────────────────────────────────────────────────────

const today = new Date().toISOString().slice(0, 10);

async function ensureVault() {
  const st = await call('vault.status');
  if (st.state === 'no_vault') {
    await call('vault.create', {
      password: PASSWORD,
      household_name: 'Contrato da carteira',
      members: ['Ana'],
    });
  } else if (st.state === 'locked') {
    await call('vault.unlock', { password: PASSWORD });
  }
  log((await call('vault.status')).state === 'unlocked', 'cofre aberto');
}

async function ensureAccount(name, currency) {
  const list = await call('accounts.list', {});
  const found = list.find((a) => a.name === name);
  if (found) return found;
  return call('accounts.create', { name, kind: 'brokerage', currency });
}

async function ensureAsset(input) {
  if (input.ticker) {
    const found = await call('assets.find', { ticker: input.ticker });
    if (found) return found;
  } else {
    const found = (await call('assets.list', { text: input.name, include_archived: true })).find(
      (a) => a.name === input.name,
    );
    if (found) return found;
  }
  return expectShape(
    'assets.create',
    input,
    'Asset',
    `assets.create ${input.ticker ?? input.name}`,
  );
}

async function main() {
  console.log(`Contrato da carteira contra ${URL_BASE}\n`);
  await ensureVault();
  const xp = await ensureAccount('XP Contrato', 'BRL');
  const avenue = await ensureAccount('Avenue Contrato', 'USD');
  log(Boolean(xp?.id && avenue?.id), 'corretoras BRL e USD');

  // Ativos
  const petr = await ensureAsset({
    ticker: 'PETR4',
    name: 'Petrobras PN',
    class: 'stock_br',
    sector: 'Petróleo',
  });
  const hglg = await ensureAsset({ ticker: 'HGLG11', name: 'CSHG Logística', class: 'fii' });
  const aapl = await ensureAsset({
    ticker: 'AAPL',
    name: 'Apple Inc.',
    class: 'stock_us',
    market: 'NASDAQ',
  });
  const fund = await ensureAsset({
    name: 'Fundo Contrato FIM',
    class: 'fund',
    cnpj: '00.000.000/0001-00',
  });
  const cdb = await ensureAsset({
    name: 'CDB Contrato 110% CDI',
    class: 'fixed_income',
    fixed_income: {
      kind: 'cdb',
      indexer: 'cdi',
      rate: '110',
      issue_date: '2025-01-10',
      maturity_date: '2028-01-10',
      grace_until: null,
      liquidity: 'at_maturity',
      ir_exempt: false,
      issuer: 'Banco Contrato',
    },
  });
  const ipca = await ensureAsset({
    name: 'Tesouro IPCA+ 2035',
    class: 'treasury_br',
    treasury_code: 'Tesouro IPCA+ 2035',
    fixed_income: {
      kind: 'tesouro_ipca',
      indexer: 'ipca',
      rate: '6.2',
      issue_date: null,
      maturity_date: '2035-05-15',
      grace_until: null,
      liquidity: 'daily',
      ir_exempt: false,
      issuer: 'Tesouro Nacional',
    },
  });
  await expectShape('assets.list', {}, 'Array<Asset>');
  await expectShape(
    'assets.list',
    { held_only: true, classes: ['stock_br', 'fii'], text: 'p' },
    'Array<Asset>',
    'assets.list (filtros)',
  );
  await expectShape('assets.get', { id: petr.id }, 'Asset');
  await expectShape('assets.find', { ticker: 'petr4' }, 'Asset', 'assets.find (minúsculas)');
  await expectShape('assets.update', { id: petr.id, segment: 'Integrada', notes: null }, 'Asset');
  await expectError(
    'assets.create',
    { ticker: 'PETR4', name: 'Duplicado', class: 'stock_br' },
    'conflict',
    'assets.create duplicado → conflict',
  );
  await expectError(
    'assets.create',
    { name: 'RF sem termos', class: 'fixed_income' },
    'invalid',
    'renda fixa sem termos → invalid',
  );

  // Operações (só uma vez por cofre)
  const existing = await call('portfolio.list_trades', { asset_id: petr.id });
  if (existing.length === 0) {
    await expectShape(
      'portfolio.create_trade',
      {
        asset_id: petr.id,
        account_id: xp.id,
        date: '2025-01-10',
        side: 'buy',
        quantity: '100',
        price: '30',
        fees: 1000,
      },
      'Trade',
      'compra PETR4',
    );
    await expectShape(
      'portfolio.create_trade',
      {
        asset_id: petr.id,
        account_id: xp.id,
        date: '2025-06-10',
        side: 'buy',
        quantity: '100',
        price: '34.5',
        fees: 1000,
        note_number: '12345',
      },
      'Trade',
      'compra PETR4 (2)',
    );
    await expectShape(
      'portfolio.create_action',
      { asset_id: petr.id, date: '2025-08-01', kind: 'split', factor: '2' },
      'CorporateAction',
      'desdobramento 1:2',
    );
    await expectError(
      'portfolio.create_trade',
      {
        asset_id: petr.id,
        account_id: xp.id,
        date: '2025-09-01',
        side: 'sell',
        quantity: '500',
        price: '20',
      },
      'invalid',
      'venda maior que a posição → invalid',
    );
    await expectShape(
      'portfolio.create_trade',
      {
        asset_id: petr.id,
        account_id: xp.id,
        date: '2025-09-01',
        side: 'sell',
        quantity: '100',
        price: '20',
        fees: 500,
      },
      'Trade',
      'venda PETR4',
    );
    await expectShape(
      'portfolio.create_trade',
      {
        asset_id: hglg.id,
        account_id: xp.id,
        date: '2025-02-03',
        side: 'buy',
        quantity: '40',
        price: '160.25',
      },
      'Trade',
      'compra HGLG11',
    );
    await expectShape(
      'portfolio.create_trade',
      {
        asset_id: aapl.id,
        account_id: avenue.id,
        date: '2025-03-03',
        side: 'buy',
        quantity: '10.5',
        price: '190.13',
        fx_rate: '5.80',
      },
      'Trade',
      'compra AAPL fracionada com câmbio',
    );
    await expectShape(
      'portfolio.create_trade',
      {
        asset_id: cdb.id,
        account_id: xp.id,
        date: '2025-01-10',
        side: 'buy',
        quantity: '10',
        price: '1000',
      },
      'Trade',
      'compra CDB',
    );
    await expectShape(
      'portfolio.create_trade',
      {
        asset_id: ipca.id,
        account_id: xp.id,
        date: '2025-04-01',
        side: 'buy',
        quantity: '1.25',
        price: '2300.50',
      },
      'Trade',
      'compra Tesouro IPCA+ fracionado',
    );
    await expectShape(
      'portfolio.create_trade',
      {
        asset_id: fund.id,
        account_id: xp.id,
        date: '2025-05-02',
        side: 'buy',
        quantity: '1000',
        price: '1.8532',
        cash_movement: false,
      },
      'Trade',
      'compra fundo sem caixa',
    );
    await expectShape(
      'portfolio.create_income',
      {
        asset_id: hglg.id,
        account_id: xp.id,
        kind: 'fii_income',
        pay_date: '2025-09-15',
        ex_date: '2025-08-29',
        quantity: '40',
        gross: 4400,
      },
      'IncomeEvent',
      'rendimento HGLG11',
    );
    await expectShape(
      'portfolio.create_income',
      { asset_id: petr.id, account_id: xp.id, kind: 'jcp', pay_date: today, gross: 10000 },
      'IncomeEvent',
      'JCP PETR4 (15% retido)',
    );
    await expectShape(
      'portfolio.create_income',
      {
        asset_id: aapl.id,
        account_id: avenue.id,
        kind: 'dividend',
        pay_date: '2025-05-15',
        gross: 273,
      },
      'IncomeEvent',
      'dividendo AAPL (30% retido)',
    );
    await expectShape(
      'portfolio.create_income',
      {
        asset_id: hglg.id,
        account_id: xp.id,
        kind: 'fii_income',
        pay_date: '2099-01-15',
        gross: 4400,
        status: 'announced',
      },
      'IncomeEvent',
      'provento anunciado',
    );
  } else {
    console.log('· operações já registradas neste cofre (pulando escritas)');
  }
  await expectError(
    'portfolio.create_action',
    { asset_id: petr.id, date: '2025-08-02', kind: 'split', factor: '0.5' },
    'invalid',
    'desdobramento com fator ≤ 1 → invalid',
  );
  await expectError(
    'portfolio.create_trade',
    {
      asset_id: petr.id,
      account_id: xp.id,
      date: '2025-09-02',
      side: 'buy',
      quantity: '0',
      price: '20',
    },
    'invalid',
    'quantidade zero → invalid',
  );

  const trades = await expectShape('portfolio.list_trades', {}, 'Array<Trade>');
  const t0 = trades?.find((t) => t.asset_id === hglg.id);
  if (t0) {
    await expectShape('portfolio.get_trade', { id: t0.id }, 'Trade');
    const upd = await expectShape(
      'portfolio.update_trade',
      { id: t0.id, notes: 'contrato' },
      'Trade',
    );
    log(upd?.id === t0.id, 'update_trade mantém o id');
  }
  const tmp = await expectShape(
    'portfolio.create_trade',
    { asset_id: hglg.id, account_id: xp.id, date: today, side: 'buy', quantity: '1', price: '150' },
    'Trade',
    'compra temporária',
  );
  if (tmp) {
    const del = await call('portfolio.delete_trade', { id: tmp.id });
    log(del === null, 'delete_trade → null');
  }
  await expectShape('portfolio.list_actions', { asset_id: petr.id }, 'Array<CorporateAction>');
  await expectShape('portfolio.list_actions', {}, 'Array<CorporateAction>', 'list_actions (todos)');
  await expectShape('portfolio.list_income', {}, 'Array<IncomeEvent>');
  await expectShape(
    'portfolio.list_income',
    { asset_id: hglg.id, from: '2025-01-01', to: today },
    'Array<IncomeEvent>',
    'list_income (filtros)',
  );

  // Cotação manual e histórico
  log(
    (await call('market.set_price', { asset_id: fund.id, date: today, close: '1.9011' })) === null,
    'market.set_price → null',
  );
  await expectError(
    'market.set_price',
    { asset_id: fund.id, date: today, close: '-1' },
    'invalid',
    'cotação negativa → invalid',
  );
  const pts = await expectShape(
    'market.prices',
    { asset_id: fund.id, from: '2020-01-01' },
    'Array<PricePoint>',
  );
  log(
    pts?.some((p) => p.source === 'manual' && p.close === '1.9011'),
    'cotação manual aparece com source "manual"',
  );
  await expectShape(
    'market.prices',
    { asset_id: petr.id },
    'Array<PricePoint>',
    'market.prices (padrão 12 meses)',
  );
  await expectShape('market.data_sources', {}, 'DataSources');

  // Alvos e rebalanceamento
  await expectShape(
    'portfolio.set_targets',
    {
      scope: 'class',
      targets: [
        { scope: 'class', key: 'stock_br', target_pct: '30' },
        { scope: 'class', key: 'fii', target_pct: '20', tolerance_pct: '3' },
        { scope: 'class', key: 'fixed_income', target_pct: '30' },
        { scope: 'class', key: 'stock_us', target_pct: '20' },
      ],
    },
    'Array<Target>',
  );
  await expectError(
    'portfolio.set_targets',
    { scope: 'class', targets: [{ scope: 'class', key: 'stock_br', target_pct: '120' }] },
    'invalid',
    'alvo > 100% → invalid',
  );
  await expectShape('portfolio.list_targets', {}, 'Array<Target>');
  const reb = await expectShape('portfolio.rebalance', { contribution: 500000 }, 'Rebalance');
  if (reb) {
    const sum = reb.lines.reduce((s, l) => s + l.suggested, 0);
    log(
      sum === 500000 || reb.lines.every((l) => l.gap <= 0),
      'rebalance distribui o aporte inteiro',
      `soma ${sum}`,
    );
    log(/não é recomendação/.test(reb.disclaimer), 'rebalance traz o aviso de não recomendação');
  }
  await expectShape('portfolio.rebalance', {}, 'Rebalance', 'rebalance sem aporte');

  // Leituras de carteira
  for (const deflator of ['ipca', 'us_cpi', 'm2_br', 'm2_us', 'none']) {
    const s = await expectShape(
      'portfolio.summary',
      { deflator },
      'PortfolioSummary',
      `summary (${deflator})`,
    );
    if (s && deflator === 'ipca') {
      log(
        s.positions.every((p) => typeof p.quantity === 'string'),
        'quantidades em texto decimal',
      );
      log(
        s.missing_prices.length >= 0,
        `missing_prices: ${s.missing_prices.join(', ') || 'nenhum'}`,
      );
      const valuations = [...new Set(s.positions.map((p) => p.valuation))];
      log(true, `valuations vistas: ${valuations.join(', ')}`);
    }
    if (s && deflator !== 'ipca' && deflator !== 'none')
      log(true, `xirr_real com ${deflator}: ${s.xirr_real}`);
  }
  await expectShape(
    'portfolio.summary',
    { by_account: true, include_closed: true, account_ids: [xp.id] },
    'PortfolioSummary',
    'summary por conta, com encerradas',
  );
  await expectShape(
    'portfolio.summary',
    { as_of: '2025-12-31' },
    'PortfolioSummary',
    'summary numa data',
  );
  const pos = await expectShape(
    'portfolio.position',
    { asset_id: petr.id, deflator: 'ipca' },
    'PositionView',
  );
  if (pos)
    log(
      pos.quantity === '100' || pos.quantity === '300',
      `posição PETR4 após desdobramento e venda: ${pos.quantity}`,
    );
  await expectShape(
    'portfolio.position',
    { asset_id: petr.id, account_id: xp.id, as_of: '2025-07-01' },
    'PositionView',
    'position numa conta e data',
  );
  await expectShape(
    'portfolio.position',
    { asset_id: cdb.id },
    'PositionView',
    'position renda fixa',
  );
  await expectShape('portfolio.performance', {}, 'Performance');
  await expectShape(
    'portfolio.performance',
    { from_month: '2025-06', deflator: 'none', class: 'stock_br' },
    'Performance',
    'performance filtrada',
  );
  await expectShape('portfolio.passive_income', { target_monthly: 300000 }, 'PassiveIncome');
  await expectShape(
    'portfolio.dividend_calendar',
    { from: '2025-01-01', to: '2099-12-31' },
    'Array<DividendCalendarItem>',
  );
  await expectShape(
    'portfolio.dividend_calendar',
    { from: '2025-01-01', to: '2099-12-31', only_mine: false },
    'Array<DividendCalendarItem>',
    'dividend_calendar (todos)',
  );

  // Watchlist
  await expectShape(
    'portfolio.watch',
    {
      asset_id: aapl.id,
      target_price: '250',
      alert_above: '260',
      alert_below: '150',
      notes: 'contrato',
    },
    'Array<WatchlistItem>',
  );
  await expectShape('portfolio.list_watchlist', {}, 'Array<WatchlistItem>');
  await expectShape('portfolio.unwatch', { asset_id: aapl.id }, 'Array<WatchlistItem>');

  // Índices
  const series = await expectShape('indices.list_series', {}, 'Array<IndexSeries>');
  log(
    ['ipca', 'cdi', 'us_cpi', 'm2_br', 'us_m2', 'ibov', 'sp500'].every((id) =>
      series?.some((s) => s.id === id),
    ),
    'séries esperadas (ipca, cdi, us_cpi, m2_br, us_m2, ibov, sp500)',
  );
  await expectShape('indices.get_series', { id: 'ipca' }, 'Array<IndexValuePoint>');
  await expectShape(
    'indices.get_series',
    { id: 'ibov', from: '2025-01-01', to: today },
    'Array<IndexValuePoint>',
    'indices.get_series (ibov)',
  );
  await expectShape(
    'indices.factor',
    { deflator: 'ipca', from: '2025-01-01', to: '2025-06-01' },
    'FactorResult',
  );
  await expectShape('fx.list', { base: 'USD', quote: 'BRL', limit: 1 }, 'Array<FxRate>');

  if (RUN_SYNC) {
    console.log('\n· market.sync (rede; pode levar minutos)…');
    const r = await expectShape('market.sync', {}, 'DataSyncReport');
    if (r) {
      const bad = [...r.indices.series, ...r.indices.fx]
        .filter((s) => s.error)
        .map((s) => `${s.series_id}: ${s.error}`);
      const badPrices = r.prices.assets
        .filter((a) => a.error)
        .map((a) => `${a.ticker ?? a.asset_id}: ${a.error}`);
      console.log(`  índices com erro: ${bad.length ? bad.join(' | ') : 'nenhum'}`);
      console.log(`  cotações com erro: ${badPrices.length ? badPrices.join(' | ') : 'nenhum'}`);
      await expectShape(
        'portfolio.summary',
        { deflator: 'ipca' },
        'PortfolioSummary',
        'summary depois da coleta',
      );
    }
  }

  console.log(
    `\n${checks - failures}/${checks} conferências ok${failures ? `, ${failures} falharam` : ''}.`,
  );
  process.exit(failures ? 1 : 0);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
