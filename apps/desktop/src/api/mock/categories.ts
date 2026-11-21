/**
 * Mock de `categories.*`: árvore editável, uso, mesclar e apagar com reatribuição, sugestões do
 * conjunto inicial e a tradução das categorias padrão quando o idioma muda.
 */
import type { Category } from '@/api/generated/Category';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import type { CategoryNode } from '@/api/generated/CategoryNode';
import type { CategoryUsage } from '@/api/generated/CategoryUsage';
import type { Locale } from '@/api/generated/Locale';
import type { MissingDefault } from '@/api/generated/MissingDefault';
import type { SpendGroup } from '@/api/generated/SpendGroup';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { nowUtc } from './ledger/dates';
import { categoryPaths, categoryTree, toCategory, withDescendants } from './ledger/domain';
import {
  cleanName,
  cleanOpt,
  conflict,
  has,
  invalid,
  notFound,
  optBool,
  str,
} from './ledger/errors';
import { type CategoryRow, del, find, newId, patch, put, rows, write } from './ledger/store';
import { fold, foldAccent } from './ledger/text';
import { mockLocale } from './locale';

/** Como no núcleo: essenciais e dívidas contam como gasto obrigatório (reserva de emergência). */
export function countsAsEssential(group: string | null | undefined): boolean {
  return group === 'essential' || group === 'debt';
}

/** Categorias do conjunto inicial do núcleo (famílias com filhos, pagamentos) que o seed do mock não tem. */
const FAMILY: [CategoryKind, string, string[]][] = [
  ['expense', 'Moradia', ['Diarista e serviços domésticos']],
  [
    'expense',
    'Saúde',
    [
      'Pediatra',
      'Terapias e psicólogo',
      'Vacinas',
      'Óculos e lentes',
      'Itens de saúde e primeiros socorros',
    ],
  ],
  [
    'expense',
    'Educação',
    [
      'Creche',
      'Material escolar',
      'Uniforme',
      'Transporte escolar',
      'Cantina e lanche escolar',
      'Faculdade',
      'Idiomas',
    ],
  ],
  ['expense', 'Seguros', ['Seguro viagem', 'Seguro educacional']],
  [
    'expense',
    'Pagamentos',
    ['Contas e boletos', 'Empréstimos', 'Financiamentos', 'Parcelas', 'Fatura do cartão'],
  ],
  [
    'expense',
    'Filhos',
    [
      'Esportes e aulas',
      'Fraldas e higiene',
      'Roupas e calçados infantis',
      'Brinquedos',
      'Babá e cuidadores',
      'Festas e aniversários',
      'Pensão alimentícia',
    ],
  ],
  [
    'expense',
    'Poupança e investimentos',
    [
      'Poupança dos filhos',
      'Reserva de emergência',
      'Previdência privada',
      'Aportes em investimentos',
    ],
  ],
  ['income', 'Salário', ['Horas extras', 'Comissões', 'Rescisão']],
  [
    'income',
    'Serviços e freelance',
    ['Projetos e consultoria', 'Bicos e diárias', 'Aulas particulares', 'Gorjetas e caixinhas'],
  ],
  ['income', 'Trabalho por aplicativo', ['Corridas', 'Entregas', 'Serviços por aplicativo']],
  [
    'income',
    'Vendas e negócio próprio',
    [
      'Faturamento do MEI',
      'Revenda por catálogo',
      'Comida e produtos caseiros',
      'Artesanato',
      'Vendas online',
    ],
  ],
  ['income', 'Rendimentos', ['Rendimentos de FII', 'Aluguel por temporada']],
  ['income', 'Benefícios', ['Vale-alimentação e refeição', 'Auxílio-creche']],
  [
    'income',
    'Aposentadoria e pensão',
    ['INSS', 'Servidor público', 'Previdência privada', 'Pensão por morte'],
  ],
  [
    'income',
    'Auxílios do governo',
    [
      'Seguro-desemprego',
      'Auxílio por incapacidade (INSS)',
      'Bolsa Família',
      'BPC/LOAS',
      'Abono salarial (PIS/Pasep)',
      'Outros programas sociais',
    ],
  ],
  ['income', 'Pensão alimentícia recebida', []],
  [
    'income',
    'Reembolsos',
    [
      'Divisão de contas',
      'Despesas do trabalho',
      'Plano de saúde',
      'Estornos e devoluções',
      'Cashback e recompensas',
    ],
  ],
  ['income', 'Prêmios e cashback', ['Cashback', 'Sorteios e loterias', 'Programas de nota fiscal']],
  ['income', 'Indenizações', ['Seguros', 'Ações judiciais']],
  ['income', 'Presentes recebidos', ['Ajuda de familiares', 'Mesada']],
  ['income', 'Herança', []],
  ['income', 'Resgate de investimentos', ['Poupança', 'Previdência privada', 'Saque do FGTS']],
  ['income', 'Venda de bens', ['Veículos', 'Imóveis', 'Usados e desapegos', 'Milhas e pontos']],
  ['income', 'Recebimento de dívidas', ['Empréstimo devolvido', 'Fiado e vendas a prazo']],
  ['income', 'Empréstimos recebidos', []],
];

// ── Nomes do conjunto inicial por idioma (como `categories::seed_name`) ────────

/** Catálogo `categorias.ftl` de cada idioma (só os nomes-base: o mock é de uma família do Brasil). */
const CATALOGS = import.meta.glob<string>('../../../../../locales/*/categorias.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
});

let catalogCache: Map<Locale, Map<string, string>> | null = null;

function catalogs(): Map<Locale, Map<string, string>> {
  if (catalogCache) return catalogCache;
  const out = new Map<Locale, Map<string, string>>();
  for (const [path, src] of Object.entries(CATALOGS)) {
    const code = /locales\/([^/]+)\/categorias\.ftl$/.exec(path)?.[1] as Locale | undefined;
    if (!code) continue;
    const names = new Map<string, string>();
    for (const line of src.split('\n')) {
      const hit = /^(categorias-[a-z0-9-]+) = (.+)$/.exec(line);
      if (hit?.[1] && hit[2] && !hit[1].includes('--')) names.set(hit[1], hit[2].trim());
    }
    out.set(code, names);
  }
  catalogCache = out;
  return out;
}

const CHAIN: Record<Locale, Locale[]> = {
  'pt-BR': ['pt-BR'],
  'pt-PT': ['pt-PT', 'pt-BR'],
  en: ['en', 'pt-BR'],
  es: ['es', 'en', 'pt-BR'],
};

/** Nome da semente no idioma (com a reserva). */
function seedName(locale: Locale, key: string): string | undefined {
  for (const l of CHAIN[locale]) {
    const name = catalogs().get(l)?.get(key);
    if (name) return name;
  }
  return undefined;
}

const slug = (s: string) =>
  foldAccent(s)
    .replace(/º/g, 'o')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');

/** Chave de origem de cada categoria padrão (no núcleo, `category.seed_key`), pelo nome em pt-BR. */
let seedKeys = new Map<string, string>();
onMockReset(() => {
  seedKeys = new Map();
});

function keyOf(c: CategoryRow): string | undefined {
  const known = seedKeys.get(c.id);
  if (known) return known;
  const br = catalogs().get('pt-BR');
  if (!br) return undefined;
  const keys = [...br].filter(([, n]) => n === c.name).map(([k]) => k);
  if (!keys.length) return undefined;
  const parent = c.parent_id ? find('category', c.parent_id) : undefined;
  const qualified =
    parent && keys.find((k) => k === `categorias-${slug(parent.name)}-${slug(c.name)}`);
  const key = qualified ?? keys[0]!;
  seedKeys.set(c.id, key);
  return key;
}

/** Nome em pt-BR (o de origem) de uma categoria padrão; as outras, o nome que têm. */
function brName(c: CategoryRow): string {
  const key = keyOf(c);
  return (key && catalogs().get('pt-BR')?.get(key)) || c.name;
}

/** Como `categories::translation_plan`: as padrão com um nome padrão que não é o do idioma atual. */
function translationPlan(): [id: string, name: string][] {
  const locale = mockLocale();
  const all = rows('category');
  const level = (c: CategoryRow) => `${c.parent_id ?? ''}|${c.kind}`;
  const taken = new Map<string, Set<string>>();
  for (const c of all) taken.set(level(c), (taken.get(level(c)) ?? new Set()).add(fold(c.name)));
  const out: [string, string][] = [];
  for (const c of all) {
    const key = keyOf(c);
    const target = key && seedName(locale, key);
    if (!key || !target || target === c.name) continue;
    if (![...catalogs().values()].some((m) => m.get(key) === c.name)) continue;
    const names = taken.get(level(c))!;
    if (fold(target) !== fold(c.name)) {
      if (names.has(fold(target))) continue;
      names.delete(fold(c.name));
      names.add(fold(target));
    }
    out.push([c.id, target]);
  }
  return out;
}

function missingDefaults(): MissingDefault[] {
  const all = rows('category');
  const out: MissingDefault[] = [];
  for (const [kind, name, children] of FAMILY) {
    const parent = all.find(
      (c) => !c.parent_id && c.kind === kind && fold(brName(c)) === fold(name),
    );
    if (!parent) {
      out.push({ kind, path: name, parent_exists: false });
      for (const c of children) out.push({ kind, path: `${name} › ${c}`, parent_exists: false });
      continue;
    }
    for (const c of children)
      if (!all.some((r) => r.parent_id === parent.id && fold(brName(r)) === fold(c)))
        out.push({ kind, path: `${parent.name} › ${c}`, parent_exists: true });
  }
  return out;
}

/** Como `categories::add_missing_defaults`: todas, ou só os caminhos pedidos (uma subcategoria cria o grupo). */
function addDefaults(paths?: string[]): number {
  const wanted = paths ? new Set(paths.map(fold)) : null;
  const picked = (path: string) => !wanted || wanted.has(fold(path));
  let n = 0;
  const now = nowUtc();
  for (const [kind, name, children] of FAMILY) {
    let parent = rows('category').find(
      (c) => !c.parent_id && c.kind === kind && fold(brName(c)) === fold(name),
    );
    if (!parent) {
      if (!picked(name) && !children.some((c) => picked(`${name} › ${c}`))) continue;
      parent = put('category', {
        ...rows('category')[0]!,
        id: newId(),
        parent_id: null,
        name,
        kind,
        is_essential: countsAsEssential(defaultGroup(kind, name)),
        spend_group: defaultGroup(kind, name),
        archived: false,
        sort_order: 100,
        created_at: now,
        updated_at: now,
      });
      n++;
    }
    for (const c of children) {
      if (rows('category').some((r) => r.parent_id === parent.id && fold(brName(r)) === fold(c)))
        continue;
      if (!picked(`${parent.name} › ${c}`)) continue;
      const group = defaultGroup(kind, name, c);
      put('category', {
        ...parent,
        id: newId(),
        parent_id: parent.id,
        name: c,
        is_essential: countsAsEssential(group),
        spend_group: group,
        sort_order: 100,
        created_at: now,
        updated_at: now,
      });
      n++;
    }
  }
  return n;
}

const kindLabel = (k: CategoryKind) => (k === 'income' ? 'receita' : 'despesa');

// ── Grupos de gasto (essenciais, dívidas, futuro, não essencial) ───────────────

export const SPEND_GROUPS: readonly SpendGroup[] = ['essential', 'debt', 'future', 'non_essential'];

/** Grupo padrão das categorias do conjunto inicial que o seed do mock não tem (pelo nome em pt-BR). */
const DEFAULT_ROOT_GROUP: Record<string, SpendGroup> = {
  Moradia: 'essential',
  Saúde: 'essential',
  Educação: 'essential',
  Seguros: 'essential',
  Pagamentos: 'essential',
  Filhos: 'essential',
  'Poupança e investimentos': 'future',
};

/** Subcategorias que fogem do grupo do pai (como `child_essential` no núcleo). */
const DEFAULT_CHILD_GROUP: Record<string, SpendGroup> = {
  'Pagamentos › Empréstimos': 'debt',
  'Pagamentos › Financiamentos': 'debt',
  'Pagamentos › Parcelas': 'debt',
  'Filhos › Brinquedos': 'non_essential',
  'Filhos › Festas e aniversários': 'non_essential',
  'Seguros › Seguro viagem': 'non_essential',
};

function defaultGroup(kind: CategoryKind, root: string, child?: string): SpendGroup | null {
  if (kind === 'income') return null;
  if (child) {
    const own = DEFAULT_CHILD_GROUP[`${root} › ${child}`];
    if (own) return own;
  }
  return DEFAULT_ROOT_GROUP[root] ?? 'non_essential';
}

function spendGroupArg(v: unknown): SpendGroup {
  if (typeof v === 'string' && (SPEND_GROUPS as readonly string[]).includes(v))
    return v as SpendGroup;
  return invalid(
    `argumentos inválidos: unknown variant \`${String(v)}\`, expected one of \`essential\`, \`debt\`, \`future\`, \`non_essential\``,
  );
}

/**
 * Como `categories::set_group`: grupo das despesas pedidas (e `is_essential` junto); as de
 * receita da lista ficam como estão.
 */
function setGroup(ids: string[], group: SpendGroup): number {
  const now = nowUtc();
  let n = 0;
  for (const id of ids) {
    const c = find('category', id) ?? notFound(`categoria ${id}`);
    if (c.kind === 'income') continue;
    if (c.spend_group === group && c.is_essential === countsAsEssential(group)) continue;
    patch('category', id, {
      spend_group: group,
      is_essential: countsAsEssential(group),
      updated_at: now,
    });
    n++;
  }
  return n;
}

function get(id: string): Category {
  const c = find('category', id) ?? notFound(`categoria ${id}`);
  return toCategory(c);
}

function flatten(nodes: CategoryNode[], out: Category[] = []): Category[] {
  for (const { children, ...c } of nodes) {
    out.push(c);
    flatten(children, out);
  }
  return out;
}

function usage(id: string): CategoryUsage {
  return {
    transactions: rows('txn').filter((t) => t.category_id === id).length,
    splits: rows('txn_split').filter((s) => s.category_id === id).length,
    budget_lines: 0,
    rules: rows('rule').filter((r) => r.actions.category_id === id).length,
    recurrences: rows('recurrence').filter((r) => r.category_id === id).length,
    installment_plans: rows('installment_plan').filter((p) => p.category_id === id).length,
    children: rows('category').filter((c) => c.parent_id === id).length,
  };
}

const inUse = (u: CategoryUsage) =>
  u.transactions + u.splits + u.budget_lines + u.rules + u.recurrences + u.installment_plans > 0;

function ensureUniqueSibling(
  parent: string | null,
  kind: CategoryKind,
  name: string,
  except?: string,
): void {
  const target = fold(name);
  const clash = rows('category').some(
    (c) => c.parent_id === parent && c.kind === kind && c.id !== except && fold(c.name) === target,
  );
  if (clash) conflict(`já existe uma categoria chamada ${name} neste nível`);
}

function checkParent(parentId: string, kind: CategoryKind): void {
  const p = find('category', parentId) ?? notFound(`categoria-pai ${parentId}`);
  if (p.kind !== kind) {
    invalid(
      `a subcategoria deve ser do mesmo tipo da categoria-pai (${p.name} é de ${kindLabel(p.kind)})`,
    );
  }
}

function reassign(from: string, into: string): CategoryUsage {
  const now = nowUtc();
  const moved: CategoryUsage = {
    transactions: 0,
    splits: 0,
    budget_lines: 0,
    rules: 0,
    recurrences: 0,
    installment_plans: 0,
    children: 0,
  };
  for (const t of rows('txn')) if (t.category_id === from) { patch('txn', t.id, { category_id: into, updated_at: now }); moved.transactions++; } // prettier-ignore
  for (const s of rows('txn_split')) if (s.category_id === from) { patch('txn_split', s.id, { category_id: into, updated_at: now }); moved.splits++; } // prettier-ignore
  for (const r of rows('rule')) if (r.actions.category_id === from) { patch('rule', r.id, { actions: { ...r.actions, category_id: into }, updated_at: now }); moved.rules++; } // prettier-ignore
  for (const r of rows('recurrence')) if (r.category_id === from) { patch('recurrence', r.id, { category_id: into, updated_at: now }); moved.recurrences++; } // prettier-ignore
  for (const p of rows('installment_plan')) if (p.category_id === from) { patch('installment_plan', p.id, { category_id: into, updated_at: now }); moved.installment_plans++; } // prettier-ignore
  return moved;
}

function checkTarget(from: CategoryRow, into: string): CategoryRow {
  if (into === from.id) invalid('escolha uma categoria diferente para receber os lançamentos');
  const target = find('category', into) ?? notFound(`categoria ${into}`);
  if (target.kind !== from.kind) {
    invalid(
      `${from.name} é de ${kindLabel(from.kind)} e ${target.name} é de ${kindLabel(target.kind)}; só é possível juntar categorias do mesmo tipo`,
    );
  }
  if (withDescendants([from.id]).includes(target.id)) {
    invalid(
      `${categoryPaths().get(target.id)} é subcategoria de ${from.name}; escolha uma categoria fora dela`,
    );
  }
  return target;
}

function mergeInner(from: string, into: string): void {
  const targetChildren = rows('category').filter((c) => c.parent_id === into);
  for (const child of rows('category').filter((c) => c.parent_id === from)) {
    const same = targetChildren.find((t) => fold(t.name) === fold(child.name));
    if (same) mergeInner(child.id, same.id);
    else patch('category', child.id, { parent_id: into, updated_at: nowUtc() });
  }
  reassign(from, into);
  del('category', from);
}

registerMockHandlers(
  guarded({
    'categories.list': (args) =>
      flatten(categoryTree(args.kind as CategoryKind | undefined, Boolean(args.include_archived))),
    'categories.tree': (args) =>
      categoryTree(args.kind as CategoryKind | undefined, Boolean(args.include_archived)),
    'categories.get': (args) => get(str(args, 'id')),
    'categories.missing_defaults': () => missingDefaults(),
    'categories.untranslated_defaults': () => translationPlan().length,
    'categories.translate_defaults': () =>
      write('Categorias padrão traduzidas', () => {
        const plan = translationPlan();
        const now = nowUtc();
        for (const [id, name] of plan) patch('category', id, { name, updated_at: now });
        return plan.length;
      }).value,
    'categories.add_defaults': (args) => {
      const paths = Array.isArray(args.paths) ? args.paths.map(String) : undefined;
      return write('Categorias sugeridas adicionadas', () => addDefaults(paths)).value;
    },
    'categories.usage': (args) => {
      const id = str(args, 'id');
      get(id);
      return usage(id);
    },
    'categories.create': (args) => {
      const name = cleanName(args.name, 'da categoria');
      const kind = args.kind as CategoryKind;
      if (kind !== 'income' && kind !== 'expense')
        invalid(`argumentos inválidos: unknown variant \`${String(args.kind)}\``);
      return write(`Categoria criada: ${name}`, () => {
        const parent = cleanOpt(args.parent_id);
        if (parent) checkParent(parent, kind);
        ensureUniqueSibling(parent, kind, name);
        const siblings = rows('category').filter((c) => c.parent_id === parent && c.kind === kind);
        const parentRow = parent ? find('category', parent) : undefined;
        if (kind === 'income' && has(args, 'spend_group') && args.spend_group != null)
          invalid('categorias de receita não têm grupo de gasto');
        // Grupo: o pedido; senão o do pai; sem pai, "não essencial". `is_essential` antigo
        // ainda vale quando vem sozinho (true = essenciais).
        const group: SpendGroup | null =
          kind === 'income'
            ? null
            : has(args, 'spend_group') && args.spend_group != null
              ? spendGroupArg(args.spend_group)
              : typeof args.is_essential === 'boolean'
                ? args.is_essential
                  ? 'essential'
                  : parentRow?.spend_group && parentRow.spend_group !== 'essential'
                    ? parentRow.spend_group
                    : 'non_essential'
                : (parentRow?.spend_group ?? 'non_essential');
        const essential = countsAsEssential(group);
        const now = nowUtc();
        const row = put('category', {
          id: newId(),
          parent_id: parent,
          name,
          kind,
          icon: cleanOpt(args.icon),
          color: cleanOpt(args.color),
          is_essential: essential,
          spend_group: group,
          archived: false,
          sort_order: Math.max(-1, ...siblings.map((s) => s.sort_order)) + 1,
          created_at: now,
          updated_at: now,
        });
        return toCategory(row);
      }).value;
    },
    'categories.update': (args) => {
      const id = str(args, 'id');
      return write('Categoria alterada', () => {
        const cur = find('category', id) ?? notFound(`categoria ${id}`);
        const name = has(args, 'name') ? cleanName(args.name, 'da categoria') : cur.name;
        const parent = has(args, 'parent_id') ? cleanOpt(args.parent_id) : cur.parent_id;
        if (parent !== cur.parent_id && parent) {
          if (parent === cur.id || withDescendants([cur.id]).includes(parent)) {
            invalid('uma categoria não pode ficar dentro dela mesma nem de uma subcategoria sua');
          }
          checkParent(parent, cur.kind);
        }
        if (parent !== cur.parent_id || fold(name) !== fold(cur.name)) {
          ensureUniqueSibling(parent, cur.kind, name, cur.id);
        }
        const changes: Partial<CategoryRow> = { name, parent_id: parent };
        if (has(args, 'icon')) changes.icon = cleanOpt(args.icon);
        if (has(args, 'color')) changes.color = cleanOpt(args.color);
        if (has(args, 'is_essential') && cur.kind === 'expense') {
          const on = Boolean(optBool(args, 'is_essential'));
          changes.is_essential = on;
          // Como no núcleo: dívida continua dívida ao ligar; desligar tira de essenciais e dívidas.
          if (on && !countsAsEssential(cur.spend_group)) changes.spend_group = 'essential';
          else if (!on && countsAsEssential(cur.spend_group)) changes.spend_group = 'non_essential';
        } else if (has(args, 'is_essential')) {
          changes.is_essential = Boolean(optBool(args, 'is_essential'));
        }
        if (has(args, 'spend_group') && args.spend_group != null) {
          if (cur.kind === 'income') invalid('categorias de receita não têm grupo de gasto');
          const group = spendGroupArg(args.spend_group);
          changes.spend_group = group;
          changes.is_essential = countsAsEssential(group);
        }
        if (has(args, 'archived')) changes.archived = Boolean(optBool(args, 'archived'));
        if (has(args, 'sort_order')) changes.sort_order = Number(args.sort_order);
        const next = { ...cur, ...changes };
        if (JSON.stringify(next) !== JSON.stringify(cur))
          patch('category', id, { ...changes, updated_at: nowUtc() });
        return get(id);
      }).value;
    },
    'categories.set_group': (args) => {
      const ids = Array.isArray(args.ids) ? args.ids.map(String) : [];
      if (!ids.length) invalid('argumentos inválidos: escolha ao menos uma categoria');
      const group = spendGroupArg(args.group);
      return {
        updated: write('Grupo de gasto das categorias alterado', () => setGroup(ids, group)).value,
      };
    },
    'categories.delete': (args) => {
      const id = str(args, 'id');
      const cat = find('category', id) ?? notFound(`categoria ${id}`);
      const path = categoryPaths().get(id) ?? cat.name;
      const into = cleanOpt(args.reassign_to);
      const reason = into
        ? `Categoria apagada com reatribuição: ${path}`
        : `Categoria apagada: ${path}`;
      return write(reason, () => {
        const u = usage(id);
        if (u.children > 0) {
          conflict(
            `${cat.name} tem ${u.children} subcategoria(s); apague, mova ou junte as subcategorias antes`,
          );
        }
        let moved: CategoryUsage = {
          transactions: 0,
          splits: 0,
          budget_lines: 0,
          rules: 0,
          recurrences: 0,
          installment_plans: 0,
          children: 0,
        };
        if (into) {
          checkTarget(cat, into);
          moved = reassign(id, into);
        } else if (inUse(u)) {
          conflict(
            `${cat.name} está em uso (${u.transactions} lançamento(s), ${u.splits} divisão(ões), ${u.budget_lines} linha(s) de orçamento, ${u.rules} regra(s), ${u.recurrences} recorrência(s), ${u.installment_plans} parcelamento(s)); escolha uma categoria para receber esse uso`,
          );
        }
        del('category', id);
        return moved;
      }).value;
    },
    'categories.merge': (args) => {
      const from =
        find('category', str(args, 'from')) ?? notFound(`categoria ${String(args.from)}`);
      const into = str(args, 'into');
      const paths = categoryPaths();
      const target = find('category', into) ?? notFound(`categoria ${into}`);
      const reason = `Categorias juntadas: ${paths.get(from.id)} → ${paths.get(target.id)}`;
      return write(reason, () => {
        checkTarget(from, into);
        mergeInner(from.id, into);
        return get(into);
      }).value;
    },
  }),
);
