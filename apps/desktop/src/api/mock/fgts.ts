/**
 * Mock de `fgts.*` (docs/brazil/fgts.md), simplificado: lê o extrato em texto (no navegador o "PDF"
 * de teste é texto), grava os lançamentos como ajustes na conta FGTS (saque com destino vira
 * transferência), confere depósitos por competência e calcula saque-aniversário, projeção e a
 * comparação com IPCA e CDI das séries simuladas. O núcleo faz a conferência completa do JAM.
 */
import type { AccountView } from '../generated/AccountView';
import type { FgtsAccountView } from '../generated/FgtsAccountView';
import type { FgtsAnniversary } from '../generated/FgtsAnniversary';
import type { FgtsCheck } from '../generated/FgtsCheck';
import type { FgtsCompare } from '../generated/FgtsCompare';
import type { FgtsEntryKind } from '../generated/FgtsEntryKind';
import type { FgtsEntryView } from '../generated/FgtsEntryView';
import type { FgtsStatementLine } from '../generated/FgtsStatementLine';
import type { FgtsStatementPreview } from '../generated/FgtsStatementPreview';
import { mockFail, onMockReset, registerMockHandlers } from '../mockRegistry';
import { familyCountry } from './country';
import { guarded } from './ledger/boot';
import { addMonths, today } from './ledger/dates';
import { balanceAt, insertTxn } from './ledger/domain';
import { fold, invoke } from './ledger/investImport';
import { newId, rows, write } from './ledger/store';
import { seriesPoints } from './marketSim';

interface Info {
  employer_name: string | null;
  employer_cnpj: string | null;
  admission_date: string | null;
  leave_date: string | null;
  rescission_base: number | null;
  last_statement_date: string | null;
  income_source_id: string | null;
}
interface Meta {
  account_id: string;
  kind: FgtsEntryKind;
  competence: string | null;
  coefficient: string | null;
  base_year: number | null;
  withdrawal_code: string | null;
  withdrawal_reason: string | null;
  statement_total: number | null;
}

const infos = new Map<string, Info>();
const metas = new Map<string, Meta>();
const modalities = new Map<string, string>();
onMockReset(() => {
  infos.clear();
  metas.clear();
  modalities.clear();
});

const LABEL: Record<FgtsEntryKind, string> = {
  deposit: 'Depósito',
  deposit_extra: 'Depósito complementar',
  jam: 'JAM (juros e correção)',
  jam_adjustment: 'Ajuste do JAM',
  distribution: 'Distribuição de resultado',
  withdrawal: 'Saque',
  reversal: 'Reposição de saque',
};
const MONTHS = [
  'janeiro',
  'fevereiro',
  'marco',
  'abril',
  'maio',
  'junho',
  'julho',
  'agosto',
  'setembro',
  'outubro',
  'novembro',
  'dezembro',
];
const CODE_REASON: Record<string, string> = {
  '01': 'rescisao',
  '50': 'emergencial',
  '19E': 'emergencial',
  '60': 'aniversario',
  '99': 'moradia',
};

function requireBrazil() {
  if (familyCountry() !== 'BR')
    mockFail(
      'invalid',
      'Esse tipo de conta só existe no Brasil: o cofre precisa estar configurado para o Brasil.',
    );
}

const money = (s: string) => {
  const neg = s.includes('-');
  const v = Number(s.replace(/[^\d,]/g, '').replace(',', '.'));
  return Math.round((neg ? -v : v) * 100);
};
const iso = (d: string) => `${d.slice(6, 10)}-${d.slice(3, 5)}-${d.slice(0, 2)}`;

function classify(h: string, amount: number): FgtsEntryKind {
  const f = fold(h);
  if (f.includes('dist') && f.includes('resultado')) return 'distribution';
  if (f.includes('reposicao')) return 'reversal';
  if (f.includes('saque') && !f.includes('cancelamento')) return 'withdrawal';
  if (f.includes('jam') && (f.includes('regulariza') || f.includes('cancelamento')))
    return 'jam_adjustment';
  if (f.includes('jam')) return 'jam';
  if (f.includes('deposito')) return f.startsWith('650') ? 'deposit_extra' : 'deposit';
  return amount < 0 ? 'withdrawal' : 'jam_adjustment';
}

function textOf(args: Record<string, unknown>): string {
  if (typeof args.text === 'string' && args.text.trim()) return args.text;
  const f = (args.files as { content_base64: string }[] | undefined)?.[0];
  if (!f) mockFail('invalid', 'Envie o PDF do extrato do FGTS ou o texto dele.');
  return new TextDecoder().decode(Uint8Array.from(atob(f.content_base64), (c) => c.charCodeAt(0)));
}

function parse(text: string) {
  const re =
    /(\d{2}\/\d{2}\/\d{4})([^\n]*?)(-?\s*R\$\s*-?\s*[\d.]+,\d{2})\s+(-?\s*R\$\s*-?\s*[\d.]+,\d{2})/g;
  const lines: FgtsStatementLine[] = [];
  let prev = 0;
  let m: RegExpExecArray | null;
  const first = text.search(/\d{2}\/\d{2}\/\d{4}[^\n]*R\$[^\n]*R\$/);
  const head = first >= 0 ? text.slice(0, first) : text;
  while ((m = re.exec(text))) {
    const pre = text.slice(prev, m.index);
    const last = lines[lines.length - 1];
    const by = /^\s*12\/(\d{4})/.exec(pre);
    if (last && last.kind === 'distribution' && last.base_year == null && by)
      last.base_year = Number(by[1]);
    const inner = m[2]!.trim();
    const history = (
      inner ||
      pre
        .trim()
        .split('\n')
        .filter((l) => !/^\s*12\/\d{4}/.test(l))
        .pop() ||
      ''
    ).trim();
    const amount = money(m[3]!);
    const kind = classify(history, amount);
    const f = fold(history).split(/\s+/);
    let competence: string | null = null;
    const mi = f.findIndex((w) => MONTHS.includes(w));
    if (
      (kind === 'deposit' || kind === 'deposit_extra') &&
      mi >= 0 &&
      /^\d{4}$/.test(f[mi + 1] ?? '')
    )
      competence = `${f[mi + 1]}-${String(MONTHS.indexOf(f[mi]!) + 1).padStart(2, '0')}`;
    const code = /COD\.?\s*([0-9]{1,3}[A-Z]?)\b(?:\s+(\d+))?/i.exec(history);
    const date = iso(m[1]!);
    lines.push({
      seq: lines.length + 1,
      date,
      history,
      kind,
      amount,
      total: money(m[4]!),
      competence,
      coefficient:
        kind === 'jam' ? (/(\d+,\d{4,})/.exec(history)?.[1]?.replace(',', '.') ?? null) : null,
      base_year: null,
      withdrawal_code: kind === 'withdrawal' ? (code?.[1] ?? null) : null,
      withdrawal_ref: kind === 'withdrawal' ? (code?.[2] ?? null) : null,
      group: kind === 'withdrawal' ? `${date}|${code?.[1] ?? ''}|${code?.[2] ?? ''}` : null,
      exists: false,
      jam_expected: null,
    });
    prev = m.index + m[0].length;
  }
  if (!lines.length)
    mockFail(
      'invalid',
      'Não achei lançamentos do FGTS neste extrato (linhas com data, valor e total).',
    );
  const cnpj = /\d{2}\.\d{3}\.\d{3}\/\d{4}-\d{2}/.exec(head);
  let employer: string | null = null;
  let admission: string | null = null;
  if (cnpj) {
    const hl = head.split('\n');
    const i = hl.findIndex((l) => l.includes(cnpj[0]));
    employer = hl[i]!.slice(0, hl[i]!.indexOf(cnpj[0])).trim();
    const next = hl[i + 1]?.trim() ?? '';
    if (next && !/\d/.test(next) && !/empregador|conta|taxa/i.test(next))
      employer = `${employer} ${next}`;
    const d = /(\d{2}\/\d{2}\/\d{4})/.exec(
      hl
        .slice(i)
        .join('\n')
        .slice(hl[i]!.indexOf(cnpj[0]) + cnpj[0].length),
    );
    admission = d ? iso(d[1]!) : null;
  }
  const rb = /R\$\s*[\d.]+,\d{2}/.exec(head);
  const issued = /emi\w*\s+em\s*:?\s*(\d{2}\/\d{2}\/\d{4})/i.exec(text);
  return {
    lines,
    header: {
      employer_name: employer,
      employer_cnpj: cnpj?.[0] ?? null,
      admission_date: admission,
      option_date: null,
      leave_date: null,
      leave_code: null,
      category: null,
      interest_rate:
        /(\d+(?:[.,]\d+)?)\s*%/.exec(head)?.[1]?.replace(',', '.').replace(/\.0$/, '') ?? null,
      rescission_base: rb ? money(rb[0]) : null,
      statement_date: issued
        ? iso(issued[1]!)
        : lines
            .map((l) => l.date)
            .sort()
            .pop()!,
      has_account_number: /\d{3,5}\/\d{6,}/.test(head),
    },
  };
}

function fgtsAccounts(): AccountView[] {
  return invoke<AccountView[]>('accounts.list', { include_archived: true }).filter(
    (a) => a.kind === 'fgts',
  );
}

function info(id: string): Info {
  let i = infos.get(id);
  if (!i) {
    i = {
      employer_name: null,
      employer_cnpj: null,
      admission_date: null,
      leave_date: null,
      rescission_base: null,
      last_statement_date: null,
      income_source_id: null,
    };
    infos.set(id, i);
  }
  return i;
}

function view(a: AccountView): FgtsAccountView {
  const i = info(a.id);
  const member = a.member_ids.length === 1 ? a.member_ids[0]! : null;
  return {
    account_id: a.id,
    name: a.name,
    member_id: member,
    balance: a.balance,
    employer_name: i.employer_name,
    employer_cnpj: i.employer_cnpj,
    admission_date: i.admission_date,
    option_date: null,
    leave_date: i.leave_date,
    leave_code: null,
    category: null,
    interest_rate: '3',
    active: i.leave_date == null,
    rescission_base: i.rescission_base,
    rescission_base_date: i.last_statement_date,
    dismissal_fine: i.rescission_base != null ? Math.floor((i.rescission_base * 40) / 100) : null,
    last_statement_date: i.last_statement_date,
    income_source_id: i.income_source_id,
    modality: member ? (modalities.get(member) ?? null) : null,
    archived: a.archived,
  };
}

function entries(accountId: string): FgtsEntryView[] {
  const list = rows('txn')
    .filter((t) => metas.has(t.id) && metas.get(t.id)!.account_id === accountId)
    .sort((a, b) => a.date.localeCompare(b.date) || a.created_at.localeCompare(b.created_at));
  const opening = invoke<AccountView>('accounts.get', { id: accountId }).opening_balance;
  let bal = opening;
  return list.map((t) => {
    const m = metas.get(t.id)!;
    bal += t.amount;
    const other = t.transfer_id
      ? rows('txn').find((o) => o.transfer_id === t.transfer_id && o.id !== t.id)
      : undefined;
    return {
      txn_id: t.id,
      date: t.date,
      description: t.description,
      kind: m.kind,
      kind_label: LABEL[m.kind],
      amount: t.amount,
      balance: bal,
      statement_total: m.statement_total,
      competence: m.competence,
      coefficient: m.coefficient,
      base_year: m.base_year,
      withdrawal_code: m.withdrawal_code,
      withdrawal_reason: m.withdrawal_reason,
      withdrawal_ref: null,
      destination_account_id: other?.account_id ?? null,
      computed: false,
    };
  });
}

function insertEntry(
  accountId: string,
  date: string,
  amount: number,
  description: string,
  meta: Omit<Meta, 'account_id'>,
  destination?: string,
) {
  let id: string;
  if (destination && amount < 0) {
    const tid = newId();
    id = insertTxn({
      account_id: accountId,
      date,
      amount,
      currency: 'BRL',
      kind: 'transfer',
      description,
      transfer_id: tid,
    }).id;
    insertTxn({
      account_id: destination,
      date,
      amount: -amount,
      currency: 'BRL',
      kind: 'transfer',
      description,
      transfer_id: tid,
    });
  } else {
    id = insertTxn({
      account_id: accountId,
      date,
      amount,
      currency: 'BRL',
      kind: 'adjustment',
      description,
    }).id;
  }
  metas.set(id, { account_id: accountId, ...meta });
  return id;
}

function preview(args: Record<string, unknown>): FgtsStatementPreview {
  requireBrazil();
  const { lines, header } = parse(textOf(args));
  const opening = lines[0]!.total - lines[0]!.amount;
  let running = opening;
  let mismatches = 0;
  for (const l of lines) {
    running += l.amount;
    if (running !== l.total) {
      mismatches += 1;
      running = l.total;
    }
  }
  const accountId =
    (typeof args.account_id === 'string' ? args.account_id : null) ??
    fgtsAccounts().find(
      (a) => header.employer_cnpj && info(a.id).employer_cnpj === header.employer_cnpj,
    )?.id ??
    null;
  if (accountId) {
    const existing = new Set(
      entries(accountId).map((e) => `${e.date}|${e.kind}|${e.amount}|${e.statement_total}`),
    );
    for (const l of lines) l.exists = existing.has(`${l.date}|${l.kind}|${l.amount}|${l.total}`);
  }
  const groups = new Map<string, FgtsStatementPreview['withdrawals'][number]>();
  for (const l of lines.filter((x) => x.kind === 'withdrawal')) {
    const key = l.group!;
    const g = groups.get(key) ?? {
      key,
      date: l.date,
      code: l.withdrawal_code,
      reference: l.withdrawal_ref,
      amount: 0,
      reason:
        (args.reasons as Record<string, string> | undefined)?.[key] ??
        (l.withdrawal_code ? (CODE_REASON[l.withdrawal_code] ?? null) : null),
    };
    g.amount -= l.amount;
    groups.set(key, g);
  }
  const existing = lines.filter((l) => l.exists).length;
  return {
    header,
    lines,
    opening_balance: opening,
    final_balance: lines[lines.length - 1]!.total,
    balance_ok: mismatches === 0,
    balance_mismatches: mismatches,
    account_id: accountId,
    account_name: accountId ? invoke<AccountView>('accounts.get', { id: accountId }).name : null,
    new_count: lines.length - existing,
    existing_count: existing,
    jam_checked: lines.filter((l) => l.kind === 'jam').length,
    jam_mismatches: 0,
    withdrawals: [...groups.values()],
    warnings: mismatches
      ? [`${mismatches} linha(s) do extrato não batem com o saldo impresso.`]
      : [],
  };
}

function check(accountId: string): FgtsCheck {
  const list = entries(accountId);
  const byComp = new Map<string, number>();
  for (const e of list.filter((x) => x.kind === 'deposit' || x.kind === 'deposit_extra'))
    if (e.competence) byComp.set(e.competence, (byComp.get(e.competence) ?? 0) + e.amount);
  const comps = [...byComp.keys()].sort();
  const deposits: FgtsCheck['deposits'] = [];
  if (comps.length) {
    let m = `${comps[0]}-01`;
    const end = addMonths(today().slice(0, 7) + '-01', -1);
    while (m <= end) {
      const comp = m.slice(0, 7);
      const due = `${addMonths(m, 1).slice(0, 7)}-20`;
      const dep = byComp.get(comp) ?? 0;
      deposits.push({
        competence: comp,
        payslip: null,
        deposited: dep,
        due_date: due,
        status: dep ? 'ok' : today() > due ? 'missing' : 'pending',
      });
      m = addMonths(m, 1);
    }
  }
  return {
    account_id: accountId,
    jam_checked: list.filter((e) => e.kind === 'jam').length,
    jam_mismatches: [],
    deposits,
    missing: deposits.filter((d) => d.status === 'missing').map((d) => d.competence),
    notes: ['Conferência simplificada do modo de demonstração.'],
  };
}

function growth(id: string, from: string, to: string, share = 1): number | null {
  const pts = seriesPoints(id).filter((p) => p.date >= from.slice(0, 7) && p.date < to);
  if (!pts.length) return null;
  return pts.reduce((f, p) => f * (1 + (share * p.value) / 100), 1);
}

function compare(accountId: string): FgtsCompare {
  const list = entries(accountId);
  const from = list[0]?.date ?? today();
  const to = today();
  const flows = list.filter((e) =>
    ['deposit', 'deposit_extra', 'withdrawal', 'reversal'].includes(e.kind),
  );
  const final = list[list.length - 1]?.balance ?? 0;
  const bench = (id: string, label: string, series: string, share: number, source: string) => {
    let v = 0;
    let ok = true;
    for (const f of flows) {
      const g = growth(series, f.date, to, share);
      if (g == null) ok = false;
      v += f.amount * (g ?? 1);
    }
    const fv = ok && flows.length ? Math.round(v) : null;
    return { id, label, final_value: fv, difference: fv != null ? fv - final : null, source };
  };
  return {
    from,
    to,
    final_balance: final,
    irr: null,
    deposits: flows.filter((f) => f.amount > 0).reduce((s, f) => s + f.amount, 0),
    withdrawals: -flows.filter((f) => f.amount < 0).reduce((s, f) => s + f.amount, 0),
    jam: list
      .filter((e) => e.kind === 'jam' || e.kind === 'jam_adjustment')
      .reduce((s, e) => s + e.amount, 0),
    distribution: list.filter((e) => e.kind === 'distribution').reduce((s, e) => s + e.amount, 0),
    benchmarks: [
      bench('ipca', 'Corrigido pelo IPCA', 'ipca', 1, 'IBGE/BCB (SGS 433)'),
      bench('cdi_85', '85% do CDI', 'cdi', 0.85, 'B3/BCB (SGS 12)'),
      bench('cdi', '100% do CDI', 'cdi', 1, 'B3/BCB (SGS 12)'),
    ],
    years: [],
    notes: ['Comparação simplificada do modo de demonstração. Não é recomendação.'],
  };
}

const BANDS: [number | null, number, number][] = [
  [50_000, 0.5, 0],
  [100_000, 0.4, 5_000],
  [500_000, 0.3, 15_000],
  [1_000_000, 0.2, 65_000],
  [1_500_000, 0.15, 115_000],
  [2_000_000, 0.1, 190_000],
  [null, 0.05, 290_000],
];

function anniversary(memberId: string | null): FgtsAnniversary {
  requireBrazil();
  const accs = fgtsAccounts().filter(
    (a) => !a.archived && (!memberId || a.member_ids.includes(memberId)),
  );
  const total = accs.reduce((s, a) => s + Math.max(0, a.balance), 0);
  const [, rate, extra] = BANDS.find(([u]) => u == null || total <= u)!;
  const modality = memberId ? (modalities.get(memberId) ?? null) : null;
  return {
    member_id: memberId,
    total_balance: total,
    amount: total > 0 ? Math.min(total, Math.round(total * rate) + extra) : 0,
    rate,
    extra,
    window_start: null,
    window_end: null,
    modality,
    blocked_on_dismissal: total,
    fine_on_dismissal: accs.reduce((s, a) => s + (view(a).dismissal_fine ?? 0), 0),
    earliest_return:
      modality === 'aniversario' ? `${addMonths(today().slice(0, 7) + '-01', 25)}` : null,
    anticipation_balance: null,
    source: 'Lei 8.036/1990, Anexo (Lei 13.932/2019)',
    notes: ['Estimativa pela tabela do saque-aniversário.'],
  };
}

registerMockHandlers(
  guarded({
    'fgts.list': () => {
      requireBrazil();
      return fgtsAccounts().map(view);
    },
    'fgts.entries': (a) => entries(String(a.account_id)),
    'fgts.statement_preview': (a) => preview(a),
    'fgts.statement_import': (a) => {
      const p = preview(a);
      let accountId = p.account_id;
      if (!accountId)
        accountId = invoke<AccountView>('accounts.create', {
          name:
            typeof a.name === 'string' && a.name
              ? a.name
              : `FGTS · ${p.header.employer_name ?? 'empregador'}`,
          kind: 'fgts',
          opening_balance: p.opening_balance,
          opening_date: p.lines.map((l) => l.date).sort()[0],
          ...(typeof a.member_id === 'string' ? { member_ids: [a.member_id] } : {}),
        }).id;
      const id = accountId;
      const i = info(id);
      Object.assign(i, {
        employer_name: p.header.employer_name ?? i.employer_name,
        employer_cnpj: p.header.employer_cnpj ?? i.employer_cnpj,
        admission_date: p.header.admission_date ?? i.admission_date,
        rescission_base: p.header.rescission_base ?? i.rescission_base,
        last_statement_date: p.header.statement_date ?? i.last_statement_date,
      });
      const reasons = Object.fromEntries(p.withdrawals.map((g) => [g.key, g.reason]));
      const dest = (a.destinations as Record<string, string> | undefined) ?? {};
      const inserted = write('Importar extrato do FGTS', () => {
        let n = 0;
        for (const l of p.lines.filter((x) => !x.exists)) {
          insertEntry(
            id,
            l.date,
            l.amount,
            l.history,
            {
              kind: l.kind,
              competence: l.competence,
              coefficient: l.coefficient,
              base_year: l.base_year,
              withdrawal_code: l.withdrawal_code,
              withdrawal_reason: l.group ? (reasons[l.group] ?? null) : null,
              statement_total: l.total,
            },
            l.group ? dest[l.group] : undefined,
          );
          n += 1;
        }
        return n;
      }).value;
      return { account_id: id, inserted, skipped: p.existing_count, preview: p };
    },
    'fgts.check': (a) => check(String(a.account_id)),
    'fgts.yield_compare': (a) => compare(String(a.account_id)),
    'fgts.project': (a) => {
      const accs = fgtsAccounts().filter((x) => !a.account_id || x.id === a.account_id);
      const bal = accs.reduce((s, x) => s + x.balance, 0);
      const date = typeof a.date === 'string' ? a.date : addMonths(today(), 12 * 20);
      const months = Math.max(0, (Number(date.slice(0, 4)) - Number(today().slice(0, 4))) * 12);
      const nominal = Math.round(bal * Math.pow(1.0025, months));
      return {
        date,
        balance_today: bal,
        balance: nominal,
        balance_real: Math.round(bal * Math.pow(0.9992, months)),
        deposits: 0,
        jam: nominal - bal,
        distribution: 0,
        tr_monthly: 0,
        ipca_12m: 4,
        notes: ['Projeção simplificada do modo de demonstração.'],
      };
    },
    'fgts.anniversary_simulate': (a) =>
      anniversary(typeof a.member_id === 'string' ? a.member_id : null),
    'fgts.record_withdrawal': (a) => {
      requireBrazil();
      const id = String(a.account_id);
      const amount = Number(a.amount);
      const date = String(a.date);
      if (!(amount > 0)) mockFail('invalid', 'O valor precisa ser maior que zero.');
      if (amount > balanceAt(id, date, false))
        mockFail('invalid', 'O valor passa do saldo da conta FGTS nessa data.');
      const code = typeof a.code === 'string' ? a.code : null;
      const reason =
        (typeof a.reason === 'string' ? a.reason : null) ??
        (code ? (CODE_REASON[code] ?? null) : null);
      const txn = write('Registrar saque do FGTS', () =>
        insertEntry(
          id,
          date,
          -amount,
          `Saque do FGTS (${reason ?? 'outros'})`,
          {
            kind: 'withdrawal',
            competence: null,
            coefficient: null,
            base_year: null,
            withdrawal_code: code,
            withdrawal_reason: reason,
            statement_total: null,
          },
          typeof a.destination_account_id === 'string' ? a.destination_account_id : undefined,
        ),
      ).value;
      return { txn_id: txn };
    },
    'fgts.use_for_housing': (a) => {
      requireBrazil();
      const id = String(a.account_id);
      const amount = Number(a.amount);
      const txn = write('Usar o FGTS no financiamento', () =>
        insertEntry(id, String(a.date), -amount, 'Amortização com FGTS', {
          kind: 'withdrawal',
          competence: null,
          coefficient: null,
          base_year: null,
          withdrawal_code: null,
          withdrawal_reason: 'moradia',
          statement_total: null,
        }),
      ).value;
      return { txn_id: txn, warnings: [] };
    },
    'fgts.set_modality': (a) => {
      requireBrazil();
      const m = String(a.member_id);
      modalities.set(m, String(a.modality));
      return anniversary(m);
    },
    'fgts.update_account': (a) => {
      const id = String(a.account_id);
      const i = info(id);
      for (const k of [
        'employer_name',
        'employer_cnpj',
        'admission_date',
        'leave_date',
        'income_source_id',
      ] as const)
        if (k in a) i[k] = (a[k] as string | null) ?? null;
      const acc = fgtsAccounts().find((x) => x.id === id);
      return acc ? view(acc) : null;
    },
  }),
);
