/**
 * Mock de `devfeedback.*`: feedback do agente ao desenvolvedor, com as mesmas regras do
 * núcleo (ordem por importância e último reforço, deduplicação pelo título ao adicionar,
 * reforço que reabre). Semeado com alguns exemplos. A interface é sempre o usuário, então
 * o modo desenvolvedor não bloqueia nada aqui (no núcleo, só os agentes dependem dele).
 */
import type { DevFeedback } from '@/api/generated/DevFeedback';
import type { DevFeedbackArea } from '@/api/generated/DevFeedbackArea';
import type { DevFeedbackKind } from '@/api/generated/DevFeedbackKind';
import type { DevFeedbackStatus } from '@/api/generated/DevFeedbackStatus';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { ready } from './ledger/boot';
import { invalid, notFound } from './ledger/errors';

const STOP = new Set(['a', 'o', 'as', 'os', 'de', 'da', 'do', 'das', 'dos', 'e', 'em', 'no']);
STOP.add('na').add('um').add('uma').add('para').add('pra').add('por').add('com').add('que');

function words(text: string): Set<string> {
  const norm = text
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ' ');
  return new Set(
    norm
      .split(' ')
      .filter((w) => w && !STOP.has(w))
      .map((w) => (w.endsWith('s') && w.length > 3 ? w.slice(0, -1) : w)),
  );
}

/** Mesma conta do núcleo (`devfeedback::title_similarity`). */
export function titleSimilarity(a: string, b: string): number {
  const wa = words(a);
  const wb = words(b);
  if (wa.size === 0 || wb.size === 0) return 0;
  const common = [...wa].filter((w) => wb.has(w)).length;
  const union = new Set([...wa, ...wb]).size;
  const jaccard = common / union;
  const shorter = Math.min(wa.size, wb.size);
  if (common === wa.size && common === wb.size) return 1;
  return shorter >= 3 && common === shorter ? Math.max(jaccard, 0.9) : jaccard;
}

let seq = 0;
const id = (p: string) => `${p}-${++seq}`;

/** Os campos que o núcleo acrescenta (V150), vazios. */
function blank(): Omit<
  DevFeedback,
  | 'id'
  | 'title'
  | 'body'
  | 'area'
  | 'kind'
  | 'importance'
  | 'status'
  | 'created_by'
  | 'created_at'
  | 'updated_at'
  | 'last_bumped_at'
  | 'bumps'
  | 'notes'
> {
  return {
    position: null,
    proposal: null,
    acceptance_criteria: null,
    code_pointers: [],
    labels: [],
    assignee: null,
    expected: null,
    actual: null,
    repro_steps: null,
    user_impact: null,
    confirmed_by_user: false,
    client_version: null,
    app_version: null,
    tool_calls: [],
    closed_at: null,
    links: [],
  };
}

function seed(): DevFeedback[] {
  const f = (
    title: string,
    body: string,
    area: DevFeedbackArea,
    kind: DevFeedbackKind,
    status: DevFeedbackStatus,
    created_at: string,
    notes: [string, string | null][],
    extra: Partial<DevFeedback> = {},
  ): DevFeedback => ({
    ...blank(),
    id: id('fb'),
    title,
    body,
    area,
    kind,
    status,
    importance: 1 + notes.length,
    created_by: 'agente (CLI de teste)',
    created_at,
    updated_at: notes.at(-1)?.[0] ?? created_at,
    last_bumped_at: notes.at(-1)?.[0] ?? null,
    bumps: notes.length,
    notes: notes.map(([at, note]) => ({
      id: id('fbn'),
      kind: 'reforco',
      note,
      client: 'agente (CLI de teste)',
      created_at: at,
    })),
    ...extra,
  });
  return [
    f(
      'Falta ferramenta para mover uma compra para outra fatura',
      'Pediram para passar uma compra parcelada para a fatura seguinte. Só existe `update_transaction`, que não mexe na fatura; tive de explicar o caminho na tela. Uma ferramenta `move_to_statement` resolveria.',
      'mcp',
      'ferramenta_faltando',
      'aberto',
      '2026-10-12T14:03:00Z',
      [
        ['2026-10-14T10:21:00Z', 'De novo, ao conciliar a fatura do cartão.'],
        ['2026-10-15T19:40:00Z', null],
      ],
      {
        proposal: 'Nova ferramenta `move_to_statement(txn_id, statement_id)`, como proposta.',
        acceptance_criteria: 'Mover uma parcela para a fatura seguinte pelo MCP, com desfazer.',
        code_pointers: [
          'crates/treasury-core/src/cards/mod.rs',
          'crates/treasury-core/src/mcp/tools.rs',
        ],
      },
    ),
    f(
      'get_dashboard não diz em que moeda vem o saldo das contas no exterior',
      'O campo `balance` das contas em USD vem sem `currency` no resumo do painel; precisei chamar `list_accounts` para saber a moeda.',
      'mcp',
      'dados',
      'em_andamento',
      '2026-10-13T09:12:00Z',
      [['2026-10-15T08:02:00Z', 'Também em get_net_worth quando falta câmbio.']],
    ),
    f(
      'Guia de importação não explica o CSV do Inter',
      'O playbook `bank_statement` cobre OFX e CSV genérico, mas o CSV do Inter tem duas linhas de cabeçalho. Errei a primeira tentativa.',
      'importacao',
      'documentacao',
      'aberto',
      '2026-10-15T16:30:00Z',
      [],
    ),
    f(
      'Erro interno ao listar proventos sem data de pagamento',
      '`list_income_events` falhava com "Erro interno" quando um provento não tinha data de pagamento.',
      'carteira',
      'bug',
      'resolvido',
      '2026-10-10T11:00:00Z',
      [],
      { closed_at: '2026-10-11T09:00:00Z' },
    ),
  ];
}

let items = seed();
onMockReset(() => {
  seq = 0;
  items = seed();
});

const OPEN: DevFeedbackStatus[] = ['aberto', 'em_andamento'];

/** Mesma ordem do núcleo: a manual primeiro, depois a importância e o reforço mais novo. */
function sorted(list: DevFeedback[]): DevFeedback[] {
  const last = (f: DevFeedback) => f.last_bumped_at ?? f.created_at;
  return [...list].sort(
    (a, b) =>
      (a.position ?? Infinity) - (b.position ?? Infinity) ||
      b.importance - a.importance ||
      last(b).localeCompare(last(a)),
  );
}

/** Põe `f` no lugar `position` (1 = topo) da coluna e renumera a coluna. */
function place(f: DevFeedback, position: number) {
  if (!Number.isInteger(position) || position < 1) invalid('a posição começa em 1');
  const col = sorted(items.filter((x) => x.status === f.status && x.id !== f.id));
  col.splice(Math.min(position - 1, col.length), 0, f);
  col.forEach((x, i) => (x.position = i + 1));
}

const STATUS_NOTE: Record<DevFeedbackStatus, string> = {
  aberto: 'Voltou a aberto',
  em_andamento: 'Em andamento',
  resolvido: 'Resolvido',
  descartado: 'Descartado',
};

function setStatus(f: DevFeedback, status: DevFeedbackStatus, reason: string | null) {
  const now = new Date().toISOString();
  if (f.status !== status) {
    f.status = status;
    f.position = null;
    f.closed_at = status === 'resolvido' || status === 'descartado' ? now : null;
  }
  f.updated_at = now;
  f.notes.push({
    id: id('fbn'),
    kind: 'status',
    note: reason ? `${STATUS_NOTE[status]}: ${reason}` : STATUS_NOTE[status],
    client: 'user',
    created_at: now,
  });
}

function find(fid: unknown): DevFeedback {
  const f = items.find((x) => x.id === fid);
  if (!f) notFound(`feedback ${String(fid)}`);
  return f;
}

function text(v: unknown, field: string, max: number): string {
  const t = typeof v === 'string' ? v.trim() : '';
  if (!t) invalid(`informe \`${field}\``);
  if (t.length > max) invalid(`\`${field}\` pode ter no máximo ${max} caracteres`);
  return t;
}

function bump(f: DevFeedback, note: string | null): DevFeedback {
  const now = new Date().toISOString();
  f.importance += 1;
  f.bumps += 1;
  f.last_bumped_at = now;
  f.updated_at = now;
  f.notes.push({ id: id('fbn'), kind: 'reforco', note, client: 'user', created_at: now });
  if (!OPEN.includes(f.status)) setStatus(f, 'aberto', null);
  return structuredClone(f);
}

registerMockHandlers({
  'devfeedback.list': (a) => {
    ready();
    const q = typeof a.query === 'string' ? words(a.query) : new Set<string>();
    return sorted(
      items.filter(
        (f) =>
          (!a.status || f.status === a.status) &&
          (!a.area || f.area === a.area) &&
          (!a.open_only || OPEN.includes(f.status)) &&
          [...q].every((w) => words(`${f.title} ${f.body}`).has(w)),
      ),
    )
      .slice(0, typeof a.limit === 'number' ? a.limit : undefined)
      .map((f) => structuredClone(f));
  },
  'devfeedback.get': (a) => {
    ready();
    return { feedback: structuredClone(find(a.id)), history: [] };
  },
  'devfeedback.add': (a) => {
    ready();
    const title = text(a.title, 'title', 160);
    const body = text(a.body, 'body', 4000);
    const similar = items
      .filter((f) => OPEN.includes(f.status))
      .map((f) => [f, titleSimilarity(title, f.title)] as const)
      .filter(([, s]) => s >= 0.6)
      .sort((x, y) => y[1] - x[1])[0];
    if (similar) {
      const feedback = bump(similar[0], `${title}: ${body}`.slice(0, 1000));
      return {
        feedback,
        deduplicated: true,
        message: `Já havia um feedback aberto parecido ("${feedback.title}"): ele foi reforçado (importância ${feedback.importance}) em vez de duplicado.`,
        redactions: 0,
      };
    }
    const now = new Date().toISOString();
    const f: DevFeedback = {
      ...blank(),
      id: id('fb'),
      title,
      body,
      area: a.area as DevFeedbackArea,
      kind: a.kind as DevFeedbackKind,
      importance: 1,
      status: 'aberto',
      created_by: 'user',
      created_at: now,
      updated_at: now,
      last_bumped_at: null,
      bumps: 0,
      notes: [],
    };
    items.push(f);
    return {
      feedback: structuredClone(f),
      deduplicated: false,
      message: 'Feedback registrado para o desenvolvedor (tela Desenvolvedor).',
      redactions: 0,
    };
  },
  'devfeedback.bump': (a) => {
    ready();
    const note = typeof a.note === 'string' && a.note.trim() ? a.note.trim() : null;
    return bump(find(a.id), note);
  },
  'devfeedback.set_status': (a) => {
    ready();
    const f = find(a.id);
    const reason = typeof a.reason === 'string' && a.reason.trim() ? a.reason.trim() : null;
    if (f.status !== a.status || reason) setStatus(f, a.status as DevFeedbackStatus, reason);
    if (typeof a.position === 'number') place(f, a.position);
    return { ...structuredClone(f), change_set_id: null };
  },
  'devfeedback.set_priority': (a) => {
    ready();
    const f = find(a.id);
    if (a.importance === undefined && a.position === undefined)
      invalid('informe a importância, a posição na coluna ou as duas');
    if (typeof a.importance === 'number') f.importance = a.importance;
    if (typeof a.position === 'number') place(f, a.position);
    f.updated_at = new Date().toISOString();
    return { ...structuredClone(f), change_set_id: null };
  },
  'devfeedback.update': (a) => {
    ready();
    const f = find(a.id);
    if (a.title !== undefined) f.title = text(a.title, 'title', 160);
    if (a.body !== undefined) f.body = text(a.body, 'body', 4000);
    if (a.area !== undefined) f.area = a.area as DevFeedbackArea;
    if (a.kind !== undefined) f.kind = a.kind as DevFeedbackKind;
    if (a.importance !== undefined) {
      const n = Number(a.importance);
      if (!Number.isInteger(n) || n < 1 || n > 1000)
        invalid('importância deve ficar entre 1 e 1000');
      f.importance = n;
    }
    const opt = (v: unknown, cur: string | null) =>
      v === undefined ? cur : typeof v === 'string' && v.trim() ? v.trim() : null;
    f.proposal = opt(a.proposal, f.proposal);
    f.acceptance_criteria = opt(a.acceptance_criteria, f.acceptance_criteria);
    f.assignee = opt(a.assignee, f.assignee);
    if (Array.isArray(a.code_pointers)) f.code_pointers = a.code_pointers as string[];
    if (Array.isArray(a.labels)) f.labels = a.labels as string[];
    f.updated_at = new Date().toISOString();
    return { ...structuredClone(f), change_set_id: null };
  },
  'devfeedback.export': (a) => {
    ready();
    return { path: String(a.path), bytes: typeof a.text === 'string' ? a.text.length : 0 };
  },

  'devfeedback.delete': (a) => {
    ready();
    find(a.id);
    items = items.filter((f) => f.id !== a.id);
    return { deleted: String(a.id), change_set_id: null };
  },
});
