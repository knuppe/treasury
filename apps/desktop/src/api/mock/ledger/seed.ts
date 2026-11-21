/**
 * Semente do banco único do mock: a família fictícia de `fixture.ts` (titulares,
 * contas, categorias com ids `mem-…`, `acc-…`, `cat-…`) e os 24 meses de lançamentos
 * determinísticos de `buildLedger` (ids `txn-…`, faturas `st-…`, recorrências
 * `rec-…`, parcelamentos `plan-…`), mais as contas de corretora da carteira.
 *
 * Tudo entra por `write`, agrupado como aconteceria de verdade (criação do cofre,
 * contas, importações mensais de extrato e fatura, recorrências, pagamentos), então o
 * histórico de alterações já começa realista e desfazível.
 */
import type { AccountKind } from '@/api/generated/AccountKind';
import type { RuleCondition } from '@/api/generated/RuleCondition';

import { mockHomeCountry } from '../../mockRegistry';
import { COUNTRIES } from '../countryList';
import { addDays, addMonths, firstOfMonth, nowUtc, parseYmd, today } from './dates';
import { usdRate } from './engine';
import {
  BASE_ACCOUNTS as ACCOUNTS,
  BASE_CATEGORIES as CATEGORIES,
  BASE_MEMBERS as MEMBERS,
  cat,
} from './fixture';
import { buildLedger, HISTORY_MONTHS, HORIZON_DAYS, RECURRENCES, type LedgerTxn } from './ledger';
import { normalize, toRrule } from './rrule';
import { type TxnRow, author, db, freshDb, newId, patch, put, replaceDb, write } from './store';
import { fingerprint } from './text';

export const SEED_HOUSEHOLD = 'Família Exemplo';

/** Contas de corretora da carteira (`mock/portfolio.ts`), sem saldo em caixa. */
const BROKERS: { id: string; name: string; currency: string; institution: string }[] = [
  { id: 'acc-inter', name: 'Inter Invest', currency: 'BRL', institution: 'Banco Inter' },
  { id: 'acc-avenue', name: 'Avenue (EUA)', currency: 'USD', institution: 'Avenue' },
  { id: 'acc-binance', name: 'Binance', currency: 'USD', institution: 'Binance' },
];

const instId = (name: string) => `inst-${name.toLowerCase().replace(/\W+/g, '-')}`;

function stamp(date: string, hour: number, minute = 0): string {
  const { y, m, d } = parseYmd(date);
  return new Date(Date.UTC(y, m - 1, d, hour + 3, minute)).toISOString();
}

/** Escreve com data retroativa (nunca no futuro). */
function writeAt<T>(at: string, reason: string, fn: () => T, who = author('user')): string | null {
  const rec = write(reason, fn, who);
  if (rec.changeSetId) {
    const set = db.changeSets.find((c) => c.id === rec.changeSetId)!;
    const when = new Date(Math.min(Date.parse(at), Date.now() - 5 * 60_000)).toISOString();
    set.created_at = when;
    set.applied_at = when;
  }
  return rec.changeSetId;
}

const TRANSFER_LIKE = new Set(['transfer', 'card_payment', 'investment']);

/** As pernas de transferência são geradas em sequência: casa as duplas e dá um `transfer_id`. */
function pairTransfers(txns: readonly LedgerTxn[]): Map<string, string> {
  const byId = [...txns].sort((a, b) => a.id.localeCompare(b.id));
  const out = new Map<string, string>();
  for (let i = 0; i < byId.length - 1; i++) {
    const a = byId[i]!;
    const b = byId[i + 1]!;
    if (out.has(a.id) || !TRANSFER_LIKE.has(a.kind) || a.kind !== b.kind) continue;
    if (a.date !== b.date || a.account_id === b.account_id) continue;
    if (a.currency === b.currency && a.amount !== -b.amount) continue;
    const id = `tr-${a.id}`;
    out.set(a.id, id);
    out.set(b.id, id);
    i++;
  }
  return out;
}

interface Group {
  key: string;
  at: string;
  reason: string;
  who: ReturnType<typeof author>;
  txns: LedgerTxn[];
  batch?: { filename: string; format: 'ofx' | 'csv'; account: string };
}

/** País da família de exemplo e a moeda dele (Brasil e reais, a menos que o teste peça outro). */
function homeOf(): { country: string; currency: string } {
  const country = mockHomeCountry();
  return { country, currency: COUNTRIES.find((c) => c[0] === country)?.[2] ?? 'BRL' };
}

export function seedLedger(): void {
  replaceDb(freshDb());
  const home = homeOf();
  // A família fictícia é brasileira: noutro país, o que era em reais nasce na moeda de lá.
  const local = (currency: string) => (currency === 'BRL' ? home.currency : currency);
  const T = today();
  const L = buildLedger(T);
  db.meta = { start: L.start, firstMonth: L.firstMonth };
  const opening = L.start;
  const created = stamp(addDays(opening, -3), 20, 14);

  // ── Família e categorias ────────────────────────────────────────
  writeAt(
    created,
    'Criação do cofre',
    () => {
      put('household', {
        id: 'hh-1',
        name: SEED_HOUSEHOLD,
        base_currency: home.currency,
        ...(home.country === 'BR' ? {} : { country: home.country }),
        created_at: created,
        updated_at: created,
      });
      for (const m of MEMBERS) put('member', { ...m, created_at: created, updated_at: created });
      for (const c of CATEGORIES) {
        const { path: _path, ...row } = c;
        put('category', { ...row, created_at: created, updated_at: created });
      }
    },
    author('system', 'criação do cofre'),
  );

  // ── Instituições e contas ───────────────────────────────────────
  const day0 = addDays(opening, -2);
  const institutions = [
    ...new Set([...ACCOUNTS.map((a) => a.institution), ...BROKERS.map((b) => b.institution)]),
  ].filter((n): n is string => Boolean(n));
  institutions.forEach((name, i) => {
    const at = stamp(day0, 9, i);
    writeAt(at, `Instituição criada: ${name}`, () => {
      put('institution', {
        id: instId(name),
        name,
        kind: /XP|Invest|Avenue|Binance/.test(name) ? 'broker' : 'bank',
        code: null,
        website: null,
        created_at: at,
        updated_at: at,
      });
    });
  });
  const accounts: {
    id: string;
    name: string;
    kind: AccountKind;
    currency: string;
    opening_balance: number;
    member_ids: string[];
    institution: string | null;
    include_in_net_worth: boolean;
    card?: (typeof ACCOUNTS)[number]['card'];
  }[] = [
    ...ACCOUNTS,
    ...BROKERS.map((b) => ({
      ...b,
      kind: 'brokerage' as const,
      opening_balance: 0,
      member_ids: ['mem-ana'],
      include_in_net_worth: true,
    })),
  ];
  accounts.forEach((a, i) => {
    const at = stamp(day0, 10, i);
    writeAt(at, `Conta criada: ${a.name}`, () => {
      put('account', {
        id: a.id,
        name: a.name,
        kind: a.kind,
        currency: local(a.currency),
        institution_id: a.institution ? instId(a.institution) : null,
        opening_balance: a.opening_balance,
        opening_date: opening,
        include_in_net_worth: a.include_in_net_worth,
        archived: false,
        color: null,
        notes: null,
        sort_order: i,
        created_at: at,
        updated_at: at,
      });
      for (const m of a.member_ids) {
        put('account_member', { id: `${a.id}|${m}`, account_id: a.id, member_id: m });
      }
      if (a.card) {
        put('credit_card', {
          id: a.id,
          credit_limit: a.card.credit_limit,
          closing_day: a.card.closing_day,
          due_day: a.card.due_day,
          payment_account_id: a.card.payment,
          network: a.id === 'acc-nucard' ? 'Mastercard' : 'Visa',
          last4: a.id === 'acc-nucard' ? '4421' : '9087',
          created_at: at,
          updated_at: at,
        });
        // O limite do cadastro é o primeiro registro do histórico (como a V156).
        if (a.card.credit_limit != null)
          put('card_limit_change', {
            id: a.id,
            account_id: a.id,
            date: opening,
            credit_limit: a.card.credit_limit,
            source: 'manual',
            created_at: at,
            updated_at: at,
          });
        // O final do cadastro é o cartão do titular (como a V155 faz nos cofres antigos).
        put('card_plastic', {
          id: a.id,
          account_id: a.id,
          kind: 'holder',
          last4: a.id === 'acc-nucard' ? '4421' : '9087',
          member_id: null,
          label: null,
          spend_limit: null,
          active: true,
          created_at: at,
          updated_at: at,
        });
      }
    });
  });

  // ── Câmbio USD/BRL (PTAX simulada; EUR fica sem câmbio) ─────────
  writeAt(
    stamp(T, 7),
    'Coleta de câmbio (PTAX)',
    () => {
      const dates = new Set<string>();
      for (let i = 0; i < HISTORY_MONTHS; i++) dates.add(addMonths(firstOfMonth(L.start), i));
      for (let i = 1; i <= 7; i++) dates.add(addDays(T, -i));
      for (const date of [...dates].filter((d) => d < T).sort()) {
        const at = stamp(date, 13);
        put('fx_rate', {
          id: `fx-usd-${date}`,
          base: 'USD',
          quote: 'BRL',
          date,
          rate: usdRate(date).toFixed(4),
          source: 'bcb_ptax_venda',
          fetched_at: at,
          created_at: at,
          updated_at: at,
        });
      }
    },
    author('system', 'coleta de dados'),
  );

  // ── Recorrências ────────────────────────────────────────────────
  const recAt = stamp(addDays(opening, -1), 19);
  writeAt(recAt, 'Recorrências cadastradas', () => {
    for (const r of RECURRENCES) {
      const spec = normalize(r.spec, opening);
      put('recurrence', {
        id: r.id,
        description: r.description,
        account_id: r.account_id,
        amount: r.amountAt(T.slice(0, 7), HISTORY_MONTHS - 1),
        currency: local(ACCOUNTS.find((a) => a.id === r.account_id)?.currency ?? 'BRL'),
        kind: r.kind,
        category_id: r.category_id,
        member_id: r.member_id,
        to_account_id: r.to_account_id ?? null,
        rrule: toRrule(spec),
        start_date: opening,
        end_date: null,
        generated_until: addDays(T, HORIZON_DAYS),
        active: true,
        created_at: recAt,
        updated_at: recAt,
      });
    }
  });

  // ── Regras de categorização ─────────────────────────────────────
  const rule = (
    name: string,
    priority: number,
    value: string,
    op: 'contains' | 'regex',
    category: string,
    active = true,
  ) => {
    const at = stamp(addDays(opening, 2), 21, priority % 60);
    writeAt(at, `Regra criada: ${name}`, () => {
      const conditions: RuleCondition[] = [{ field: 'description', op, value }];
      put('rule', {
        id: newId(),
        name,
        priority,
        active,
        match_all: true,
        conditions,
        actions: { category_id: category },
        created_at: at,
        updated_at: at,
      });
    });
  };
  rule('Corridas de aplicativo', 10, '^(uber|99)', 'regex', cat('transporte', 'aplicativos'));
  rule('Descrição contém “IFOOD”', 20, 'ifood', 'contains', cat('alimentacao', 'delivery'));
  rule('Postos de combustível', 30, '^posto', 'regex', cat('transporte', 'combustivel'));
  rule('Farmácias', 40, 'drogasil|droga raia|pague menos', 'regex', cat('saude', 'farmacia'));
  rule('Padaria do bairro', 50, 'padaria', 'contains', cat('alimentacao', 'padaria'), false);

  // ── Lançamentos, agrupados como foram gravados ──────────────────
  const transfers = pairTransfers(L.txns);
  const reconciledBefore = firstOfMonth(addMonths(firstOfMonth(T), -1));
  const accName = (id: string) => accounts.find((a) => a.id === id)?.name ?? id;
  const toRow = (t: LedgerTxn, at: string, extra: Partial<TxnRow>): TxnRow => ({
    id: t.id,
    account_id: t.account_id,
    date: t.date,
    amount: t.amount,
    currency: local(t.currency),
    kind: t.kind,
    status:
      t.status === 'cleared' &&
      t.date < reconciledBefore &&
      (t.account_id === 'acc-itau' || t.account_id === 'acc-nuconta')
        ? 'reconciled'
        : t.status,
    description: t.description,
    payee: null,
    notes: null,
    category_id: t.category_id,
    member_id: t.member_id,
    transfer_id: transfers.get(t.id) ?? null,
    statement_id: t.statement_id,
    installment_plan_id: t.installment?.plan ?? null,
    installment_number: t.installment?.n ?? null,
    recurrence_id: t.recurrence_id ?? null,
    import_id: null,
    external_id: null,
    fingerprint: fingerprint(t.date, t.amount, t.description),
    created_at: at,
    updated_at: at,
    ...extra,
  });
  const statements = L.statements;

  const groups = new Map<string, Group>();
  const plans = new Map<string, LedgerTxn[]>();
  for (const t of L.txns) {
    if (t.installment) {
      const list = plans.get(t.installment.plan) ?? [];
      list.push(t);
      plans.set(t.installment.plan, list);
      continue;
    }
    const month = t.date.slice(0, 7);
    let key: string;
    let reason: string;
    let who = author('user');
    let batch: Group['batch'];
    if (t.date > T) {
      key = 'futuro';
      reason = 'Lançamentos previstos de recorrências';
      who = author('system', 'rotina');
    } else if (t.recurrence_id) {
      key = `rec:${month}`;
      reason = 'Lançamentos de recorrências efetivados';
      who = author('system', 'rotina');
    } else if (t.kind === 'card_payment') {
      const pair = transfers.get(t.id) ?? t.id;
      key = `pay:${pair}`;
      const card = t.statement_id ? statements.get(t.statement_id)?.account_id : undefined;
      reason = `Pagamento da fatura ${accName(card ?? t.account_id)} com vencimento em ${t.date.split('-').reverse().join('/')}`;
    } else if (transfers.has(t.id)) {
      key = `tr:${transfers.get(t.id)}`;
      reason = 'Transferência entre contas';
    } else if (t.account_id === 'acc-nucard' || t.account_id === 'acc-itaucard') {
      const file = `${t.account_id === 'acc-nucard' ? 'nubank-cartao' : 'itau-cartao'}-${month}.csv`;
      key = `imp:${file}`;
      reason = `Importação de ${file}`;
      who = author('import', file);
      batch = { filename: file, format: 'csv', account: t.account_id };
    } else if (t.account_id === 'acc-itau' || t.account_id === 'acc-nuconta') {
      const file = `extrato-${t.account_id === 'acc-itau' ? 'itau' : 'nubank'}-${month}.ofx`;
      key = `imp:${file}`;
      reason = `Importação de ${file}`;
      who = author('import', file);
      batch = { filename: file, format: 'ofx', account: t.account_id };
    } else {
      key = `man:${t.account_id}:${month}`;
      reason = `Lançamentos de ${accName(t.account_id)} em ${month.slice(5)}/${month.slice(0, 4)}`;
    }
    const g = groups.get(key) ?? { key, at: '', reason, who, txns: [], batch };
    g.txns.push(t);
    const last = g.txns.reduce((m, x) => (x.date > m ? x.date : m), '');
    g.at = key === 'futuro' ? stamp(T, 6) : stamp(last, key.startsWith('imp:') ? 22 : 12, 10);
    groups.set(key, g);
  }
  for (const [planId, list] of plans) {
    const first = [...list].sort((a, b) => a.installment!.n - b.installment!.n)[0]!;
    const purchase = addMonths(first.date, -(first.installment!.n - 1));
    const desc = first.description.replace(/ · \d+\/\d+$/, '');
    groups.set(`plan:${planId}`, {
      key: `plan:${planId}`,
      at: stamp(purchase <= T ? purchase : T, 20, 40),
      reason: `Compra parcelada em ${first.installment!.of}x: ${desc}`,
      who: author('user'),
      txns: list,
    });
  }
  const ordered = [...groups.values()].sort(
    (a, b) => a.at.localeCompare(b.at) || a.key.localeCompare(b.key),
  );
  for (const g of ordered) {
    writeAt(
      g.at,
      g.reason,
      () => {
        let batchId: string | null = null;
        if (g.batch) {
          batchId = `imp-${g.batch.filename.replace(/\W+/g, '-')}`;
          put('import_batch', {
            id: batchId,
            filename: g.batch.filename,
            format: g.batch.format,
            file_sha256: newId().replace(/-/g, ''),
            account_id: g.batch.account,
            total_rows: g.txns.length,
            inserted: g.txns.length,
            duplicates: 0,
            errors: [],
            created_at: g.at,
            updated_at: g.at,
          });
        }
        if (g.key.startsWith('plan:')) {
          const first = [...g.txns].sort((a, b) => a.installment!.n - b.installment!.n)[0]!;
          put('installment_plan', {
            id: first.installment!.plan,
            account_id: first.account_id,
            description: first.description.replace(/ · \d+\/\d+$/, ''),
            total_amount: g.txns.reduce((s, t) => s - t.amount, 0),
            currency: local(first.currency),
            installments: first.installment!.of,
            purchase_date: addMonths(first.date, -(first.installment!.n - 1)),
            category_id: first.category_id,
            member_id: first.member_id,
            created_at: g.at,
            updated_at: g.at,
          });
        }
        for (const t of g.txns) {
          const st = t.statement_id ? statements.get(t.statement_id) : undefined;
          if (st && !db.t.card_statement.has(st.id)) {
            put('card_statement', {
              ...st,
              informed_total: null,
              created_at: g.at,
              updated_at: g.at,
            });
          }
          put(
            'txn',
            toRow(t, g.at, {
              ...(batchId ? { import_id: batchId } : {}),
              ...(g.batch?.format === 'ofx' ? { external_id: `FIT${t.id.slice(4)}` } : {}),
            }),
          );
          t.splits?.forEach((s, i) => {
            put('txn_split', {
              id: `${t.id}-p${i + 1}`,
              txn_id: t.id,
              category_id: s.category_id,
              member_id: s.member_id,
              amount: s.amount,
              notes: null,
              seq: i,
              created_at: g.at,
              updated_at: g.at,
            });
          });
        }
      },
      g.who,
    );
  }

  // ── Toques finais: tags, total informado da fatura e uma ação da IA ──
  writeAt(stamp(addDays(T, -20), 20), 'Tags nos lançamentos da viagem', () => {
    const now = nowUtc();
    put('tag', { id: 'tag-viagem', name: 'viagem', color: null, created_at: now, updated_at: now });
    for (const t of db.t.txn.values()) {
      if (t.installment_plan_id === 'plan-viagem') {
        put('txn_tag', { id: `${t.id}|tag-viagem`, txn_id: t.id, tag_id: 'tag-viagem' });
      }
    }
  });
  const pharmacy = [...db.t.txn.values()]
    .filter((t) => /Drogasil|Droga Raia|Pague Menos/.test(t.description) && t.date <= T)
    .sort((a, b) => b.date.localeCompare(a.date))[0];
  if (pharmacy) {
    writeAt(stamp(pharmacy.date, 21), `Lançamento alterado: ${pharmacy.description}`, () => {
      const now = nowUtc();
      put('tag', {
        id: 'tag-reembolsavel',
        name: 'reembolsável',
        color: null,
        created_at: now,
        updated_at: now,
      });
      put('txn_tag', {
        id: `${pharmacy.id}|tag-reembolsavel`,
        txn_id: pharmacy.id,
        tag_id: 'tag-reembolsavel',
      });
      patch('txn', pharmacy.id, { notes: 'Pedir reembolso ao plano', updated_at: now });
    });
  }
  const lastClosed = [...db.t.card_statement.values()]
    .filter((s) => s.account_id === 'acc-nucard' && s.closing_date < T)
    .sort((a, b) => b.closing_date.localeCompare(a.closing_date))[0];
  if (lastClosed) {
    const total = [...db.t.txn.values()]
      .filter((t) => t.statement_id === lastClosed.id && t.kind !== 'card_payment')
      .reduce((s, t) => s - t.amount, 0);
    writeAt(stamp(addDays(lastClosed.closing_date, 1), 8), 'Total informado da fatura', () => {
      patch('card_statement', lastClosed.id, {
        informed_total: total + 1_290,
        updated_at: nowUtc(),
      });
    });
  }
}
