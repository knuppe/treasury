/**
 * Markdown das respostas do agente (subconjunto do CommonMark + GFM): títulos, parágrafos,
 * ênfase, código, citações, listas (aninhadas, numeradas e de tarefas), tabelas, linhas
 * horizontais e links. Gera uma árvore simples que a interface desenha com elementos React;
 * HTML do texto nunca é interpretado (vira texto). Tolerante a texto incompleto: durante o
 * streaming, um `**` ou uma cerca de código sem fechamento continuam legíveis.
 */

export type Inline =
  | { t: 'text'; v: string }
  | { t: 'code'; v: string }
  | { t: 'strong'; c: Inline[] }
  | { t: 'em'; c: Inline[] }
  | { t: 'del'; c: Inline[] }
  | { t: 'link'; href: string; c: Inline[] }
  | { t: 'br' };

export type Align = 'left' | 'center' | 'right' | null;

export interface ListItem {
  checked: boolean | null;
  blocks: Block[];
}

export type Block =
  | { t: 'heading'; level: 1 | 2 | 3 | 4 | 5 | 6; c: Inline[] }
  | { t: 'paragraph'; c: Inline[] }
  | { t: 'code'; lang: string; v: string }
  | { t: 'quote'; blocks: Block[] }
  | { t: 'list'; ordered: boolean; start: number; items: ListItem[] }
  | { t: 'table'; align: Align[]; head: Inline[][]; rows: Inline[][][] }
  | { t: 'hr' };

// --- Inline ------------------------------------------------------------------------------

const ESCAPABLE = '\\`*_{}[]()#+-.!|~<>';

/** Endereço que pode virar link: https/http, e-mail e telas do app (`/agente?…`). */
export function safeHref(raw: string): string | null {
  const href = raw.trim().replace(/^<|>$/g, '');
  if (/^https?:\/\/[^\s]+$/i.test(href)) return href;
  if (/^mailto:[^\s@]+@[^\s@]+$/i.test(href)) return href;
  if (/^\/(?!\/)[^\s]*$/.test(href)) return href;
  return null;
}

function pushText(out: Inline[], v: string) {
  if (!v) return;
  const last = out[out.length - 1];
  if (last?.t === 'text') last.v += v;
  else out.push({ t: 'text', v });
}

/** Fecho do delimitador `d` a partir de `from`, sem atravessar código. */
function findClose(s: string, d: string, from: number): number {
  let i = from;
  while (i < s.length) {
    if (s[i] === '\\') {
      i += 2;
      continue;
    }
    if (s[i] === '`') {
      const end = s.indexOf('`', i + 1);
      if (end < 0) return -1;
      i = end + 1;
      continue;
    }
    if (s.startsWith(d, i)) {
      // `*` simples não fecha num `**`.
      if (d.length === 1 && s[i + 1] === d) {
        i += 2;
        continue;
      }
      if (i > from && s[i - 1] !== ' ') return i;
    }
    i += 1;
  }
  return -1;
}

const BARE_URL = /^https?:\/\/[^\s<>()]+[^\s<>().,;:!?'"]/i;

export function parseInline(src: string): Inline[] {
  const out: Inline[] = [];
  let i = 0;
  let buf = '';
  const flush = () => {
    pushText(out, buf);
    buf = '';
  };
  while (i < src.length) {
    const ch = src[i]!;
    const rest = src.slice(i);
    if (ch === '\\' && i + 1 < src.length && ESCAPABLE.includes(src[i + 1]!)) {
      buf += src[i + 1];
      i += 2;
      continue;
    }
    if (ch === '\n') {
      flush();
      out.push({ t: 'br' });
      i += 1;
      continue;
    }
    if (ch === '`') {
      const run = /^`+/.exec(rest)![0];
      const end = src.indexOf(run, i + run.length);
      if (end > 0) {
        flush();
        out.push({ t: 'code', v: src.slice(i + run.length, end).replace(/^ (.+) $/, '$1') });
        i = end + run.length;
        continue;
      }
    }
    if (ch === '[') {
      const m = /^\[([^\]\n]+)\]\(([^)\s]+)(?:\s+"[^"]*")?\)/.exec(rest);
      if (m) {
        const href = safeHref(m[2]!);
        flush();
        if (href) out.push({ t: 'link', href, c: parseInline(m[1]!) });
        else out.push(...parseInline(m[1]!));
        i += m[0].length;
        continue;
      }
    }
    if (ch === '<') {
      const m = /^<(https?:\/\/[^\s>]+)>/i.exec(rest);
      if (m) {
        flush();
        out.push({ t: 'link', href: m[1]!, c: [{ t: 'text', v: m[1]! }] });
        i += m[0].length;
        continue;
      }
    }
    if ((ch === 'h' || ch === 'H') && (i === 0 || /[\s(]/.test(src[i - 1]!))) {
      const m = BARE_URL.exec(rest);
      if (m) {
        flush();
        out.push({ t: 'link', href: m[0], c: [{ t: 'text', v: m[0] }] });
        i += m[0].length;
        continue;
      }
    }
    if (ch === '*' || ch === '_' || ch === '~') {
      const double = src[i + 1] === ch;
      // `_` dentro de palavra (nome_de_arquivo) é texto.
      const intraword = ch === '_' && i > 0 && /\w/.test(src[i - 1]!);
      if ((ch !== '~' || double) && !intraword && src[i + (double ? 2 : 1)] !== ' ') {
        const d = double ? ch + ch : ch;
        const close = findClose(src, d, i + d.length);
        const after = close >= 0 ? src[close + d.length] : undefined;
        const closesWord = ch !== '_' || after === undefined || !/\w/.test(after);
        if (close > i + d.length && closesWord) {
          flush();
          const inner = parseInline(src.slice(i + d.length, close));
          if (ch === '~') out.push({ t: 'del', c: inner });
          else if (double) out.push({ t: 'strong', c: inner });
          else out.push({ t: 'em', c: inner });
          i = close + d.length;
          continue;
        }
      }
    }
    buf += ch;
    i += 1;
  }
  flush();
  return out;
}

// --- Blocos ------------------------------------------------------------------------------

const HEADING = /^ {0,3}(#{1,6})(?:[ \t]+(.*?))?(?:[ \t]+#+)?[ \t]*$/;
const FENCE = /^ {0,3}(`{3,}|~{3,})[ \t]*([^`\s]*)[^`]*$/;
const HR = /^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$/;
const QUOTE = /^ {0,3}> ?/;
const BULLET = /^( {0,12})([-*+])[ \t]+(.*)$/;
const ORDERED = /^( {0,12})(\d{1,9})[.)][ \t]+(.*)$/;
const TABLE_SEP = /^ {0,3}\|?[ \t]*:?-+:?[ \t]*(\|[ \t]*:?-+:?[ \t]*)*\|?[ \t]*$/;

function splitRow(line: string): string[] {
  let s = line.trim();
  if (s.startsWith('|')) s = s.slice(1);
  if (s.endsWith('|') && !s.endsWith('\\|')) s = s.slice(0, -1);
  const cells: string[] = [];
  let cur = '';
  let code = false;
  for (let i = 0; i < s.length; i++) {
    const c = s[i]!;
    if (c === '\\' && s[i + 1] === '|') {
      cur += '|';
      i += 1;
    } else if (c === '`') {
      code = !code;
      cur += c;
    } else if (c === '|' && !code) {
      cells.push(cur.trim());
      cur = '';
    } else cur += c;
  }
  cells.push(cur.trim());
  return cells;
}

function alignOf(cell: string): Align {
  const c = cell.trim();
  const l = c.startsWith(':');
  const r = c.endsWith(':');
  return l && r ? 'center' : r ? 'right' : l ? 'left' : null;
}

function isBlockStart(line: string): boolean {
  return (
    HEADING.test(line) ||
    FENCE.test(line) ||
    HR.test(line) ||
    QUOTE.test(line) ||
    BULLET.test(line) ||
    ORDERED.test(line)
  );
}

function indentOf(line: string): number {
  return /^ */.exec(line)![0].length;
}

function parseList(lines: string[], start: number): { block: Block; next: number } {
  const first = lines[start]!;
  const ordered = !BULLET.test(first);
  const m0 = (ordered ? ORDERED : BULLET).exec(first)!;
  const base = m0[1]!.length;
  const items: ListItem[] = [];
  let i = start;
  while (i < lines.length) {
    const line = lines[i]!;
    const m = (ordered ? ORDERED : BULLET).exec(line);
    if (!m || m[1]!.length !== base) break;
    const marker = line.length - m[3]!.length - m[1]!.length;
    const contentIndent = base + marker;
    const body: string[] = [m[3]!];
    i += 1;
    let blankRun = 0;
    while (i < lines.length) {
      const l = lines[i]!;
      if (l.trim() === '') {
        blankRun += 1;
        body.push('');
        i += 1;
        continue;
      }
      const ind = indentOf(l);
      if (ind >= Math.min(contentIndent, base + 2)) {
        body.push(l.slice(Math.min(ind, contentIndent)));
        blankRun = 0;
        i += 1;
        continue;
      }
      // Continuação sem recuo (texto colado na linha de cima).
      if (blankRun === 0 && !isBlockStart(l) && !TABLE_SEP.test(lines[i + 1] ?? '')) {
        body.push(l.trim());
        i += 1;
        continue;
      }
      break;
    }
    while (body.length && body[body.length - 1] === '') body.pop();
    let checked: boolean | null = null;
    const task = /^\[([ xX])\][ \t]+/.exec(body[0]!);
    if (task) {
      checked = task[1] !== ' ';
      body[0] = body[0]!.slice(task[0].length);
    }
    items.push({ checked, blocks: parseBlocks(body.join('\n')) });
    // Uma linha em branco entre itens continua a mesma lista.
    if (i < lines.length && lines[i]!.trim() === '') {
      let j = i;
      while (j < lines.length && lines[j]!.trim() === '') j += 1;
      const nm = j < lines.length ? (ordered ? ORDERED : BULLET).exec(lines[j]!) : null;
      if (nm && nm[1]!.length === base) i = j;
    }
  }
  const start1 = ordered ? Number.parseInt(m0[2]!, 10) : 1;
  return { block: { t: 'list', ordered, start: start1, items }, next: i };
}

export function parseBlocks(src: string): Block[] {
  const lines = src.replace(/\r\n?/g, '\n').replace(/\t/g, '    ').split('\n');
  const blocks: Block[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i]!;
    if (line.trim() === '') {
      i += 1;
      continue;
    }
    const fence = FENCE.exec(line);
    if (fence) {
      const marker = fence[1]!;
      const body: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i]!.trim().startsWith(marker)) {
        body.push(lines[i]!);
        i += 1;
      }
      i += 1; // a cerca de fechamento (ou o fim do texto, no streaming)
      blocks.push({ t: 'code', lang: fence[2] ?? '', v: body.join('\n') });
      continue;
    }
    const h = HEADING.exec(line);
    if (h) {
      blocks.push({
        t: 'heading',
        level: h[1]!.length as 1 | 2 | 3 | 4 | 5 | 6,
        c: parseInline(h[2] ?? ''),
      });
      i += 1;
      continue;
    }
    if (HR.test(line)) {
      blocks.push({ t: 'hr' });
      i += 1;
      continue;
    }
    if (QUOTE.test(line)) {
      const body: string[] = [];
      while (i < lines.length && lines[i]!.trim() !== '' && QUOTE.test(lines[i]!)) {
        body.push(lines[i]!.replace(QUOTE, ''));
        i += 1;
      }
      blocks.push({ t: 'quote', blocks: parseBlocks(body.join('\n')) });
      continue;
    }
    if (BULLET.test(line) || ORDERED.test(line)) {
      // `---` é linha horizontal, não item.
      const r = parseList(lines, i);
      blocks.push(r.block);
      i = r.next;
      continue;
    }
    if (line.includes('|') && i + 1 < lines.length && TABLE_SEP.test(lines[i + 1]!)) {
      const head = splitRow(line);
      const align = splitRow(lines[i + 1]!).map(alignOf);
      if (align.length === head.length) {
        i += 2;
        const rows: Inline[][][] = [];
        while (i < lines.length && lines[i]!.trim() !== '' && lines[i]!.includes('|')) {
          const cells = splitRow(lines[i]!);
          rows.push(head.map((_, k) => parseInline(cells[k] ?? '')));
          i += 1;
        }
        blocks.push({ t: 'table', align, head: head.map((c) => parseInline(c)), rows });
        continue;
      }
    }
    // Parágrafo: até uma linha em branco ou o começo de outro bloco.
    const para: string[] = [line.trim()];
    i += 1;
    while (i < lines.length) {
      const l = lines[i]!;
      if (l.trim() === '' || isBlockStart(l)) break;
      if (l.includes('|') && TABLE_SEP.test(lines[i + 1] ?? '')) break;
      // Setext (`Título` + `===`): vira título.
      if (/^ {0,3}=+[ \t]*$/.test(l) || (/^ {0,3}-+[ \t]*$/.test(l) && para.length === 1)) {
        break;
      }
      para.push(l.trim());
      i += 1;
    }
    const next = lines[i];
    if (next !== undefined && /^ {0,3}(=+|-+)[ \t]*$/.test(next) && para.length === 1) {
      blocks.push({
        t: 'heading',
        level: next.trim().startsWith('=') ? 1 : 2,
        c: parseInline(para[0]!),
      });
      i += 1;
      continue;
    }
    blocks.push({ t: 'paragraph', c: parseInline(para.join('\n')) });
  }
  return blocks;
}

/** Texto sem marcação (resumos, leitores de tela). */
export function inlineText(c: Inline[]): string {
  return c
    .map((n) => {
      switch (n.t) {
        case 'text':
        case 'code':
          return n.v;
        case 'br':
          return ' ';
        default:
          return inlineText(n.c);
      }
    })
    .join('');
}
