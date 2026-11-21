/**
 * Mock de `income.*` (docs/income.md): rendas por membro e a previsão dos próximos meses pelos
 * valores informados (o núcleo também usa o último holerite e calcula INSS e IRRF pelas regras
 * do ano; aqui, só o 13º e o terço de férias). Holerite em PDF só no app: a leitura usa o
 * leitor de PDF do núcleo.
 */
import type { IncomeActual } from '../generated/IncomeActual';
import type { IncomeMonth } from '../generated/IncomeMonth';
import type { IncomeSource } from '../generated/IncomeSource';
import type { IncomeSourceInput } from '../generated/IncomeSourceInput';
import type { IncomeSourceKind } from '../generated/IncomeSourceKind';
import type { IncomeSummary } from '../generated/IncomeSummary';
import type { Member } from '../generated/Member';
import type { Payslip } from '../generated/Payslip';
import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addMonths, monthOf, today } from './ledger/dates';
import { invalid, notFound } from './ledger/errors';
import { invoke } from './ledger/investImport';

const FORMAL: IncomeSourceKind[] = ['clt', 'public_servant', 'retirement', 'pension', 'pj'];
const WITHHOLDING: IncomeSourceKind[] = ['clt', 'public_servant', 'retirement', 'pension'];

let sources: IncomeSource[] = [];
let payslips: Payslip[] = [];

/** Rótulo do tipo de holerite (o mesmo texto do núcleo). */
const EXTRA_LABEL: Partial<Record<Payslip['kind'], string>> = {
  thirteenth_1: '1ª parcela do 13º',
  thirteenth_2: '2ª parcela do 13º',
  vacation: 'férias',
  plr: 'PLR',
  advance: 'adiantamento',
  other: 'outro pagamento',
};
let seq = 0;
let seeded = false;

/**
 * Simulação de demissão simplificada (o núcleo segue a CLT inteira, com INSS, IRRF e as
 * tabelas do seguro-desemprego): saldo, aviso de 30 + 3 dias por ano, 13º e férias
 * proporcionais, multa de 40% e seguro pela faixa de 2026, só para a dispensa sem justa causa.
 */
function simulateTermination(a: Record<string, unknown>) {
  const src = sources.find((s) => s.id === a.source_id);
  const salary = Number(a.gross_monthly ?? src?.gross_monthly ?? 0);
  if (!salary) invalid('informe o salário bruto ou escolha uma renda com salário bruto');
  const end = typeof a.termination_date === 'string' ? a.termination_date : '';
  const start = typeof a.admission_date === 'string' ? a.admission_date : (src?.start_date ?? '');
  if (!/^\d{4}-\d{2}-\d{2}$/.test(end) || !/^\d{4}-\d{2}-\d{2}$/.test(start))
    invalid('informe as datas de admissão e de desligamento');
  const years = Math.max(0, Number(end.slice(0, 4)) - Number(start.slice(0, 4)));
  const dismissed = a.reason === 'sem_justa_causa';
  const days = Math.min(30, Number(end.slice(8, 10)));
  const notice = dismissed ? Math.min(90, 30 + 3 * years) : 0;
  const month = Number(end.slice(5, 7));
  const line = (code: string, label: string, detail: string, amount: number) => ({
    code,
    label,
    detail,
    amount,
    taxable: false,
  });
  const lines = [
    line('saldo_salario', 'Saldo de salário', `${days} dias`, Math.round((salary * days) / 30)),
    ...(notice
      ? [
          line(
            'aviso_previo',
            'Aviso prévio indenizado',
            `${notice} dias`,
            Math.round((salary * notice) / 30),
          ),
        ]
      : []),
    line(
      'decimo_terceiro',
      '13º salário proporcional',
      `${month}/12`,
      Math.round((salary * month) / 12),
    ),
  ];
  const gross = lines.reduce((s, l) => s + l.amount, 0);
  const balance = typeof a.fgts_balance === 'number' ? a.fgts_balance : salary * 0.08 * 12 * years;
  const fine = dismissed ? Math.round(balance * 0.4) : 0;
  const installment = Math.min(251_865, Math.max(162_100, Math.round(salary * 0.8)));
  const unemployment = {
    eligible: dismissed,
    reason: dismissed ? null : 'O seguro-desemprego é só para dispensa sem justa causa.',
    average_salary: salary,
    installment: dismissed ? installment : 0,
    installments: dismissed ? 5 : 0,
    total: dismissed ? installment * 5 : 0,
    table: 'MTE, tabela de 11/01/2026 (Lei 7.998/1990; Res. CODEFAT 957/2022)',
  };
  const withdrawable = dismissed ? Math.round(balance) + fine : 0;
  const total = gross + withdrawable + unemployment.total;
  return {
    reason: a.reason,
    notice: dismissed ? 'indenizado' : null,
    notice_days: notice,
    projected_end: end,
    years_of_service: years,
    lines,
    gross,
    deductions: 0,
    net: gross,
    fgts: {
      balance: Math.round(balance),
      deposit: 0,
      fine_rate: dismissed ? 40 : 0,
      fine,
      withdrawal_allowed: dismissed,
      withdrawal_rate: dismissed ? 100 : 0,
      withdrawable,
      anniversary_blocks_balance: false,
    },
    unemployment,
    total_available: total,
    months_covered: src?.net_monthly ? Math.round((total / src.net_monthly) * 10) / 10 : null,
    warnings: ['Estimativa simplificada da demonstração.'],
    payroll_rules: 'Demonstração',
  };
}

function seed() {
  if (seeded) return;
  seeded = true;
  sources = [
    source('mem-ana', {
      member_id: 'mem-ana',
      name: 'Salário · Alfa Engenharia',
      kind: 'clt',
      employer: 'Alfa Engenharia',
      gross_monthly: 1_850_000,
      net_monthly: 1_380_000,
      has_thirteenth: true,
      vacation_month: 1,
      pay_day: 5,
    }),
    source('mem-bruno', {
      member_id: 'mem-bruno',
      name: 'Studio Norte (PJ)',
      kind: 'pj',
      gross_monthly: 1_050_000,
      net_monthly: 970_000,
      pay_day: 5,
    }),
    source('mem-bruno', {
      member_id: 'mem-bruno',
      name: 'Aulas de design',
      kind: 'informal',
      net_monthly: 80_000,
      variable: true,
    }),
  ];
  // Holerites da Ana nos últimos cinco meses: os dois mais antigos antes do aumento, para a
  // figura de renda mostrar o real ao lado da previsão pelo último holerite.
  const ana = sources[0]!;
  const start = `${monthOf(today())}-01`;
  payslips = [-5, -4, -3, -2, -1].map((k) => {
    const raise = k >= -3;
    const gross = raise ? 1_850_000 : 1_700_000;
    const net = raise ? 1_380_000 : 1_265_000;
    const inss = 95_163;
    const other = 0;
    const irrf = gross - inss - other - net;
    return {
      id: `slip-${k}`,
      source_id: ana.id,
      source_name: ana.name,
      member_name: ana.member_name,
      reference_month: monthOf(addMonths(start, k)),
      kind: 'monthly',
      pay_date: null,
      gross,
      social_security: inss,
      income_tax: irrf,
      other_deductions: other,
      net,
      fgts: Math.round(gross * 0.08),
      items: [],
      txn_id: null,
    };
  });
  sources[0] = { ...ana, payslips: payslips.length };
}

onMockReset(() => {
  sources = [];
  payslips = [];
  seeded = false;
});

function memberName(id: string): string {
  const m = invoke<Member[]>('household.list_members', { include_archived: true }).find(
    (x) => x.id === id,
  );
  if (!m) notFound(`membro ${id}`);
  return m.name;
}

function source(memberId: string, i: IncomeSourceInput, id?: string): IncomeSource {
  if ((i.gross_monthly ?? 0) < 0 || (i.net_monthly ?? 0) < 0)
    invalid('o valor não pode ser negativo');
  if (i.gross_monthly != null && i.net_monthly != null && i.net_monthly > i.gross_monthly)
    invalid('o líquido não pode ser maior que o bruto');
  if (!i.name?.trim()) invalid('informe o nome da renda');
  seq += 1;
  return {
    id: id ?? `inc-${seq}`,
    member_id: memberId,
    member_name: memberName(memberId),
    name: i.name.trim(),
    kind: i.kind,
    formal: FORMAL.includes(i.kind),
    employer: i.employer ?? null,
    employer_cnpj: i.employer_cnpj ?? null,
    gross_monthly: i.gross_monthly ?? null,
    net_monthly: i.net_monthly ?? null,
    variable: i.variable ?? i.kind === 'informal',
    pay_day: i.pay_day ?? null,
    has_thirteenth: i.has_thirteenth ?? WITHHOLDING.includes(i.kind),
    vacation_month: i.vacation_month ?? null,
    dependents: i.dependents ?? 0,
    account_id: i.account_id ?? null,
    category_id: i.category_id ?? null,
    start_date: i.start_date ?? null,
    end_date: i.end_date ?? null,
    archived: false,
    notes: i.notes ?? null,
    payslips: payslips.filter((p) => p.source_id === id).length,
    last_payslip: null,
  };
}

const empty = (month: string): IncomeMonth => ({
  month,
  gross: 0,
  social_security: 0,
  income_tax: 0,
  other_deductions: 0,
  net: 0,
  extras: [],
});

function add(a: IncomeMonth, b: IncomeMonth) {
  a.gross += b.gross;
  a.social_security += b.social_security;
  a.income_tax += b.income_tax;
  a.other_deductions += b.other_deductions;
  a.net += b.net;
  for (const e of b.extras) if (!a.extras.includes(e)) a.extras.push(e);
}

/** Um mês da previsão de uma renda pelos valores informados (13º e férias no mês certo). */
function projectMonth(s: IncomeSource, month: string): IncomeMonth {
  const gross = s.gross_monthly ?? s.net_monthly ?? 0;
  const net = s.net_monthly ?? s.gross_monthly ?? 0;
  const ded = gross - net;
  const x = empty(month);
  const mo = Number(month.slice(5, 7));
  const on =
    (!s.start_date || s.start_date.slice(0, 7) <= month) &&
    (!s.end_date || s.end_date.slice(0, 7) >= month);
  if (!on || gross <= 0) return x;
  x.gross = gross;
  x.net = net;
  x.other_deductions = ded;
  if (s.has_thirteenth && mo === 11) {
    x.gross += Math.trunc(gross / 2);
    x.net += Math.trunc(gross / 2);
    x.extras.push('1ª parcela do 13º');
  }
  if (s.has_thirteenth && mo === 12) {
    x.gross += gross - Math.trunc(gross / 2);
    x.net += net - Math.trunc(gross / 2);
    x.other_deductions += ded;
    x.extras.push('2ª parcela do 13º');
  }
  if (s.vacation_month === mo) {
    x.gross += Math.trunc(gross / 3);
    x.net += Math.trunc(net / 3);
    x.other_deductions += Math.trunc(gross / 3) - Math.trunc(net / 3);
    x.extras.push('terço de férias');
  }
  return x;
}

function summary(a: Record<string, unknown>): IncomeSummary {
  seed();
  const months = Math.min(Math.max(Number(a.months ?? 12), 1), 24);
  const first = `${monthOf(today())}-01`;
  const monthly = Array.from({ length: months }, (_, k) => empty(monthOf(addMonths(first, k + 1))));
  const members = invoke<Member[]>('household.list_members', {});
  const famNext = empty('next');
  const out = members.map((m) => {
    const mine = sources.filter((s) => s.member_id === m.id && !s.archived);
    const next = empty('next');
    const projection = mine.map((s) => {
      const gross = s.gross_monthly ?? s.net_monthly ?? 0;
      const ms = monthly.map((mm) => projectMonth(s, mm.month));
      ms.forEach((x, i) => {
        add(next, x);
        add(monthly[i]!, x);
      });
      return {
        source_id: s.id,
        name: s.name,
        kind: s.kind,
        basis: gross > 0 ? 'declared' : 'none',
        months: ms,
      };
    });
    add(famNext, next);
    return {
      member_id: m.id,
      member_name: m.name,
      sources: mine,
      projection,
      next,
      last_12m: empty('12m'),
      effective_tax_rate: null,
    };
  });
  // Meses passados com holerite, com a previsão de hoje para o mesmo mês.
  const since = monthOf(addMonths(first, -11));
  const until = monthOf(first);
  const byMonth = new Map<string, IncomeActual>();
  for (const p of payslips) {
    if (p.reference_month < since || p.reference_month > until) continue;
    // PLR e outros avulsos ficam fora do "real × previsto" (como no núcleo).
    if (p.kind === 'plr' || p.kind === 'other') continue;
    const src = sources.find((x) => x.id === p.source_id);
    const e = byMonth.get(p.reference_month) ?? {
      real: empty(p.reference_month),
      expected: empty(p.reference_month),
    };
    add(e.real, { ...p, month: p.reference_month, extras: [] });
    if (src) add(e.expected, projectMonth(src, p.reference_month));
    byMonth.set(p.reference_month, e);
  }
  const actual = [...byMonth.values()].sort((x, y) => x.real.month.localeCompare(y.real.month));
  const base = payslips
    .filter((p) => p.kind === 'monthly')
    .map((p) => p.reference_month)
    .sort()
    .at(-1);
  return {
    from: monthOf(addMonths(first, 1)),
    months,
    currency: 'BRL',
    members: out,
    monthly,
    next: famNext,
    last_12m: empty('12m'),
    actual,
    payslips: payslips
      .filter((p) => p.reference_month >= since && p.reference_month <= until)
      .sort((x, y) => y.reference_month.localeCompare(x.reference_month))
      .map((p) => ({
        month: p.reference_month,
        kind: p.kind,
        label: EXTRA_LABEL[p.kind] ?? 'mensal',
        member_name: p.member_name,
        source_name: p.source_name,
        gross: p.gross,
        net: p.net,
        extra: p.kind === 'plr' || p.kind === 'other',
      })),
    forecast_base_month: base ?? null,
    // Como no núcleo: o mês corrente sem holerite entra como previsão.
    current_month: (() => {
      if (byMonth.has(until)) return null;
      const cur = empty(until);
      for (const s of sources.filter((x) => !x.archived)) add(cur, projectMonth(s, until));
      return cur.gross > 0 ? cur : null;
    })(),
    notes: [
      'Modo de demonstração: a previsão usa os valores informados (no app, também o último holerite e o INSS e o IRRF pelas regras do ano).',
    ],
  };
}

registerMockHandlers(
  guarded({
    'income.list_sources': (a) => {
      seed();
      return sources.filter(
        (s) =>
          (typeof a.member_id !== 'string' || s.member_id === a.member_id) &&
          (a.include_archived === true || !s.archived),
      );
    },
    'income.summary': (a) => summary(a),
    'income.simulate_termination': (a) => simulateTermination(a),
    'income.create_source': (a) => {
      seed();
      const i = a as unknown as IncomeSourceInput;
      const s = source(i.member_id, i);
      sources = [...sources, s];
      return s;
    },
    'income.update_source': (a) => {
      seed();
      const id = String(a.id);
      const cur = sources.find((s) => s.id === id);
      if (!cur) notFound(`renda ${id}`);
      const i = a.source as IncomeSourceInput;
      const next = { ...source(i.member_id, i, id), archived: a.archived === true };
      sources = sources.map((s) => (s.id === id ? next : s));
      return next;
    },
    'income.delete_source': (a) => {
      seed();
      sources = sources.filter((s) => s.id !== a.id);
      payslips = payslips.filter((p) => p.source_id !== a.id);
      return { deleted: a.id };
    },
    'income.list_payslips': () => payslips,
    'income.payslip_preview': () =>
      invalid('a leitura de holerite em PDF funciona no app instalado (modo de demonstração)'),
    'income.payslip_import': () =>
      invalid('a leitura de holerite em PDF funciona no app instalado (modo de demonstração)'),
    'income.delete_payslip': (a) => {
      payslips = payslips.filter((p) => p.id !== a.id);
      return { deleted: a.id };
    },
  }),
);
