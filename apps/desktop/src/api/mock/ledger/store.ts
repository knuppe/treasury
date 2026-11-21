/**
 * Banco em memória do livro-caixa do mock, com as mesmas tabelas do núcleo
 * (`migrations/V2__fluxo_de_caixa.sql`) e o mesmo histórico de alterações:
 * toda escrita passa por `write(motivo, fn)`, que compara as tabelas antes e
 * depois e grava um conjunto de alterações com o antes/depois de cada linha.
 * Desfazer/refazer conferem se a linha ainda está como o conjunto deixou
 * (senão: `conflict`, como o núcleo).
 *
 * Regra de ouro: linhas são imutáveis. Para mudar, use `put`/`patch`/`del`
 * (que trocam o objeto); assim a comparação é por referência e barata.
 */
import type { Author } from '@/api/generated/Author';
import type { AuthorKind } from '@/api/generated/AuthorKind';
import type { ChangeItem } from '@/api/generated/ChangeItem';
import type { ChangeSetDetail } from '@/api/generated/ChangeSetDetail';
import type { ChangeSetSummary } from '@/api/generated/ChangeSetSummary';
import type { ChangeStatus } from '@/api/generated/ChangeStatus';
import type { AccountKind } from '@/api/generated/AccountKind';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import type { InstitutionKind } from '@/api/generated/InstitutionKind';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import type { ImportFormat } from '@/api/generated/ImportFormat';
import type { ImportRowError } from '@/api/generated/ImportRowError';
import type { RuleActions } from '@/api/generated/RuleActions';
import type { RuleCondition } from '@/api/generated/RuleCondition';
import type { ScreenQuery } from '@/api/generated/ScreenQuery';
import type { SpendGroup } from '@/api/generated/SpendGroup';

import { conflict, invalid, notFound } from './errors';
import { nowUtc } from './dates';

// ── Linhas (colunas do banco) ───────────────────────────────────

import type { MemberProfile } from '@/api/generated/MemberProfile';

/** Cadastro vazio do membro. */
export const EMPTY_PROFILE: MemberProfile = {
  birth_date: null,
  relationship: null,
  irpf_dependent: false,
  occupation: null,
  notes: null,
  sex: null,
};

export interface HouseholdRow {
  id: string;
  name: string;
  base_currency: string;
  /** País da família (V123); sem ele, Brasil. */
  country?: string;
  created_at: string;
  updated_at: string;
}
export interface MemberRow {
  id: string;
  name: string;
  color: string | null;
  is_primary: boolean;
  archived: boolean;
  sort_order: number;
  /** Cadastro do membro (V120); ausente = vazio. */
  profile?: MemberProfile;
  created_at: string;
  updated_at: string;
}
export interface InstitutionRow {
  id: string;
  name: string;
  kind: InstitutionKind;
  code: string | null;
  website: string | null;
  created_at: string;
  updated_at: string;
}
export interface AccountRow {
  id: string;
  name: string;
  kind: AccountKind;
  currency: string;
  institution_id: string | null;
  opening_balance: number;
  opening_date: string;
  include_in_net_worth: boolean;
  archived: boolean;
  color: string | null;
  notes: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
export interface AccountMemberRow {
  id: string;
  account_id: string;
  member_id: string;
}
export interface CreditCardRow {
  /** = account_id */
  id: string;
  credit_limit: number | null;
  closing_day: number;
  due_day: number;
  payment_account_id: string | null;
  network: string | null;
  last4: string | null;
  created_at: string;
  updated_at: string;
}
export interface StatementRow {
  id: string;
  account_id: string;
  reference_month: string;
  period_start: string;
  closing_date: string;
  due_date: string;
  informed_total: number | null;
  /** V156: de onde vêm as datas e o que a fatura impressa mostra; ausentes = calculadas/nulos. */
  dates_source?: 'calc' | 'manual' | 'import';
  informed_minimum?: number | null;
  informed_limit?: number | null;
  informed_previous_balance?: number | null;
  created_at: string;
  updated_at: string;
}
export interface CategoryRow {
  id: string;
  parent_id: string | null;
  name: string;
  kind: CategoryKind;
  icon: string | null;
  color: string | null;
  is_essential: boolean;
  /** Grupo de gasto (só despesas); `is_essential` acompanha `essential`. */
  spend_group: SpendGroup | null;
  archived: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
export interface TxnRow {
  id: string;
  account_id: string;
  date: string;
  amount: number;
  currency: string;
  kind: TxnKind;
  status: TxnStatus;
  description: string;
  payee: string | null;
  notes: string | null;
  category_id: string | null;
  member_id: string | null;
  transfer_id: string | null;
  statement_id: string | null;
  installment_plan_id: string | null;
  installment_number: number | null;
  recurrence_id: string | null;
  import_id: string | null;
  external_id: string | null;
  fingerprint: string | null;
  /** Projeto (V105); ausente = sem projeto. */
  project_id?: string | null;
  /** De onde veio o membro (V122): `auto` (da conta) ou `manual`; ausente = sem origem. */
  member_origin?: 'auto' | 'manual' | null;
  /** Plástico do cartão da compra (V155); ausente = sem plástico. */
  card_plastic_id?: string | null;
  /** Compra no exterior (V157): valor com o sinal de `amount` e moeda de origem. */
  orig_amount?: number | null;
  orig_currency?: string | null;
  created_at: string;
  updated_at: string;
}
/** Plástico do cartão (`card_plastic`, V155): titular, adicional, virtual ou temporário. */
export interface CardPlasticRow {
  id: string;
  account_id: string;
  kind: 'holder' | 'additional' | 'virtual' | 'temporary';
  last4: string | null;
  member_id: string | null;
  label: string | null;
  spend_limit: number | null;
  active: boolean;
  created_at: string;
  updated_at: string;
}
/** Histórico do limite do cartão (`card_limit_change`, V156). */
export interface LimitChangeRow {
  id: string;
  account_id: string;
  date: string;
  credit_limit: number;
  source: 'manual' | 'import';
  created_at: string;
  updated_at: string;
}
/** Anuidade do cartão (`card_annual_fee`, V156). */
export interface AnnualFeeRow {
  id: string;
  account_id: string;
  amount: number;
  installments: number;
  anniversary_month: number | null;
  waiver_kind: 'none' | 'full' | 'spend' | 'investment' | 'discount';
  waiver_threshold: number | null;
  waiver_discount: string | null;
  starts_on: string;
  ends_on: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}
/** Programa de cashback ou pontos do cartão (`card_reward_program`, V158). */
export interface RewardProgramRow {
  id: string;
  account_id: string;
  kind: 'cashback' | 'points';
  base_rate: string;
  rate_currency: string | null;
  cycle_cap: number | null;
  destination: 'statement' | 'account' | 'investment';
  destination_account_id: string | null;
  payout_days: number;
  starts_on: string;
  ends_on: string | null;
  notes: string | null;
  /** `card_reward_multiplier` (V158), junto do programa no mock. */
  multipliers: { category_id: string; rate: string }[];
  /** Programa de pontos onde os pontos caem (V159). */
  loyalty_program_id?: string | null;
  created_at: string;
  updated_at: string;
}
/** Programa de pontos e milhas (`loyalty_program`, V159). */
export interface LoyaltyProgramRow {
  id: string;
  name: string;
  kind: 'bank' | 'airline' | 'retail' | 'other';
  member_id: string | null;
  unit_label: string | null;
  value_per_1000: number | null;
  value_currency: string | null;
  active: boolean;
  notes: string | null;
  created_at: string;
  updated_at: string;
}
/** Movimento de pontos (`loyalty_entry`, V159). */
export interface LoyaltyEntryRow {
  id: string;
  program_id: string;
  date: string;
  points: number;
  kind:
    'earn' | 'bonus' | 'purchase' | 'transfer_in' | 'transfer_out' | 'redeem' | 'expire' | 'adjust';
  expires_on: string | null;
  statement_id: string | null;
  transfer_id: string | null;
  cost: number | null;
  cost_currency: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}
/** Série de parcelas marcada como "não é parcela" (`detected_installment_ignore`, V163). */
export interface InstallmentIgnoreRow {
  /** A chave da série. */
  id: string;
  account_id: string | null;
  description: string;
  created_at: string;
  updated_at: string;
}
/** Projeto (`project`, V105): agrupador de gastos, opcionalmente de um bem. */
export interface ProjectRow {
  id: string;
  name: string;
  physical_asset_id: string | null;
  status: 'planned' | 'active' | 'done';
  budget: number | null;
  start_date: string | null;
  end_date: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}
/** Item lançado à mão no projeto (`project_item`, V106): + receita, − despesa. */
export interface ProjectItemRow {
  id: string;
  project_id: string;
  date: string;
  amount: number;
  description: string;
  category_id: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}
export interface SplitRow {
  id: string;
  txn_id: string;
  category_id: string | null;
  member_id: string | null;
  amount: number;
  notes: string | null;
  /** Ordem de inserção (o núcleo ordena por rowid). */
  seq: number;
  created_at: string;
  updated_at: string;
}
export interface TagRow {
  id: string;
  name: string;
  color: string | null;
  created_at: string;
  updated_at: string;
}
export interface TxnTagRow {
  id: string;
  txn_id: string;
  tag_id: string;
}
export interface AttachmentRow {
  id: string;
  txn_id: string | null;
  filename: string;
  mime: string;
  size_bytes: number;
  sha256: string;
  data: string;
  created_at: string;
  updated_at: string;
}
export interface PlanRow {
  id: string;
  account_id: string;
  description: string;
  total_amount: number;
  currency: string;
  installments: number;
  purchase_date: string;
  category_id: string | null;
  member_id: string | null;
  created_at: string;
  updated_at: string;
}
export interface RecurrenceRow {
  id: string;
  description: string;
  account_id: string;
  amount: number;
  currency: string;
  kind: TxnKind;
  category_id: string | null;
  member_id: string | null;
  to_account_id: string | null;
  rrule: string;
  start_date: string;
  end_date: string | null;
  generated_until: string | null;
  active: boolean;
  created_at: string;
  updated_at: string;
}
export interface RuleRow {
  id: string;
  name: string;
  priority: number;
  active: boolean;
  match_all: boolean;
  conditions: RuleCondition[];
  actions: RuleActions;
  created_at: string;
  updated_at: string;
}
export interface BatchRow {
  id: string;
  filename: string;
  format: ImportFormat;
  file_sha256: string;
  account_id: string;
  total_rows: number;
  inserted: number;
  duplicates: number;
  errors: ImportRowError[];
  created_at: string;
  updated_at: string;
}
export interface FxRow {
  id: string;
  base: string;
  quote: string;
  date: string;
  rate: string;
  source: string;
  fetched_at: string;
  created_at: string;
  updated_at: string;
}

/** Tela salva do screener (`screen`): o filtro fica como objeto. */
export interface ScreenRow {
  id: string;
  name: string;
  description: string | null;
  query: ScreenQuery;
  created_at: string;
  updated_at: string;
  /** Alerta de tela (V145). */
  alert?: boolean;
}

export interface Tables {
  household: Map<string, HouseholdRow>;
  member: Map<string, MemberRow>;
  institution: Map<string, InstitutionRow>;
  account: Map<string, AccountRow>;
  account_member: Map<string, AccountMemberRow>;
  credit_card: Map<string, CreditCardRow>;
  card_statement: Map<string, StatementRow>;
  card_plastic: Map<string, CardPlasticRow>;
  card_limit_change: Map<string, LimitChangeRow>;
  card_annual_fee: Map<string, AnnualFeeRow>;
  card_reward_program: Map<string, RewardProgramRow>;
  loyalty_program: Map<string, LoyaltyProgramRow>;
  loyalty_entry: Map<string, LoyaltyEntryRow>;
  category: Map<string, CategoryRow>;
  tag: Map<string, TagRow>;
  txn: Map<string, TxnRow>;
  txn_split: Map<string, SplitRow>;
  txn_tag: Map<string, TxnTagRow>;
  attachment: Map<string, AttachmentRow>;
  installment_plan: Map<string, PlanRow>;
  recurrence: Map<string, RecurrenceRow>;
  rule: Map<string, RuleRow>;
  import_batch: Map<string, BatchRow>;
  fx_rate: Map<string, FxRow>;
  screen: Map<string, ScreenRow>;
  project: Map<string, ProjectRow>;
  detected_installment_ignore: Map<string, InstallmentIgnoreRow>;
  project_item: Map<string, ProjectItemRow>;
}

export type TableName = keyof Tables;
type RowOf<T extends TableName> = Tables[T] extends Map<string, infer R> ? R : never;

const TABLE_NAMES: TableName[] = [
  'household',
  'member',
  'institution',
  'account',
  'account_member',
  'credit_card',
  'card_statement',
  'card_plastic',
  'card_limit_change',
  'card_annual_fee',
  'card_reward_program',
  'loyalty_program',
  'loyalty_entry',
  'category',
  'tag',
  'txn',
  'txn_split',
  'txn_tag',
  'attachment',
  'installment_plan',
  'recurrence',
  'rule',
  'import_batch',
  'fx_rate',
  'screen',
  'project',
  'detected_installment_ignore',
  'project_item',
];

function emptyTables(): Tables {
  return Object.fromEntries(TABLE_NAMES.map((t) => [t, new Map()])) as unknown as Tables;
}

// ── Estado técnico (fora do histórico) ───────────────────────────

interface StoredChangeSet {
  id: string;
  created_at: string;
  author: Author;
  reason: string;
  status: ChangeStatus;
  applied_at: string | null;
  undone_at: string | null;
  items: StoredItem[];
  /** Alteração de um mock com estado próprio (ex.: carteira), fora das tabelas daqui. */
  external?: ExternalChange;
}

/**
 * Conjunto de alterações de um mock que guarda o estado fora deste banco (a carteira
 * e os ativos): o histórico mostra o resumo, e desfazer/refazer chamam o próprio mock.
 */
export interface ExternalChange {
  /** `{ insert: { trade: 3 }, update: { trade: 1 } }`, como `ChangeSetSummary.summary`. */
  summary: Record<string, Record<string, number>>;
  undo: () => void;
  redo: () => void;
}

interface StoredItem {
  table: TableName;
  rowId: string;
  before: object | null;
  after: object | null;
}

export interface RuleHit {
  hits: number;
  last_hit_at: string | null;
}

export interface Db {
  t: Tables;
  changeSets: StoredChangeSet[];
  ruleHits: Map<string, RuleHit>;
  /** Contador de ids (ordem lexicográfica = ordem de criação). */
  seq: number;
  /** Muda a cada escrita, desfazer ou refazer (cache das visões derivadas). */
  version: number;
  /** Dados da semente (ex.: início do livro). */
  meta: Record<string, string>;
}

function emptyDb(): Db {
  return { t: emptyTables(), changeSets: [], ruleHits: new Map(), seq: 0, version: 0, meta: {} };
}

export let db: Db = emptyDb();

type Listener = (tables: ReadonlySet<TableName> | 'all') => void;
const listeners: Listener[] = [];

/** Avisado depois de cada mudança no banco, com as tabelas tocadas (espelhos e caches). */
export function onStoreChange(fn: Listener): void {
  listeners.push(fn);
}

function changed(tables: ReadonlySet<TableName> | 'all'): void {
  db.version += 1;
  for (const fn of listeners) fn(tables);
}

/** Tabelas copiadas na primeira escrita de cada uma dentro do `write` em curso. */
let tx: Map<TableName, Map<string, unknown>> | null = null;

function touch(table: TableName): void {
  if (tx && !tx.has(table)) tx.set(table, new Map(db.t[table] as Map<string, unknown>));
}

/** Troca o banco inteiro (usado pela semente e pela restauração de backup). */
export function replaceDb(next: Db): void {
  next.version = db.version;
  db = next;
  changed('all');
}

export function freshDb(): Db {
  return emptyDb();
}

export function newId(): string {
  db.seq += 1;
  const n = String(db.seq).padStart(10, '0');
  return `01mock00-${n.slice(0, 4)}-7${n.slice(4, 7)}-8${n.slice(7)}0-${n}00`;
}

// ── Acesso ──────────────────────────────────────────────────────

export function rows<T extends TableName>(table: T): RowOf<T>[] {
  return [...(db.t[table] as Map<string, RowOf<T>>).values()];
}

export function find<T extends TableName>(
  table: T,
  id: string | null | undefined,
): RowOf<T> | undefined {
  if (id == null) return undefined;
  return (db.t[table] as Map<string, RowOf<T>>).get(id);
}

/** Linha obrigatória; senão `not_found` com a descrição dada. */
export function must<T extends TableName>(table: T, id: string, what: string): RowOf<T> {
  return find(table, id) ?? notFound(what);
}

export function put<T extends TableName>(table: T, row: RowOf<T>): RowOf<T> {
  touch(table);
  (db.t[table] as Map<string, RowOf<T>>).set((row as { id: string }).id, row);
  return row;
}

/** Troca campos de uma linha (cria um objeto novo). Devolve a linha nova. */
export function patch<T extends TableName>(
  table: T,
  id: string,
  changes: Partial<RowOf<T>>,
): RowOf<T> {
  const cur = find(table, id);
  if (!cur) notFound(`${table} ${id}`);
  const next = { ...cur, ...changes } as RowOf<T>;
  if (shallowEqual(cur, next as object)) return cur;
  return put(table, next);
}

export function del(table: TableName, id: string): boolean {
  if (!db.t[table].has(id)) return false;
  touch(table);
  return db.t[table].delete(id);
}

function shallowEqual(a: object, b: object): boolean {
  const ka = Object.keys(a);
  const kb = Object.keys(b);
  if (ka.length !== kb.length) return false;
  for (const k of ka) {
    const va = (a as Record<string, unknown>)[k];
    const vb = (b as Record<string, unknown>)[k];
    if (va !== vb && JSON.stringify(va) !== JSON.stringify(vb)) return false;
  }
  return true;
}

// ── Escrita com histórico ────────────────────────────────────────

export const USER: Author = { kind: 'user', detail: null };

export function author(kind: AuthorKind, detail: string | null = null): Author {
  return { kind, detail };
}

export interface Recorded<T> {
  value: T;
  changeSetId: string | null;
}

function diff(before: ReadonlyMap<TableName, Map<string, unknown>>): StoredItem[] {
  const items: StoredItem[] = [];
  for (const table of TABLE_NAMES) {
    const a = before.get(table) as Map<string, object> | undefined;
    if (!a) continue;
    const b = db.t[table] as Map<string, object>;
    for (const [id, row] of b) {
      const old = a.get(id);
      if (old === undefined) items.push({ table, rowId: id, before: null, after: row });
      else if (old !== row && !shallowEqual(old, row)) {
        items.push({ table, rowId: id, before: old, after: row });
      }
    }
    for (const [id, row] of a) {
      if (!b.has(id)) items.push({ table, rowId: id, before: row, after: null });
    }
  }
  return items;
}

/**
 * Executa `fn` como uma transação: se lançar, nada muda; se mudar alguma linha,
 * grava um conjunto de alterações com o autor e o motivo. Dentro de outro `write`,
 * entra no conjunto de fora.
 */
export function write<T>(reason: string, fn: () => T, who: Author = USER): Recorded<T> {
  if (!reason.trim()) invalid('toda alteração precisa de um motivo');
  if (tx) return { value: fn(), changeSetId: null };
  const before = new Map<TableName, Map<string, unknown>>();
  tx = before;
  const seqBefore = db.seq;
  let value: T;
  try {
    value = fn();
  } catch (e) {
    for (const [table, rowsBefore] of before)
      (db.t as unknown as Record<string, unknown>)[table] = rowsBefore;
    db.seq = seqBefore;
    throw e;
  } finally {
    tx = null;
  }
  const items = diff(before);
  if (items.length === 0) return { value, changeSetId: null };
  changed(new Set(items.map((it) => it.table)));
  const at = nowUtc();
  const set: StoredChangeSet = {
    id: newId(),
    created_at: at,
    author: who,
    reason,
    status: 'applied',
    applied_at: at,
    undone_at: null,
    items,
  };
  db.changeSets.push(set);
  return { value, changeSetId: set.id };
}

/** Grava no histórico uma alteração feita por um mock com estado próprio. */
export function recordExternal(reason: string, who: Author, external: ExternalChange): string {
  const at = nowUtc();
  const set: StoredChangeSet = {
    id: newId(),
    created_at: at,
    author: who,
    reason,
    status: 'applied',
    applied_at: at,
    undone_at: null,
    items: [],
    external,
  };
  db.changeSets.push(set);
  db.version += 1;
  return set.id;
}

// ── Leitura do histórico ─────────────────────────────────────────

/** Forma da linha como o núcleo devolve (booleanos 0/1, sem campos internos). */
function dbShape(table: TableName, row: object | null): Record<string, unknown> | null {
  if (!row) return null;
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(row)) {
    if (k === 'seq' && table === 'txn_split') continue;
    if (k === 'data' && table === 'attachment') continue;
    if (k === 'id' && (table === 'account_member' || table === 'txn_tag')) continue;
    if (table === 'credit_card' && k === 'id') {
      out.account_id = v;
      continue;
    }
    if (table === 'rule' && (k === 'conditions' || k === 'actions' || k === 'match_all')) {
      if (k === 'match_all') continue;
      out[k] = JSON.stringify(
        k === 'conditions' ? { match_all: (row as RuleRow).match_all, items: v as unknown } : v,
      );
      continue;
    }
    if (table === 'import_batch' && k === 'errors') {
      out[k] = JSON.stringify(v);
      continue;
    }
    out[k] = typeof v === 'boolean' ? (v ? 1 : 0) : v;
  }
  return out;
}

function summaryOf(set: StoredChangeSet): Record<string, Record<string, number>> {
  if (set.external) return set.external.summary;
  const out: Record<string, Record<string, number>> = {};
  for (const it of set.items) {
    const op = it.before == null ? 'insert' : it.after == null ? 'delete' : 'update';
    const byTable = (out[op] ??= {});
    byTable[it.table] = (byTable[it.table] ?? 0) + 1;
  }
  return out;
}

export function changeSummary(set: StoredChangeSet): ChangeSetSummary {
  return {
    id: set.id,
    created_at: set.created_at,
    author: set.author,
    reason: set.reason,
    status: set.status,
    applied_at: set.applied_at,
    undone_at: set.undone_at,
    summary: summaryOf(set),
    item_count: set.external
      ? Object.values(set.external.summary).reduce(
          (n, byTable) => n + Object.values(byTable).reduce((a, b) => a + b, 0),
          0,
        )
      : set.items.length,
  };
}

export function changeDetail(set: StoredChangeSet): ChangeSetDetail {
  const items: ChangeItem[] = set.items.map((it, seq) => ({
    seq,
    table_name: it.table,
    row_id:
      it.table === 'account_member' || it.table === 'txn_tag'
        ? it.rowId.replace('|', ':')
        : it.rowId,
    op: it.before == null ? 'insert' : it.after == null ? 'delete' : 'update',
    before: dbShape(it.table, it.before),
    after: dbShape(it.table, it.after),
  }));
  return { ...changeSummary(set), items };
}

export function changeSet(id: string): StoredChangeSet {
  return db.changeSets.find((c) => c.id === id) ?? notFound(`conjunto de alterações ${id}`);
}

const CONFLICT =
  'alterações posteriores mudaram estes dados; desfaça primeiro as alterações mais recentes que tocam nas mesmas linhas';

function sameRow(a: object | undefined | null, b: object | null): boolean {
  if (a == null || b == null) return a == null && b == null;
  return a === b || shallowEqual(a, b);
}

/** Aplica os itens na direção pedida, conferindo o estado atual de cada linha. */
function applyItems(set: StoredChangeSet, direction: 'undo' | 'redo'): void {
  const ordered = direction === 'undo' ? [...set.items].reverse() : set.items;
  for (const it of ordered) {
    const expected = direction === 'undo' ? it.after : it.before;
    const current = db.t[it.table].get(it.rowId) as object | undefined;
    if (!sameRow(current, expected)) conflict(CONFLICT);
  }
  for (const it of ordered) {
    const target = direction === 'undo' ? it.before : it.after;
    if (target == null) db.t[it.table].delete(it.rowId);
    else (db.t[it.table] as Map<string, object>).set(it.rowId, target);
  }
}

export function undo(id: string): ChangeSetSummary {
  const set = changeSet(id);
  if (set.status !== 'applied') conflict('só é possível desfazer alterações aplicadas');
  if (set.external) set.external.undo();
  else applyItems(set, 'undo');
  changed(new Set(set.items.map((it) => it.table)));
  set.status = 'undone';
  set.undone_at = nowUtc();
  return changeSummary(set);
}

export function redo(id: string): ChangeSetSummary {
  const set = changeSet(id);
  if (set.status !== 'undone') conflict('só é possível refazer alterações desfeitas');
  if (set.external) set.external.redo();
  else applyItems(set, 'redo');
  changed(new Set(set.items.map((it) => it.table)));
  set.status = 'applied';
  set.applied_at = nowUtc();
  set.undone_at = null;
  return changeSummary(set);
}

/** Linhas que um conjunto inseriu numa tabela (ex.: lançamentos de uma importação). */
export function insertedBy(id: string, table: TableName): string[] {
  return changeSet(id)
    .items.filter((it) => it.table === table && it.before == null)
    .map((it) => it.rowId);
}

export function findChangeSetInserting(table: TableName, rowId: string): string | null {
  for (let i = db.changeSets.length - 1; i >= 0; i--) {
    const set = db.changeSets[i]!;
    if (set.items.some((it) => it.table === table && it.rowId === rowId && it.before == null)) {
      return set.id;
    }
  }
  return null;
}
