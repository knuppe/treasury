/**
 * Mock de `rules.*`: regras de categorização com o mesmo motor do núcleo
 * (prioridade, E/OU, texto sem acentos, regex, faixas de valor), simulação,
 * aplicação num conjunto de alterações e sugestão a partir de um lançamento.
 */
import type { LearnedRule } from '@/api/generated/LearnedRule';
import type { Rule } from '@/api/generated/Rule';
import type { RuleActions } from '@/api/generated/RuleActions';
import type { RuleApplyResult } from '@/api/generated/RuleApplyResult';
import type { RuleCondition } from '@/api/generated/RuleCondition';
import type { RuleInput } from '@/api/generated/RuleInput';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnView } from '@/api/generated/TxnView';

import { extraMockHandler, onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { nowUtc } from './ledger/dates';
import {
  categoryPaths,
  filterTxns,
  isTransferLike,
  replaceTags,
  sortTxns,
  txnViews,
} from './ledger/domain';
import { invalid, notFound, str } from './ledger/errors';
import {
  type RuleRow,
  type TxnRow,
  db,
  del,
  find,
  newId,
  patch,
  put,
  rows,
  write,
} from './ledger/store';
import { fold, foldAccent, normalizeDescription } from './ledger/text';

export interface Subject {
  description: string;
  payee: string | null;
  notes: string | null;
  amount: number;
  account_id: string;
  kind: TxnKind;
}

type Matcher = (s: Subject) => boolean;

function textValue(c: RuleCondition): string {
  if (typeof c.value === 'string' && c.value.trim()) return c.value;
  if (typeof c.value === 'number') return String(c.value);
  return invalid('a condição precisa de um texto');
}

function amountValue(v: unknown): number {
  if (typeof v === 'number') {
    if (!Number.isInteger(v))
      invalid('valor da condição deve ser inteiro, em centavos (ex.: -5000 = −R$ 50,00)');
    return v;
  }
  if (typeof v === 'string') {
    const n = Number(v.replace(/\./g, '').replace(',', '.'));
    if (Number.isNaN(n)) invalid(`valor inválido: ${v}`);
    return Math.round(n * 100);
  }
  return invalid('a condição de valor precisa de um número');
}

function fieldText(s: Subject, field: RuleCondition['field']): string {
  return field === 'description'
    ? s.description
    : field === 'payee'
      ? (s.payee ?? '')
      : field === 'notes'
        ? (s.notes ?? '')
        : '';
}

function compile(c: RuleCondition): Matcher {
  switch (c.field) {
    case 'description':
    case 'payee':
    case 'notes': {
      if (c.op === 'regex') {
        let re: RegExp;
        try {
          re = new RegExp(foldAccent(textValue(c)), 'i');
        } catch (e) {
          return invalid(`expressão regular inválida: ${(e as Error).message}`);
        }
        return (s) => re.test(foldAccent(fieldText(s, c.field)));
      }
      if (!['contains', 'equals', 'starts_with', 'ends_with'].includes(c.op)) {
        invalid('condições de texto usam contém, igual, começa com, termina com ou regex');
      }
      const needle = fold(textValue(c));
      if (!needle) invalid('o texto da condição precisa ter letras ou números');
      return (s) => {
        const hay = fold(fieldText(s, c.field));
        if (c.op === 'contains') return hay.includes(needle);
        if (c.op === 'equals') return hay === needle;
        if (c.op === 'starts_with') return hay.startsWith(needle);
        return hay.endsWith(needle);
      };
    }
    case 'amount': {
      const a = amountValue(c.value);
      if (c.op === 'equals') return (s) => s.amount === a;
      if (c.op === 'gte') return (s) => s.amount >= a;
      if (c.op === 'lte') return (s) => s.amount <= a;
      if (c.op === 'between') {
        if (c.value2 == null) invalid('a condição "entre" precisa de value2');
        const lo = Math.min(a, c.value2);
        const hi = Math.max(a, c.value2);
        return (s) => s.amount >= lo && s.amount <= hi;
      }
      return invalid('condições de valor usam igual, entre, maior ou igual, menor ou igual');
    }
    case 'account_id':
      if (c.op !== 'equals' || typeof c.value !== 'string' || !c.value) {
        invalid('condição de conta usa "igual" com o id da conta');
      }
      return (s) => s.account_id === c.value;
    case 'kind':
      if (c.op !== 'equals' || typeof c.value !== 'string') {
        invalid('condição de tipo usa "igual" com o tipo do lançamento');
      }
      return (s) => s.kind === c.value;
  }
}

export interface CompiledRule {
  id: string;
  priority: number;
  matches: (s: Subject) => boolean;
  actions: RuleActions;
}

export function compileRule(
  id: string,
  priority: number,
  input: Pick<RuleInput, 'conditions' | 'match_all' | 'actions'>,
): CompiledRule {
  if (!input.conditions?.length) invalid('a regra precisa de pelo menos uma condição');
  const matchers = input.conditions.map(compile);
  const all = input.match_all !== false;
  return {
    id,
    priority,
    actions: input.actions,
    matches: (s) => (all ? matchers.every((m) => m(s)) : matchers.some((m) => m(s))),
  };
}

export interface Outcome {
  matched: string[];
  applied: string[];
  category_id?: string;
  member_id?: string;
  tags?: string[];
  rename_to?: string;
  kind?: TxnKind;
}

export function loadRules(ids?: string[] | null): CompiledRule[] {
  const list = ids
    ? ids.map((id) => find('rule', id) ?? notFound(`regra ${id}`))
    : rows('rule').filter((r) => r.active);
  return list.map((r) => compileRule(r.id, r.priority, r)).sort((a, b) => a.priority - b.priority);
}

export function evaluate(rules: CompiledRule[], s: Subject): Outcome {
  const out: Outcome = { matched: [], applied: [] };
  for (const r of rules) {
    if (!r.matches(s)) continue;
    out.matched.push(r.id);
    const a = r.actions;
    let contributed = false;
    if (out.category_id === undefined && a.category_id) { out.category_id = a.category_id; contributed = true; } // prettier-ignore
    if (out.member_id === undefined && a.member_id) { out.member_id = a.member_id; contributed = true; } // prettier-ignore
    if (out.tags === undefined && a.tags?.length) { out.tags = a.tags; contributed = true; } // prettier-ignore
    if (out.rename_to === undefined && a.rename_to?.trim()) { out.rename_to = a.rename_to.trim(); contributed = true; } // prettier-ignore
    if (out.kind === undefined && a.kind) { out.kind = a.kind; contributed = true; } // prettier-ignore
    if (contributed) out.applied.push(r.id);
  }
  return out;
}

export function recordHits(hits: Map<string, number>): void {
  const now = nowUtc();
  for (const [id, n] of hits) {
    if (n <= 0) continue;
    const cur = db.ruleHits.get(id) ?? { hits: 0, last_hit_at: null };
    db.ruleHits.set(id, { hits: cur.hits + n, last_hit_at: now });
  }
}

function toRule(r: RuleRow): Rule {
  const h = db.ruleHits.get(r.id);
  return {
    id: r.id,
    name: r.name,
    priority: r.priority,
    active: r.active,
    match_all: r.match_all,
    conditions: r.conditions,
    actions: r.actions,
    hits: h?.hits ?? 0,
    last_hit_at: h?.last_hit_at ?? null,
    created_at: r.created_at,
    updated_at: r.updated_at,
  };
}

function cleanActions(a: RuleActions): RuleActions {
  const out: RuleActions = {};
  if (a.category_id) out.category_id = a.category_id;
  if (a.member_id) out.member_id = a.member_id;
  if (a.kind) out.kind = a.kind;
  if (a.rename_to != null) out.rename_to = a.rename_to.trim();
  if (a.tags) {
    const tags: string[] = [];
    for (const t of a.tags.map((x) => x.trim()))
      if (!tags.some((o) => o.toLowerCase() === t.toLowerCase())) tags.push(t);
    out.tags = tags;
  }
  return out;
}

function validate(input: RuleInput): void {
  if (!input.name?.trim()) invalid('dê um nome à regra');
  compileRule('', 0, input);
  const a = input.actions ?? {};
  if (!a.category_id && !a.member_id && !a.tags?.length && !a.rename_to && !a.kind) {
    invalid('a regra precisa de pelo menos uma ação (categoria, membro, tags, renomear ou tipo)');
  }
  if (a.category_id && !find('category', a.category_id)) notFound(`categoria ${a.category_id}`);
  if (a.member_id && !find('member', a.member_id)) notFound(`membro ${a.member_id}`);
  if (a.rename_to != null && !a.rename_to.trim()) invalid('o novo nome não pode ficar vazio');
  if (a.tags?.some((t) => !t.trim())) invalid('tags não podem ficar vazias');
  for (const c of input.conditions) {
    if (c.field === 'account_id' && typeof c.value === 'string' && !find('account', c.value)) {
      notFound(`conta ${c.value}`);
    }
  }
}

function subject(t: TxnRow): Subject {
  return {
    description: t.description,
    payee: t.payee,
    notes: t.notes,
    amount: t.amount,
    account_id: t.account_id,
    kind: t.kind,
  };
}

function splitIds(): Set<string> {
  return new Set([...rows('txn_split')].map((s) => s.txn_id));
}

interface Planned {
  id: string;
  category_id?: string | null;
  member_id?: string;
  description?: string;
  kind?: TxnKind;
  add_tags: string[];
  rules: string[];
}

function plan(args: Record<string, unknown>): { matched: number; changes: Planned[] } {
  const set = loadRules(Array.isArray(args.rule_ids) ? args.rule_ids.map(String) : null);
  const onlyUncategorized = args.only_uncategorized !== false;
  const facts = sortTxns(args.filter ? filterTxns(args.filter) : rows('txn'), 'date_desc');
  let matched = 0;
  const changes: Planned[] = [];
  const withSplits = splitIds();
  for (const f of facts) {
    const split = withSplits.has(f.id);
    if (onlyUncategorized && (f.category_id || split)) continue;
    const out = evaluate(set, subject(f));
    if (out.matched.length === 0) continue;
    matched++;
    const p: Planned = { id: f.id, add_tags: [], rules: out.applied };
    const finalKind = out.kind ?? f.kind;
    if (out.kind && out.kind !== f.kind) p.kind = out.kind;
    const mayOverride = !f.category_id || !onlyUncategorized;
    if (isTransferLike(finalKind)) {
      if (f.category_id && p.kind) p.category_id = null;
    } else if (out.category_id && mayOverride && !split && f.category_id !== out.category_id) {
      p.category_id = out.category_id;
    }
    if (out.member_id && (!f.member_id || !onlyUncategorized) && f.member_id !== out.member_id) {
      p.member_id = out.member_id;
    }
    if (out.rename_to && out.rename_to !== f.description) p.description = out.rename_to;
    if (out.tags) {
      const have = rows('txn_tag')
        .filter((l) => l.txn_id === f.id)
        .map((l) => find('tag', l.tag_id)?.name.toLowerCase());
      p.add_tags = out.tags.filter((t) => !have.includes(t.toLowerCase()));
    }
    if (p.category_id !== undefined || p.member_id || p.description || p.kind || p.add_tags.length)
      changes.push(p);
  }
  return { matched, changes };
}

function simulate(changes: Planned[]): TxnView[] {
  const sample = changes.slice(0, 50);
  const views = txnViews(sample.map((p) => find('txn', p.id)!));
  const paths = categoryPaths();
  return views.map((v, i) => {
    const p = sample[i]!;
    const out = { ...v, tags: [...v.tags] };
    if (p.category_id !== undefined) {
      out.category_id = p.category_id;
      out.category_path = p.category_id ? (paths.get(p.category_id) ?? null) : null;
    }
    if (p.member_id) {
      out.member_id = p.member_id;
      out.member_name = find('member', p.member_id)?.name ?? null;
    }
    if (p.description) out.description = p.description;
    if (p.kind) out.kind = p.kind;
    for (const t of p.add_tags) out.tags.push({ id: '', name: t, color: null });
    return out;
  });
}

function apply(args: Record<string, unknown>): RuleApplyResult {
  const { matched, changes } = plan(args);
  if (args.dry_run === true || changes.length === 0) {
    return { matched, updated: changes.length, sample: simulate(changes), change_set_id: null };
  }
  const rec = write('Aplicação de regras de categorização', () => {
    const hits = new Map<string, number>();
    for (const p of changes) {
      const t = find('txn', p.id)!;
      const changesRow: Partial<TxnRow> = {};
      if (p.category_id !== undefined) changesRow.category_id = p.category_id;
      if (p.member_id) changesRow.member_id = p.member_id;
      if (p.description) changesRow.description = p.description;
      if (p.kind) changesRow.kind = p.kind;
      patch('txn', t.id, { ...changesRow, updated_at: nowUtc() });
      if (p.add_tags.length) {
        const names = rows('txn_tag')
          .filter((l) => l.txn_id === t.id)
          .map((l) => find('tag', l.tag_id)!.name);
        replaceTags(t.id, [...names, ...p.add_tags]);
      }
      for (const r of p.rules) hits.set(r, (hits.get(r) ?? 0) + 1);
    }
    recordHits(hits);
  });
  return {
    matched,
    updated: changes.length,
    sample: txnViews(changes.slice(0, 50).map((p) => find('txn', p.id)!)),
    change_set_id: rec.changeSetId,
  };
}

const NOISE = new Set(
  'a o e as os de da do das dos em no na nos nas para pra pelo pela com por ao compra compras pag pagto pgto pagamento pagamentos debito credito cartao card pix ted doc tef transf transferencia enviado enviada recebido recebida boleto int internet online app ltda me sa eireli epp mei cia br bra brasil www http https net org parcela parc aut automatico deb cred visa master mastercard elo nacional'.split(
    ' ',
  ),
);

const significant = (w: string) => w.length >= 3 && !/^\d+$/.test(w) && !NOISE.has(w);

export function significantRun(description: string): string[] {
  const words = normalizeDescription(description).split(' ').filter(Boolean);
  const start = words.findIndex(significant);
  if (start < 0) return words.filter((w) => w.length >= 2).slice(0, 2);
  const out: string[] = [];
  for (const w of words.slice(start)) {
    if (!significant(w) || out.length >= 3) break;
    out.push(w);
  }
  return out;
}

function suggestFromTxn(txnId: string): RuleInput {
  const f = find('txn', txnId) ?? notFound(`lançamento ${txnId}`);
  const run = significantRun(f.description);
  if (run.length === 0) invalid('a descrição do lançamento não tem palavras para montar uma regra');
  const others = rows('txn')
    .filter((t) => t.category_id && t.id !== txnId)
    .map((t) => [fold(t.description), t.category_id] as const);
  let chosen = run.join(' ');
  for (let n = 1; n <= run.length; n++) {
    const candidate = run.slice(0, n).join(' ');
    if (candidate.length < 4 && n < run.length) continue;
    if (!others.some(([d, c]) => d.includes(candidate) && c !== f.category_id)) {
      chosen = candidate;
      break;
    }
  }
  return {
    name: `Descrição contém “${chosen.toUpperCase()}”`,
    priority: 100,
    active: true,
    match_all: true,
    conditions: [{ field: 'description', op: 'contains', value: chosen }],
    actions: isTransferLike(f.kind) || !f.category_id ? {} : { category_id: f.category_id },
  };
}

let learnedIgnored = new Set<string>();
onMockReset(() => {
  learnedIgnored = new Set();
});

function invoke<T>(cmd: string, args: Record<string, unknown> = {}): T {
  return extraMockHandler(cmd)!(args) as T;
}

registerMockHandlers(
  guarded({
    'rules.list': () =>
      rows('rule')
        .sort(
          (a, b) =>
            a.priority - b.priority ||
            a.created_at.localeCompare(b.created_at) ||
            a.id.localeCompare(b.id),
        )
        .map(toRule),
    'rules.get': (args) =>
      toRule(find('rule', str(args, 'id')) ?? notFound(`regra ${String(args.id)}`)),
    'rules.create': (args) => {
      const input = args as unknown as RuleInput;
      const id = write(`Regra criada: ${String(input.name ?? '').trim()}`, () => {
        validate(input);
        const now = nowUtc();
        return put('rule', {
          id: newId(),
          name: input.name.trim(),
          priority: input.priority ?? 100,
          active: input.active ?? true,
          match_all: input.match_all ?? true,
          conditions: input.conditions,
          actions: cleanActions(input.actions),
          created_at: now,
          updated_at: now,
        }).id;
      }).value;
      return toRule(find('rule', id)!);
    },
    'rules.update': (args) => {
      const id = str(args, 'id');
      const input = args as unknown as RuleInput;
      write(`Regra alterada: ${String(input.name ?? '').trim()}`, () => {
        const cur = find('rule', id) ?? notFound(`regra ${id}`);
        validate(input);
        patch('rule', id, {
          name: input.name.trim(),
          priority: input.priority ?? cur.priority,
          active: input.active ?? cur.active,
          match_all: input.match_all ?? true,
          conditions: input.conditions,
          actions: cleanActions(input.actions),
          updated_at: nowUtc(),
        });
      });
      return toRule(find('rule', id)!);
    },
    'rules.delete': (args) => {
      const r = find('rule', str(args, 'id')) ?? notFound(`regra ${String(args.id)}`);
      write(`Regra excluída: ${r.name}`, () => del('rule', r.id));
      return null;
    },
    'rules.test': (args) => {
      const input = args.rule as RuleInput;
      const rule = compileRule('teste', 0, input);
      const limit = Math.min(Math.max(Number(args.limit ?? 50), 1), 500);
      const hits: TxnRow[] = [];
      for (const t of sortTxns(rows('txn'), 'date_desc')) {
        if (rule.matches(subject(t))) {
          hits.push(t);
          if (hits.length >= limit) break;
        }
      }
      return txnViews(hits);
    },
    'rules.apply': apply,
    'rules.suggest_from_txn': (args) => suggestFromTxn(str(args, 'txn_id')),
    // Exemplo fixo (a regra de verdade mora no núcleo, `rules/learned.rs`): o lançamento
    // categorizado mais recente vira uma sugestão, até existir regra para ele ou ser dispensado.
    'rules.learned': (): LearnedRule[] => {
      const sample = rows('txn')
        .filter((t) => (t.kind === 'expense' || t.kind === 'income') && t.category_id)
        .sort((a, b) => b.date.localeCompare(a.date) || a.id.localeCompare(b.id))[0];
      if (!sample?.category_id) return [];
      const rule = suggestFromTxn(sample.id);
      const key = String(rule.conditions[0]?.value ?? '');
      const taken = rows('rule').some((r) => r.name === rule.name);
      if (!key || taken || learnedIgnored.has(key)) return [];
      return [
        {
          key,
          category_id: sample.category_id,
          category_path:
            invoke<{ id: string; path: string }[]>('categories.list', {
              include_archived: true,
            }).find((c) => c.id === sample.category_id)?.path ?? '',
          count: 3,
          uncategorized: 0,
          sample_txn_id: sample.id,
          sample_description: sample.description,
          rule,
        },
      ];
    },
    'rules.learned_ignore': (args) => {
      learnedIgnored.add(str(args, 'key'));
      return null;
    },
  }),
);
