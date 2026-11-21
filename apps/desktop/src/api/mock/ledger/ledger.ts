/**
 * Livro-caixa fictício e determinístico: 24 meses de lançamentos de uma família (salários,
 * contas fixas, mercado, cartões com faturas, parcelamentos, transferências), mais os
 * previstos dos próximos ~60 dias. Os relatórios, o orçamento e as metas do mock são
 * calculados a partir dele com as regras do núcleo, então os números batem entre si.
 *
 * A data de hoje é a do relógio (ou `setMockToday` nos testes); cada mês usa a própria
 * semente, então o histórico não muda quando o dia muda.
 */
import type { RuleSpec } from '@/api/generated/RuleSpec';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import {
  addDays,
  addMonthsToDate,
  clampDay,
  monthBounds,
  monthOf,
  monthsBetween,
  parseIso,
  shiftMonth,
} from '@/lib/months';

import { ACCOUNTS, BASE_ACCOUNTS, cat } from './fixture';
import { ledgerView } from './view';

export interface LedgerSplit {
  category_id: string | null;
  member_id: string | null;
  amount: number;
}

export interface LedgerTxn {
  id: string;
  date: string;
  account_id: string;
  /** Com sinal, na moeda da conta. */
  amount: number;
  currency: string;
  kind: TxnKind;
  status: TxnStatus;
  description: string;
  category_id: string | null;
  member_id: string | null;
  splits?: LedgerSplit[];
  statement_id: string | null;
  /** Pernas de transferência/aporte compartilham o id. */
  transfer_id?: string;
  recurrence_id?: string;
  installment?: { plan: string; n: number; of: number };
}

export interface LedgerStatement {
  id: string;
  account_id: string;
  reference_month: string;
  period_start: string;
  closing_date: string;
  due_date: string;
}

export interface RecurrenceDef {
  id: string;
  description: string;
  account_id: string;
  kind: TxnKind;
  category_id: string | null;
  member_id: string | null;
  to_account_id?: string;
  day: number;
  /** Valor com sinal no mês (reajustes). */
  amountAt: (month: string, index: number) => number;
  /** Só em alguns meses do ano (1–12). */
  months?: number[];
  spec: RuleSpec;
}

export interface Ledger {
  today: string;
  /** Primeiro dia do livro (abertura das contas). */
  start: string;
  firstMonth: string;
  txns: LedgerTxn[];
  statements: Map<string, LedgerStatement>;
  recurrences: RecurrenceDef[];
}

/** Compra mensal de ativos na corretora (igual ao aporte). */
export const PORTFOLIO_PURCHASE = 200_000;

/** Meses de histórico (o atual incluso). */
export const HISTORY_MONTHS = 24;
/** Horizonte dos previstos (como `recurrence::DEFAULT_HORIZON_DAYS`). */
export const HORIZON_DAYS = 62;

export { mockToday, setMockToday } from './clock';

/** O gerador usa sempre as contas da semente (as listas vivas espelham o banco). */
const baseAccount = (id: string) => BASE_ACCOUNTS.find((a) => a.id === id);

/** O banco do mock é refeito no `resetMock` (ver `boot.ts`); nada a limpar aqui. */
export function resetLedger(): void {}

/**
 * O livro-caixa visto pelos relatórios, orçamento, metas e carteira: derivado do banco
 * único do mock (`store.ts`), que é semeado com `buildLedger` e recebe as escritas das
 * telas de contas e lançamentos. Assim um lançamento criado, editado ou desfeito
 * aparece nos relatórios, e os ids (`acc-…`, `cat-…`, `txn-…`) são os mesmos em tudo.
 */
export function getLedger(): Ledger {
  return ledgerView();
}

// ── Aleatoriedade determinística ────────────────────────────────

function hash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function rng(seed: string) {
  let a = hash(seed);
  const next = () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  return {
    next,
    /** Inteiro em [a, b]. */
    int: (lo: number, hi: number) => lo + Math.floor(next() * (hi - lo + 1)),
    /** Valor em centavos em [lo, hi], arredondado a R$ 0,10. */
    cents: (lo: number, hi: number) => Math.round((lo + next() * (hi - lo)) / 10) * 10,
    chance: (p: number) => next() < p,
    pick: <T>(list: readonly T[]): T => list[Math.floor(next() * list.length)]!,
  };
}

// ── Faturas ──────────────────────────────────────────────────────

/** Mesma regra de `cards::statement_dates`. */
export function statementDates(date: string, closingDay: number, dueDay: number) {
  const { y, m } = parseIso(date);
  const thisClose = clampDay(y, m, closingDay);
  const closeMonth = date <= thisClose ? monthOf(date) : shiftMonth(monthOf(date), 1);
  const cm = parseIso(closeMonth);
  const closing = clampDay(cm.y, cm.m, closingDay);
  const dueMonth = dueDay > closingDay ? closeMonth : shiftMonth(closeMonth, 1);
  const dm = parseIso(dueMonth);
  const due = clampDay(dm.y, dm.m, dueDay);
  const pm = parseIso(shiftMonth(closeMonth, -1));
  const prevClose = clampDay(pm.y, pm.m, closingDay);
  return {
    reference_month: monthOf(due),
    period_start: addDays(prevClose, 1),
    closing_date: closing,
    due_date: due,
  };
}

// ── Recorrências ─────────────────────────────────────────────────

const monthly = (day: number): RuleSpec => ({ freq: 'MONTHLY', interval: 1, by_month_day: day });

/** Reajuste: valor `before` até o mês `index` da janela, depois `after`. */
const step = (before: number, after: number, at: number) => (_m: string, i: number) =>
  i < at ? before : after;
const fixed = (v: number) => () => v;

export const RECURRENCES: RecurrenceDef[] = [
  {
    id: 'rec-salario-ana',
    description: 'Salário Ana · Alfa Engenharia',
    account_id: 'acc-itau',
    kind: 'income',
    category_id: cat('salario'),
    member_id: 'mem-ana',
    day: 5,
    amountAt: step(1_300_000, 1_380_000, 12),
    spec: monthly(5),
  },
  {
    id: 'rec-salario-bruno',
    description: 'Salário Bruno · Studio Norte',
    account_id: 'acc-nuconta',
    kind: 'income',
    category_id: cat('salario'),
    member_id: 'mem-bruno',
    day: 5,
    amountAt: step(920_000, 970_000, 14),
    spec: monthly(5),
  },
  {
    id: 'rec-aluguel',
    description: 'Aluguel · Imobiliária Lar',
    account_id: 'acc-itau',
    kind: 'expense',
    category_id: cat('moradia', 'aluguel'),
    member_id: 'mem-ana',
    day: 10,
    amountAt: step(-320_000, -334_000, 12),
    spec: monthly(10),
  },
  {
    id: 'rec-condominio',
    description: 'Condomínio Edifício Ipê',
    account_id: 'acc-itau',
    kind: 'expense',
    category_id: cat('moradia', 'condominio'),
    member_id: 'mem-ana',
    day: 10,
    amountAt: step(-78_000, -82_000, 15),
    spec: monthly(10),
  },
  {
    id: 'rec-plano',
    description: 'Unimed · plano familiar',
    account_id: 'acc-itau',
    kind: 'expense',
    category_id: cat('saude', 'plano'),
    member_id: 'mem-ana',
    day: 15,
    amountAt: step(-138_000, -152_000, 16),
    spec: monthly(15),
  },
  {
    id: 'rec-escola',
    description: 'Colégio Horizonte · mensalidade',
    account_id: 'acc-itau',
    kind: 'expense',
    category_id: cat('educacao', 'escola'),
    member_id: 'mem-ana',
    day: 5,
    amountAt: step(-198_000, -210_000, 15),
    spec: monthly(5),
  },
  {
    id: 'rec-natacao',
    description: 'Natação · Clube Aquático',
    account_id: 'acc-itau',
    kind: 'expense',
    category_id: cat('filhos', 'atividades'),
    member_id: 'mem-ana',
    day: 7,
    amountAt: fixed(-28_000),
    spec: monthly(7),
  },
  {
    id: 'rec-seguro-vida',
    description: 'Seguro de vida · Porto',
    account_id: 'acc-itau',
    kind: 'expense',
    category_id: cat('seguros', 'seguro-vida'),
    member_id: 'mem-ana',
    day: 25,
    amountAt: fixed(-16_000),
    spec: monthly(25),
  },
  {
    id: 'rec-tarifas',
    description: 'Tarifa pacote de serviços',
    account_id: 'acc-itau',
    kind: 'expense',
    category_id: cat('impostos', 'tarifas'),
    member_id: null,
    day: 2,
    amountAt: fixed(-4_590),
    spec: monthly(2),
  },
  {
    id: 'rec-internet',
    description: 'Vivo Fibra 500 Mb',
    account_id: 'acc-nucard',
    kind: 'expense',
    category_id: cat('moradia', 'internet'),
    member_id: 'mem-bruno',
    day: 12,
    amountAt: fixed(-11_990),
    spec: monthly(12),
  },
  {
    id: 'rec-netflix',
    description: 'Netflix',
    account_id: 'acc-nucard',
    kind: 'expense',
    category_id: cat('assinaturas', 'streaming'),
    member_id: 'mem-bruno',
    day: 17,
    amountAt: fixed(-5_590),
    spec: monthly(17),
  },
  {
    id: 'rec-spotify',
    description: 'Spotify Família',
    account_id: 'acc-nucard',
    kind: 'expense',
    category_id: cat('assinaturas', 'streaming'),
    member_id: 'mem-bruno',
    day: 9,
    amountAt: fixed(-3_490),
    spec: monthly(9),
  },
  {
    id: 'rec-disney',
    description: 'Disney+',
    account_id: 'acc-nucard',
    kind: 'expense',
    category_id: cat('assinaturas', 'streaming'),
    member_id: 'mem-ana',
    day: 21,
    amountAt: fixed(-4_390),
    spec: monthly(21),
  },
  {
    id: 'rec-icloud',
    description: 'iCloud+ 200 GB',
    account_id: 'acc-itaucard',
    kind: 'expense',
    category_id: cat('assinaturas', 'software'),
    member_id: 'mem-ana',
    day: 6,
    amountAt: fixed(-1_490),
    spec: monthly(6),
  },
  {
    id: 'rec-chatgpt',
    description: 'Assistente IA Plus',
    account_id: 'acc-wise',
    kind: 'expense',
    category_id: cat('assinaturas', 'software'),
    member_id: 'mem-bruno',
    day: 19,
    amountAt: fixed(-2_000),
    spec: monthly(19),
  },
  {
    id: 'rec-telefone',
    description: 'Claro · 2 linhas',
    account_id: 'acc-nucard',
    kind: 'expense',
    category_id: cat('assinaturas', 'telefone'),
    member_id: 'mem-bruno',
    day: 14,
    amountAt: fixed(-17_980),
    spec: monthly(14),
  },
  {
    id: 'rec-academia',
    description: 'Smart Fit · 2 planos',
    account_id: 'acc-itaucard',
    kind: 'expense',
    category_id: cat('cuidados', 'academia'),
    member_id: 'mem-ana',
    day: 3,
    amountAt: fixed(-25_980),
    spec: monthly(3),
  },
  {
    id: 'rec-ingles',
    description: 'Curso de inglês · Bruno',
    account_id: 'acc-itaucard',
    kind: 'expense',
    category_id: cat('educacao', 'cursos'),
    member_id: 'mem-bruno',
    day: 8,
    amountAt: fixed(-42_000),
    spec: monthly(8),
  },
  {
    id: 'rec-seguro-auto',
    description: 'Seguro do carro · parcela',
    account_id: 'acc-itaucard',
    kind: 'expense',
    category_id: cat('transporte', 'seguro-auto'),
    member_id: 'mem-bruno',
    day: 22,
    amountAt: fixed(-21_000),
    spec: monthly(22),
  },
  {
    id: 'rec-racao',
    description: 'Ração Premier · assinatura',
    account_id: 'acc-nucard',
    kind: 'expense',
    category_id: cat('pets', 'racao'),
    member_id: 'mem-ana',
    day: 11,
    amountAt: fixed(-23_000),
    spec: monthly(11),
  },
  {
    id: 'rec-mesada',
    description: 'Mesada',
    account_id: 'acc-carteira',
    kind: 'expense',
    category_id: cat('filhos', 'mesada'),
    member_id: 'mem-ana',
    day: 1,
    amountAt: fixed(-15_000),
    spec: monthly(1),
  },
  {
    id: 'rec-doacao',
    description: 'Doação mensal · Médicos Sem Fronteiras',
    account_id: 'acc-nucard',
    kind: 'expense',
    category_id: cat('presentes', 'doacoes'),
    member_id: 'mem-bruno',
    day: 20,
    amountAt: fixed(-5_000),
    spec: monthly(20),
  },
  {
    id: 'rec-iptu',
    description: 'IPTU · parcela',
    account_id: 'acc-itau',
    kind: 'expense',
    category_id: cat('moradia', 'iptu'),
    member_id: 'mem-ana',
    day: 10,
    amountAt: fixed(-21_000),
    months: [2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    spec: { freq: 'MONTHLY', interval: 1, by_month_day: 10 },
  },
  {
    id: 'rec-poupanca',
    description: 'Reserva · poupança',
    account_id: 'acc-itau',
    kind: 'transfer',
    category_id: null,
    member_id: null,
    to_account_id: 'acc-poupanca',
    day: 6,
    amountAt: fixed(-120_000),
    spec: monthly(6),
  },
  {
    id: 'rec-aporte',
    description: 'Aporte mensal · XP',
    account_id: 'acc-nuconta',
    kind: 'investment',
    category_id: null,
    member_id: null,
    to_account_id: 'acc-xp',
    day: 6,
    amountAt: fixed(-200_000),
    spec: monthly(6),
  },
];

// ── Geração ──────────────────────────────────────────────────────

export function buildLedger(today: string): Ledger {
  const current = monthOf(today);
  const firstMonth = shiftMonth(current, -(HISTORY_MONTHS - 1));
  const start = monthBounds(firstMonth).from;
  const horizon = addDays(today, HORIZON_DAYS);
  const txns: LedgerTxn[] = [];
  let seq = 0;

  const add = (
    date: string,
    account_id: string,
    amount: number,
    kind: TxnKind,
    category_id: string | null,
    member_id: string | null,
    description: string,
    extra: Partial<LedgerTxn> = {},
  ) => {
    if (date < start || date > horizon) return;
    const acc = baseAccount(account_id)!;
    txns.push({
      id: `txn-${String(++seq).padStart(5, '0')}`,
      date,
      account_id,
      amount,
      currency: acc.currency,
      kind,
      status: date <= today ? 'cleared' : 'planned',
      description,
      category_id,
      member_id,
      statement_id: null,
      ...extra,
    });
  };
  /** Lançamento aleatório: só até hoje (o futuro só tem previstos). */
  const addPast = (...args: Parameters<typeof add>) => {
    if (args[0] <= today) add(...args);
  };

  const months = monthsBetween(firstMonth, shiftMonth(current, 2));
  months.forEach((month, index) => {
    const { y, m } = parseIso(month);
    const day = (d: number) => clampDay(y, m, d);
    const isPastOrCurrent = month <= current;
    const r = rng(`mes:${month}`);
    const lastDay = month === current ? parseIso(today).d : 31;
    const someDay = () => day(r.int(1, Math.max(1, Math.min(lastDay, 28))));

    // Recorrências (efetivadas no passado, previstas à frente).
    for (const rec of RECURRENCES) {
      if (rec.months && !rec.months.includes(m)) continue;
      const amount = rec.amountAt(month, index);
      const date = day(rec.day);
      if (rec.to_account_id) {
        const to = baseAccount(rec.to_account_id)!;
        add(date, rec.account_id, amount, rec.kind, null, null, rec.description, {
          recurrence_id: rec.id,
        });
        add(date, to.id, -amount, rec.kind, null, null, rec.description, { recurrence_id: rec.id });
      } else {
        add(
          date,
          rec.account_id,
          amount,
          rec.kind,
          rec.category_id,
          rec.member_id,
          rec.description,
          {
            recurrence_id: rec.id,
          },
        );
      }
    }
    // Compra de ativos com o aporte do mês (a operação gera a saída do caixa da corretora).
    addPast(
      day(7),
      'acc-xp',
      -PORTFOLIO_PURCHASE,
      'investment',
      null,
      null,
      'Compra de ativos · carteira',
    );
    // Saque para o dinheiro da carteira.
    add(day(3), 'acc-itau', -40_000, 'transfer', null, null, 'Saque 24h');
    add(day(3), 'acc-carteira', 40_000, 'transfer', null, null, 'Saque 24h');
    // Repasse da conta do Bruno para a conta conjunta.
    add(day(6), 'acc-nuconta', -650_000, 'transfer', null, null, 'Repasse para conta conjunta');
    add(day(6), 'acc-itau', 650_000, 'transfer', null, null, 'Repasse para conta conjunta');

    if (!isPastOrCurrent) return;

    // Receitas variáveis.
    if (m === 11) {
      addPast(
        day(30),
        'acc-itau',
        780_000,
        'income',
        cat('salario', '13'),
        'mem-ana',
        '13º salário · 1ª parcela',
      );
      addPast(
        day(30),
        'acc-nuconta',
        550_000,
        'income',
        cat('salario', '13'),
        'mem-bruno',
        '13º salário · 1ª parcela',
      );
    }
    if (m === 12) {
      addPast(
        day(19),
        'acc-itau',
        680_000,
        'income',
        cat('salario', '13'),
        'mem-ana',
        '13º salário · 2ª parcela',
      );
      addPast(
        day(19),
        'acc-nuconta',
        480_000,
        'income',
        cat('salario', '13'),
        'mem-bruno',
        '13º salário · 2ª parcela',
      );
    }
    if (m === 2)
      addPast(
        day(25),
        'acc-itau',
        520_000,
        'income',
        cat('salario', 'ferias'),
        'mem-ana',
        'Férias · Ana',
      );
    if (m === 7)
      addPast(
        day(25),
        'acc-nuconta',
        370_000,
        'income',
        cat('salario', 'ferias'),
        'mem-bruno',
        'Férias · Bruno',
      );
    if (m === 3)
      addPast(
        day(28),
        'acc-itau',
        1_000_000,
        'income',
        cat('salario', 'plr'),
        'mem-ana',
        'PLR 2º semestre',
      );
    if (r.chance(0.42))
      addPast(
        someDay(),
        'acc-nuconta',
        r.cents(150_000, 320_000),
        'income',
        cat('freelance'),
        'mem-bruno',
        r.pick(['Projeto identidade visual', 'Freela · site institucional', 'Consultoria UX']),
      );
    addPast(
      day(1),
      'acc-poupanca',
      r.cents(11_800, 15_600),
      'income',
      cat('rendimentos', 'juros-rec'),
      'mem-ana',
      'Rendimento poupança',
    );
    if ([3, 6, 9, 12].includes(m))
      addPast(
        day(15),
        'acc-xp',
        r.cents(38_000, 62_000),
        'income',
        cat('rendimentos', 'dividendos'),
        null,
        'Dividendos · carteira de ações',
      );
    if (r.chance(0.18))
      addPast(
        someDay(),
        'acc-itau',
        r.cents(9_000, 24_000),
        'income',
        cat('reembolsos'),
        'mem-ana',
        'Reembolso Unimed',
      );

    // Moradia variável.
    const summer = [12, 1, 2, 3].includes(m);
    addPast(
      day(15),
      'acc-itau',
      -r.cents(summer ? 24_000 : 15_000, summer ? 33_000 : 22_000),
      'expense',
      cat('moradia', 'energia'),
      'mem-bruno',
      'Enel · energia',
    );
    addPast(
      day(18),
      'acc-itau',
      -r.cents(8_500, 14_000),
      'expense',
      cat('moradia', 'agua'),
      'mem-bruno',
      'Sabesp · água',
    );
    addPast(
      day(20),
      'acc-itau',
      -r.cents(6_000, 9_500),
      'expense',
      cat('moradia', 'gas'),
      'mem-bruno',
      'Comgás',
    );
    if (r.chance(0.2))
      addPast(
        someDay(),
        'acc-itaucard',
        -r.cents(18_000, 95_000),
        'expense',
        cat('moradia', 'manutencao'),
        'mem-ana',
        r.pick(['Eletricista', 'Encanador', 'Pintura da sala']),
      );

    // Alimentação.
    const cards = ['acc-nucard', 'acc-itaucard'] as const;
    const people = ['mem-ana', 'mem-bruno'] as const;
    const groceries = r.int(4, 5);
    for (let i = 0; i < groceries; i++) {
      const amount = -r.cents(26_000, 52_000);
      const who = r.pick(people);
      if (i === 0) {
        const pet = -r.cents(6_000, 12_000);
        addPast(
          someDay(),
          r.pick(cards),
          amount + pet,
          'expense',
          null,
          who,
          'Pão de Açúcar · compra do mês',
          {
            splits: [
              { category_id: cat('alimentacao', 'mercado'), member_id: who, amount },
              { category_id: cat('pets', 'racao'), member_id: who, amount: pet },
            ],
          },
        );
      } else {
        addPast(
          someDay(),
          r.pick(cards),
          amount,
          'expense',
          cat('alimentacao', 'mercado'),
          who,
          r.pick(['Carrefour', 'Pão de Açúcar', 'Hortifruti', 'Atacadão']),
        );
      }
    }
    for (let i = r.int(8, 12); i > 0; i--)
      addPast(
        someDay(),
        r.pick(['acc-carteira', 'acc-nuconta']),
        -r.cents(1_200, 4_500),
        'expense',
        cat('alimentacao', 'padaria'),
        r.pick(people),
        'Padaria Real',
      );
    for (let i = r.int(3, 5); i > 0; i--)
      addPast(
        someDay(),
        r.pick(cards),
        -r.cents(7_000, 21_000),
        'expense',
        cat('alimentacao', 'restaurantes'),
        r.pick(people),
        r.pick([
          'Restaurante Coco Bambu',
          'Outback',
          'Bar do Juarez',
          'Sushi Kenzo',
          'Cantina Roperto',
        ]),
      );
    for (let i = r.int(4, 8); i > 0; i--)
      addPast(
        someDay(),
        'acc-nucard',
        -r.cents(4_500, 11_000),
        'expense',
        cat('alimentacao', 'delivery'),
        'mem-bruno',
        r.pick(['iFood', 'Rappi', 'iFood · pizzaria']),
      );

    // Transporte.
    for (let i = r.int(3, 4); i > 0; i--)
      addPast(
        someDay(),
        'acc-itaucard',
        -r.cents(18_000, 27_000),
        'expense',
        cat('transporte', 'combustivel'),
        'mem-bruno',
        r.pick(['Posto Shell', 'Posto Ipiranga']),
      );
    for (let i = r.int(4, 10); i > 0; i--)
      addPast(
        someDay(),
        'acc-nucard',
        -r.cents(1_800, 6_000),
        'expense',
        cat('transporte', 'aplicativos'),
        r.pick(people),
        r.pick(['Uber', '99']),
      );
    for (let i = r.int(0, 3); i > 0; i--)
      addPast(
        someDay(),
        'acc-carteira',
        -r.cents(1_500, 4_000),
        'expense',
        cat('transporte', 'estacionamento'),
        'mem-bruno',
        'Estacionamento',
      );
    if (r.chance(0.15))
      addPast(
        someDay(),
        'acc-itaucard',
        -r.cents(40_000, 180_000),
        'expense',
        cat('transporte', 'veiculo'),
        'mem-bruno',
        r.pick(['Revisão 40 mil km', 'Troca de pneus', 'Oficina · freios']),
      );
    if ([1, 2, 3].includes(m))
      addPast(
        day(20),
        'acc-itau',
        -82_000,
        'expense',
        cat('transporte', 'ipva'),
        'mem-bruno',
        `IPVA · parcela ${m}/3`,
      );

    // Saúde.
    for (let i = r.int(2, 4); i > 0; i--)
      addPast(
        someDay(),
        r.pick(cards),
        -r.cents(4_000, 18_000),
        'expense',
        cat('saude', 'farmacia'),
        r.pick(people),
        r.pick(['Drogasil', 'Droga Raia', 'Pague Menos']),
      );
    if (r.chance(0.25))
      addPast(
        someDay(),
        'acc-itau',
        -35_000,
        'expense',
        cat('saude', 'consultas'),
        'mem-ana',
        'Consulta dermatologista',
      );
    if (r.chance(0.1))
      addPast(
        someDay(),
        'acc-itau',
        -r.cents(15_000, 30_000),
        'expense',
        cat('saude', 'exames'),
        'mem-bruno',
        'Laboratório Fleury',
      );
    if (r.chance(0.1))
      addPast(
        someDay(),
        'acc-itaucard',
        -r.cents(25_000, 45_000),
        'expense',
        cat('saude', 'dentista'),
        'mem-ana',
        'Dentista · limpeza',
      );

    // Educação.
    if (m === 1 || m === 2)
      addPast(
        day(12),
        'acc-itaucard',
        -r.cents(60_000, 95_000),
        'expense',
        cat('educacao', 'livros'),
        'mem-ana',
        'Material escolar',
      );

    // Lazer.
    if (m === 1)
      addPast(
        day(8),
        'acc-itaucard',
        -r.cents(380_000, 560_000),
        'expense',
        cat('lazer', 'viagens'),
        'mem-ana',
        'Pousada Trancoso · férias',
      );
    for (let i = r.int(1, 3); i > 0; i--)
      addPast(
        someDay(),
        r.pick(cards),
        -r.cents(8_000, 28_000),
        'expense',
        cat('lazer', 'passeios'),
        r.pick(people),
        r.pick([
          'Parque Ibirapuera · bicicletas',
          'Museu Catavento',
          'Sítio do Picapau',
          'Zoológico',
        ]),
      );
    for (let i = r.int(1, 2); i > 0; i--)
      addPast(
        someDay(),
        'acc-nucard',
        -r.cents(6_000, 15_000),
        'expense',
        cat('lazer', 'cinema'),
        r.pick(people),
        r.pick(['Cinemark', 'Ingresso Rápido · show']),
      );
    if (r.chance(0.3))
      addPast(
        someDay(),
        'acc-nucard',
        -r.cents(10_000, 40_000),
        'expense',
        cat('lazer', 'hobbies'),
        'mem-bruno',
        r.pick(['Loja de música', 'Kalunga · aquarela', 'Steam']),
      );

    // Compras.
    for (let i = r.int(1, 2); i > 0; i--)
      addPast(
        someDay(),
        r.pick(cards),
        -r.cents(15_000, 55_000),
        'expense',
        cat('compras', 'vestuario'),
        r.pick(people),
        r.pick(['Renner', 'Zara', 'C&A', 'Nike']),
      );
    if (r.chance(0.3))
      addPast(
        someDay(),
        'acc-itaucard',
        -r.cents(10_000, 60_000),
        'expense',
        cat('compras', 'casa'),
        'mem-ana',
        r.pick(['Tok&Stok', 'Leroy Merlin', 'Camicado']),
      );
    if (r.chance(0.08))
      addPast(
        someDay(),
        'acc-nucard',
        -r.cents(80_000, 250_000),
        'expense',
        cat('compras', 'eletronicos'),
        'mem-bruno',
        r.pick(['Fone Bluetooth', 'Monitor 27"', 'Smartwatch']),
      );

    // Cuidados pessoais.
    addPast(
      someDay(),
      'acc-itaucard',
      -r.cents(12_000, 25_000),
      'expense',
      cat('cuidados', 'cabelo'),
      'mem-ana',
      'Salão Beleza Natural',
    );
    for (let i = r.int(1, 2); i > 0; i--)
      addPast(
        someDay(),
        'acc-nucard',
        -r.cents(4_000, 15_000),
        'expense',
        cat('cuidados', 'cosmeticos'),
        r.pick(people),
        r.pick(['Sephora', 'O Boticário', 'Natura']),
      );

    // Pets, presentes, impostos, outras.
    if (r.chance(0.15))
      addPast(
        someDay(),
        'acc-itaucard',
        -r.cents(18_000, 45_000),
        'expense',
        cat('pets', 'veterinario'),
        'mem-ana',
        'Clínica veterinária',
      );
    for (let i = m === 12 ? r.int(3, 6) : r.chance(0.3) ? 1 : 0; i > 0; i--)
      addPast(
        someDay(),
        r.pick(cards),
        -r.cents(8_000, 30_000),
        'expense',
        cat('presentes', 'presentes-dados'),
        r.pick(people),
        r.pick(['Presente aniversário', 'Amazon · presente', 'Livraria Cultura']),
      );
    if (r.chance(0.05))
      addPast(
        someDay(),
        'acc-itau',
        -r.cents(1_500, 9_000),
        'expense',
        cat('impostos', 'juros'),
        null,
        'Juros cheque especial',
      );
    if (r.chance(0.3))
      addPast(
        someDay(),
        'acc-carteira',
        -r.cents(3_000, 20_000),
        'expense',
        cat('outras-despesas'),
        r.pick(people),
        r.pick(['Chaveiro', 'Cartório', 'Correios']),
      );
  });

  // Sem categoria: o mês atual tem alguns lançamentos à espera de revisão.
  const tr = rng(`sem-categoria:${current}`);
  const { y: cy, m: cm, d: cd } = parseIso(today);
  const cday = () => clampDay(cy, cm, tr.int(1, Math.max(1, cd)));
  add(
    cday(),
    'acc-nuconta',
    -tr.cents(5_000, 25_000),
    'expense',
    null,
    'mem-bruno',
    'PIX ENVIADO · JOAO P S',
  );
  add(
    cday(),
    'acc-nucard',
    -tr.cents(3_000, 18_000),
    'expense',
    null,
    'mem-bruno',
    'COMPRA *MERCPAGO 4421',
  );
  add(cday(), 'acc-itau', -tr.cents(9_000, 30_000), 'expense', null, null, 'TED 341 0001 · DOC');
  add(cday(), 'acc-itau', tr.cents(15_000, 40_000), 'income', null, null, 'PIX RECEBIDO · MARIA C');
  const prev = shiftMonth(current, -1);
  add(
    clampDay(parseIso(prev).y, parseIso(prev).m, 17),
    'acc-nucard',
    -6_790,
    'expense',
    null,
    'mem-bruno',
    'COMPRA *PAG 2231',
  );

  // Parcelamentos no cartão (parcela n na data da compra + n meses).
  const plans: {
    id: string;
    account: string;
    desc: string;
    total: number;
    n: number;
    purchase: string;
    category: string;
    member: string;
  }[] = [
    {
      id: 'plan-notebook',
      account: 'acc-itaucard',
      desc: 'Notebook Dell · Fast Shop',
      total: 450_000,
      n: 10,
      purchase: clampDay(
        parseIso(shiftMonth(current, -2)).y,
        parseIso(shiftMonth(current, -2)).m,
        14,
      ),
      category: cat('compras', 'eletronicos'),
      member: 'mem-bruno',
    },
    {
      id: 'plan-geladeira',
      account: 'acc-nucard',
      desc: 'Geladeira Brastemp · Magalu',
      total: 456_000,
      n: 12,
      purchase: clampDay(
        parseIso(shiftMonth(current, -5)).y,
        parseIso(shiftMonth(current, -5)).m,
        22,
      ),
      category: cat('compras', 'casa'),
      member: 'mem-ana',
    },
  ];
  const july = cm >= 7 ? `${cy}-07` : `${cy - 1}-07`;
  plans.push({
    id: 'plan-viagem',
    account: 'acc-nucard',
    desc: 'LATAM · passagens Lisboa',
    total: 780_000,
    n: 6,
    purchase: `${july}-04`,
    category: cat('lazer', 'viagens'),
    member: 'mem-ana',
  });
  for (const p of plans) {
    const each = Math.floor(p.total / p.n);
    for (let k = 0; k < p.n; k++) {
      const amount = k === p.n - 1 ? p.total - each * (p.n - 1) : each;
      const date = addMonthsToDate(p.purchase, k);
      if (date < start) continue;
      txns.push({
        id: `txn-${String(++seq).padStart(5, '0')}`,
        date,
        account_id: p.account,
        amount: -amount,
        currency: 'BRL',
        kind: 'expense',
        status: date <= today ? 'cleared' : 'planned',
        description: `${p.desc} · ${k + 1}/${p.n}`,
        category_id: p.category,
        member_id: p.member,
        statement_id: null,
        installment: { plan: p.id, n: k + 1, of: p.n },
      });
    }
  }

  // Faturas: cada lançamento de cartão entra na fatura da data.
  const statements = new Map<string, LedgerStatement>();
  for (const t of txns) {
    const acc = baseAccount(t.account_id);
    if (!acc?.card) continue;
    const d = statementDates(t.date, acc.card.closing_day, acc.card.due_day);
    const id = `st-${acc.id}-${d.reference_month}`;
    if (!statements.has(id)) statements.set(id, { id, account_id: acc.id, ...d });
    t.statement_id = id;
  }
  // Pagamento das faturas vencidas, no vencimento, pela conta do cartão.
  for (const st of statements.values()) {
    if (st.due_date > today) continue;
    const total = -txns.filter((t) => t.statement_id === st.id).reduce((s, t) => s + t.amount, 0);
    if (total <= 0) continue;
    const acc = baseAccount(st.account_id)!;
    const desc = `Pagamento fatura ${acc.name}`;
    add(st.due_date, acc.card!.payment, -total, 'card_payment', null, null, desc);
    add(st.due_date, acc.id, total, 'card_payment', null, null, desc, { statement_id: st.id });
  }

  txns.sort((a, b) =>
    a.date === b.date ? a.id.localeCompare(b.id) : a.date.localeCompare(b.date),
  );
  return { today, start, firstMonth, txns, statements, recurrences: RECURRENCES };
}

/** Contas do livro (para saldos). */
export const LEDGER_ACCOUNTS = ACCOUNTS;
