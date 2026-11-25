/**
 * Mock de `b3.*` (importação da Área do Investidor B3).
 *
 * O tipo de relatório vem do nome do arquivo ("negociacao", "movimentacao", "posicao")
 * ou do cabeçalho do CSV. O conteúdo é um conjunto fixo e realista de linhas (três
 * instituições: XP, Inter e Nu Invest, esta sem conta no seed), mas a situação de cada
 * linha é calculada contra a carteira do mock: conta escolhida, duplicada (já importada),
 * possível duplicada (operação manual igual), venda maior que a posição, evento sem
 * fator. O commit grava pela carteira e pelos ativos do mock, num conjunto de alterações
 * que `changes.undo` desfaz.
 */
import type { AssetClass } from '../generated/AssetClass';
import type { B3AccountChoice } from '../generated/B3AccountChoice';
import type { B3ActionApproval } from '../generated/B3ActionApproval';
import type { B3ActionProposal } from '../generated/B3ActionProposal';
import type { B3AssetOverride } from '../generated/B3AssetOverride';
import type { B3AssetProposal } from '../generated/B3AssetProposal';
import type { B3CommitError } from '../generated/B3CommitError';
import type { B3CommitResult } from '../generated/B3CommitResult';
import type { B3Counts } from '../generated/B3Counts';
import type { B3DetectResult } from '../generated/B3DetectResult';
import type { B3File } from '../generated/B3File';
import type { B3IgnoredRow } from '../generated/B3IgnoredRow';
import type { B3IncomeProposal } from '../generated/B3IncomeProposal';
import type { B3Institution } from '../generated/B3Institution';
import type { B3MatchReason } from '../generated/B3MatchReason';
import type { B3NewAccount } from '../generated/B3NewAccount';
import type { B3PositionLine } from '../generated/B3PositionLine';
import type { B3Preview } from '../generated/B3Preview';
import type { B3ReportKind } from '../generated/B3ReportKind';
import type { B3RowStatus } from '../generated/B3RowStatus';
import type { B3SheetInfo } from '../generated/B3SheetInfo';
import type { B3Source } from '../generated/B3Source';
import type { B3TradeProposal } from '../generated/B3TradeProposal';
import type { Asset } from '../generated/Asset';
import type { CorporateActionKind } from '../generated/CorporateActionKind';
import type { IncomeEvent } from '../generated/IncomeEvent';
import type { IncomeKind } from '../generated/IncomeKind';
import type { PortfolioSummary } from '../generated/PortfolioSummary';
import type { Trade } from '../generated/Trade';
import type { TradeSide } from '../generated/TradeSide';
import { mockFail, onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { today } from './ledger/dates';
import {
  cents,
  custodyAccounts,
  fold,
  heldQuantity,
  invoke,
  recordImport,
} from './ledger/investImport';

const XP = 'XP INVESTIMENTOS CCTVM S/A';
const INTER = 'INTER DISTRIBUIDORA DE TITULOS E VALORES MOBILIARIOS LTDA';
const NU = 'NU INVEST CORRETORA DE VALORES S.A.';

/** Nomes de produto como a B3 escreve. */
const PRODUCT: Record<string, string> = {
  PETR4: 'PETR4 - PETROLEO BRASILEIRO S.A. PETROBRAS',
  HGLG11: 'HGLG11 - CSHG LOGÍSTICA FUNDO DE INVESTIMENTO IMOBILIARIO - FII',
  KLBN11: 'KLBN11 - KLABIN S.A.',
  MGLU3: 'MGLU3 - MAGAZINE LUIZA S.A.',
  BBAS3: 'BBAS3 - BANCO DO BRASIL S.A.',
  WEGE3: 'WEGE3 - WEG S.A.',
  ITUB4: 'ITUB4 - ITAU UNIBANCO HOLDING S.A.',
};

interface TradeRow {
  date: string;
  institution: string;
  ticker: string;
  side: TradeSide;
  quantity: number;
  price: number;
  line: number;
}

const TRADE_ROWS: TradeRow[] = [
  // Igual a uma compra registrada à mão no seed (mesma data, ativo e quantidade).
  {
    date: '2026-03-10',
    institution: XP,
    ticker: 'PETR4',
    side: 'buy',
    quantity: 100,
    price: 36.9,
    line: 2,
  },
  {
    date: '2026-09-03',
    institution: XP,
    ticker: 'PETR4',
    side: 'buy',
    quantity: 100,
    price: 38.12,
    line: 3,
  },
  {
    date: '2026-09-10',
    institution: INTER,
    ticker: 'HGLG11',
    side: 'buy',
    quantity: 10,
    price: 160.2,
    line: 4,
  },
  {
    date: '2026-09-15',
    institution: XP,
    ticker: 'KLBN11',
    side: 'buy',
    quantity: 200,
    price: 21.4,
    line: 5,
  },
  // A XP não tem MGLU3 (foi vendida na Inter): venda maior que a posição.
  {
    date: '2026-09-22',
    institution: XP,
    ticker: 'MGLU3',
    side: 'sell',
    quantity: 300,
    price: 9.1,
    line: 6,
  },
  {
    date: '2026-09-24',
    institution: NU,
    ticker: 'BBAS3',
    side: 'buy',
    quantity: 100,
    price: 27.8,
    line: 7,
  },
  {
    date: '2026-10-01',
    institution: INTER,
    ticker: 'WEGE3',
    side: 'sell',
    quantity: 50,
    price: 51.3,
    line: 8,
  },
];

interface IncomeRow {
  date: string;
  institution: string;
  ticker: string;
  kind: IncomeKind;
  movement: string;
  quantity: number;
  /** Líquido, em reais (como a B3 informa). */
  net: number;
  line: number;
}

const INCOME_ROWS: IncomeRow[] = [
  {
    date: '2026-09-14',
    institution: XP,
    ticker: 'HGLG11',
    kind: 'fii_income',
    movement: 'Rendimento',
    quantity: 75,
    net: 82.5,
    line: 3,
  },
  {
    date: '2026-09-20',
    institution: XP,
    ticker: 'PETR4',
    kind: 'dividend',
    movement: 'Dividendo',
    quantity: 400,
    net: 248,
    line: 5,
  },
  {
    date: '2026-10-01',
    institution: XP,
    ticker: 'ITUB4',
    kind: 'jcp',
    movement: 'Juros Sobre Capital Próprio',
    quantity: 400,
    net: 6.8,
    line: 9,
  },
  {
    date: '2026-09-29',
    institution: NU,
    ticker: 'BBAS3',
    kind: 'jcp',
    movement: 'Juros Sobre Capital Próprio',
    quantity: 100,
    net: 17.85,
    line: 8,
  },
];

interface ActionRow {
  date: string;
  institution: string;
  ticker: string;
  kind: CorporateActionKind;
  movement: string;
  quantity: number;
  unitCost: string;
  line: number;
}

const ACTION_ROWS: ActionRow[] = [
  {
    date: '2026-09-26',
    institution: INTER,
    ticker: 'WEGE3',
    kind: 'bonus',
    movement: 'Bonificação em Ativos',
    quantity: 16,
    unitCost: '9.15',
    line: 7,
  },
];

const SHEET_TRADES = 'Negociação';
const SHEET_MOVES = 'Movimentação';
const SHEET_POSITION = 'Ações';

function kindsOf(file: B3File): B3ReportKind[] {
  const name = fold(file.filename);
  const kinds: B3ReportKind[] = [];
  if (/negocia/.test(name)) kinds.push('trades');
  if (/moviment/.test(name)) kinds.push('movements');
  if (/posic/.test(name)) kinds.push('position');
  if (kinds.length) return kinds;
  let head: string;
  try {
    head = fold(atob(file.content_base64).slice(0, 400));
  } catch {
    head = '';
  }
  if (head.includes('data do negocio')) kinds.push('trades');
  if (head.includes('entrada/saida') && head.includes('movimentacao')) kinds.push('movements');
  if (head.includes('preco de fechamento')) kinds.push('position');
  return kinds;
}

function formatOf(filename: string): 'xlsx' | 'csv' {
  return /\.csv$/i.test(filename) ? 'csv' : 'xlsx';
}

function checkFile(file: B3File): void {
  if (!file.filename?.trim()) mockFail('invalid', 'informe o nome do arquivo');
  if (!/\.(xlsx|csv)$/i.test(file.filename))
    mockFail(
      'invalid',
      'formato não reconhecido: envie o Excel (.xlsx) ou o CSV da Área do Investidor',
    );
}

function sheetsOf(file: B3File, kinds: readonly B3ReportKind[]): B3SheetInfo[] {
  const sheet = (name: string, kind: B3ReportKind, headers: string[], rows: string[][]) => ({
    name: formatOf(file.filename) === 'csv' ? file.filename : name,
    kind,
    position_class: kind === 'position' ? ('stock_br' as AssetClass) : null,
    headers,
    header_line: 1,
    row_count: rows.length,
    sample_rows: rows.slice(0, 5),
  });
  const out: B3SheetInfo[] = [];
  if (kinds.includes('trades'))
    out.push(
      sheet(
        SHEET_TRADES,
        'trades',
        [
          'Data do Negócio',
          'Tipo de Movimentação',
          'Mercado',
          'Instituição',
          'Código de Negociação',
          'Quantidade',
          'Preço',
          'Valor',
        ],
        TRADE_ROWS.map((t) => [
          t.date.split('-').reverse().join('/'),
          t.side === 'buy' ? 'Compra' : 'Venda',
          'Mercado à Vista',
          t.institution,
          t.ticker,
          String(t.quantity),
          t.price.toFixed(2).replace('.', ','),
          (t.quantity * t.price).toFixed(2).replace('.', ','),
        ]),
      ),
    );
  if (kinds.includes('movements'))
    out.push(
      sheet(
        SHEET_MOVES,
        'movements',
        [
          'Entrada/Saída',
          'Data',
          'Movimentação',
          'Produto',
          'Instituição',
          'Quantidade',
          'Preço unitário',
          'Valor da Operação',
        ],
        INCOME_ROWS.map((i) => [
          'Credito',
          i.date.split('-').reverse().join('/'),
          i.movement,
          PRODUCT[i.ticker] ?? i.ticker,
          i.institution,
          String(i.quantity),
          '-',
          i.net.toFixed(2).replace('.', ','),
        ]),
      ),
    );
  if (kinds.includes('position'))
    out.push(
      sheet(
        SHEET_POSITION,
        'position',
        [
          'Produto',
          'Instituição',
          'Conta',
          'Código de Negociação',
          'Quantidade',
          'Preço de Fechamento',
          'Valor Atualizado',
        ],
        [[PRODUCT.PETR4!, XP, '123456', 'PETR4', '400', '38,40', '15.360,00']],
      ),
    );
  if (!out.length)
    out.push({
      name: formatOf(file.filename) === 'csv' ? file.filename : 'Planilha1',
      kind: null,
      position_class: null,
      headers: ['Coluna A', 'Coluna B'],
      header_line: null,
      row_count: 2,
      sample_rows: [['texto', '1']],
    });
  return out;
}

function institutionsOf(kinds: readonly B3ReportKind[]): string[] {
  const names = new Set<string>();
  if (kinds.includes('trades') || kinds.includes('movements'))
    for (const t of TRADE_ROWS) names.add(t.institution);
  if (kinds.includes('movements')) for (const i of INCOME_ROWS) names.add(i.institution);
  if (kinds.includes('position')) [XP, INTER, NU].forEach((n) => names.add(n));
  return [...names];
}

function periodOf(kinds: readonly B3ReportKind[]): [string | null, string | null] {
  const dates = [
    ...(kinds.includes('trades') || kinds.includes('movements')
      ? TRADE_ROWS.map((t) => t.date)
      : []),
    ...(kinds.includes('movements') ? [...INCOME_ROWS, ...ACTION_ROWS].map((i) => i.date) : []),
  ].sort();
  return [dates[0] ?? null, dates[dates.length - 1] ?? null];
}

function detect(file: B3File): B3DetectResult {
  checkFile(file);
  const kinds = kindsOf(file);
  const [from, to] = periodOf(kinds);
  return {
    filename: file.filename.trim(),
    format: formatOf(file.filename),
    encoding: formatOf(file.filename) === 'csv' ? 'UTF-8' : null,
    sheets: sheetsOf(file, kinds),
    kinds,
    institutions: institutionsOf(kinds),
    period_from: from,
    period_to: to,
    warnings: kinds.length
      ? []
      : [
          'nenhuma aba com relatório da Área do Investidor B3 (negociação, movimentação ou posição)',
        ],
  };
}

// ── Prévia ───────────────────────────────────────────────────────

const INSTITUTION_STOPWORDS = new Set(
  's a sa ltda cctvm ctvm dtvm cvm corretora de do da dos das e valores mobiliarios titulos cambio investimentos investimento invest banco securities financeira distribuidora cooperativa credito unibanco pagamentos ip scfi'.split(
    ' ',
  ),
);

const tokensOf = (name: string) =>
  fold(name)
    .split(/[^a-z0-9]+/)
    .filter((w) => w && !INSTITUTION_STOPWORDS.has(w));

/** Renamed brokers: the names of one custodian (same group as in the core). */
const groupOf = (name: string) =>
  /^nu invest|^easynvest/.test(fold(name)) ? 'known:nu-invest' : `name:${tokensOf(name).join(' ')}`;

/** Custodian → account saved by earlier imports (mock: kept in memory). */
let savedMapping = new Map<string, string>();
onMockReset(() => {
  savedMapping = new Map();
});

/** Conta sugerida: mapping saved before, else a word of the institution in the account's name (prefix counts). */
function suggestAccount(
  institution: string,
  listed = true,
): { id: string; reason: B3MatchReason } | null {
  // Listed assets go only to a brokerage account (as in the core).
  const accounts = custodyAccounts().filter(
    (a) => !listed || a.kind === 'brokerage' || a.kind === 'foreign',
  );
  const saved = savedMapping.get(groupOf(institution));
  if (saved && accounts.some((a) => a.id === saved)) return { id: saved, reason: 'saved' };
  const tokens = tokensOf(institution);
  const scored = accounts
    .map((a) => {
      const words = fold(`${a.name} ${a.institution_name ?? ''}`).split(/[^a-z0-9]+/);
      const score = tokens.reduce(
        (n, t) =>
          n +
          (words.includes(t) ? 10 : t.length >= 2 && words.some((w) => w.startsWith(t)) ? 6 : 0),
        0,
      );
      return { a, score: score * 2 + (a.kind === 'brokerage' ? 1 : 0) };
    })
    .filter((x) => x.score > 1)
    .sort((x, y) => y.score - x.score);
  if (!scored.length || (scored[1] && scored[1].score === scored[0]!.score)) return null;
  return { id: scored[0]!.a.id, reason: 'similar_name' };
}

/** The account offered when nothing matches. */
function proposedAccount(name: string, from: string | null): B3NewAccount {
  const f = fold(name);
  const broker = /corretora|ctvm|dtvm|invest/.test(f);
  const tail = name.split(' - ').pop()?.trim() ?? '';
  const raw =
    groupOf(name) === 'known:nu-invest'
      ? 'Nu Invest'
      : !tail.includes(' ') && tail !== name
        ? tail
        : (name.split(/[^\p{L}\p{N}]+/u).find((w) => w && !INSTITUTION_STOPWORDS.has(fold(w))) ??
          name);
  const short =
    raw === 'Nu Invest' ? raw : raw.charAt(0).toUpperCase() + raw.slice(1).toLowerCase();
  return {
    name: broker ? short : `${short} Investimentos`,
    kind: broker ? 'brokerage' : 'other',
    institution_name: short,
    institution_kind: broker ? 'broker' : 'bank',
    opening_date: from ?? undefined,
  };
}

function findAsset(ticker: string): Asset | null {
  return invoke<Asset | null>('assets.find', { ticker });
}

function classFor(
  ticker: string,
  product: string,
): { class: AssetClass; reason: string; review: boolean } {
  if (/11$/.test(ticker) && /\bfii\b|imobiliario/i.test(fold(product)))
    return { class: 'fii', reason: 'nome do produto com "FII"/"imobiliário"', review: false };
  if (/11$/.test(ticker))
    return {
      class: 'stock_br',
      reason: 'final 11 sem "FII" no nome: tratado como unit de ação',
      review: true,
    };
  return { class: 'stock_br', reason: `final ${ticker.slice(4)} no código: ação`, review: false };
}

interface Stored {
  token: string;
  filenames: string[];
  preview: B3Preview;
  /** Id do ativo existente por chave (para o commit). */
  assetIds: Map<string, string | null>;
  /** Accounts to create on commit, by the placeholder id the preview used. */
  newAccounts: { placeholder: string; spec: B3NewAccount }[];
}

let stored: Stored[] = [];
let seq = 0;
onMockReset(() => {
  stored = [];
  seq = 0;
});

const src = (filename: string, sheet: string, line: number): B3Source => ({
  filename,
  sheet,
  line,
});

function preview(args: Record<string, unknown>): B3Preview {
  const files = (Array.isArray(args.files) ? args.files : []) as B3File[];
  if (!files.length) mockFail('invalid', 'envie pelo menos um relatório da B3');
  files.forEach(checkFile);
  const choices = (Array.isArray(args.accounts) ? args.accounts : []) as B3AccountChoice[];
  const fileKinds = files.map((f) => ({ f, kinds: kindsOf(f) }));
  const all = new Set(fileKinds.flatMap((x) => x.kinds));
  if (!all.size) mockFail('invalid', 'nenhum relatório da B3 reconhecido nos arquivos enviados');
  const fileFor = (k: B3ReportKind) => fileKinds.find((x) => x.kinds.includes(k))?.f.filename ?? '';
  const hasTrades = all.has('trades');
  const hasMoves = all.has('movements');
  const hasPosition = all.has('position');
  const accountsList = custodyAccounts();
  const newAccounts: { placeholder: string; spec: B3NewAccount }[] = [];
  const accountName = (id: string | null) =>
    accountsList.find((a) => a.id === id)?.name ??
    newAccounts.find((n) => n.placeholder === id)?.spec.name ??
    null;
  const [periodFrom] = periodOf([...all]);

  // Instituições e contas.
  const instNames = institutionsOf([...all]);
  const institutions: B3Institution[] = instNames.map((name) => {
    const choice = choices.find((c) => fold(c.institution) === fold(name));
    const suggestion = suggestAccount(name);
    const suggested = suggestion?.id ?? null;
    let accountId = choice ? choice.account_id : suggested;
    if (choice?.new_account) {
      accountId = `new:${newAccounts.length + 1}`;
      newAccounts.push({ placeholder: accountId, spec: choice.new_account });
    }
    const reason: B3MatchReason | null = choice?.new_account
      ? 'created'
      : choice
        ? choice.account_id
          ? 'chosen'
          : null
        : (suggestion?.reason ?? null);
    const rows =
      (hasTrades || hasMoves ? TRADE_ROWS.filter((t) => t.institution === name).length : 0) +
      (hasMoves ? INCOME_ROWS.filter((i) => i.institution === name).length : 0) +
      (hasMoves ? ACTION_ROWS.filter((a) => a.institution === name).length : 0);
    return {
      name,
      account_id: accountId,
      account_name: accountName(accountId),
      suggested_account_id: suggested,
      from_suggestion: !choice && suggested != null,
      rows,
      match_reason: reason,
      aliases: [name],
      b3_accounts: [],
      mapped: reason === 'saved',
      proposed_new_account: !accountId && !suggested ? proposedAccount(name, periodFrom) : null,
      new_account: choice?.new_account ?? null,
      summary: {
        trades: 0,
        incomes: 0,
        actions: 0,
        period_from: null,
        period_to: null,
        assets: [],
      },
    };
  });
  const accountOf = (inst: string) => institutions.find((i) => i.name === inst)?.account_id ?? null;

  // Ativos.
  const tickers = new Set<string>();
  if (hasTrades || hasMoves) TRADE_ROWS.forEach((t) => tickers.add(t.ticker));
  if (hasMoves) [...INCOME_ROWS, ...ACTION_ROWS].forEach((r) => tickers.add(r.ticker));
  if (hasPosition) tickers.add('PETR4');
  const assetIds = new Map<string, string | null>();
  const assets: B3AssetProposal[] = [...tickers].sort().map((ticker) => {
    const existing = findAsset(ticker);
    assetIds.set(ticker, existing?.id ?? null);
    const product = PRODUCT[ticker] ?? ticker;
    const c = classFor(ticker, product);
    return {
      key: ticker,
      ticker,
      name: existing?.name ?? product.split(' - ')[1] ?? ticker,
      class: existing?.class ?? c.class,
      class_reason: existing ? 'já cadastrado' : c.reason,
      existing_asset_id: existing?.id ?? null,
      create: !existing,
      needs_review: !existing && c.review,
      note: !existing && c.review ? 'confira a classe antes de importar' : null,
      fixed_income: null,
    };
  });

  let index = 0;
  const existingTrades = invoke<Trade[]>('portfolio.list_trades', { limit: 5000 });
  const existingIncome = invoke<IncomeEvent[]>('portfolio.list_income', {});
  // Quantidade simulada por ativo e conta: a carteira mais as linhas novas desta importação.
  const pending: { assetKey: string; account: string; date: string; delta: number }[] = [];
  const simulated = (assetKey: string, account: string, date: string, assetId?: string) => {
    const id = assetId ?? assetIds.get(assetKey);
    const base = id ? heldQuantity(id, account, date) : 0;
    return (
      base +
      pending
        .filter((p) => p.assetKey === assetKey && p.account === account && p.date <= date)
        .reduce((s, p) => s + p.delta, 0)
    );
  };

  // Operações (do relatório de negociação ou, sem ele, das liquidações).
  const trades: B3TradeProposal[] = [];
  if (hasTrades || hasMoves) {
    const origin = hasTrades ? 'trades' : 'settlement';
    const filename = hasTrades ? fileFor('trades') : fileFor('movements');
    for (const t of [...TRADE_ROWS].sort((a, b) => a.date.localeCompare(b.date))) {
      const account = accountOf(t.institution);
      const assetId = assetIds.get(t.ticker) ?? null;
      const externalId = `b3:${t.date}:${t.ticker}:${t.side}:${t.quantity}:${fold(t.institution).split(' ')[0]}`;
      let status: B3RowStatus = 'new';
      let duplicateOf: string | null = null;
      let note: string | null =
        origin === 'settlement' ? 'data estimada: 2 dias úteis antes da liquidação' : null;
      let error: string | null = null;
      const same = existingTrades.find(
        (x) =>
          x.asset_id === assetId &&
          x.account_id === account &&
          x.date === t.date &&
          x.side === t.side &&
          Number(x.quantity) === t.quantity,
      );
      if (!account) {
        status = 'no_account';
        note = 'instituição sem conta escolhida';
      } else if (same?.notes?.includes(externalId)) {
        status = 'duplicate';
        duplicateOf = same.id;
        note = 'já importada da B3';
      } else if (same) {
        status = 'possible_duplicate';
        duplicateOf = same.id;
        note = 'parecida com uma operação lançada à mão (mesma data, ativo e quantidade)';
      } else if (t.side === 'sell') {
        const held = simulated(t.ticker, account, t.date);
        if (t.quantity > held + 1e-9) {
          status = 'error';
          error = `venda de ${t.quantity} maior que a posição de ${held} nesta conta na data`;
        }
      }
      if (status === 'new' && account)
        pending.push({
          assetKey: t.ticker,
          account,
          date: t.date,
          delta: t.side === 'buy' ? t.quantity : -t.quantity,
        });
      trades.push({
        index: index++,
        source: src(filename, hasTrades ? SHEET_TRADES : SHEET_MOVES, t.line),
        origin,
        date: t.date,
        asset_key: t.ticker,
        ticker: t.ticker,
        institution: t.institution,
        account_id: account,
        side: t.side,
        quantity: String(t.quantity),
        price: t.price.toFixed(2),
        gross: cents(t.quantity * t.price),
        market: 'Mercado à Vista',
        external_id: externalId,
        status,
        duplicate_of: duplicateOf,
        note,
        error,
      });
    }
  }

  // Proventos.
  const incomes: B3IncomeProposal[] = [];
  if (hasMoves) {
    for (const i of INCOME_ROWS) {
      const account = accountOf(i.institution);
      const assetId = assetIds.get(i.ticker) ?? null;
      const net = cents(i.net);
      // JCP: 15% retidos até 2025, 17,5% desde 2026 (LC 224/2025), como `gross_and_tax` do núcleo.
      const jcpRate = i.date >= '2026-01-01' ? 0.175 : 0.15;
      const gross = i.kind === 'jcp' ? Math.round(net / (1 - jcpRate)) : net;
      const same = existingIncome.find(
        (x) =>
          x.asset_id === assetId &&
          x.account_id === account &&
          x.kind === i.kind &&
          x.pay_date === i.date,
      );
      const status: B3RowStatus = !account ? 'no_account' : same ? 'duplicate' : 'new';
      incomes.push({
        index: index++,
        source: src(fileFor('movements'), SHEET_MOVES, i.line),
        pay_date: i.date,
        asset_key: i.ticker,
        ticker: i.ticker,
        institution: i.institution,
        account_id: account,
        kind: i.kind,
        movement: i.movement,
        quantity: String(i.quantity),
        gross,
        tax_withheld: gross - net,
        net,
        external_id: `b3:${i.date}:${i.ticker}:${i.kind}`,
        status,
        duplicate_of: same?.id ?? null,
        note: !account
          ? 'instituição sem conta escolhida'
          : same
            ? 'provento já registrado nesta data'
            : i.kind === 'jcp'
              ? 'JCP vem líquido da B3: bruto = líquido ÷ 0,85'
              : null,
        error: null,
      });
    }
  }

  // Eventos corporativos.
  const actions: B3ActionProposal[] = [];
  if (hasMoves) {
    for (const a of ACTION_ROWS) {
      const account = accountOf(a.institution);
      const held = account ? simulated(a.ticker, account, a.date) : null;
      actions.push({
        index: index++,
        sources: [src(fileFor('movements'), SHEET_MOVES, a.line)],
        date: a.date,
        asset_key: a.ticker,
        ticker: a.ticker,
        kind: a.kind,
        movement: a.movement,
        file_quantity: String(a.quantity),
        held_quantity: held == null ? null : String(held),
        factor: null,
        unit_cost: a.unitCost,
        status: account ? 'review' : 'no_account',
        duplicate_of: null,
        note: account
          ? `${a.quantity} ações recebidas sobre ${held} não dão um fator exato: informe o fator (ações novas por ação)`
          : 'instituição sem conta escolhida',
        error: null,
      });
    }
  }

  // Linhas ignoradas.
  const ignored: B3IgnoredRow[] = [];
  if (hasMoves) {
    const f = fileFor('movements');
    ignored.push(
      {
        source: src(f, SHEET_MOVES, 2),
        report: 'movements',
        rule: 'option',
        description: 'Opção de Compra · PETRA300 · 100',
        reason: 'opções não entram na carteira (só ações, FIIs, ETFs, BDRs, Tesouro e renda fixa)',
      },
      {
        source: src(f, SHEET_MOVES, 4),
        report: 'movements',
        rule: 'lending',
        description: 'Empréstimo · BBAS3 · 100',
        reason:
          'aluguel de ações não muda a posição; a remuneração entra como provento quando paga',
      },
      {
        source: src(f, SHEET_MOVES, 6),
        report: 'movements',
        rule: 'balance_update',
        description: 'Atualização · Tesouro IPCA+ 2035',
        reason: 'atualização de saldo do Tesouro não é operação',
      },
    );
    if (hasTrades)
      ignored.push({
        source: src(f, SHEET_MOVES, 10),
        report: 'movements',
        rule: 'settlement_duplicate',
        description: 'Transferência - Liquidação · PETR4 · 100',
        reason: 'a compra já veio do relatório de negociação (data do negócio)',
      });
  }

  // Conferência com a posição da B3.
  let position_check: B3Preview['position_check'] = null;
  if (hasPosition) {
    const date = today();
    const lines: B3PositionLine[] = [];
    const mapped = institutions.filter((i) => i.account_id);
    const summary = invoke<PortfolioSummary>('portfolio.summary', { by_account: true });
    const listed = summary.positions.filter(
      (p) =>
        ['stock_br', 'fii', 'etf_br', 'bdr'].includes(p.asset.class) &&
        mapped.some((m) => m.account_id === p.account_id) &&
        p.quantity !== '0',
    );
    const keys = new Set<string>();
    for (const p of listed) {
      const inst = mapped.find((m) => m.account_id === p.account_id)!;
      const key = p.asset.ticker ?? p.asset.name;
      keys.add(`${key}|${inst.name}`);
      const computed = simulated(key, inst.account_id!, date, p.asset.id);
      // A B3 tem 100 PETR4 a mais na XP: uma compra que não foi lançada.
      const b3 = key === 'PETR4' && inst.name === XP ? computed + 100 : computed;
      lines.push(
        positionLine(key, p.asset.name, inst, p.asset.id, b3, computed, fileFor('position')),
      );
    }
    for (const t of pending) {
      const inst = institutions.find((i) => i.account_id === t.account)!;
      if (keys.has(`${t.assetKey}|${inst.name}`)) continue;
      keys.add(`${t.assetKey}|${inst.name}`);
      const q = simulated(t.assetKey, t.account, date);
      if (q > 0)
        lines.push(
          positionLine(
            t.assetKey,
            assets.find((a) => a.key === t.assetKey)?.name ?? t.assetKey,
            inst,
            assetIds.get(t.assetKey) ?? null,
            q,
            q,
            fileFor('position'),
          ),
        );
    }
    const nu = institutions.find((i) => i.name === NU);
    if (nu && !nu.account_id)
      lines.push({
        asset_key: 'BBAS3',
        ticker: 'BBAS3',
        name: 'BANCO DO BRASIL S.A.',
        institution: NU,
        account_id: null,
        account_name: null,
        asset_id: assetIds.get('BBAS3') ?? null,
        b3_quantity: '100',
        computed_quantity: null,
        difference: null,
        close_price: '27.95',
        b3_value: 279500,
        status: 'no_account',
        source: src(fileFor('position'), SHEET_POSITION, 9),
        note: 'instituição sem conta escolhida',
        unavailable_quantity: null,
        unavailable_reason: null,
      });
    lines.sort((a, b) => a.asset_key.localeCompare(b.asset_key));
    position_check = {
      date,
      lines,
      matched: lines.filter((l) => l.status === 'match').length,
      divergent: lines.filter((l) => l.status === 'divergent' || l.status === 'missing_in_b3')
        .length,
    };
  }

  const rows = [...trades, ...incomes, ...actions];
  const count = (s: B3RowStatus) => rows.filter((r) => r.status === s).length;
  const counts: B3Counts = {
    new_assets: assets.filter((a) => a.create).length,
    trades: trades.filter((t) => t.status === 'new').length,
    incomes: incomes.filter((t) => t.status === 'new').length,
    actions: actions.filter((t) => t.status === 'new').length,
    duplicates: count('duplicate'),
    possible_duplicates: count('possible_duplicate'),
    no_account: count('no_account'),
    review: count('review'),
    errors: count('error'),
    ignored: ignored.length,
  };
  const [from, to] = periodOf([...all]);
  seq += 1;
  const token = `b3-preview-${seq}`;
  const warnings: string[] = [];
  if (!hasTrades && hasMoves)
    warnings.push(
      'sem o relatório de negociação: as compras e vendas vieram das liquidações, com a data estimada',
    );
  if (institutions.some((i) => !i.account_id))
    warnings.push('há instituições sem conta: as linhas delas não serão importadas');
  const out: B3Preview = {
    token,
    files: fileKinds.map(({ f, kinds }) => ({
      filename: f.filename,
      sha256: `mock-${fold(f.filename).replace(/[^a-z0-9]/g, '')}`,
      format: formatOf(f.filename),
      kinds,
    })),
    period_from: from,
    period_to: to,
    uses_settlements: !hasTrades && hasMoves,
    institutions,
    assets,
    trades,
    incomes,
    actions,
    ignored,
    position_check,
    openings: [],
    counts,
    warnings,
  };
  // What each custodian's account receives.
  for (const inst of institutions) {
    if (!inst.account_id) continue;
    const mine = <T extends { account_id: string | null; status: B3RowStatus }>(xs: T[]) =>
      xs.filter((x) => x.account_id === inst.account_id && x.status === 'new');
    const ts = mine(trades);
    const is = mine(incomes);
    const dates = [...ts.map((t) => t.date), ...is.map((i) => i.pay_date)].sort();
    const qty = new Map<string, number>();
    for (const t of ts)
      qty.set(
        t.asset_key,
        (qty.get(t.asset_key) ?? 0) + (t.side === 'buy' ? 1 : -1) * Number(t.quantity),
      );
    for (const i of is) if (!qty.has(i.asset_key)) qty.set(i.asset_key, 0);
    inst.summary = {
      trades: ts.length,
      incomes: is.length,
      actions: 0,
      period_from: dates[0] ?? null,
      period_to: dates[dates.length - 1] ?? null,
      assets: [...qty].map(([key, q]) => ({
        asset_key: key,
        ticker: key,
        name: assets.find((a) => a.key === key)?.name ?? key,
        before: '0',
        after: String(q),
      })),
    };
  }
  stored = [
    ...stored.slice(-3),
    { token, filenames: files.map((f) => f.filename), preview: out, assetIds, newAccounts },
  ];
  return out;
}

function positionLine(
  key: string,
  name: string,
  inst: B3Institution,
  assetId: string | null,
  b3: number,
  computed: number,
  filename: string,
): B3PositionLine {
  const diff = computed - b3;
  return {
    asset_key: key,
    ticker: key,
    name,
    institution: inst.name,
    account_id: inst.account_id,
    account_name: inst.account_name,
    asset_id: assetId,
    b3_quantity: String(b3),
    computed_quantity: String(computed),
    difference: String(diff),
    close_price: null,
    b3_value: null,
    status: diff === 0 ? 'match' : 'divergent',
    source: src(filename, SHEET_POSITION, 2),
    note:
      diff === 0
        ? null
        : diff < 0
          ? 'a B3 tem mais do que o app: falta lançar operações'
          : 'o app tem mais do que a B3',
    unavailable_quantity: null,
    unavailable_reason: null,
  };
}

// ── Commit ───────────────────────────────────────────────────────

function commit(args: Record<string, unknown>): B3CommitResult {
  const token = typeof args.token === 'string' ? args.token : '';
  const s = stored.find((x) => x.token === token);
  if (!s)
    mockFail(
      'not_found',
      'Não encontrado: prévia da importação B3 (expirada ou já gravada); gere a prévia de novo.',
    );
  const p = s.preview;
  const skip = new Set((Array.isArray(args.skip) ? args.skip : []) as number[]);
  const includePossible = args.include_possible_duplicates === true;
  const cash = args.cash_movement === true;
  const approvals = new Map<number, B3ActionApproval>();
  for (const a of (Array.isArray(args.approve_actions)
    ? args.approve_actions
    : []) as B3ActionApproval[]) {
    if (!p.actions.some((x) => x.index === a.index))
      mockFail('invalid', `a linha ${a.index} da prévia não é um evento corporativo`);
    approvals.set(a.index, a);
  }
  const overrides = new Map<string, B3AssetOverride>();
  for (const o of (Array.isArray(args.assets) ? args.assets : []) as B3AssetOverride[]) {
    if (!p.assets.some((a) => a.key === o.key))
      mockFail('invalid', `ativo ${o.key} não está na prévia`);
    overrides.set(o.key, o);
  }

  const writable = (status: B3RowStatus, index: number) =>
    !skip.has(index) &&
    (status === 'new' ||
      (status === 'possible_duplicate' && includePossible) ||
      (status === 'review' && approvals.has(index)));

  const errors: B3CommitError[] = [];
  const counts = { assets: 0, trades: 0, incomes: 0, actions: 0 };
  let written = 0;
  const files = s.filenames.join(', ');
  const created: string[] = [];
  const accountIds = new Map<string, string>();
  const { changeSetId } = recordImport(`Importação B3: ${files}`, `B3: ${files}`, (log) => {
    counts.assets = counts.trades = counts.incomes = counts.actions = 0;
    written = 0;
    errors.length = 0;
    const ids = new Map(s.assetIds);
    created.length = 0;
    accountIds.clear();
    for (const n of s.newAccounts) {
      const acc = invoke<{ id: string }>('accounts.create', {
        name: n.spec.name,
        kind: n.spec.kind,
        institution_name: n.spec.institution_name,
        opening_date: n.spec.opening_date ?? undefined,
      });
      log.accounts.push(acc.id);
      created.push(acc.id);
      accountIds.set(n.placeholder, acc.id);
    }
    const accountFor = (id: string | null) => (id ? (accountIds.get(id) ?? id) : id);
    if (args.save_mapping !== false)
      for (const inst of p.institutions)
        if (inst.account_id) savedMapping.set(groupOf(inst.name), accountFor(inst.account_id)!);
    const assetFor = (key: string): string => {
      const o = overrides.get(key);
      if (o?.asset_id) return o.asset_id;
      const cur = ids.get(key) ?? findAsset(key)?.id;
      if (cur) return cur;
      const proposal = p.assets.find((a) => a.key === key)!;
      const created = invoke<Asset>('assets.create', {
        ticker: proposal.ticker,
        name: o?.name ?? proposal.name,
        class: o?.class ?? proposal.class,
        notes: 'Cadastrado pela importação da B3',
      });
      log.assets.push(created.id);
      ids.set(key, created.id);
      counts.assets += 1;
      return created.id;
    };
    type Job = { date: string; order: number; index: number; run: () => void };
    const jobs: Job[] = [];
    for (const a of p.actions)
      if (writable(a.status, a.index))
        jobs.push({
          date: a.date,
          order: 0,
          index: a.index,
          run: () => {
            const ap = approvals.get(a.index);
            const factor = ap?.factor ?? a.factor;
            if (!factor) mockFail('invalid', 'informe o fator do evento');
            const rec = invoke<{ id: string }>('portfolio.create_action', {
              asset_id: assetFor(a.asset_key),
              date: a.date,
              kind: a.kind,
              factor,
              unit_cost: ap?.unit_cost ?? a.unit_cost,
              notes: `Importado da B3 · ${a.movement}`,
            });
            log.actions.push(rec.id);
            counts.actions += 1;
          },
        });
    for (const t of p.trades)
      if (writable(t.status, t.index))
        jobs.push({
          date: t.date,
          order: t.side === 'buy' ? 1 : 2,
          index: t.index,
          run: () => {
            const rec = invoke<Trade>('portfolio.create_trade', {
              asset_id: assetFor(t.asset_key),
              account_id: accountFor(t.account_id),
              date: t.date,
              side: t.side,
              quantity: t.quantity,
              price: t.price,
              fees: 0,
              taxes: 0,
              cash_movement: cash,
              notes: `Importado da B3 · ${t.external_id ?? ''}`,
            });
            log.trades.push(rec.id);
            counts.trades += 1;
          },
        });
    for (const i of p.incomes)
      if (writable(i.status, i.index))
        jobs.push({
          date: i.pay_date,
          order: 3,
          index: i.index,
          run: () => {
            const rec = invoke<IncomeEvent>('portfolio.create_income', {
              asset_id: assetFor(i.asset_key),
              account_id: accountFor(i.account_id),
              kind: i.kind,
              pay_date: i.pay_date,
              quantity: i.quantity,
              gross: i.gross,
              tax_withheld: i.tax_withheld,
              status: 'received',
              cash_movement: cash,
            });
            log.incomes.push(rec.id);
            counts.incomes += 1;
          },
        });
    jobs.sort((a, b) => a.date.localeCompare(b.date) || a.order - b.order || a.index - b.index);
    for (const j of jobs) {
      try {
        j.run();
        written += 1;
      } catch (e) {
        const message =
          typeof e === 'object' && e && 'message' in e ? String(e.message) : String(e);
        errors.push({ index: j.index, message });
      }
    }
  });
  stored = stored.filter((x) => x.token !== token);
  const total = p.trades.length + p.incomes.length + p.actions.length;
  return {
    change_set_id: changeSetId,
    assets_created: counts.assets,
    trades: counts.trades,
    incomes: counts.incomes,
    actions: counts.actions,
    skipped: total - written,
    errors: errors.sort((a, b) => a.index - b.index),
    backup_id: null,
    accounts_created: created,
  };
}

registerMockHandlers(
  guarded({
    'b3.detect': (args) => detect(args as unknown as B3File),
    'b3.preview': preview,
    'b3.commit': commit,
  }),
);
