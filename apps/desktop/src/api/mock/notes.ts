/**
 * Mock de `notes.*` (notas de corretagem SINACOR).
 *
 * Lê o texto da nota como o núcleo (aceita texto puro; um PDF de verdade vira uma nota
 * de exemplo, com o número tirado do nome do arquivo): operações, custos, IRRF e
 * líquido. Identifica o ativo pelo código no título ou por uma tabela de nomes do
 * pregão, rateia taxas e IRRF e compara com as operações da carteira do mock (nova,
 * só completa os custos, duplicada, sem ativo).
 *
 * Arquivo com "senha" no nome é tratado como PDF protegido: devolve `needs_password`
 * até vir uma senha (qualquer uma, menos "errada"). A senha não é guardada.
 */
import type { Asset } from '../generated/Asset';
import type { AssetClass } from '../generated/AssetClass';
import type { NoteAssetChoice } from '../generated/NoteAssetChoice';
import type { NoteCommitResult } from '../generated/NoteCommitResult';
import type { NoteCosts } from '../generated/NoteCosts';
import type { NoteFile } from '../generated/NoteFile';
import type { NoteFileError } from '../generated/NoteFileError';
import type { NoteNewAsset } from '../generated/NoteNewAsset';
import type { NotePreview } from '../generated/NotePreview';
import type { NoteSummary } from '../generated/NoteSummary';
import type { NoteTrade } from '../generated/NoteTrade';
import type { Trade } from '../generated/Trade';
import { mockFail, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { brokerageAccounts, fold, invoke, recordImport } from './ledger/investImport';

/** Nota de exemplo (a mesma dos testes do núcleo), usada quando o arquivo é um PDF. */
export const SAMPLE_NOTE = [
  'NOTA DE NEGOCIAÇÃO',
  'Nr. nota Folha Data pregão',
  '123456 1 13/10/2026',
  '1-BOVESPA C VISTA PETROBRAS PN N2 100 28,50 2.850,00 D',
  '1-BOVESPA C FRACIONARIO ITAUSA PN N1 7 10,00 70,00 D',
  '1-BOVESPA V VISTA FII HGLG PAX HGLG11 CI # 10 160,00 1.600,00 C',
  'Taxa de liquidação 0,95 D',
  'Emolumentos 0,23 D',
  'I.R.R.F. s/ operações, base R$ 1.600,00 0,08',
  'Líquido para 15/10/2026 1.321,26 D',
].join('\n');

/** Nome no pregão (sem o tipo de mercado) → código. */
const NAMES: Record<string, { ticker: string; name: string; class: AssetClass }> = {
  'PETROBRAS PN': { ticker: 'PETR4', name: 'Petrobras PN', class: 'stock_br' },
  'PETROBRAS ON': { ticker: 'PETR3', name: 'Petrobras ON', class: 'stock_br' },
  'VALE ON': { ticker: 'VALE3', name: 'Vale ON', class: 'stock_br' },
  'ITAUUNIBANCO PN': { ticker: 'ITUB4', name: 'Itaú Unibanco PN', class: 'stock_br' },
  'BRASIL ON': { ticker: 'BBAS3', name: 'Banco do Brasil ON', class: 'stock_br' },
  'WEG ON': { ticker: 'WEGE3', name: 'WEG ON', class: 'stock_br' },
  'MAGAZ LUIZA ON': { ticker: 'MGLU3', name: 'Magazine Luiza ON', class: 'stock_br' },
  'KLABIN S/A UNT': { ticker: 'KLBN11', name: 'Klabin Units', class: 'stock_br' },
  'TAESA UNT': { ticker: 'TAEE11', name: 'Taesa Units', class: 'stock_br' },
};

const toCents = (s: string) => Math.round(Number(s.replace(/\./g, '').replace(',', '.')) * 100);
const toDec = (s: string) => String(Number(s.replace(/\./g, '').replace(',', '.')));
const brl = (c: number) =>
  (c / 100).toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' });
const isoDate = (dmy: string) => dmy.split('/').reverse().join('-');

interface ParsedLine {
  side: 'buy' | 'sell';
  market: string;
  spec: string;
  dayTrade: boolean;
  quantity: string;
  price: string;
  gross: number;
}

interface ParsedNote {
  number: string | null;
  date: string | null;
  lines: ParsedLine[];
  costs: NoteCosts;
  net: number | null;
}

function decodeText(file: NoteFile): string | null {
  let bin: string;
  try {
    bin = atob(file.content_base64);
  } catch {
    mockFail('invalid', `${file.filename}: o conteúdo não está em base64`);
  }
  if (bin.startsWith('%PDF')) {
    // O mock não lê PDF: usa a nota de exemplo com o número tirado do nome do arquivo.
    let h = 0;
    for (const ch of file.filename) h = (h * 31 + ch.charCodeAt(0)) % 900000;
    const sample = SAMPLE_NOTE.replace('123456', String(100000 + h));
    return /incompleta/i.test(file.filename)
      ? sample.replace(/^1-BOVESPA C FRACIONARIO.*\n/m, '')
      : sample;
  }
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(
      Uint8Array.from(bin, (c) => c.charCodeAt(0)),
    );
  } catch {
    return null;
  }
}

const TRADE_LINE =
  /^\s*\d-BOVESPA\s+([CV])\s+(VISTA|FRACIONARIO)\s+(.+?)\s+(?:([#D])\s+)?(\d[\d.]*)\s+([\d.]*\d,\d+)\s+([\d.]*\d,\d{2})\s+([DC])\s*$/i;

function costOf(text: string, re: RegExp): number {
  const m = re.exec(text);
  return m ? toCents(m[1]!) : 0;
}

function parse(text: string): ParsedNote | null {
  if (!/NOTA DE (NEGOCIA|CORRETAGEM)/i.test(text)) return null;
  const head = /Nr\.?\s*nota[^\n]*\n\s*(\d+)\s+\d+\s+(\d{2}\/\d{2}\/\d{4})/i.exec(text);
  const lines: ParsedLine[] = [];
  for (const raw of text.split(/\r?\n/)) {
    const m = TRADE_LINE.exec(raw);
    if (!m) continue;
    lines.push({
      side: m[1]!.toUpperCase() === 'C' ? 'buy' : 'sell',
      market: m[2]!.toUpperCase(),
      spec: m[3]!.trim(),
      dayTrade: m[4] === 'D',
      quantity: toDec(m[5]!),
      price: toDec(m[6]!),
      gross: toCents(m[7]!),
    });
  }
  const costs: NoteCosts = {
    settlement: costOf(text, /Taxa de liquida[çc][ãa]o\s+([\d.,]+)/i),
    registration: costOf(text, /Taxa de registro\s+([\d.,]+)/i),
    emoluments: costOf(text, /Emolumentos\s+([\d.,]+)/i),
    brokerage: costOf(text, /Corretagem\s+([\d.,]+)/i),
    iss: costOf(text, /\bISS\b[^\d\n]*([\d.,]+)/i),
    other: costOf(text, /Outros\s+([\d.,]+)/i),
    irrf: costOf(text, /I\.?R\.?R\.?F\.?[^\n]*?([\d.]*\d,\d{2})\s*$/im),
    total: 0,
  };
  costs.total =
    costs.settlement +
    costs.registration +
    costs.emoluments +
    costs.brokerage +
    costs.iss +
    costs.other;
  const liq = /L[íi]quido para\s+\d{2}\/\d{2}\/\d{4}\s+([\d.,]+)\s*([DC])/i.exec(text);
  return {
    number: head?.[1] ?? null,
    date: head ? isoDate(head[2]!) : null,
    lines,
    costs,
    net: liq ? toCents(liq[1]!) * (liq[2]!.toUpperCase() === 'D' ? -1 : 1) : null,
  };
}

/** Rateio proporcional; a sobra do arredondamento vai para as maiores operações (como o núcleo). */
function allocate(total: number, weights: readonly number[]): number[] {
  const sum = weights.reduce((a, b) => a + b, 0);
  if (sum <= 0 || total === 0) return weights.map(() => 0);
  const out = weights.map((w) => Math.trunc((total * w) / sum));
  const rest = total - out.reduce((a, b) => a + b, 0);
  const order = weights.map((_, i) => i).sort((a, b) => weights[b]! - weights[a]!);
  for (let k = 0; k < Math.abs(rest); k++) out[order[k % order.length]!]! += Math.sign(rest);
  return out;
}

/** Marcas de observação que a nota acrescenta a algumas execuções. */
const MARKS = new Set(['@', '#', '@#', '#@', 'ER', 'EJ', 'ED', 'EX', 'ATZ']);
const stripMarks = (spec: string) =>
  spec
    .split(/\s+/)
    .filter((t) => t && !MARKS.has(t))
    .join(' ');

/** Soma as execuções da mesma ordem (título sem marcas, lado, mercado, day trade). */
function groupLines(lines: ParsedLine[]): { line: ParsedLine; executions: number }[] {
  const out: { line: ParsedLine; executions: number; qty: number }[] = [];
  for (const l of lines) {
    const key = stripMarks(l.spec);
    const g = out.find(
      (o) =>
        stripMarks(o.line.spec) === key &&
        o.line.side === l.side &&
        o.line.market === l.market &&
        o.line.dayTrade === l.dayTrade,
    );
    if (g) {
      g.qty += Number(l.quantity);
      g.line = { ...g.line, gross: g.line.gross + l.gross, spec: key };
      g.executions += 1;
    } else out.push({ line: { ...l }, executions: 1, qty: Number(l.quantity) });
  }
  return out.map((o) =>
    o.executions > 1
      ? {
          executions: o.executions,
          line: {
            ...o.line,
            quantity: String(o.qty),
            price: String(Math.round((o.line.gross / 100 / o.qty) * 1e6) / 1e6),
          },
        }
      : { line: o.line, executions: 1 },
  );
}

function resolveSpec(
  spec: string,
  choices: ReadonlyMap<string, NoteAssetChoice>,
): { ticker: string | null; asset: Asset | null; create: NoteNewAsset | null } {
  const chosen = choices.get(spec) ?? choices.get(stripMarks(spec));
  if (chosen?.asset_id) {
    const a = invoke<Asset>('assets.get', { id: chosen.asset_id });
    return { ticker: a.ticker, asset: a, create: null };
  }
  if (chosen?.ticker) {
    const ticker = chosen.ticker.trim().toUpperCase();
    const asset = invoke<Asset | null>('assets.find', { ticker });
    if (asset) return { ticker, asset, create: null };
    return {
      ticker,
      asset: null,
      create: {
        ticker,
        name: spec,
        class: chosen.class ?? (/^FII\b/i.test(spec) || /11$/.test(ticker) ? 'fii' : 'stock_br'),
      },
    };
  }
  const code = /\b([A-Z]{4}\d{1,2})\b/.exec(spec)?.[1];
  const named = Object.entries(NAMES).find(([k]) => spec.toUpperCase().startsWith(k))?.[1];
  const ticker = code ?? named?.ticker ?? null;
  if (!ticker) return { ticker: null, asset: null, create: null };
  const asset = invoke<Asset | null>('assets.find', { ticker });
  if (asset) return { ticker, asset, create: null };
  return {
    ticker,
    asset: null,
    create: {
      ticker,
      name: named?.name ?? spec,
      class: /^FII\b/i.test(spec) ? 'fii' : (named?.class ?? 'stock_br'),
    },
  };
}

interface Planned {
  preview: NotePreview;
}

function plan(
  files: NoteFile[],
  accountId: string,
  password: string | undefined,
  choices: ReadonlyMap<string, NoteAssetChoice>,
  group = true,
): Planned {
  if (!files.length) mockFail('invalid', 'envie ao menos uma nota');
  const account = brokerageAccounts().find((a) => a.id === accountId);
  if (!account) mockFail('not_found', `Não encontrado: conta ${accountId}.`);
  if (account.currency !== 'BRL') mockFail('invalid', 'notas SINACOR são de contas em reais (BRL)');
  const existing = invoke<Trade[]>('portfolio.list_trades', { account_id: accountId, limit: 5000 });
  const notes: NoteSummary[] = [];
  const trades: NoteTrade[] = [];
  const errors: NoteFileError[] = [];
  const unresolved = new Set<string>();
  const seenNotes = new Set<string>();
  let index = 0;
  for (const f of files) {
    if (/senha/i.test(fold(f.filename)) && (!password || password === 'errada')) {
      errors.push({
        filename: f.filename,
        message: password ? 'senha incorreta' : 'o PDF é protegido por senha',
        needs_password: true,
      });
      continue;
    }
    const text = decodeText(f);
    const note = text ? parse(text) : null;
    if (!note) {
      errors.push({
        filename: f.filename,
        message: 'não parece uma nota de corretagem no padrão SINACOR',
        needs_password: false,
      });
      continue;
    }
    const warnings: string[] = [];
    if (!note.date) warnings.push('nota sem data: nada será gravado');
    if (!note.lines.length) warnings.push('nenhuma operação lida nesta nota');
    if (note.number && seenNotes.has(note.number))
      warnings.push(`a nota ${note.number} foi enviada duas vezes`);
    if (note.number) seenNotes.add(note.number);
    const grouped = group
      ? groupLines(note.lines)
      : note.lines.map((line) => ({ line, executions: 1 }));
    const lines = grouped.map((g) => g.line);
    const fees = allocate(
      note.costs.total,
      lines.map((l) => l.gross),
    );
    const irrf = allocate(
      note.costs.irrf,
      lines.map((l) => (l.side === 'sell' ? l.gross : 0)),
    );
    let computed = -note.costs.total - note.costs.irrf;
    lines.forEach((l, i) => {
      computed += l.side === 'sell' ? l.gross : -l.gross;
      const r = resolveSpec(l.spec, choices);
      const date = note.date ?? '';
      const same = r.asset
        ? existing.filter(
            (t) =>
              t.asset_id === r.asset!.id &&
              t.date === date &&
              t.side === l.side &&
              Number(t.quantity) === Number(l.quantity),
          )
        : [];
      const withNote = same.find((t) => t.note_number && t.note_number === note.number);
      const withCosts = same.find((t) => t.fees > 0 || t.taxes > 0);
      const bare = same.find((t) => t.fees === 0 && t.taxes === 0 && !t.note_number);
      let status: NoteTrade['status'];
      let existingId: string | null = null;
      let text: string | null;
      if (!r.asset && !r.create) {
        status = 'unresolved';
        unresolved.add(l.spec);
        text = 'ativo não identificado pelo nome do pregão: escolha o ativo';
      } else if (withNote || (withCosts && !bare)) {
        status = 'duplicate';
        existingId = (withNote ?? withCosts)!.id;
        text = withNote ? 'já registrada com esta nota' : 'já registrada com custos';
      } else if (bare) {
        status = 'fill_costs';
        existingId = bare.id;
        text = 'já existe sem custos (ex.: do relatório da B3): ganha os custos e o número da nota';
      } else {
        status = 'new';
        text = r.create ? `${r.create.ticker} será cadastrado` : null;
      }
      trades.push({
        index: index++,
        filename: f.filename,
        note_number: note.number,
        date,
        side: l.side,
        market: l.market === 'FRACIONARIO' ? 'Fracionário' : 'Vista',
        spec: l.spec,
        ticker: r.ticker,
        asset_id: r.asset?.id ?? null,
        create_asset: r.create,
        quantity: l.quantity,
        price: l.price,
        gross: l.gross,
        fees: fees[i]!,
        taxes: irrf[i]!,
        day_trade: l.dayTrade,
        executions: grouped[i]!.executions,
        chosen: choices.has(l.spec) || choices.has(stripMarks(l.spec)),
        warning: null,
        status,
        existing_trade_id: existingId,
        note: text,
      });
    });
    if (note.net != null && Math.abs(note.net - computed) > 1)
      warnings.push(
        `o líquido da nota (${brl(note.net)}) não bate com as operações e custos lidos (${brl(computed)}): confira se alguma linha ficou de fora`,
      );
    notes.push({
      filename: f.filename,
      number: note.number,
      date: note.date,
      trades: note.lines.length,
      costs: note.costs,
      net: note.net,
      computed_net: computed,
      warnings,
    });
  }
  return { preview: { notes, trades, unresolved_specs: [...unresolved], errors } };
}

function choicesOf(args: Record<string, unknown>): Map<string, NoteAssetChoice> {
  const list = (Array.isArray(args.assets) ? args.assets : []) as NoteAssetChoice[];
  const out = new Map<string, NoteAssetChoice>();
  for (const c of list) {
    if (c.asset_id) invoke('assets.get', { id: c.asset_id });
    else if (!c.ticker?.trim())
      mockFail('invalid', `escolha o ativo ou informe o código para "${c.spec}"`);
    out.set(c.spec, c);
  }
  return out;
}

function filesOf(args: Record<string, unknown>): NoteFile[] {
  return (Array.isArray(args.files) ? args.files : []) as NoteFile[];
}

function commit(args: Record<string, unknown>): NoteCommitResult {
  const files = filesOf(args);
  const accountId = typeof args.account_id === 'string' ? args.account_id : '';
  const password = typeof args.password === 'string' ? args.password : undefined;
  const planned = plan(files, accountId, password, choicesOf(args), args.group !== false);
  const err = planned.preview.errors[0];
  if (err) mockFail('invalid', `${err.filename}: ${err.message}`);
  const skip = new Set((Array.isArray(args.skip) ? args.skip : []) as number[]);
  const todo = planned.preview.trades.filter(
    (t) => !skip.has(t.index) && (t.status === 'new' || t.status === 'fill_costs'),
  );
  const skipped = planned.preview.trades.length - todo.length;
  const cash = args.cash_movement === true;
  let created = 0;
  let updated = 0;
  let assetsCreated = 0;
  const names = files.map((f) => f.filename).join(', ');
  const { changeSetId } = todo.length
    ? recordImport(
        `Importação de nota de corretagem: ${names}`,
        `Nota de corretagem: ${names}`,
        (log) => {
          created = updated = assetsCreated = 0;
          const newAssets = new Map<string, string>();
          for (const t of todo) {
            if (t.status === 'fill_costs') {
              const cur = invoke<Trade>('portfolio.get_trade', { id: t.existing_trade_id });
              log.updates.push({
                id: cur.id,
                before: {
                  fees: cur.fees,
                  taxes: cur.taxes,
                  note_number: cur.note_number,
                  day_trade: cur.day_trade,
                },
              });
              invoke('portfolio.update_trade', {
                id: cur.id,
                fees: t.fees,
                taxes: t.taxes,
                note_number: t.note_number,
                day_trade: t.day_trade,
              });
              updated += 1;
              continue;
            }
            let assetId = t.asset_id;
            if (!assetId && t.create_asset) {
              assetId = newAssets.get(t.create_asset.ticker) ?? null;
              if (!assetId) {
                const a = invoke<Asset>('assets.create', {
                  ...t.create_asset,
                  notes: 'Cadastrado pela importação de nota de corretagem',
                });
                log.assets.push(a.id);
                newAssets.set(t.create_asset.ticker, a.id);
                assetId = a.id;
                assetsCreated += 1;
              }
            }
            const rec = invoke<Trade>('portfolio.create_trade', {
              asset_id: assetId,
              account_id: accountId,
              date: t.date,
              side: t.side,
              quantity: t.quantity,
              price: t.price,
              fees: t.fees,
              taxes: t.taxes,
              day_trade: t.day_trade,
              note_number: t.note_number,
              cash_movement: cash,
            });
            log.trades.push(rec.id);
            created += 1;
          }
        },
      )
    : { changeSetId: null };
  return { created, updated, assets_created: assetsCreated, skipped, change_set_id: changeSetId };
}

registerMockHandlers(
  guarded({
    'notes.preview': (args) =>
      plan(
        filesOf(args),
        typeof args.account_id === 'string' ? args.account_id : '',
        typeof args.password === 'string' ? args.password : undefined,
        choicesOf(args),
        args.group !== false,
      ).preview,
    'notes.commit': commit,
  }),
);
