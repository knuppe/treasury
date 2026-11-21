/**
 * Mock de `import.*`: lê OFX (SGML/XML) e CSV com presets e mapeamento, classifica
 * (saldo, pagamento de fatura, sinais do cartão), deduplica (FITID, impressão
 * digital, mesma data e valor com descrição parecida), aplica as regras e grava
 * a prévia num único conjunto de alterações com autor "importação".
 */
import type { CsvMapping } from '@/api/generated/CsvMapping';
import type { CsvPreset } from '@/api/generated/CsvPreset';
import type { DetectResult } from '@/api/generated/DetectResult';
import type { FileKind } from '@/api/generated/FileKind';
import type { ImportBatch } from '@/api/generated/ImportBatch';
import type { InboxItem } from '@/api/generated/InboxItem';
import type { InboxStatus } from '@/api/generated/InboxStatus';
import type { ImportFormat } from '@/api/generated/ImportFormat';
import type { ImportPreview } from '@/api/generated/ImportPreview';
import type { ImportResult } from '@/api/generated/ImportResult';
import type { ImportRowError } from '@/api/generated/ImportRowError';
import type { OfxAccountInfo } from '@/api/generated/OfxAccountInfo';
import type { PreviewCounts } from '@/api/generated/PreviewCounts';
import type { PreviewRow } from '@/api/generated/PreviewRow';
import type { RowOverride } from '@/api/generated/RowOverride';
import type { TxnKind } from '@/api/generated/TxnKind';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { linkCategoryPayments } from './debt';
import { guarded } from './ledger/boot';
import { brDate, isIsoDate } from './ledger/dates';
import {
  currencyAndKind,
  deleteRows,
  insertTxn,
  isTransferLike,
  replaceTags,
} from './ledger/domain';
import { invalid, notFound, str, text } from './ledger/errors';
import { rememberOfx } from './ledger/ofxMatch';
import {
  author,
  find,
  findChangeSetInserting,
  newId,
  patch,
  put,
  rows,
  write,
} from './ledger/store';
import { fingerprint, fold, hashHex, verySimilar } from './ledger/text';
import { evaluate, loadRules, recordHits } from './rules';

/** Previsto (fora transferências e parcelas) que um lançamento real efetiva: mesma conta e
 * sentido, valor até 5% diferente (mínimo R$ 1,00) e data até 7 dias; o mais perto em valor e
 * depois em data. */
function plannedMatch(accountId: string, date: string, amount: number, taken: Set<string>) {
  if (!amount) return undefined;
  const day = (iso: string) => Date.parse(`${iso}T00:00:00Z`) / 86_400_000;
  const tol = Math.max(100, Math.trunc((Math.abs(amount) * 5) / 100));
  return rows('txn')
    .filter(
      (t) =>
        t.account_id === accountId &&
        t.status === 'planned' &&
        (t.kind === 'expense' || t.kind === 'income') &&
        !t.transfer_id &&
        !t.installment_plan_id &&
        t.amount < 0 === amount < 0 &&
        Math.abs(t.amount - amount) <= tol &&
        Math.abs(day(t.date) - day(date)) <= 7 &&
        !taken.has(t.id),
    )
    .sort(
      (a, b) =>
        Math.abs(a.amount - amount) - Math.abs(b.amount - amount) ||
        Math.abs(day(a.date) - day(date)) - Math.abs(day(b.date) - day(date)) ||
        a.id.localeCompare(b.id),
    )[0];
}

const PRESETS: CsvPreset[] = [
  { id: 'nubank_conta', name: 'Nubank — conta', mapping: m(',', 'Data', '%d/%m/%Y', 'Valor', null, null, 'Descrição', 'Identificador', false, false) },
  { id: 'nubank_cartao', name: 'Nubank — cartão de crédito', mapping: m(',', 'date', '%Y-%m-%d', 'amount', null, null, 'title', null, false, true) },
  { id: 'inter', name: 'Banco Inter — conta', mapping: m(';', 'Data Lançamento', '%d/%m/%Y', 'Valor', null, null, 'Histórico+Descrição', null, true, false) },
  { id: 'itau', name: 'Itaú — conta corrente', mapping: m(';', 'data', '%d/%m/%Y', 'valor (R$)', null, null, 'lançamento', null, true, false) },
  { id: 'c6', name: 'C6 Bank — conta', mapping: m(';', 'Data Lançamento', '%d/%m/%Y', null, 'Saída(R$)', 'Entrada(R$)', 'Título+Descrição', null, true, false) },
  { id: 'c6_cartao', name: 'C6 Bank — cartão de crédito', mapping: m(';', 'Data de Compra', '%d/%m/%Y', 'Valor (em R$)', null, null, 'Descrição+Parcela', null, false, true) },
  { id: 'flash', name: 'Flash — vale-alimentação e refeição', mapping: { ...m(',', 'Data', '%d/%m/%Y', 'Valor', null, null, 'Movimentação', 'Data+Hora+Movimentação+Valor', true, false), payee_column: 'Movimentação', card_descriptor: true } },
  { id: 'generico', name: 'Genérico (data; descrição; valor)', mapping: m(null, '0', '%d/%m/%Y|%Y-%m-%d', '2', null, null, '1', null, true, false) },
]; // prettier-ignore

/** Como o núcleo devolve: campos opcionais ausentes chegam como `null`. */
function m(
  delimiter: string | null,
  date: string,
  dateFormat: string,
  amount: string | null,
  debit: string | null,
  credit: string | null,
  description: string,
  externalId: string | null,
  decimalComma: boolean,
  invertSign: boolean,
): CsvMapping {
  return {
    delimiter,
    has_header: true,
    skip_rows: null,
    date_column: date,
    date_format: dateFormat,
    amount_column: amount,
    debit_column: debit,
    credit_column: credit,
    description_column: description,
    payee_column: null,
    external_id_column: externalId,
    decimal_comma: decimalComma,
    invert_sign: invertSign,
    encoding: null,
  } as unknown as CsvMapping;
}

// ── Arquivo ─────────────────────────────────────────────────────

function decodeBase64(b64: string): Uint8Array {
  let bin: string;
  try {
    bin = atob(b64.replace(/\s/g, ''));
  } catch {
    return invalid('conteúdo do arquivo inválido (base64)');
  }
  if (bin.length === 0) invalid('o arquivo está vazio');
  if (bin.length > 25 * 1024 * 1024) invalid('o arquivo passa de 25 MB');
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

function decodeText(
  bytes: Uint8Array,
  hint?: string | null,
  declared?: string | null,
): { text: string; encoding: string } {
  const label = (hint ?? declared ?? '').toLowerCase();
  const tryDecode = (enc: string, fatal: boolean) => new TextDecoder(enc, { fatal }).decode(bytes);
  if (label && label !== 'utf-8' && label !== 'utf8') {
    const enc = label === '1252' || label === 'cp1252' ? 'windows-1252' : label;
    try {
      return { text: tryDecode(enc, false), encoding: enc === 'latin1' ? 'iso-8859-1' : enc };
    } catch {
      invalid(`codificação desconhecida: ${label}`);
    }
  }
  try {
    const text = tryDecode('utf-8', true);
    return { text: text.replace(/^\uFEFF/, ''), encoding: 'utf-8' };
  } catch {
    return { text: tryDecode('windows-1252', false), encoding: 'windows-1252' };
  }
}

function detectFormat(text: string, filename: string): ImportFormat {
  if (/<OFX>|OFXHEADER|<\?OFX/i.test(text.slice(0, 4000))) return 'ofx';
  if (/\.ofx$/i.test(filename))
    invalid('o arquivo tem extensão .ofx, mas não parece um OFX válido');
  if (/\.(csv|txt)$/i.test(filename) || /[;,\t]/.test(text.slice(0, 2000))) return 'csv';
  return invalid('formato não reconhecido: envie um extrato OFX ou CSV');
}

// ── OFX ─────────────────────────────────────────────────────────

interface RawRow {
  line: number | null;
  date: string | null;
  amount: number | null;
  description: string;
  payee: string | null;
  notes: string | null;
  external_id: string | null;
  trn_type: string | null;
  error: string | null;
}

interface OfxStatement {
  info: OfxAccountInfo;
  txns: RawRow[];
}

const tag = (block: string, name: string) => {
  const v = new RegExp(`<${name}>([^<\\r\\n]*)`, 'i').exec(block)?.[1]?.trim();
  return v ? v : null;
};

function ofxDate(raw: string | null): string | null {
  const match = raw ? /^(\d{4})(\d{2})(\d{2})/.exec(raw) : null;
  if (!match) return null;
  const iso = `${match[1]}-${match[2]}-${match[3]}`;
  return isIsoDate(iso) ? iso : null;
}

/** Favorecido do descritor da maquininha, como `card_payee` do núcleo. */
function cardPayee(descriptor: string): string {
  let s = descriptor.trim();
  if (fold(s).startsWith('deposito')) return '';
  const star = s.indexOf('*');
  if (star >= 0 && star <= 10 && /^[A-Za-z0-9 ]*$/.test(s.slice(0, star)))
    s = s.slice(star + 1).trimStart();
  for (const suffix of [' BRA', ' BR']) {
    if (s.length > suffix.length && s.toUpperCase().endsWith(suffix)) {
      s = s.slice(0, -suffix.length).trimEnd();
      break;
    }
  }
  return s;
}

function parseAmount(raw: string, decimalComma: boolean): number | null {
  let s = raw.trim().replace(/R\$|\s/gi, '');
  let neg = false;
  if (/^\(.*\)$/.test(s)) {
    neg = true;
    s = s.slice(1, -1);
  }
  if (/^[-−]/.test(s)) {
    neg = !neg;
    s = s.slice(1);
  } else if (s.startsWith('+')) s = s.slice(1);
  if (/[-−]$/.test(s)) {
    neg = !neg;
    s = s.slice(0, -1);
  }
  if (decimalComma) s = s.replace(/\./g, '').replace(',', '.');
  else s = s.replace(/,/g, '');
  if (!/^\d+(\.\d{1,2})?$|^\.\d{1,2}$/.test(s)) return null;
  const [whole = '0', frac = ''] = s.split('.');
  const cents = Number(whole || '0') * 100 + Number(frac.padEnd(2, '0') || '0');
  return neg && cents !== 0 ? -cents : cents;
}

function combine(a: string | null, b: string | null): string {
  if (a && b) {
    const fa = fold(a);
    const fb = fold(b);
    if (fa.includes(fb)) return a;
    if (fb.includes(fa)) return b;
    return `${a} - ${b}`;
  }
  return a ?? b ?? '';
}

function parseOfx(text: string): OfxStatement[] {
  const blocks = [...text.matchAll(/<(CC)?STMTRS>([\s\S]*?)<\/(CC)?STMTRS>/gi)];
  if (blocks.length === 0) invalid('OFX sem extrato (STMTRS ou CCSTMTRS)');
  return blocks.map((b) => {
    const body = b[2] ?? '';
    const credit = Boolean(b[1]);
    const currency = tag(body, 'CURDEF');
    const ledger = /<LEDGERBAL>([\s\S]*?)(<\/LEDGERBAL>|<AVAILBAL>|$)/i.exec(body)?.[1] ?? '';
    const ledgerRaw = tag(ledger, 'BALAMT');
    const txns = [...body.matchAll(/<STMTTRN>([\s\S]*?)<\/STMTTRN>/gi)].map((t) => {
      const block = t[1] ?? '';
      const name = tag(block, 'NAME');
      const payee = tag(block, 'PAYEE');
      const memo = tag(block, 'MEMO');
      const fitid = tag(block, 'FITID');
      const check = tag(block, 'CHECKNUM');
      const type = tag(block, 'TRNTYPE');
      const dateRaw = tag(block, 'DTPOSTED');
      const amountRaw = tag(block, 'TRNAMT');
      const row: RawRow = {
        line: null,
        date: ofxDate(dateRaw),
        amount: null,
        description: combine(name ?? payee, memo) || type || 'Lançamento',
        payee: payee && payee !== combine(name ?? payee, memo) ? payee : null,
        notes: check && check !== fitid ? `Documento nº ${check}` : null,
        external_id: fitid,
        trn_type: type,
        error: null,
      };
      if (!row.date) row.error = `data inválida: ${dateRaw ?? '(vazia)'}`;
      if (amountRaw == null) row.error ??= 'lançamento sem valor (TRNAMT)';
      else {
        let a = parseAmount(amountRaw, amountRaw.includes(',') && !amountRaw.includes('.'));
        if (a == null) row.error ??= `valor inválido: ${amountRaw}`;
        else {
          if (type === 'DEBIT' && a > 0) a = -a;
          row.amount = a;
        }
      }
      return row;
    });
    const acct = credit
      ? (/<CCACCTFROM>([\s\S]*?)<\/CCACCTFROM>/i.exec(body)?.[1] ?? '')
      : (/<BANKACCTFROM>([\s\S]*?)<\/BANKACCTFROM>/i.exec(body)?.[1] ?? '');
    const info: OfxAccountInfo = {
      bank_id: tag(acct, 'BANKID'),
      branch_id: tag(acct, 'BRANCHID'),
      account_id: tag(acct, 'ACCTID'),
      account_type: tag(acct, 'ACCTTYPE'),
      credit_card: credit,
      currency,
      ledger_balance: ledgerRaw ? parseAmount(ledgerRaw, false) : null,
      balance_date: ofxDate(tag(ledger, 'DTASOF')),
      date_start: ofxDate(tag(body, 'DTSTART')),
      date_end: ofxDate(tag(body, 'DTEND')),
      statements: blocks.length,
    };
    return { info, txns };
  });
}

function declaredCharset(text: string): string | null {
  const c =
    /CHARSET:\s*([\w-]+)/i.exec(text.slice(0, 600))?.[1] ??
    /encoding="([\w-]+)"/i.exec(text.slice(0, 300))?.[1];
  if (!c || /none/i.test(c)) return null;
  return c === '1252' ? 'windows-1252' : c.toLowerCase();
}

function pickStatement(
  stmts: OfxStatement[],
  wanted: string | null | undefined,
  card: boolean,
): OfxStatement {
  if (wanted?.trim()) {
    return (
      stmts.find((s) => s.info.account_id === wanted.trim()) ??
      notFound(`extrato da conta ${wanted} no arquivo`)
    );
  }
  return (
    stmts.find((s) => s.info.credit_card === card && s.txns.length > 0) ??
    stmts.find((s) => s.txns.length > 0) ??
    stmts[0]!
  );
}

// ── CSV ─────────────────────────────────────────────────────────

interface Rec {
  line: number;
  cells: string[];
}

function readRecords(text: string, delim: string): Rec[] {
  const out: Rec[] = [];
  let cells: string[] = [];
  let cell = '';
  let quoted = false;
  let line = 1;
  let startLine = 1;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i]!;
    if (quoted) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          cell += '"';
          i++;
        } else quoted = false;
      } else {
        if (ch === '\n') line++;
        cell += ch;
      }
      continue;
    }
    if (ch === '"' && cell.trim() === '') {
      quoted = true;
      cell = '';
    } else if (ch === delim) {
      cells.push(cell);
      cell = '';
    } else if (ch === '\n' || ch === '\r') {
      if (ch === '\r' && text[i + 1] === '\n') i++;
      cells.push(cell);
      if (cells.some((c) => c.trim() !== ''))
        out.push({ line: startLine, cells: cells.map((c) => c.trim()) });
      cells = [];
      cell = '';
      line++;
      startLine = line;
    } else cell += ch;
  }
  cells.push(cell);
  if (cells.some((c) => c.trim() !== ''))
    out.push({ line: startLine, cells: cells.map((c) => c.trim()) });
  return out;
}

function detectDelimiter(text: string): string {
  const lines = text
    .split(/\r?\n/)
    .filter((l) => l.trim())
    .slice(0, 30);
  let best = ',';
  let bestScore = -1;
  for (const d of [';', ',', '\t', '|']) {
    const counts = lines.map((l) => l.split(d).length - 1).filter((n) => n > 0);
    const score = counts.length * 10 + Math.min(...(counts.length ? counts : [0]));
    if (score > bestScore) {
      best = d;
      bestScore = score;
    }
  }
  return best;
}

const delimiterName = (d: string) => (d === '\t' ? '\\t' : d);
const delimiterFrom = (d: string | null | undefined) =>
  d === '\\t' || d === 'tab' ? '\t' : d || null;

function guessDate(c: string): boolean {
  return /^\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}/.test(c) || /^\d{4}-\d{2}-\d{2}/.test(c);
}

const looksLikeData = (c: string) =>
  c.trim() !== '' &&
  (guessDate(c) || parseAmount(c, true) != null || parseAmount(c, false) != null);

function detectLayout(text: string, delim: string) {
  const recs = readRecords(text, delim);
  const counts = new Map<number, number>();
  for (const r of recs) counts.set(r.cells.length, (counts.get(r.cells.length) ?? 0) + 1);
  const mode = [...counts.entries()].sort((a, b) => b[1] - a[1] || b[0] - a[0])[0]?.[0] ?? 0;
  const first = recs.findIndex((r) => r.cells.length === mode);
  if (first < 0) return { skip_rows: 0, has_header: false, headers: [] as string[] };
  const rec = recs[first]!;
  const next = recs[first + 1];
  const hasHeader = !rec.cells.some(looksLikeData) && (!next || next.cells.some(looksLikeData));
  return {
    skip_rows: rec.line - 1,
    has_header: hasHeader,
    headers: hasHeader ? rec.cells : rec.cells.map((_, i) => `Coluna ${i + 1}`),
  };
}

function skipLines(text: string, n: number): string {
  return text.split(/\r?\n/).slice(n).join('\n');
}

function referencedColumns(mp: CsvMapping): string[] {
  return [
    mp.date_column,
    mp.amount_column,
    mp.debit_column,
    mp.credit_column,
    mp.description_column,
    mp.external_id_column,
  ]
    .filter((c): c is string => typeof c === 'string' && c !== '')
    .flatMap((c) => c.split('+'));
}

function suggestPreset(headers: string[]): string | null {
  const have = headers.map(fold);
  let best: [number, string] | null = null;
  for (const p of PRESETS) {
    if (p.id === 'generico') continue;
    const cols = referencedColumns(p.mapping);
    if (cols.every((c) => have.includes(fold(c))) && (!best || cols.length > best[0]))
      best = [cols.length, p.id];
  }
  return best?.[1] ?? null;
}

function resolveColumns(spec: string, headers: string[] | null): number[] {
  const s = spec.trim();
  if (!s) invalid('coluna não informada');
  if (headers) {
    const i = headers.findIndex((h) => fold(h) === fold(s));
    if (i >= 0) return [i];
  }
  if (/^\d+$/.test(s)) return [Number(s)];
  if (s.includes('+')) return s.split('+').flatMap((p) => resolveColumns(p, headers));
  return invalid(
    headers
      ? `coluna não encontrada: ${s} (colunas do arquivo: ${headers.join(', ')})`
      : `coluna não encontrada: ${s} (o arquivo não tem cabeçalho; use o índice)`,
  );
}

function parseCsvDate(cell: string, format: string): string | null {
  for (const f of format.split('|')) {
    const parts: string[] = [];
    let re = '';
    for (let i = 0; i < f.length; i++) {
      if (f[i] === '%' && i + 1 < f.length) {
        const t = f[++i]!;
        parts.push(t);
        re +=
          t === 'Y'
            ? '(\\d{4})'
            : t === 'y'
              ? '(\\d{2})'
              : t === 'H' || t === 'M' || t === 'S'
                ? '(\\d{1,2})'
                : '(\\d{1,2})';
      } else re += f[i]!.replace(/[.*+?^${}()|[\]\\/]/g, '\\$&');
    }
    const match = new RegExp(`^${re}`).exec(cell.trim());
    if (!match) continue;
    let y = 0;
    let mo = 0;
    let d = 0;
    parts.forEach((p, i) => {
      const v = Number(match[i + 1]);
      if (p === 'Y') y = v;
      else if (p === 'y') y = 2000 + v;
      else if (p === 'm') mo = v;
      else if (p === 'd') d = v;
    });
    const iso = `${String(y).padStart(4, '0')}-${String(mo).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
    if (isIsoDate(iso) && y >= 1900 && y <= 2200) return iso;
  }
  return null;
}

function parseCsv(text: string, mapping: CsvMapping) {
  const delim = delimiterFrom(mapping.delimiter) ?? detectDelimiter(text);
  const skip = mapping.skip_rows ?? detectLayout(text, delim).skip_rows;
  const recs = readRecords(skipLines(text, skip), delim);
  const headers = mapping.has_header ? (recs.shift()?.cells ?? []) : null;
  const opt = (s: string | null | undefined) => (s && s.trim() ? resolveColumns(s, headers) : null);
  const cols = {
    date: resolveColumns(mapping.date_column, headers),
    amount: opt(mapping.amount_column),
    debit: opt(mapping.debit_column),
    credit: opt(mapping.credit_column),
    description: resolveColumns(mapping.description_column, headers),
    payee: opt(mapping.payee_column),
    external: opt(mapping.external_id_column),
  };
  if (!cols.amount && !cols.debit && !cols.credit)
    invalid('informe a coluna de valor ou as colunas de débito e crédito');
  const joined = (cells: string[], c: number[] | null) =>
    (c ?? [])
      .map((i) => cells[i]?.trim() ?? '')
      .filter(Boolean)
      .join(' - ');
  const rowsOut: RawRow[] = [];
  for (const rec of recs) {
    const dateCell = joined(rec.cells, cols.date);
    const amountCells = [cols.amount, cols.debit, cols.credit]
      .filter(Boolean)
      .map((c) => joined(rec.cells, c));
    if (!dateCell && amountCells.every((c) => !c)) continue;
    const row: RawRow = {
      line: rec.line + skip,
      date: parseCsvDate(dateCell, mapping.date_format),
      amount: null,
      description: joined(rec.cells, cols.description),
      payee: cols.payee
        ? (mapping.card_descriptor
            ? cardPayee(joined(rec.cells, cols.payee))
            : joined(rec.cells, cols.payee)) || null
        : null,
      notes: null,
      external_id: cols.external ? joined(rec.cells, cols.external) || null : null,
      trn_type: null,
      error: null,
    };
    if (!row.date) row.error = `data inválida: "${dateCell}" (formato ${mapping.date_format})`;
    let amount: number | null = null;
    const a = cols.amount ? joined(rec.cells, cols.amount) : '';
    if (cols.amount && (a || (!cols.debit && !cols.credit))) {
      amount = parseAmount(a, mapping.decimal_comma);
      if (amount == null) row.error ??= `valor inválido: ${a}`;
    } else {
      const d = joined(rec.cells, cols.debit);
      const c = joined(rec.cells, cols.credit);
      if (!d && !c) row.error ??= 'linha sem valor';
      else {
        const pc = c ? parseAmount(c, mapping.decimal_comma) : 0;
        const pd = d ? parseAmount(d, mapping.decimal_comma) : 0;
        if (pc == null || pd == null) row.error ??= `valor inválido: ${pc == null ? c : d}`;
        else amount = Math.abs(pc) - Math.abs(pd);
      }
    }
    if (amount != null) row.amount = mapping.invert_sign ? -amount : amount;
    rowsOut.push(row);
  }
  return { delimiter: delim, skip_rows: skip, headers: headers ?? [], rows: rowsOut };
}

// ── Classificação e deduplicação ────────────────────────────────

const BILL_PAYMENT = ['pagamento de fatura', 'pagamento da fatura', 'pagamento fatura', 'pagto fatura', 'pgto fatura', 'pag fatura', 'pagamento cartao de credito', 'pagamento cartao credito', 'pagto cartao', 'pgto cartao', 'pag cartao credito', 'fatura cartao', 'fatura do cartao', 'fatura de cartao']; // prettier-ignore
const CARD_SIDE = ['pagamento recebido', 'pagamento efetuado', 'pagamento obrigado', 'pagamento em ', 'credito de pagamento']; // prettier-ignore

function isBillPayment(desc: string, cardSide: boolean): boolean {
  const d = `${fold(desc)} `;
  return (
    BILL_PAYMENT.some((p) => d.includes(p)) || (cardSide && CARD_SIDE.some((p) => d.includes(p)))
  );
}

function isBalanceLine(desc: string): boolean {
  const d = fold(desc);
  return (
    d === 'saldo' ||
    d.startsWith('saldo ') ||
    d.includes('saldo anterior') ||
    d.includes('saldo do dia') ||
    d.includes('saldo final')
  );
}

interface Pending {
  row: PreviewRow;
  notes: string | null;
  applied: string[];
}

function buildRows(accountId: string, raw: RawRow[], card: boolean, warnings: string[]): Pending[] {
  const ignored = raw.map((r) =>
    isBalanceLine(r.description)
      ? 'linha de saldo, não é lançamento'
      : r.error
        ? null
        : r.amount === 0
          ? 'valor zero'
          : null,
  );
  if (card) {
    const purchases = raw.filter(
      (r, i) => !r.error && !ignored[i] && !isBillPayment(r.description, true) && r.amount != null,
    );
    const positives = purchases.filter((r) => r.amount! > 0).length;
    if (purchases.length >= 3 && positives * 3 > purchases.length * 2) {
      for (const r of raw) if (r.amount != null) r.amount = -r.amount;
      warnings.push(
        'o arquivo mostra compras como valores positivos; os sinais foram invertidos para que compras fiquem negativas no cartão',
      );
    }
  }
  const rowsOut: PreviewRow[] = raw.map((r, index) => {
    let kind: TxnKind | null = null;
    let note: string | null = ignored[index] ?? null;
    if (r.amount != null && !r.error && !ignored[index]) {
      const payment = r.trn_type === 'PAYMENT';
      if (card && (isBillPayment(r.description, true) || (payment && r.amount > 0))) {
        if (r.amount < 0) {
          r.amount = -r.amount;
          note = 'pagamento de fatura: sinal ajustado para crédito no cartão';
        }
        kind = 'card_payment';
      } else if (!card && r.amount < 0 && isBillPayment(r.description, false))
        kind = 'card_payment';
      else kind = r.amount < 0 ? 'expense' : 'income';
    }
    return {
      index,
      line: r.line,
      date: r.date,
      amount: r.amount,
      description: r.description,
      payee: r.payee,
      external_id: r.external_id,
      status: ignored[index] ? 'ignored' : r.error ? 'error' : 'new',
      duplicate_of: null,
      kind,
      suggested_category_id: null,
      suggested_member_id: null,
      suggested_tags: [],
      suggested_description: null,
      rule_id: null,
      error: ignored[index] ? null : r.error,
      note,
      card_last4: null,
      card_plastic_id: null,
      orig_amount: null,
      orig_currency: null,
      planned_id: null,
    };
  });

  // 1. FITID repetido no próprio arquivo.
  const seen = new Map<string, string[]>();
  let collisions = 0;
  for (const row of rowsOut) {
    if (row.status !== 'new' || !row.external_id) continue;
    const ext = row.external_id;
    const fp = fingerprint(row.date ?? '', row.amount ?? 0, row.description);
    const fps = seen.get(ext) ?? [];
    if (fps.includes(fp)) {
      row.status = 'duplicate';
      row.note = `FITID ${ext} repetido no próprio arquivo`;
    } else if (fps.length === 0) {
      fps.push(fp);
    } else {
      fps.push(fp);
      const id = `${ext}#${fps.length}`;
      row.note = `o banco repetiu o FITID ${ext} em lançamentos diferentes; gravado como ${id}`;
      row.external_id = id;
      collisions++;
    }
    seen.set(ext, fps);
  }
  if (collisions) {
    warnings.push(
      `${collisions} lançamento(s) com FITID repetido e conteúdo diferente receberam um id próprio`,
    );
  }
  // 2. Mesmo id externo já gravado na conta.
  const existing = rows('txn').filter((t) => t.account_id === accountId && t.status !== 'planned');
  for (const row of rowsOut) {
    if (row.status !== 'new' || !row.external_id) continue;
    const hit = existing.find((t) => t.external_id === row.external_id);
    if (hit) {
      row.status = 'duplicate';
      row.duplicate_of = hit.id;
      row.note = 'já importado (mesmo identificador)';
    }
  }
  // 3. Possíveis duplicados.
  const consumed = new Set<string>();
  for (const row of rowsOut) {
    if (row.status !== 'new' || !row.date || row.amount == null) continue;
    const fp = fingerprint(row.date, row.amount, row.description);
    let hit = existing
      .filter((t) => t.fingerprint === fp)
      .sort((a, b) => a.created_at.localeCompare(b.created_at))
      .find((t) => !consumed.has(t.id));
    hit ??= existing.find(
      (t) =>
        !consumed.has(t.id) &&
        t.date === row.date &&
        t.amount === row.amount &&
        verySimilar(t.description, row.description),
    );
    if (hit) {
      consumed.add(hit.id);
      row.status = 'possible_duplicate';
      row.duplicate_of = hit.id;
      row.note = 'mesma data e valor de um lançamento existente, com descrição igual ou parecida';
    }
  }
  // 4. Previsto que a linha efetiva (mesmas regras de `import::engine::planned_match`).
  const taken = new Set<string>();
  for (const row of rowsOut) {
    if (row.status !== 'new' || !row.date || row.amount == null) continue;
    if (row.kind !== 'expense' && row.kind !== 'income') continue;
    const hit = plannedMatch(accountId, row.date, row.amount, taken);
    if (!hit) continue;
    taken.add(hit.id);
    const note = `efetiva o previsto “${hit.description}” de ${brDate(hit.date)}`;
    row.note = row.note ? `${row.note} · ${note}` : note;
    row.planned_id = hit.id;
  }
  // Regras.
  const set = loadRules();
  return rowsOut.map((row, i) => {
    const p: Pending = { row, notes: raw[i]!.notes, applied: [] };
    const eligible =
      (row.status === 'new' || row.status === 'possible_duplicate') &&
      row.kind &&
      row.kind !== 'card_payment';
    if (eligible && set.length) {
      const out = evaluate(set, {
        description: row.description,
        payee: row.payee,
        notes: raw[i]!.notes,
        amount: row.amount ?? 0,
        account_id: accountId,
        kind: row.kind!,
      });
      if (out.kind) row.kind = out.kind;
      row.suggested_category_id = isTransferLike(row.kind!) ? null : (out.category_id ?? null);
      row.suggested_member_id = out.member_id ?? null;
      row.suggested_tags = out.tags ?? [];
      row.suggested_description = out.rename_to ?? null;
      row.rule_id = out.applied[0] ?? out.matched[0] ?? null;
      p.applied = out.applied;
    }
    return p;
  });
}

function counts(list: Pending[]): PreviewCounts {
  const c: PreviewCounts = { new: 0, duplicate: 0, possible_duplicate: 0, error: 0, ignored: 0 };
  for (const p of list) c[p.row.status]++;
  return c;
}

// ── Prévias guardadas na "sessão" ───────────────────────────────

interface Stored {
  token: string;
  account_id: string;
  currency: string;
  filename: string;
  format: ImportFormat;
  sha: string;
  rows: Pending[];
  /** Conta lida do OFX, para lembrar a conta escolhida ao gravar. */
  ofx: OfxAccountInfo | null;
}

let previews: Stored[] = [];
onMockReset(() => {
  previews = [];
});

function detect(args: Record<string, unknown>): DetectResult {
  const filename = str(args, 'filename');
  const bytes = decodeBase64(str(args, 'content_base64'));
  const probe = new TextDecoder('windows-1252').decode(bytes.slice(0, 4000));
  const format = detectFormat(probe, filename);
  if (format === 'ofx') {
    const dec = decodeText(bytes, args.encoding as string | undefined, declaredCharset(probe));
    const stmts = parseOfx(dec.text);
    const s = pickStatement(stmts, null, false);
    return {
      format,
      encoding: dec.encoding,
      delimiter: null as unknown as undefined,
      skip_rows: 0,
      has_header: true,
      headers: ['Data', 'Valor', 'Descrição', 'FITID'],
      sample_rows: s.txns
        .slice(0, 10)
        .map((t) => [
          t.date ?? '',
          t.amount != null ? (t.amount / 100).toFixed(2) : '',
          t.description,
          t.external_id ?? '',
        ]),
      row_count: s.txns.length,
      suggested_preset_id: null,
      suggested_mapping: null,
      ofx: s.info,
    };
  }
  const dec = decodeText(bytes, args.encoding as string | undefined, null);
  const delim = detectDelimiter(dec.text);
  const layout = detectLayout(dec.text, delim);
  const recs = readRecords(skipLines(dec.text, layout.skip_rows), delim);
  const data = layout.has_header ? recs.slice(1) : recs;
  const suggested = suggestPreset(layout.headers);
  const preset = PRESETS.find((p) => p.id === suggested);
  return {
    format,
    encoding: dec.encoding,
    delimiter: delimiterName(delim),
    skip_rows: layout.skip_rows,
    has_header: layout.has_header,
    headers: layout.headers,
    sample_rows: data.slice(0, 10).map((r) => r.cells),
    row_count: data.length,
    suggested_preset_id: suggested,
    suggested_mapping: preset
      ? {
          ...preset.mapping,
          delimiter: delimiterName(delim),
          skip_rows: layout.skip_rows,
          encoding: dec.encoding,
        }
      : null,
    ofx: null,
  };
}

function preview(args: Record<string, unknown>): ImportPreview {
  const filename = text(args.filename).trim();
  if (!filename) invalid('informe o nome do arquivo');
  const content = str(args, 'content_base64');
  const bytes = decodeBase64(content);
  const accountId = str(args, 'account_id');
  const acc = currencyAndKind(accountId);
  const sha = hashHex(content);
  const probe = new TextDecoder('windows-1252').decode(bytes.slice(0, 4000));
  const format = (args.format as ImportFormat | undefined) ?? detectFormat(probe, filename);
  const warnings: string[] = [];
  let raw: RawRow[];
  let ofx: OfxAccountInfo | null = null;
  let mapping: CsvMapping | null = null;
  let presetId: string | null = null;
  let encoding: string;
  const card = acc.kind === 'credit_card';
  if (format === 'ofx') {
    const dec = decodeText(bytes, args.encoding as string | undefined, declaredCharset(probe));
    encoding = dec.encoding;
    const stmts = parseOfx(dec.text);
    const s = pickStatement(stmts, args.ofx_account as string | undefined, card);
    if (stmts.length > 1) {
      warnings.push(
        `o arquivo tem ${stmts.length} extratos; importando o da conta ${s.info.account_id ?? 'sem número'} (escolha outro com ofx_account)`,
      );
    }
    if (s.info.currency && s.info.currency.toUpperCase() !== acc.currency) {
      warnings.push(
        `o extrato está em ${s.info.currency} e a conta em ${acc.currency}; os valores foram lidos na moeda da conta`,
      );
    }
    if (s.info.credit_card !== card) {
      warnings.push(
        s.info.credit_card
          ? 'o arquivo é de cartão de crédito e a conta escolhida não é cartão'
          : 'a conta escolhida é cartão, mas o arquivo é extrato de conta',
      );
    }
    raw = s.txns;
    ofx = s.info;
  } else {
    const explicit = args.csv_mapping as CsvMapping | undefined;
    const dec = decodeText(
      bytes,
      explicit?.encoding ?? (args.encoding as string | undefined),
      null,
    );
    encoding = dec.encoding;
    if (explicit) {
      mapping = { ...explicit };
      presetId = (args.preset_id as string | undefined) ?? null;
    } else if (typeof args.preset_id === 'string') {
      const p =
        PRESETS.find((x) => x.id === args.preset_id) ??
        notFound(`preset ${String(args.preset_id)}`);
      mapping = { ...p.mapping };
      presetId = p.id;
    } else {
      const layout = detectLayout(dec.text, detectDelimiter(dec.text));
      const s = suggestPreset(layout.headers);
      if (!s) {
        invalid(
          `não reconheci o layout do CSV (colunas: ${layout.headers.join(', ')}); escolha um preset ou informe o mapeamento`,
        );
      }
      mapping = { ...PRESETS.find((p) => p.id === s)!.mapping };
      presetId = s;
    }
    const parsed = parseCsv(dec.text, mapping);
    mapping = {
      ...mapping,
      delimiter: delimiterName(parsed.delimiter),
      skip_rows: parsed.skip_rows,
      encoding: dec.encoding,
    };
    raw = parsed.rows;
  }
  if (raw.length === 0) invalid('nenhum lançamento encontrado no arquivo');
  const pending = buildRows(accountId, raw, card, warnings);
  const previous = rows('import_batch')
    .filter((b) => b.account_id === accountId && b.file_sha256 === sha)
    .sort((a, b) => b.created_at.localeCompare(a.created_at))[0];
  if (previous) warnings.unshift('este mesmo arquivo já foi importado nesta conta');
  const token = newId();
  previews.push({
    token,
    account_id: accountId,
    currency: acc.currency,
    filename,
    format,
    sha,
    rows: pending,
    ofx,
  });
  if (previews.length > 8) previews = previews.slice(-8);
  return {
    token,
    format,
    encoding,
    account_id: accountId,
    filename,
    file_sha256: sha,
    already_imported: Boolean(previous),
    already_imported_batch_id: previous?.id ?? null,
    currency: acc.currency,
    rows: pending.map((p) => p.row),
    counts: counts(pending),
    ofx,
    csv_mapping: mapping,
    preset_id: presetId,
    warnings,
  };
}

function commit(args: Record<string, unknown>): ImportResult {
  const token = str(args, 'token');
  const stored = previews.find((p) => p.token === token);
  if (!stored) notFound('prévia de importação (expirada ou já gravada); gere a prévia de novo');
  currencyAndKind(stored.account_id);
  const skip = new Set((Array.isArray(args.skip_indexes) ? args.skip_indexes : []).map(Number));
  const includePossible = args.include_possible_duplicates === true;
  const overrides = new Map<number, RowOverride>(
    (Array.isArray(args.overrides) ? (args.overrides as RowOverride[]) : []).map((o) => [
      o.index,
      o,
    ]),
  );
  for (const o of overrides.values()) {
    if (o.index >= stored.rows.length) invalid(`linha ${o.index} não existe na prévia`);
    if (o.category_id && !find('category', o.category_id)) notFound(`categoria ${o.category_id}`);
    if (o.member_id && !find('member', o.member_id)) notFound(`membro ${o.member_id}`);
  }
  let duplicates = 0;
  const errors: ImportRowError[] = [];
  const toInsert: { p: Pending; o?: RowOverride }[] = [];
  for (const p of stored.rows) {
    const r = p.row;
    if (r.status === 'duplicate' || (r.status === 'possible_duplicate' && !includePossible))
      duplicates++;
    if (r.status === 'error') errors.push({ index: r.index, line: r.line, message: r.error ?? '' });
    const insertable = r.status === 'new' || (r.status === 'possible_duplicate' && includePossible);
    if (!insertable || skip.has(r.index) || !r.date || r.amount == null || !r.kind) continue;
    toInsert.push({ p, o: overrides.get(r.index) });
  }
  const rec = write(
    `Importação de ${stored.filename}`,
    () => {
      const batch = newId();
      const now = new Date().toISOString();
      put('import_batch', {
        id: batch,
        filename: stored.filename,
        format: stored.format,
        file_sha256: stored.sha,
        account_id: stored.account_id,
        total_rows: stored.rows.length,
        inserted: 0,
        duplicates: 0,
        errors: [],
        created_at: now,
        updated_at: now,
      });
      let inserted = 0;
      let settled = 0;
      const ids: string[] = [];
      let late = 0;
      const seenExt = new Set<string>();
      const hits = new Map<string, number>();
      for (const { p, o } of toInsert) {
        const r = p.row;
        if (r.external_id) {
          const taken = rows('txn').some(
            (t) => t.account_id === stored.account_id && t.external_id === r.external_id,
          );
          if (taken || seenExt.has(r.external_id)) {
            late++;
            continue;
          }
          seenExt.add(r.external_id);
        }
        const kind = o?.kind ?? r.kind!;
        let category =
          o?.category_id ??
          (r.suggested_category_id && find('category', r.suggested_category_id)
            ? r.suggested_category_id
            : null);
        if (isTransferLike(kind) && !o?.category_id) category = null;
        const member =
          o?.member_id ??
          (r.suggested_member_id && find('member', r.suggested_member_id)
            ? r.suggested_member_id
            : null);
        const newDesc = o?.description?.trim() || r.suggested_description || null;
        const renamed = Boolean(newDesc && newDesc !== r.description);
        const notes = [p.notes, renamed ? `Descrição no extrato: ${r.description}` : null]
          .filter(Boolean)
          .join(' · ');
        const t = insertTxn({
          account_id: stored.account_id,
          date: r.date!,
          amount: r.amount!,
          currency: stored.currency,
          kind,
          description: newDesc ?? r.description,
          payee: r.payee,
          notes: notes || null,
          category_id: category,
          member_id: member,
          external_id: r.external_id,
          import_id: batch,
        });
        if (renamed)
          patch('txn', t.id, { fingerprint: fingerprint(r.date!, r.amount!, r.description) });
        const tags = o?.tags ?? r.suggested_tags;
        if (tags.length) replaceTags(t.id, tags);
        if (!o?.category_id) for (const id of p.applied) hits.set(id, (hits.get(id) ?? 0) + 1);
        const planned = r.status === 'new' && r.planned_id ? find('txn', r.planned_id) : undefined;
        if (planned && planned.status === 'planned') {
          const real = find('txn', t.id)!;
          patch('txn', t.id, {
            category_id: real.category_id ?? planned.category_id,
            member_id: real.member_id ?? planned.member_id,
            recurrence_id: real.recurrence_id ?? planned.recurrence_id,
          });
          deleteRows([planned]);
          settled++;
        }
        ids.push(t.id);
        inserted++;
      }
      patch('import_batch', batch, { inserted, duplicates: duplicates + late, errors });
      recordHits(hits);
      return { batch, inserted, paid: linkCategoryPayments(ids), settled };
    },
    author('import', stored.filename),
  );
  // A próxima importação do mesmo OFX já vem com a conta escolhida.
  if (stored.format === 'ofx' && stored.ofx) rememberOfx(stored.ofx, stored.account_id);
  previews = previews.filter((p) => p.token !== token);
  return {
    batch_id: rec.value.batch,
    inserted: rec.value.inserted,
    installments_paid: rec.value.paid,
    transfers_linked: 0,
    planned_settled: rec.value.settled,
    skipped: stored.rows.length - rec.value.inserted,
    change_set_id: rec.changeSetId,
    backup_id: toInsert.length > 200 ? newId() : null,
  };
}

function listBatches(args: Record<string, unknown>): ImportBatch[] {
  const limit = Math.min(Math.max(Number(args.limit ?? 100), 1), 1000);
  return rows('import_batch')
    .filter((b) => !args.account_id || b.account_id === args.account_id)
    .sort((a, b) => b.created_at.localeCompare(a.created_at) || b.id.localeCompare(a.id))
    .slice(0, limit)
    .map((b) => ({
      id: b.id,
      filename: b.filename,
      format: b.format,
      file_sha256: b.file_sha256,
      account_id: b.account_id,
      account_name: find('account', b.account_id)?.name ?? '?',
      total_rows: b.total_rows,
      inserted: b.inserted,
      duplicates: b.duplicates,
      errors: b.errors,
      created_at: b.created_at,
      change_set_id: findChangeSetInserting('import_batch', b.id),
    }));
}

/** Espelho simples de `import.sniff`: pelo conteúdo e pela extensão. */
/**
 * Espelho de `import::image::analyze`: o mock não tem OCR; o nome do arquivo diz o que a
 * imagem é (`cupom…`, `comprovante…`, `boleto…`), o resto é documento.
 */
function analyzeImage(args: Record<string, unknown>) {
  const name = (typeof args.filename === 'string' ? args.filename : '').toLowerCase();
  if (name.includes('cupom')) {
    return {
      kind: 'receipt',
      label: 'Cupom fiscal lido no computador: 2 item(ns)',
      note: 'a conta fechou; revise e grave a compra',
      reading: null,
      lines: 6,
    };
  }
  if (name.includes('comprovante'))
    return {
      kind: 'payment_proof',
      label: 'Comprovante de pagamento ou transferência',
      note: 'o agente de IA registra o lançamento a partir da imagem',
      reading: null,
      lines: 5,
    };
  if (name.includes('boleto'))
    return {
      kind: 'bill',
      label: 'Boleto',
      note: 'o agente de IA registra a conta a pagar a partir da imagem',
      reading: null,
      lines: 7,
    };
  return {
    kind: 'document',
    label: 'Imagem com texto, sem importador',
    note: 'só o agente de IA lê este tipo de imagem',
    reading: null,
    lines: 3,
  };
}

function sniff(args: Record<string, unknown>) {
  const filename = typeof args.filename === 'string' ? args.filename : '';
  const lower = filename.toLowerCase();
  const b64 = typeof args.content_base64 === 'string' ? args.content_base64 : '';
  let head: string;
  try {
    head = atob(b64).slice(0, 4000);
  } catch {
    head = '';
  }
  const out = (kind: string, format: string | null, label: string, note: string | null = null) => ({
    filename,
    kind,
    format,
    label,
    note,
    needs_password: false,
    layout: null,
  });
  const img = head.startsWith('\x89PNG')
    ? 'png'
    : head.startsWith('\xFF\xD8\xFF')
      ? 'jpeg'
      : head.startsWith('GIF8')
        ? 'gif'
        : head.slice(0, 4) === 'RIFF' && head.slice(8, 12) === 'WEBP'
          ? 'webp'
          : head.slice(4, 8) === 'ftyp' && /^(hei[cxms]|hev[cx]|mif1|msf1)$/.test(head.slice(8, 12))
            ? 'heic'
            : null;
  if (img)
    return out(
      'image',
      img,
      `Imagem ${img.toUpperCase()}`,
      'lida no computador (QR e texto): cupom fiscal abre a compra preenchida; o resto vai ao agente de IA só se você quiser',
    );
  if (/<infNFe|portalfiscal\.inf\.br\/nfe/.test(head))
    return out(
      'receipt',
      'xml',
      'Nota fiscal em XML (NF-e/NFC-e)',
      'abre a compra com os itens para revisar e gravar em Consumo',
    );
  if (head.startsWith('%PDF')) {
    // Extrato por período da Caixa (no app real, a página é uma imagem lida pelo OCR).
    if (/EXTRATO POR PER.{1,3}ODO/i.test(head))
      return out(
        'statement',
        'pdf',
        'Extrato da Caixa (PDF, lido no computador)',
        'importe na conta corrente da Caixa; os lançamentos e os saldos de cada dia foram lidos sem IA',
      );
    if (/NFC-?e|Consumidor Eletr/i.test(head) || lower.includes('cupom'))
      return out(
        'receipt',
        'pdf',
        'Cupom fiscal em PDF (DANFE NFC-e)',
        'abre a compra com os itens para revisar e gravar em Consumo',
      );
    return /NOTA DE (CORRETAGEM|NEGOCIA)/i.test(head) || lower.includes('nota')
      ? out('brokerage_note', 'pdf', 'Nota de corretagem (SINACOR)')
      : out(
          'unknown',
          'pdf',
          'PDF',
          'não parece uma nota de corretagem; o agente pode ler o documento',
        );
  }
  if (/\.(xlsx|xls|ods)$/.test(lower))
    return out('b3_report', 'xlsx', 'Relatório da Área do Investidor B3');
  if (/<OFX>|OFXHEADER/i.test(head) || /\.(ofx|qfx)$/.test(lower))
    return out('statement', 'ofx', 'Extrato OFX');
  if (/\.csv$/.test(lower) && /data|date/i.test(head))
    return out('statement', 'csv', 'Extrato CSV');
  return out(
    'unknown',
    lower.includes('.') ? lower.split('.').pop()! : null,
    'Formato não reconhecido',
    'não é um extrato OFX/CSV, uma nota de corretagem nem um relatório da B3',
  );
}

// Caixa de entrada de arquivos (`import/inbox.rs`): sem pasta de verdade no navegador; os testes
// põem arquivos com `mockInboxAdd`.
let inbox: (InboxItem & { content_base64: string })[] = [];
onMockReset(() => {
  inbox = [];
});

/** Faz o papel de um arquivo que chegou na pasta de entrada (o mesmo conteúdo não duplica). */
export function mockInboxAdd(filename: string, content: string): InboxItem | null {
  const content_base64 = btoa(content);
  if (inbox.some((i) => i.content_base64 === content_base64)) return null;
  const s = sniff({ filename, content_base64 });
  const item = {
    id: `inbox-${inbox.length + 1}`,
    filename,
    size: content.length,
    sha256: `sha-${inbox.length + 1}`,
    detected_kind: s.kind as FileKind,
    label: s.label,
    status: s.kind === 'unknown' ? 'unrecognized' : 'pending',
    note: s.note,
    created_at: new Date().toISOString(),
    content_base64,
  } satisfies InboxItem & { content_base64: string };
  inbox.unshift(item);
  const { content_base64: _c, ...view } = item;
  return view;
}

function inboxView(i: InboxItem & { content_base64: string }): InboxItem {
  const { content_base64: _c, ...view } = i;
  return view;
}

registerMockHandlers(
  guarded({
    'import.inbox_list': (a) =>
      inbox
        .filter((i) => a.include_done === true || ['pending', 'unrecognized'].includes(i.status))
        .map(inboxView),
    'import.inbox_scan': () => ({
      folder: '~/Treasury/Inbox',
      added: [],
      duplicates: 0,
      too_large: [],
    }),
    'import.inbox_file': (a) => {
      const i = inbox.find((x) => x.id === str(a, 'id'));
      if (!i) notFound(`arquivo ${str(a, 'id')} da caixa de entrada`);
      return { id: i.id, filename: i.filename, content_base64: i.content_base64 };
    },
    'import.inbox_set_status': (a) => {
      const i = inbox.find((x) => x.id === str(a, 'id'));
      if (!i) notFound(`arquivo ${str(a, 'id')} da caixa de entrada`);
      i.status = str(a, 'status') as InboxStatus;
      return inboxView(i);
    },
    // Documento como texto (`pdf_markdown.rs`): no navegador, o conteúdo como veio.
    'import.read_document': (a) => {
      const raw = atob(str(a, 'content_base64'));
      const markdown = `<!-- página 1 -->\n\n${raw}`;
      return {
        filename: str(a, 'filename'),
        format: 'markdown',
        engine: 'layout',
        pages: 1,
        plain_pages: [],
        markdown,
        offset: 0,
        next_offset: null,
        total_chars: markdown.length,
      };
    },
    'import.sniff': sniff,
    'import.analyze_image': analyzeImage,
    'import.detect': detect,
    'import.preview': preview,
    'import.commit': commit,
    'import.list_batches': listBatches,
    'import.list_presets': () => PRESETS,
  }),
);
