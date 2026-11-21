/**
 * Leitura amigável de um conjunto de alterações (propostas da IA e histórico): nomes no
 * lugar de ids (conta, categoria, membro, ativo…), campos no idioma do app (`agente.ftl`),
 * dinheiro, datas e enums formatados, um título por linha ("PAO DE ACUCAR · 08/10/2026") e
 * frases de resumo ("Altera a categoria de 2 lançamentos: Outros → Alimentação › Mercado"),
 * cada uma uma mensagem só do catálogo, com a ordem das palavras do idioma.
 *
 * Tudo aqui é puro: os nomes chegam num `NameBook` montado por `useChangeNames`. O diff
 * exato (`diff.ts`/`DiffView`) continua disponível em "Ver detalhes técnicos".
 */
import type { ChangeItem } from '@/api/generated/ChangeItem';
import type { ChangeOp } from '@/api/generated/ChangeOp';
import type { RuleInput } from '@/api/generated/RuleInput';
import {
  INDEXER_LABEL as LOAN_INDEXER,
  KIND_LABEL as LOAN_KIND,
  SYSTEM_LABEL,
} from '@/features/debts/labels';
import { goalKindLabel } from '@/features/goals/model';
import { KIND_LABEL as PHYSICAL_KIND } from '@/features/physical/labels';
import { formatQuantity, formatUnitPrice } from '@/features/portfolio/decimal';
import {
  ACTION_KIND_LABEL,
  CLASS_LABEL,
  FIXED_INCOME_KIND_LABEL,
  INCOME_KIND_LABEL,
  INDEXER_LABEL,
  LIQUIDITY_LABEL,
} from '@/features/portfolio/labels';
import { groupName, isSpendGroup } from '@/features/reports/model/groups';
import { conditionsSummary } from '@/features/rules/RuleDialog';
import {
  ACCOUNT_KIND,
  INSTITUTION_KIND,
  RULE_FIELD,
  TXN_KIND,
  TXN_STATUS,
} from '@/features/transactions/ui/labels';
import { t } from '@/i18n';
import {
  baseCurrency,
  formatDate,
  formatDateTime,
  formatMoney,
  formatMonthLong,
} from '@/lib/format';

import { sameValue } from './diff';
import { MEMORY_KIND_LABEL, intlLocale, labels, tableCount, tableLabel } from './format';

// ---------------------------------------------------------------------------
// Nomes das entidades citadas

export type RefKind =
  | 'category'
  | 'account'
  | 'member'
  | 'institution'
  | 'asset'
  | 'loan'
  | 'tag'
  | 'rule'
  | 'goal'
  | 'recurrence'
  | 'txn'
  | 'physical';

/** Lançamento resumido (para dar nome a linhas que só trazem o id). */
export interface TxnBrief {
  description: string;
  date: string;
  amount: number;
  currency: string;
}

export interface NameBook {
  name: (kind: RefKind, id: string) => string | undefined;
  txn: (id: string) => TxnBrief | undefined;
}

export const EMPTY_BOOK: NameBook = { name: () => undefined, txn: () => undefined };

/** Tabela de cada entidade (para registrar as que nascem na própria proposta). */
const TABLE_KIND: Record<string, RefKind> = {
  category: 'category',
  account: 'account',
  member: 'member',
  institution: 'institution',
  asset: 'asset',
  loan: 'loan',
  tag: 'tag',
  rule: 'rule',
  goal: 'goal',
  recurrence: 'recurrence',
  txn: 'txn',
  physical_asset: 'physical',
};

/** Campo que aponta para outra entidade (`category_id` → categoria). */
export function refKind(table: string, field: string): RefKind | null {
  switch (field) {
    case 'category_id':
      return 'category';
    case 'parent_id':
      return table === 'category' ? 'category' : null;
    case 'account_id':
    case 'to_account_id':
    case 'debit_account_id':
    case 'payment_account_id':
      return 'account';
    case 'member_id':
      return 'member';
    case 'institution_id':
    case 'issuer_id':
      return 'institution';
    case 'asset_id':
    case 'new_asset_id':
      return 'asset';
    case 'loan_id':
      return 'loan';
    case 'tag_id':
      return 'tag';
    case 'rule_id':
      return 'rule';
    case 'goal_id':
      return 'goal';
    case 'recurrence_id':
      return 'recurrence';
    case 'txn_id':
    case 'transfer_id':
      return 'txn';
    case 'physical_asset_id':
      return 'physical';
    default:
      return null;
  }
}

/** Nome de quem nasce na própria proposta (ex.: categoria criada e usada logo depois). */
export function namesFromItems(items: readonly ChangeItem[]): Map<string, string> {
  const out = new Map<string, string>();
  for (const it of items) {
    const kind = TABLE_KIND[it.table_name];
    const row = it.after ?? it.before;
    if (!kind || !row) continue;
    const label = str(row.ticker) ?? str(row.name) ?? str(row.description);
    if (label) out.set(`${kind}:${it.row_id}`, label);
  }
  return out;
}

// ---------------------------------------------------------------------------
// Campos

/** Campos de controle: ficam fora da leitura amigável (aparecem nos detalhes técnicos). */
const TECHNICAL = new Set([
  'id',
  'created_at',
  'updated_at',
  'fingerprint',
  'external_id',
  'import_id',
  'change_set_id',
  'generated_until',
  'hits',
  'last_hit_at',
  'created_by',
  'fetched_at',
]);

export function isTechnicalField(field: string): boolean {
  return TECHNICAL.has(field);
}

const FIELD_LABEL: Record<string, string> = labels({
  name: 'agente-campo-nome',
  full_name: 'agente-campo-nome-completo',
  short_name: 'agente-campo-nome-curto',
  kind: 'agente-campo-tipo',
  class: 'agente-campo-classe',
  currency: 'agente-campo-moeda',
  amount: 'agente-campo-valor',
  date: 'agente-campo-data',
  status: 'agente-campo-situacao',
  description: 'agente-campo-descricao',
  payee: 'agente-campo-favorecido',
  notes: 'agente-campo-notas',
  note: 'agente-campo-nota',
  content: 'agente-campo-conteudo',
  title: 'agente-campo-titulo',
  category_id: 'agente-campo-categoria',
  parent_id: 'agente-campo-categoria-pai',
  member_id: 'agente-campo-membro',
  account_id: 'agente-campo-conta',
  to_account_id: 'agente-campo-conta-destino',
  debit_account_id: 'agente-campo-conta-debito',
  payment_account_id: 'agente-campo-conta-pagamento',
  institution_id: 'agente-campo-instituicao',
  issuer_id: 'agente-campo-emissor',
  issuer: 'agente-campo-emissor',
  asset_id: 'agente-campo-ativo',
  new_asset_id: 'agente-campo-ativo-novo',
  loan_id: 'agente-campo-contrato',
  tag_id: 'agente-campo-etiqueta',
  rule_id: 'agente-campo-regra',
  goal_id: 'agente-campo-meta',
  recurrence_id: 'agente-campo-recorrencia',
  txn_id: 'agente-campo-lancamento',
  transfer_id: 'agente-campo-transferencia',
  statement_id: 'agente-campo-fatura',
  installment_plan_id: 'agente-campo-parcelamento',
  installment_number: 'agente-campo-parcela',
  installments: 'agente-campo-parcelas',
  budget_id: 'agente-campo-orcamento',
  physical_asset_id: 'agente-campo-bem',
  archived: 'agente-campo-arquivado',
  active: 'agente-campo-ativa',
  enabled: 'agente-campo-ligada',
  is_essential: 'agente-campo-essencial',
  spend_group: 'agente-campo-grupo-gasto',
  is_primary: 'agente-campo-principal',
  include_in_net_worth: 'agente-campo-patrimonio',
  match_all: 'agente-campo-todas-condicoes',
  ir_exempt: 'agente-campo-isento-ir',
  day_trade: 'agente-campo-day-trade',
  pinned: 'agente-campo-fixado',
  opening_balance: 'agente-campo-saldo-inicial',
  opening_date: 'agente-campo-data-inicial',
  credit_limit: 'agente-campo-limite',
  closing_day: 'agente-campo-dia-fechamento',
  due_day: 'agente-campo-dia-vencimento',
  informed_total: 'agente-campo-total-informado',
  total_amount: 'agente-campo-valor-total',
  purchase_date: 'agente-campo-data-compra',
  first_due_month: 'agente-campo-primeira-parcela',
  priority: 'agente-campo-prioridade',
  conditions: 'agente-campo-condicoes',
  actions: 'agente-campo-acoes',
  rrule: 'agente-campo-repeticao',
  start_date: 'agente-campo-inicio',
  end_date: 'agente-campo-fim',
  rate: 'agente-campo-taxa',
  annual_rate: 'agente-campo-taxa-ano',
  monthly_rate: 'agente-campo-taxa-mes',
  indexer: 'agente-campo-indexador',
  index_pct: 'agente-campo-pct-indexador',
  liquidity: 'agente-campo-liquidez',
  issue_date: 'agente-campo-emissao',
  maturity_date: 'agente-campo-vencimento',
  grace_until: 'agente-campo-carencia',
  ticker: 'agente-campo-ticker',
  market: 'agente-campo-mercado',
  country: 'agente-campo-pais',
  sector: 'agente-campo-setor',
  segment: 'agente-campo-segmento',
  price_provider: 'agente-campo-fonte-cotacao',
  provider_symbol: 'agente-campo-simbolo-fonte',
  side: 'agente-campo-operacao',
  quantity: 'agente-campo-quantidade',
  price: 'agente-campo-preco',
  unit_price: 'agente-campo-preco-unitario',
  unit_cost: 'agente-campo-custo-unitario',
  fees: 'agente-campo-custos',
  fees_amount: 'agente-campo-custos',
  taxes: 'agente-campo-impostos',
  fx_rate: 'agente-campo-cambio',
  note_number: 'agente-campo-nota-corretagem',
  factor: 'agente-campo-fator',
  ex_date: 'agente-campo-data-com',
  pay_date: 'agente-campo-pagamento',
  gross: 'agente-campo-bruto',
  net: 'agente-campo-liquido',
  tax_withheld: 'agente-campo-ir-retido',
  target_price: 'agente-campo-preco-alvo',
  alert_above: 'agente-campo-alerta-acima',
  alert_below: 'agente-campo-alerta-abaixo',
  target_amount: 'agente-campo-valor-meta',
  target_date: 'agente-campo-prazo',
  initial_amount: 'agente-campo-valor-inicial',
  monthly_contribution: 'agente-campo-aporte-mensal',
  principal: 'agente-campo-valor-financiado',
  term_months: 'agente-campo-prazo-meses',
  system: 'agente-campo-sistema',
  contract_date: 'agente-campo-data-contrato',
  contract_number: 'agente-campo-numero-contrato',
  first_due_date: 'agente-campo-primeiro-vencimento',
  color: 'agente-campo-cor',
  icon: 'agente-campo-icone',
  sort_order: 'agente-campo-ordem',
  month: 'agente-campo-mes',
  value: 'agente-campo-valor',
  cost: 'agente-campo-custo',
  source: 'agente-campo-origem',
  domain: 'agente-campo-dominio',
  label: 'agente-campo-rotulo',
  topic: 'agente-campo-assunto',
  steps: 'agente-campo-passos',
  branch: 'agente-campo-agencia',
  account_number: 'agente-campo-conta',
});

/** Siglas que ficam iguais em todos os idiomas. */
const FIELD_CODE: Record<string, string> = { isin: 'ISIN', cnpj: 'CNPJ' };

export function fieldLabel(field: string): string {
  if (FIELD_CODE[field]) return FIELD_CODE[field];
  if (FIELD_LABEL[field]) return FIELD_LABEL[field];
  const words = field.replace(/_id$/, '').replace(/_/g, ' ');
  return words.charAt(0).toUpperCase() + words.slice(1);
}

const BOOL = new Set([
  'archived',
  'active',
  'enabled',
  'is_essential',
  'is_primary',
  'include_in_net_worth',
  'match_all',
  'ir_exempt',
  'day_trade',
  'pinned',
  'estimated',
  'builtin',
  'persist_session',
  'agent_navigation',
]);

const MONEY = new Set([
  'amount',
  'gross',
  'net',
  'fees',
  'taxes',
  'tax_withheld',
  'cost',
  'principal',
  'opening_balance',
  'credit_limit',
  'total_amount',
  'informed_total',
  'limit_amount',
  'target_amount',
  'initial_amount',
  'monthly_contribution',
  'paid_amount',
  'purchase_price',
  'sale_price',
  'sale_costs',
  'insurance_monthly',
  'fees_monthly',
  'property_value',
  'balance_after',
  'interest',
  'amortization',
  'extra_payment',
  'total',
  'value',
]);

const UNIT_PRICE = new Set([
  'price',
  'unit_price',
  'unit_cost',
  'target_price',
  'alert_above',
  'alert_below',
  'amount_per_share',
]);

/** Campos que o modo privacidade esconde (dinheiro, preços, quantidades). */
export function isSensitiveField(field: string): boolean {
  return (
    MONEY.has(field) ||
    field.endsWith('_amount') ||
    field.endsWith('_cents') ||
    UNIT_PRICE.has(field) ||
    field === 'quantity'
  );
}

const SIDE: Record<string, string> = labels({
  buy: 'agente-lado-compra',
  sell: 'agente-lado-venda',
});

/** Rótulo de um valor de enum, pela tabela e pelo campo. */
function enumLabel(table: string, field: string, value: string): string | undefined {
  const pick = (map: Record<string, string>) => map[value];
  if (field === 'side') return pick(SIDE);
  if (field === 'spend_group') return isSpendGroup(value) ? groupName(value) : undefined;
  if (field === 'class') return pick(CLASS_LABEL);
  if (field === 'liquidity') return pick(LIQUIDITY_LABEL);
  if (field === 'system') return pick(SYSTEM_LABEL);
  if (field === 'indexer') return pick(table === 'loan' ? LOAN_INDEXER : INDEXER_LABEL);
  if (field === 'status' && (table === 'txn' || table === 'recurrence')) return pick(TXN_STATUS);
  if (field !== 'kind') return undefined;
  switch (table) {
    case 'txn':
    case 'recurrence':
      return pick(TXN_KIND);
    case 'account':
      return pick(ACCOUNT_KIND);
    case 'institution':
      return pick(INSTITUTION_KIND);
    case 'income_event':
      return pick(INCOME_KIND_LABEL);
    case 'corporate_action':
      return pick(ACTION_KIND_LABEL);
    case 'ai_memory':
      return pick(MEMORY_KIND_LABEL);
    case 'loan':
      return pick(LOAN_KIND);
    case 'fixed_income_terms':
      return pick(FIXED_INCOME_KIND_LABEL);
    case 'physical_asset':
      return pick(PHYSICAL_KIND);
    case 'goal':
      return goalKindLabel(value as never);
    default:
      return undefined;
  }
}

const NONE: Record<string, string> = labels({
  category_id: 'agente-sem-categoria',
  member_id: 'agente-sem-membro',
  parent_id: 'agente-sem-pai',
});

/** Texto entre aspas, no estilo do idioma. */
function quote(text: string): string {
  return t('agente-aspas', { texto: text });
}

function yesNo(v: boolean): string {
  return v ? t('agente-sim') : t('agente-nao');
}

function str(v: unknown): string | undefined {
  return typeof v === 'string' && v.trim() !== '' ? v : undefined;
}

/** Colunas JSON chegam como texto do SQLite (núcleo) ou já como objeto (mock). */
function parseJson(v: unknown): unknown {
  if (typeof v !== 'string') return v;
  const t = v.trim();
  if (!t.startsWith('[') && !t.startsWith('{')) return v;
  try {
    return JSON.parse(t) as unknown;
  } catch {
    return v;
  }
}

function safe<T>(fn: () => T, fallback: T): T {
  try {
    return fn();
  } catch {
    return fallback;
  }
}

function refName(kind: RefKind, id: string, book: NameBook): string {
  if (kind === 'txn') {
    const txn = book.txn(id);
    return txn ? quote(txn.description || t('agente-sem-descricao')) : t('agente-outro-lancamento');
  }
  return book.name(kind, id) ?? t('agente-nao-encontrado', { ref: REF_FALLBACK[kind] });
}

const REF_FALLBACK: Record<RefKind, string> = labels({
  category: 'agente-ref-categoria',
  account: 'agente-ref-conta',
  member: 'agente-ref-membro',
  institution: 'agente-ref-instituicao',
  asset: 'agente-ref-ativo',
  loan: 'agente-ref-contrato',
  tag: 'agente-ref-etiqueta',
  rule: 'agente-ref-regra',
  goal: 'agente-ref-meta',
  recurrence: 'agente-ref-recorrencia',
  txn: 'agente-ref-lancamento',
  physical: 'agente-ref-bem',
});

/** Ações de uma regra (`{category_id, tags, member_id}`) em texto. */
function ruleActionsText(v: unknown, book: NameBook): string {
  if (!v || typeof v !== 'object') return '—';
  const a = v as Record<string, unknown>;
  const parts: string[] = [];
  if (str(a.category_id))
    parts.push(
      t('agente-regra-categoria', { nome: refName('category', a.category_id as string, book) }),
    );
  if (str(a.member_id))
    parts.push(t('agente-regra-membro', { nome: refName('member', a.member_id as string, book) }));
  if (Array.isArray(a.tags) && a.tags.length)
    parts.push(
      t('agente-regra-etiquetas', {
        etiquetas: a.tags.map((tag) => `#${String(tag)}`).join(' '),
      }),
    );
  if (str(a.payee)) parts.push(t('agente-regra-favorecido', { nome: a.payee as string }));
  if (str(a.kind))
    parts.push(
      t('agente-regra-tipo', {
        tipo: TXN_KIND[a.kind as keyof typeof TXN_KIND] ?? String(a.kind),
      }),
    );
  return parts.length ? parts.join(' · ') : t('agente-regra-nenhuma');
}

function ruleConditionsText(v: unknown, matchAll: unknown): string {
  if (!Array.isArray(v)) return '—';
  return safe(
    () =>
      conditionsSummary({
        conditions: v as RuleInput['conditions'],
        match_all: matchAll === false || matchAll === 0 ? false : true,
      }),
    v.map((c) => RULE_FIELD[(c as { field: keyof typeof RULE_FIELD }).field] ?? '?').join(', '),
  );
}

/**
 * Valor de um campo em texto amigável. `row` é a linha (antes ou depois) de onde vêm a
 * moeda e os campos vizinhos; `null`/ausente vira "—" (ou "Sem categoria", etc.).
 */
export function formatFieldValue(
  table: string,
  field: string,
  value: unknown,
  row: Record<string, unknown> | null,
  book: NameBook,
): string {
  if (value === undefined || value === null || value === '') return NONE[field] ?? '—';
  const currency = str(row?.currency) ?? baseCurrency();
  const ref = refKind(table, field);
  if (ref && typeof value === 'string') return refName(ref, value, book);
  if (field === 'conditions') return ruleConditionsText(parseJson(value), row?.match_all);
  if (field === 'actions' && table === 'rule') return ruleActionsText(parseJson(value), book);
  if (BOOL.has(field) || field.startsWith('is_'))
    return value === true || value === 1
      ? yesNo(true)
      : value === false || value === 0
        ? yesNo(false)
        : JSON.stringify(value);
  if (
    typeof value === 'number' &&
    Number.isInteger(value) &&
    (MONEY.has(field) || field.endsWith('_amount') || field.endsWith('_cents'))
  )
    return formatMoney(value, currency);
  if (typeof value === 'string') {
    const e = enumLabel(table, field, value);
    if (e) return e;
    if (UNIT_PRICE.has(field)) return safe(() => formatUnitPrice(value, currency), value);
    if (field === 'quantity') return safe(() => formatQuantity(value), value);
    if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return safe(() => formatDate(value), value);
    if (/^\d{4}-\d{2}$/.test(value)) return safe(() => formatMonthLong(value), value);
    if (/^\d{4}-\d{2}-\d{2}T/.test(value)) return safe(() => formatDateTime(value), value);
    if (/^-?\d+\.\d+$/.test(value))
      return Number(value).toLocaleString(intlLocale(), { maximumFractionDigits: 8 });
    return value;
  }
  if (typeof value === 'number')
    return value.toLocaleString(intlLocale(), { maximumFractionDigits: 8 });
  if (typeof value === 'boolean') return yesNo(value);
  const parsed = parseJson(value);
  if (Array.isArray(parsed))
    return parsed.map((v) => (typeof v === 'string' ? v : JSON.stringify(v))).join(', ');
  return JSON.stringify(parsed);
}

// ---------------------------------------------------------------------------
// Linhas

export const OP_VERB: Record<ChangeOp, string> = labels({
  insert: 'agente-op-criacao',
  update: 'agente-op-alteracao',
  delete: 'agente-op-exclusao',
});

/** Título de uma linha: o nome dela e, à parte, o que o modo privacidade esconde. */
export interface RowTitle {
  name: string;
  detail?: string;
  /** Valor (dinheiro, quantidade × preço): escondido no modo privacidade. */
  money?: string;
}

function merged(item: ChangeItem): Record<string, unknown> {
  return { ...(item.before ?? {}), ...(item.after ?? {}) };
}

function tickerOf(id: unknown, book: NameBook): string {
  const fallback = REF_FALLBACK.asset;
  return typeof id === 'string' ? (book.name('asset', id) ?? fallback) : fallback;
}

function truncate(s: string, n = 80): string {
  return s.length > n ? `${s.slice(0, n - 1)}…` : s;
}

export function rowTitle(item: ChangeItem, book: NameBook): RowTitle {
  const r = merged(item);
  const table = item.table_name;
  const cur = str(r.currency) ?? baseCurrency();
  const date = (v: unknown) =>
    str(v) ? safe(() => formatDate(v as string), v as string) : undefined;
  const money = (v: unknown) =>
    typeof v === 'number' && Number.isInteger(v) ? formatMoney(v, cur) : undefined;
  switch (table) {
    case 'txn': {
      const known = book.txn(item.row_id);
      const desc = str(r.description) ?? known?.description;
      return {
        name: desc || t('agente-lancamento-sem-descricao'),
        detail: date(r.date ?? known?.date),
        money: money(r.amount) ?? (known ? formatMoney(known.amount, known.currency) : undefined),
      };
    }
    case 'txn_split':
    case 'txn_tag': {
      const parent = str(r.txn_id) ? book.txn(r.txn_id as string) : undefined;
      const of = parent ? quote(parent.description || t('agente-sem-descricao')) : REF_FALLBACK.txn;
      if (table === 'txn_tag') {
        const tag = str(r.tag_id) ? refName('tag', r.tag_id as string, book) : REF_FALLBACK.tag;
        return {
          name: t('agente-titulo-etiqueta-em', { etiqueta: tag.replace(/^#/, ''), lancamento: of }),
        };
      }
      return { name: t('agente-titulo-parte-de', { lancamento: of }), money: money(r.amount) };
    }
    case 'trade': {
      const side = SIDE[str(r.side) ?? ''] ?? t('agente-operacao');
      const qty = str(r.quantity) ? safe(() => formatQuantity(r.quantity as string), '') : '';
      const price = str(r.price) ? safe(() => formatUnitPrice(r.price as string, cur), '') : '';
      return {
        name: t('agente-titulo-de', { tipo: side, ativo: tickerOf(r.asset_id, book) }),
        detail: date(r.date),
        money: qty && price ? `${qty} × ${price}` : qty || undefined,
      };
    }
    case 'income_event':
      return {
        name: t('agente-titulo-de', {
          tipo: INCOME_KIND_LABEL[r.kind as keyof typeof INCOME_KIND_LABEL] ?? t('agente-provento'),
          ativo: tickerOf(r.asset_id, book),
        }),
        detail: date(r.pay_date ?? r.ex_date),
        money: money(r.net ?? r.gross),
      };
    case 'corporate_action':
      return {
        name: t('agente-titulo-de', {
          tipo: ACTION_KIND_LABEL[r.kind as keyof typeof ACTION_KIND_LABEL] ?? t('agente-evento'),
          ativo: tickerOf(r.asset_id, book),
        }),
        detail: date(r.date),
      };
    case 'watchlist_item':
    case 'fixed_income_terms':
      return { name: tickerOf(r.asset_id ?? item.row_id, book) };
    case 'asset': {
      const ticker = str(r.ticker) ?? book.name('asset', item.row_id);
      const name = str(r.name);
      return {
        name:
          ticker && name && ticker !== name
            ? `${ticker} · ${name}`
            : (ticker ?? name ?? tableLabel('asset')),
      };
    }
    case 'budget_line':
      return {
        name: str(r.category_id)
          ? refName('category', r.category_id as string, book)
          : tableLabel('category'),
        money: money(r.amount),
      };
    case 'recurrence':
      return {
        name:
          str(r.description) ?? book.name('recurrence', item.row_id) ?? tableLabel('recurrence'),
        money: money(r.amount),
      };
    case 'dev_feedback':
      return { name: truncate(str(r.title) ?? tableLabel(table)) };
    case 'dev_feedback_note':
      return { name: truncate(str(r.note) ?? tableLabel(table)) };
    case 'ai_memory':
    case 'ai_procedure':
      return { name: truncate(str(r.content) ?? str(r.title) ?? t('agente-nota-gestor')) };
    case 'account_member':
    case 'goal_account':
    case 'physical_asset_member': {
      const parts = [
        str(r.member_id) ? refName('member', r.member_id as string, book) : null,
        str(r.account_id) ? refName('account', r.account_id as string, book) : null,
        str(r.goal_id)
          ? t('agente-titulo-meta', { nome: refName('goal', r.goal_id as string, book) })
          : null,
      ].filter(Boolean);
      return { name: parts.join(' · ') || tableLabel(table) };
    }
    default: {
      const kind = TABLE_KIND[table];
      const name =
        (kind === 'category' ? book.name('category', item.row_id) : undefined) ??
        str(r.name) ??
        (kind ? book.name(kind, item.row_id) : undefined) ??
        str(r.description) ??
        str(r.title) ??
        str(r.label) ??
        str(r.domain);
      return { name: name ?? tableLabel(table) };
    }
  }
}

export interface FriendlyField {
  field: string;
  label: string;
  before: string;
  after: string;
  change: 'added' | 'removed' | 'changed';
  sensitive: boolean;
}

export interface FriendlyRow {
  key: string;
  table: string;
  tableLabel: string;
  op: ChangeOp;
  opLabel: string;
  title: RowTitle;
  fields: FriendlyField[];
}

/** Campos que contam: na alteração, só os que mudam; na inclusão/exclusão, os preenchidos. */
export function friendlyRow(item: ChangeItem, book: NameBook): FriendlyRow {
  const before = item.before ?? {};
  const after = item.after ?? {};
  const keys = [...Object.keys(before)];
  for (const k of Object.keys(after)) if (!keys.includes(k)) keys.push(k);
  const fields: FriendlyField[] = [];
  for (const field of keys) {
    if (isTechnicalField(field)) continue;
    const b = before[field];
    const a = after[field];
    if (item.op === 'update') {
      if (!(field in after) || sameValue(b, a)) continue;
    } else {
      const v = item.op === 'insert' ? a : b;
      if (v === null || v === undefined || v === '') continue;
      // A moeda aparece junto do valor; o nome já está no título.
      if (field === 'currency' && keys.some((k) => MONEY.has(k))) continue;
    }
    fields.push({
      field,
      label: fieldLabel(field),
      before:
        item.op === 'insert' ? '' : formatFieldValue(item.table_name, field, b, item.before, book),
      after:
        item.op === 'delete'
          ? ''
          : formatFieldValue(item.table_name, field, a, item.after ?? item.before, book),
      change: item.op === 'insert' ? 'added' : item.op === 'delete' ? 'removed' : 'changed',
      sensitive: isSensitiveField(field),
    });
  }
  return {
    key: `${item.seq}:${item.table_name}:${item.row_id}`,
    table: item.table_name,
    tableLabel: tableLabel(item.table_name),
    op: item.op,
    opLabel: OP_VERB[item.op],
    title: rowTitle(item, book),
    fields,
  };
}

export function friendlyRows(items: readonly ChangeItem[], book: NameBook): FriendlyRow[] {
  return [...items].sort((a, b) => a.seq - b.seq).map((it) => friendlyRow(it, book));
}

// ---------------------------------------------------------------------------
// Frases de resumo

/** Frase da inclusão por tabela (registrar uma operação, não "criar"). */
const INSERT_SENTENCE = {
  trade: 'agente-frase-registra',
  income_event: 'agente-frase-registra',
  corporate_action: 'agente-frase-registra',
  physical_valuation: 'agente-frase-registra',
  physical_improvement: 'agente-frase-registra',
  ai_memory: 'agente-frase-grava',
  watchlist_item: 'agente-frase-acompanha',
  account_member: 'agente-frase-inclui',
  txn_tag: 'agente-frase-poe',
} as const;

type Sentence = (typeof INSERT_SENTENCE)[keyof typeof INSERT_SENTENCE] | 'agente-frase-cria';

/** Tabelas de ligação que só acompanham a principal (não viram frase própria). */
const QUIET = new Set(['rule_hit', 'goal_account']);

function quoteList(names: string[], max = 3): string {
  const q = names.map(quote);
  if (q.length <= max) return joinWords(q);
  return t('agente-lista-e-mais', { inicio: q.slice(0, max).join(', '), n: q.length - max });
}

function joinWords(words: string[]): string {
  return words.length > 1
    ? t('agente-lista-e', { inicio: words.slice(0, -1).join(', '), fim: words.at(-1)! })
    : (words[0] ?? '');
}

/**
 * O que a proposta faz, em frases curtas: uma por tabela e operação.
 * Ex.: "Altera a categoria de 2 lançamentos: Outros → Alimentação › Mercado".
 */
export function describeChanges(items: readonly ChangeItem[], book: NameBook): string[] {
  const rows = friendlyRows(items, book);
  const groups = new Map<string, FriendlyRow[]>();
  for (const r of rows) {
    if (QUIET.has(r.table)) continue;
    const k = `${r.table}:${r.op}`;
    groups.set(k, [...(groups.get(k) ?? []), r]);
  }
  const out: string[] = [];
  for (const group of groups.values()) {
    const first = group[0]!;
    const n = group.length;
    const itens = tableCount(first.table, n);
    const names = group.map((r) => r.title.name);
    if (first.op !== 'update') {
      const key: Sentence | 'agente-frase-apaga' =
        first.op === 'delete'
          ? 'agente-frase-apaga'
          : ((INSERT_SENTENCE as Record<string, Sentence>)[first.table] ?? 'agente-frase-cria');
      out.push(t(key, { itens, nomes: quoteList(names) }));
      continue;
    }
    const changed = group.filter((r) => r.fields.length > 0);
    if (changed.length === 0) continue;
    const fields = [...new Set(changed.flatMap((r) => r.fields.map((f) => f.field)))];
    const alvo = n === 1 ? quote(first.title.name) : itens;
    const what = joinWords(fields.map((f) => ARTICLE[f] ?? fieldLabel(f).toLowerCase()));
    if (fields.length === 1 && !isSensitiveField(fields[0]!)) {
      const only = changed.map((r) => r.fields[0]!);
      const befores = new Set(only.map((f) => f.before));
      const afters = new Set(only.map((f) => f.after));
      const to = afters.size === 1 ? only[0]!.after : null;
      const from = befores.size === 1 ? only[0]!.before : null;
      const campo = ARTICLE[fields[0]!] ?? t('agente-artigo-campo');
      if (to && from)
        out.push(t('agente-frase-altera-de-para', { campo, alvo, de: from, para: to }));
      else if (to) out.push(t('agente-frase-altera-para', { campo, alvo, para: to }));
      else out.push(t('agente-frase-altera', { campo, alvo }));
      continue;
    }
    out.push(t('agente-frase-altera', { campo: what, alvo }));
  }
  return out;
}

/** Artigo + nome do campo na frase ("a categoria", "o membro"). */
const ARTICLE: Record<string, string> = labels({
  category_id: 'agente-artigo-categoria',
  amount: 'agente-artigo-valor',
  member_id: 'agente-artigo-membro',
  account_id: 'agente-artigo-conta',
  status: 'agente-artigo-situacao',
  kind: 'agente-artigo-tipo',
  name: 'agente-artigo-nome',
  description: 'agente-artigo-descricao',
  payee: 'agente-artigo-favorecido',
  notes: 'agente-artigo-notas',
  date: 'agente-artigo-data',
  active: 'agente-artigo-ativacao',
  archived: 'agente-artigo-arquivamento',
  priority: 'agente-artigo-prioridade',
  parent_id: 'agente-artigo-categoria-pai',
  institution_id: 'agente-artigo-instituicao',
  is_essential: 'agente-artigo-essencial',
  spend_group: 'agente-artigo-grupo-gasto',
  conditions: 'agente-artigo-condicoes',
  actions: 'agente-artigo-acoes',
  color: 'agente-artigo-cor',
});

/** Ids citados por campos de referência (para buscar só o que falta). */
export function referencedIds(items: readonly ChangeItem[]): { kind: RefKind; id: string }[] {
  const out = new Map<string, { kind: RefKind; id: string }>();
  for (const it of items) {
    const kind = TABLE_KIND[it.table_name];
    if (kind === 'txn') out.set(`txn:${it.row_id}`, { kind: 'txn', id: it.row_id });
    for (const row of [it.before, it.after]) {
      if (!row) continue;
      for (const [field, v] of Object.entries(row)) {
        const ref = refKind(it.table_name, field);
        if (ref && typeof v === 'string' && v) out.set(`${ref}:${v}`, { kind: ref, id: v });
      }
    }
  }
  return [...out.values()];
}
