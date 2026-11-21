/**
 * Mock de `pension.*` (previdência VGBL/PGBL), com as mesmas regras do núcleo: aportes pelas
 * operações do ativo (os resgates consomem os mais antigos), valor pela última cota, IR do
 * regressivo pela idade de cada aporte (35% até 2 anos … 10% depois de 10) ou 15% na fonte
 * no progressivo; VGBL tributa só o rendimento, PGBL o valor todo.
 */
import type { PensionLot } from '../generated/PensionLot';
import type { PensionPlan } from '../generated/PensionPlan';
import type { PensionPlanInput } from '../generated/PensionPlanInput';
import type { PensionStatementPreview } from '../generated/PensionStatementPreview';
import type { PensionStatementRow } from '../generated/PensionStatementRow';
import type { PensionView } from '../generated/PensionView';
import type { PricePoint } from '../generated/PricePoint';
import type { Trade } from '../generated/Trade';
import type { WithdrawalSim } from '../generated/WithdrawalSim';
import type { Asset } from '../generated/Asset';
import { mockFail, onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addDays, addMonths, today } from './ledger/dates';
import { invoke } from './ledger/investImport';

const plans = new Map<string, PensionPlan>();
onMockReset(() => plans.clear());

interface Lot {
  date: string;
  qty: number;
  unit: number;
}

export function regressiveRate(date: string, at: string): [number, [number, string] | null] {
  const rates = [35, 30, 25, 20, 15];
  for (let k = 0; k < rates.length; k += 1) {
    const until = addMonths(date, 24 * (k + 1));
    // Lei 11.053/2004 art. 1º: "inferior ou igual" a 2, 4… anos; a faixa seguinte começa no dia depois.
    if (at <= until) return [rates[k]!, [rates[k]! - 5, addDays(until, 1)]];
  }
  return [10, null];
}

function lotsOf(assetId: string, at: string): { lots: Lot[]; last: number | null } {
  const trades = invoke<Trade[]>('portfolio.list_trades', { asset_id: assetId, limit: 5000 })
    .filter((t) => t.date <= at)
    .sort((a, b) => a.date.localeCompare(b.date) || a.side.localeCompare(b.side));
  const lots: Lot[] = [];
  let last: number | null = null;
  for (const t of trades) {
    const qty = Number(t.quantity);
    const price = Number(t.price);
    last = price;
    if (t.side === 'buy') lots.push({ date: t.date, qty, unit: price + t.fees / 100 / qty });
    else {
      let left = qty;
      for (const l of lots) {
        const take = Math.min(l.qty, left);
        l.qty -= take;
        left -= take;
      }
    }
  }
  return { lots: lots.filter((l) => l.qty > 1e-9), last };
}

function priceOf(assetId: string, at: string): { price: number; date: string } | null {
  const pts = invoke<PricePoint[]>('market.prices', {
    asset_id: assetId,
    from: '1990-01-01',
    to: at,
  });
  const p = pts.at(-1);
  return p ? { price: Number(p.close), date: p.date } : null;
}

const cents = (v: number) => Math.round(v * 100);

function simulate(
  lots: Lot[],
  price: number,
  plan: PensionPlan | undefined,
  amount: number | null,
  at: string,
): WithdrawalSim {
  const pgbl = plan?.plan_type === 'pgbl';
  const progressive = plan?.tax_regime === 'progressive';
  let left = amount == null ? null : amount / 100;
  let gross = 0;
  let base = 0;
  let tax = 0;
  for (const l of lots) {
    const value = l.qty * price;
    if (value <= 0) continue;
    if (left != null && left <= 0) break;
    const part = left == null ? value : Math.min(left, value);
    const frac = part / value;
    const gain = Math.max(0, (price - l.unit) * l.qty * frac);
    const b = pgbl ? part : gain;
    const rate = progressive ? 15 : regressiveRate(l.date, at)[0];
    gross += part;
    base += b;
    tax += (b * rate) / 100;
    if (left != null) left -= part;
  }
  const g = cents(gross);
  const t = cents(tax);
  return {
    gross: g,
    tax_base: cents(base),
    tax: t,
    net: g - t,
    effective_rate_pct: String(g ? Math.round((t * 10000) / g) / 100 : 0),
    note: `${pgbl ? 'PGBL' : 'VGBL'} ${progressive ? 'progressivo: 15% retidos na fonte' : 'regressivo: IR definitivo pela idade de cada aporte'}.`,
  };
}

function view(assetId: string): PensionView {
  const asset = invoke<Asset>('assets.get', { id: assetId });
  const at = today();
  const plan = plans.get(assetId);
  const { lots, last } = lotsOf(assetId, at);
  const p = priceOf(assetId, at);
  const price = p?.price ?? last ?? 0;
  const warnings: string[] = [];
  if (!plan)
    warnings.push(
      'informe o tipo do plano (VGBL ou PGBL) e o regime de tributação para o IR sair certo; por enquanto o cálculo supõe VGBL regressivo',
    );
  if (!p && lots.length)
    warnings.push(
      'sem cota recente: o valor de hoje usa a cota do último aporte. Informe o CNPJ do fundo (coleta da CVM) ou a cota atual',
    );
  const soon = addMonths(at, 3);
  let changing = 0;
  const out: PensionLot[] = lots.map((l) => {
    const value = cents(l.qty * price);
    const cost = cents(l.qty * l.unit);
    const [rate, next] = regressiveRate(l.date, at);
    if (next && next[1] <= soon) changing += value;
    return {
      date: l.date,
      quantity: String(l.qty),
      cost,
      value,
      gain: value - cost,
      regressive_rate_pct: rate,
      next_rate_pct: next?.[0] ?? null,
      next_rate_date: next?.[1] ?? null,
    };
  });
  const contributed = out.reduce((a, l) => a + l.cost, 0);
  const value = out.reduce((a, l) => a + l.value, 0);
  return {
    asset_id: assetId,
    asset_name: asset.name,
    plan: plan ?? null,
    lots: out,
    contributed,
    value,
    gain: value - contributed,
    price: price ? String(price) : null,
    price_date: p?.date ?? null,
    full_withdrawal: simulate(lots, price, plan, null, at),
    changing_band_soon: plan?.tax_regime === 'progressive' ? 0 : changing,
    warnings,
    accounts: planAccounts(assetId),
    pending_payments: 0,
    pending_amount: 0,
    pending_since: null,
    payments: [],
    payment_issues: [],
  };
}

/** Contas que guardam as cotas do plano (as das operações do ativo). */
function planAccounts(assetId: string): PensionView['accounts'] {
  const trades = invoke<Trade[]>('portfolio.list_trades', { asset_id: assetId, limit: 5000 });
  const seen = new Map<string, string>();
  for (const t of trades) {
    if (!seen.has(t.account_id)) seen.set(t.account_id, t.account_name);
  }
  return [...seen].map(([id, name]) => ({ id, name }));
}

// Extrato de previdência: o mock não lê PDF; qualquer arquivo vira este extrato de exemplo
// (fundo de pensão, layout da Quanta), com as mesmas regras de dedupe e gravação do núcleo.
const SAMPLE_ROWS: Omit<PensionStatementRow, 'index' | 'status'>[] = [
  { competence: '2022-03', due_date: '2022-03-15', paid_date: '2022-03-15', kind: 'participant', description: 'Contribuição Mensal para Aposentadoria', paid: 18000, retirement: 18000, current_value: 22880, risk: 0, quotas: '140.12345678', quota_price: '1.2845814979', external_id: 'prev:2022-03:2022-03-15:p:18000:140.12345678' },
  { competence: '2022-03', due_date: '2022-03-15', paid_date: '2022-03-15', kind: 'risk_disability', description: 'Contribuição risco invalidez', paid: 1400, retirement: 0, current_value: 0, risk: 1400, quotas: '0', quota_price: null, external_id: 'prev:2022-03:2022-03-15:i:1400:0' },
  { competence: '2022-07', due_date: '2022-08-12', paid_date: '2022-08-12', kind: 'employer', description: 'Contribuição extra do empregador', paid: 21600, retirement: 21600, current_value: 26208, risk: 0, quotas: '160.50432109', quota_price: '1.3457581611', external_id: 'prev:2022-07:2022-08-12:e:21600:160.50432109' },
]; // prettier-ignore

function statementPreview(args: Record<string, unknown>): PensionStatementPreview {
  const hasInput =
    (Array.isArray(args.files) && args.files.length > 0) ||
    (typeof args.text === 'string' && args.text.trim() !== '');
  if (!hasInput) mockFail('invalid', 'envie o PDF do extrato ou o texto dele');
  const assetId = typeof args.asset_id === 'string' ? args.asset_id : null;
  // O mock não guarda o id externo da operação: a mesma data e as mesmas cotas bastam.
  const key = (date: string, qty: string | number) => `${date}|${Number(qty)}`;
  const seen = new Set(
    assetId
      ? invoke<Trade[]>('portfolio.list_trades', { asset_id: assetId }).map((t) =>
          key(t.date, t.quantity),
        )
      : [],
  );
  const rows = SAMPLE_ROWS.map((r, index) => ({
    ...r,
    index,
    status:
      r.kind !== 'participant' && r.kind !== 'employer' && !args.risk_account_id
        ? ('info' as const)
        : seen.has(key(r.paid_date, r.quotas))
          ? ('duplicate' as const)
          : ('new' as const),
  }));
  return {
    header: {
      plan_name: 'PLANOPREV-Arrojado',
      plan_cnpj: '11.444.777/0001-61',
      cnpb: '2010.0042-19',
      administrator: null,
      adhesion: '2022-03-14',
      admin_fee_pct: '0.25',
      loading_fee_pct: '0.00',
      tax_regime: 'regressive',
    },
    rows,
    totals: {
      participant: 18000,
      employer: 21600,
      risk: 1400,
      quotas: '300.63',
      quota_balance: '300.63',
      quota_value: '1.63284017',
      quota_date: today(),
      balance: 49088,
      computed_balance: 49088,
    },
    asset_id: assetId,
    asset_name: 'PLANOPREV-Arrojado',
    warnings: [],
    layout: {
      id: 'pension.quanta.v1-colado',
      subject: 'pension_statement',
      format: 'pdf',
      issuer: 'Quanta Previdência (e fundos com o mesmo extrato)',
      version: 'v1 (2023), texto colado',
      description: 'O mesmo extrato como o leitor de PDF do app entrega',
    },
  };
}

registerMockHandlers(
  guarded({
    'pension.statement_preview': (args) => statementPreview(args),
    'pension.statement_import': (args) => {
      const account = typeof args.account_id === 'string' ? args.account_id : null;
      if (!account) mockFail('invalid', 'escolha a conta das cotas da previdência');
      const p = statementPreview(args);
      let assetId = p.asset_id;
      const created = !assetId;
      if (!assetId)
        assetId = invoke<Asset>('assets.create', {
          name: p.asset_name,
          class: 'pension',
          cnpj: p.header.plan_cnpj,
          price_provider: 'manual',
        }).id;
      invoke('pension.set_plan', {
        asset_id: assetId,
        plan_type: plans.get(assetId)?.plan_type ?? 'efpc',
        tax_regime: p.header.tax_regime ?? 'regressive',
        admin_fee_pct: p.header.admin_fee_pct,
        loading_fee_pct: p.header.loading_fee_pct,
        start_date: p.header.adhesion,
      });
      let contributions = 0;
      let skipped = 0;
      for (const r of p.rows) {
        if (r.status === 'duplicate') skipped += 1;
        if (r.status !== 'new' || !r.quota_price) continue;
        invoke('portfolio.create_trade', {
          asset_id: assetId,
          account_id: account,
          date: r.paid_date,
          side: 'buy',
          quantity: r.quotas,
          price: r.quota_price,
          cash_movement: false,
          external_id: r.external_id,
        });
        contributions += 1;
      }
      invoke('market.set_price', {
        asset_id: assetId,
        date: p.totals.quota_date,
        close: p.totals.quota_value,
      });
      return {
        asset_id: assetId,
        created_asset: created,
        contributions,
        skipped,
        risk_expenses: 0,
        prices: contributions + 1,
      };
    },
    // Como `pension::set_balance`: o saldo vira a cota do dia (saldo ÷ cotas).
    'pension.set_balance': (args) => {
      const assetId = String(args.asset_id);
      const date = String(args.date);
      const balance = Number(args.balance);
      if (!(balance > 0)) mockFail('invalid', 'o saldo precisa ser maior que zero');
      const qty = lotsOf(assetId, date).lots.reduce((acc, l) => acc + l.qty, 0);
      if (qty <= 0)
        mockFail(
          'invalid',
          'o plano não tem cotas na data: registre os aportes antes de informar o saldo',
        );
      const price = String(Number((balance / 100 / qty).toFixed(10)));
      invoke('market.set_price', { asset_id: assetId, date, close: price });
      return { price };
    },
    'pension.get': (args) => view(String(args.asset_id)),
    'pension.set_plan': (args) => {
      const i = args as unknown as PensionPlanInput;
      const a = invoke<Asset>('assets.get', { id: i.asset_id });
      if (a.class !== 'pension')
        mockFail('invalid', 'o plano de previdência é para ativos da classe Previdência');
      plans.set(i.asset_id, {
        asset_id: i.asset_id,
        plan_type: i.plan_type,
        tax_regime: i.tax_regime,
        insurer: i.insurer?.trim() || null,
        admin_fee_pct: i.admin_fee_pct?.trim().replace(',', '.') || null,
        loading_fee_pct: i.loading_fee_pct?.trim().replace(',', '.') || null,
        start_date: i.start_date ?? null,
        notes: i.notes?.trim() || null,
      });
      return view(i.asset_id);
    },
    'pension.simulate': (args) => {
      const id = String(args.asset_id);
      const at = typeof args.date === 'string' ? args.date : today();
      const amount = typeof args.amount === 'number' ? args.amount : null;
      if (amount != null && amount <= 0)
        mockFail('invalid', 'informe um valor de resgate positivo');
      const { lots, last } = lotsOf(id, at);
      const price = priceOf(id, at)?.price ?? last ?? 0;
      const total = cents(lots.reduce((a, l) => a + l.qty * price, 0));
      if (amount != null && amount > total)
        mockFail('invalid', 'o resgate passa do saldo do plano');
      return simulate(lots, price, plans.get(id), amount, at);
    },
  }),
);
