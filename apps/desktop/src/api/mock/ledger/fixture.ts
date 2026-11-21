/**
 * Família fictícia do núcleo falso: titulares, contas e a árvore de categorias padrão
 * do núcleo (mesmos nomes de `categories::defaults`), com ids estáveis e legíveis.
 * Não é registrado como mock: os módulos `reports`, `budget` e `goals` o usam.
 *
 * `BASE_*` são a semente do banco único do mock (`seed.ts`); `ACCOUNTS`, `CATEGORIES` e
 * `MEMBERS` são listas vivas, espelhadas do banco a cada escrita (`syncLive`), para que
 * contas, categorias e titulares criados nas telas apareçam nos relatórios.
 */
import type { Account } from '@/api/generated/Account';
import type { AccountKind } from '@/api/generated/AccountKind';
import type { Category } from '@/api/generated/Category';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import type { Member } from '@/api/generated/Member';
import type { SpendGroup } from '@/api/generated/SpendGroup';

/**
 * Moeda base da família no banco do mock (`household.base_currency`), espelhada a cada escrita
 * (`view.ts`). As contas da família de exemplo que são em reais nascem nela (`seed.ts`).
 */
export let BASE = 'BRL';

export function setFixtureBase(code: string): void {
  BASE = code;
}

export const BASE_MEMBERS: readonly Member[] = [
  {
    id: 'mem-ana',
    name: 'Ana',
    color: null,
    is_primary: true,
    archived: false,
    sort_order: 0,
    profile: {
      birth_date: '1988-04-12',
      relationship: 'titular',
      irpf_dependent: false,
      occupation: 'Engenheira civil',
      notes: null,
      sex: 'female',
    },
  },
  {
    id: 'mem-bruno',
    name: 'Bruno',
    color: null,
    is_primary: false,
    archived: false,
    sort_order: 1,
    profile: {
      birth_date: '1987-09-02',
      relationship: 'conjuge',
      irpf_dependent: false,
      occupation: 'Designer',
      notes: null,
      sex: 'male',
    },
  },
];

export interface FixtureAccount {
  id: string;
  name: string;
  kind: AccountKind;
  currency: string;
  opening_balance: number;
  member_ids: string[];
  institution: string | null;
  include_in_net_worth: boolean;
  card?: { closing_day: number; due_day: number; credit_limit: number; payment: string };
}

export const BASE_ACCOUNTS: readonly FixtureAccount[] = [
  {
    id: 'acc-itau',
    name: 'Itaú Conta Corrente',
    kind: 'checking',
    currency: 'BRL',
    opening_balance: 1_850_000,
    member_ids: ['mem-ana', 'mem-bruno'],
    institution: 'Itaú Unibanco',
    include_in_net_worth: true,
  },
  {
    id: 'acc-nuconta',
    name: 'Nubank Conta',
    kind: 'checking',
    currency: 'BRL',
    opening_balance: 320_000,
    member_ids: ['mem-bruno'],
    institution: 'Nubank',
    include_in_net_worth: true,
  },
  {
    id: 'acc-poupanca',
    name: 'Poupança Caixa',
    kind: 'savings',
    currency: 'BRL',
    opening_balance: 2_200_000,
    member_ids: ['mem-ana'],
    institution: 'Caixa',
    include_in_net_worth: true,
  },
  {
    id: 'acc-carteira',
    name: 'Carteira',
    kind: 'cash',
    currency: 'BRL',
    opening_balance: 35_000,
    member_ids: [],
    institution: null,
    include_in_net_worth: true,
  },
  {
    id: 'acc-nucard',
    name: 'Cartão Nubank',
    kind: 'credit_card',
    currency: 'BRL',
    opening_balance: 0,
    member_ids: ['mem-bruno'],
    institution: 'Nubank',
    include_in_net_worth: true,
    card: { closing_day: 3, due_day: 10, credit_limit: 1_800_000, payment: 'acc-itau' },
  },
  {
    id: 'acc-itaucard',
    name: 'Cartão Itaú Visa',
    kind: 'credit_card',
    currency: 'BRL',
    opening_balance: 0,
    member_ids: ['mem-ana'],
    institution: 'Itaú Unibanco',
    include_in_net_worth: true,
    card: { closing_day: 25, due_day: 5, credit_limit: 2_500_000, payment: 'acc-itau' },
  },
  {
    id: 'acc-xp',
    name: 'XP Investimentos',
    kind: 'brokerage',
    currency: 'BRL',
    opening_balance: 420_000,
    member_ids: ['mem-ana', 'mem-bruno'],
    institution: 'XP',
    include_in_net_worth: true,
  },
  {
    id: 'acc-wise',
    name: 'Wise USD',
    kind: 'foreign',
    currency: 'USD',
    opening_balance: 120_000,
    member_ids: ['mem-bruno'],
    institution: 'Wise',
    include_in_net_worth: true,
  },
  {
    id: 'acc-eur',
    name: 'Revolut EUR',
    kind: 'foreign',
    currency: 'EUR',
    opening_balance: 64_000,
    member_ids: ['mem-ana'],
    institution: 'Revolut',
    include_in_net_worth: true,
  },
];

export const MEMBERS: Member[] = [...BASE_MEMBERS];
export const ACCOUNTS: FixtureAccount[] = [...BASE_ACCOUNTS];

export function fixtureAccount(id: string): FixtureAccount | undefined {
  return ACCOUNTS.find((a) => a.id === id);
}

/** Conta no formato do núcleo (`Account`), com a data de abertura do livro-caixa. */
export function toAccount(a: FixtureAccount, openingDate: string, sortOrder: number): Account {
  return {
    id: a.id,
    name: a.name,
    kind: a.kind,
    currency: a.currency,
    institution_id: a.institution
      ? `inst-${a.institution.toLowerCase().replace(/\W+/g, '-')}`
      : null,
    institution_name: a.institution,
    member_ids: a.member_ids,
    opening_balance: a.opening_balance,
    opening_date: openingDate,
    include_in_net_worth: a.include_in_net_worth,
    archived: false,
    color: null,
    notes: null,
    sort_order: sortOrder,
    overdraft: null,
    branch: null,
    account_number: null,
    retirement_plan: null,
    retirement_coverage: null,
    card: a.card
      ? {
          credit_limit: a.card.credit_limit,
          closing_day: a.card.closing_day,
          due_day: a.card.due_day,
          payment_account_id: a.card.payment,
          network: a.id === 'acc-nucard' ? 'Mastercard' : 'Visa',
          last4: a.id === 'acc-nucard' ? '4421' : '9087',
        }
      : null,
  };
}

// ── Categorias ───────────────────────────────────────────────────

type Def = [slug: string, name: string, essential: boolean, children?: [string, string, boolean][]];

const EXPENSE: Def[] = [
  [
    'moradia',
    'Moradia',
    true,
    [
      ['aluguel', 'Aluguel', true],
      ['condominio', 'Condomínio', true],
      ['energia', 'Energia', true],
      ['agua', 'Água', true],
      ['gas', 'Gás', true],
      ['internet', 'Internet', true],
      ['iptu', 'IPTU', true],
      ['manutencao', 'Manutenção', true],
    ],
  ],
  [
    'alimentacao',
    'Alimentação',
    true,
    [
      ['mercado', 'Mercado', true],
      ['restaurantes', 'Restaurantes', false],
      ['delivery', 'Delivery', false],
      ['padaria', 'Padaria', true],
    ],
  ],
  [
    'transporte',
    'Transporte',
    true,
    [
      ['combustivel', 'Combustível', true],
      ['publico', 'Transporte público', true],
      ['aplicativos', 'Aplicativos', false],
      ['estacionamento', 'Estacionamento', true],
      ['veiculo', 'Manutenção do veículo', true],
      ['ipva', 'IPVA e licenciamento', true],
      ['seguro-auto', 'Seguro do veículo', true],
    ],
  ],
  [
    'saude',
    'Saúde',
    true,
    [
      ['plano', 'Plano de saúde', true],
      ['farmacia', 'Farmácia', true],
      ['consultas', 'Consultas', true],
      ['exames', 'Exames', true],
      ['dentista', 'Dentista', true],
    ],
  ],
  [
    'educacao',
    'Educação',
    true,
    [
      ['escola', 'Escola', true],
      ['cursos', 'Cursos', true],
      ['livros', 'Livros e material', true],
    ],
  ],
  [
    'lazer',
    'Lazer',
    false,
    [
      ['viagens', 'Viagens', false],
      ['passeios', 'Passeios', false],
      ['cinema', 'Cinema e shows', false],
      ['hobbies', 'Hobbies', false],
    ],
  ],
  [
    'compras',
    'Compras',
    false,
    [
      ['vestuario', 'Vestuário', false],
      ['eletronicos', 'Eletrônicos', false],
      ['casa', 'Casa e decoração', false],
    ],
  ],
  [
    'cuidados',
    'Cuidados pessoais',
    false,
    [
      ['cabelo', 'Cabelo e estética', false],
      ['academia', 'Academia', false],
      ['cosmeticos', 'Cosméticos', false],
    ],
  ],
  [
    'assinaturas',
    'Assinaturas',
    false,
    [
      ['streaming', 'Streaming', false],
      ['software', 'Software e apps', false],
      ['telefone', 'Telefone', false],
    ],
  ],
  [
    'impostos',
    'Impostos e taxas',
    true,
    [
      ['tarifas', 'Tarifas bancárias', true],
      ['juros', 'Juros e multas', true],
      ['iof', 'IOF', true],
      ['ir', 'Imposto de renda', true],
    ],
  ],
  [
    'seguros',
    'Seguros',
    true,
    [
      ['seguro-vida', 'Seguro de vida', true],
      ['seguro-casa', 'Seguro residencial', true],
    ],
  ],
  [
    'pets',
    'Pets',
    false,
    [
      ['racao', 'Ração', false],
      ['veterinario', 'Veterinário', false],
    ],
  ],
  [
    'presentes',
    'Presentes e doações',
    false,
    [
      ['presentes-dados', 'Presentes', false],
      ['doacoes', 'Doações', false],
    ],
  ],
  [
    'filhos',
    'Filhos',
    true,
    [
      ['mesada', 'Mesada', true],
      ['atividades', 'Atividades', true],
    ],
  ],
  ['outras-despesas', 'Outras despesas', false],
];

const INCOME: Def[] = [
  [
    'salario',
    'Salário',
    false,
    [
      ['13', '13º salário', false],
      ['ferias', 'Férias', false],
      ['plr', 'PLR e bônus', false],
    ],
  ],
  ['pro-labore', 'Pró-labore', false],
  ['freelance', 'Serviços e freelance', false],
  [
    'rendimentos',
    'Rendimentos',
    false,
    [
      ['dividendos', 'Dividendos', false],
      ['juros-rec', 'Juros', false],
      ['jcp', 'JCP', false],
      ['alugueis', 'Aluguéis recebidos', false],
    ],
  ],
  ['reembolsos', 'Reembolsos', false],
  ['restituicao', 'Restituição de IR', false],
  ['presentes-rec', 'Presentes recebidos', false],
  ['outras-receitas', 'Outras receitas', false],
];

/**
 * Grupo de gasto das categorias do livro que fogem da regra "essencial → Essenciais, senão Não
 * essencial" (como o grupo padrão pela semente no núcleo).
 */
const GROUP_EXCEPTIONS: Record<string, SpendGroup> = {
  'cat-impostos-juros': 'debt',
};

function groupOf(id: string, kind: CategoryKind, essential: boolean): SpendGroup | null {
  if (kind === 'income') return null;
  return GROUP_EXCEPTIONS[id] ?? (essential ? 'essential' : 'non_essential');
}

function build(defs: Def[], kind: CategoryKind): Category[] {
  const out: Category[] = [];
  defs.forEach(([slug, name, essential, children], i) => {
    const id = `cat-${slug}`;
    const group = groupOf(id, kind, essential);
    out.push({
      id,
      parent_id: null,
      name,
      kind,
      icon: null,
      color: null,
      is_essential: group === 'essential' || group === 'debt',
      spend_group: group,
      archived: false,
      sort_order: i,
      path: name,
    });
    children?.forEach(([cslug, cname, cess], j) => {
      const cid = `cat-${slug}-${cslug}`;
      const cgroup = groupOf(cid, kind, cess);
      out.push({
        id: cid,
        parent_id: id,
        name: cname,
        kind,
        icon: null,
        color: null,
        is_essential: cgroup === 'essential' || cgroup === 'debt',
        spend_group: cgroup,
        archived: false,
        sort_order: j,
        path: `${name} › ${cname}`,
      });
    });
  });
  return out;
}

/** Categorias na ordem da árvore (pais antes dos filhos), despesas e depois receitas. */
export const BASE_CATEGORIES: readonly Category[] = [
  ...build(EXPENSE, 'expense'),
  ...build(INCOME, 'income'),
];
export const CATEGORIES: Category[] = [...BASE_CATEGORIES];

const BY_ID = new Map(CATEGORIES.map((c) => [c.id, c]));

/** Espelha as listas vivas a partir do banco do mock (chamado a cada escrita). */
export function syncLive(next: {
  accounts: FixtureAccount[];
  categories: Category[];
  members: Member[];
}): void {
  ACCOUNTS.splice(0, ACCOUNTS.length, ...next.accounts);
  CATEGORIES.splice(0, CATEGORIES.length, ...next.categories);
  MEMBERS.splice(0, MEMBERS.length, ...next.members);
  BY_ID.clear();
  for (const c of CATEGORIES) BY_ID.set(c.id, c);
}

export function category(id: string | null | undefined): Category | undefined {
  return id ? BY_ID.get(id) : undefined;
}

/** Id da categoria de primeiro nível. */
export function rootOf(id: string): string {
  let c = BY_ID.get(id);
  while (c?.parent_id) c = BY_ID.get(c.parent_id);
  return c?.id ?? id;
}

/** A categoria e todas as descendentes. */
export function withDescendants(ids: readonly string[]): Set<string> {
  const out = new Set<string>();
  const visit = (id: string) => {
    if (out.has(id)) return;
    out.add(id);
    for (const c of CATEGORIES) if (c.parent_id === id) visit(c.id);
  };
  ids.forEach(visit);
  return out;
}

/** Ids de categoria de um caminho curto: `cat('alimentacao', 'mercado')`. */
export function cat(root: string, child?: string): string {
  return child ? `cat-${root}-${child}` : `cat-${root}`;
}
