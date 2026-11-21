/**
 * Mock de `transactions.operation_candidates`, `link_operation` e `unlink_operation`
 * (espelho de `transactions::operations`): o lançamento vira aporte (sem categoria) ligado a
 * uma operação da carteira, às cotas capital ou a uma meta; desvincular devolve o tipo e a
 * categoria de antes.
 */
import type { OperationCandidate } from '@/api/generated/OperationCandidate';
import type { OperationLinkPreview } from '@/api/generated/OperationLinkPreview';
import type { TxnOperationKind } from '@/api/generated/TxnOperationKind';

import { registerMockHandlers } from '../mockRegistry';
import { capitalShareAccounts } from './capitalShare';
import { guarded } from './ledger/boot';
import { diffDays, nowUtc } from './ledger/dates';
import { conflict, invalid, notFound, str } from './ledger/errors';
import { goalsList } from './ledger/goalsState';
import { operationLinks } from './ledger/opLinks';
import { find, patch, rows, write } from './ledger/store';
import { fold } from './ledger/text';
import { tradeViews } from './portfolio';

const CANDIDATE_DAYS = 7;

const brDate = (d: string) => `${d.slice(8, 10)}/${d.slice(5, 7)}/${d.slice(0, 4)}`;

function tradeLabel(id: string): string | null {
  const t = tradeViews().find((x) => x.id === id);
  if (!t) return null;
  const buy = t.side === 'buy';
  return `${buy ? 'Aporte' : 'Resgate'} em ${t.ticker ?? t.asset_name} (${buy ? 'compra' : 'venda'} de ${t.quantity} em ${brDate(t.date)})`;
}

function label(kind: TxnOperationKind, id: string): string | null {
  if (kind === 'trade') return tradeLabel(id);
  if (kind === 'capital_share') {
    const s = capitalShareAccounts().find((c) => c.account_id === id);
    return s ? `Cotas capital · ${s.name}` : null;
  }
  const g = goalsList(false).find((x) => x.id === id);
  return g ? `Meta ${g.name}` : null;
}

function candidates(txnId: string): OperationCandidate[] {
  const t = find('txn', txnId) ?? notFound('lançamento');
  const desc = fold(t.description);
  const abs = Math.abs(t.amount);
  const taken = new Set(
    [...operationLinks().values()].filter((l) => l.kind === 'trade').map((l) => l.target_id),
  );
  const out: OperationCandidate[] = [];
  const side = t.amount < 0 ? 'buy' : 'sell';
  for (const tr of tradeViews()) {
    if (tr.side !== side || taken.has(tr.id) || tr.txn_id === t.id) continue;
    const gap = Math.abs(diffDays(tr.date, t.date));
    if (gap > CANDIDATE_DAYS || tr.currency !== t.currency) continue;
    const total = Math.abs(tr.cash_amount);
    if (total <= 0) continue;
    const diff = Math.abs(total - abs) / total;
    if (diff > 0.1) continue;
    const named = desc.includes(fold(tr.ticker ?? '\u0000')) || desc.includes(fold(tr.asset_name));
    out.push({
      kind: 'trade',
      target_id: tr.id,
      label: tradeLabel(tr.id) ?? '',
      reason: `${diff < 0.005 ? 'mesmo valor' : 'valor parecido'} ${gap === 0 ? 'no mesmo dia' : `a ${gap} dia(s)`}`,
      score: Math.max(5, Math.min(100, 90 - Math.round(diff * 300) - gap * 5 + (named ? 10 : 0))),
    });
  }
  for (const s of capitalShareAccounts()) {
    const byText = s.pattern
      .split('|')
      .map((x) => fold(x.trim()))
      .some((x) => x && desc.includes(x));
    if (!byText && s.account_id !== t.account_id) continue;
    out.push({
      kind: 'capital_share',
      target_id: s.account_id,
      label: `Cotas capital · ${s.name}`,
      reason: byText ? 'o texto do extrato fala em cota capital' : 'a conta tem cotas capital',
      score: byText ? 85 : 40,
    });
  }
  const programmed = ['poupanca programada', 'capitalizacao', 'aplicacao programada'].some((k) =>
    desc.includes(k),
  );
  for (const g of goalsList(false)) {
    const named = fold(g.name)
      .split(/\s+/)
      .some((w) => w.length >= 4 && desc.includes(w));
    out.push({
      kind: 'goal',
      target_id: g.id,
      label: `Meta ${g.name}`,
      reason: named
        ? 'o texto do extrato cita a meta'
        : programmed
          ? 'o texto fala em poupança programada'
          : 'meta ativa',
      score: named ? 70 : programmed ? 45 : 10,
    });
  }
  return out.sort((a, b) => b.score - a.score);
}

const KINDS: readonly TxnOperationKind[] = ['trade', 'capital_share', 'goal'];

/** Por que o lançamento não se vincula à operação (`null` = pode). */
function blocker(txnId: string, kind: TxnOperationKind, targetId: string): string | null {
  const t = find('txn', txnId) ?? notFound('lançamento');
  if (t.status === 'planned')
    return 'o lançamento ainda está previsto; efetive-o antes de vincular';
  if (t.transfer_id)
    return 'o lançamento é uma perna de transferência; desligue a transferência antes';
  if (rows('txn_split').some((s) => s.txn_id === txnId))
    return 'lançamentos divididos não se vinculam a uma operação';
  const cur = operationLinks().get(txnId);
  if (cur) return `o lançamento já está vinculado a ${cur.label}; desvincule antes`;
  if (kind === 'trade') {
    const tr = tradeViews().find((x) => x.id === targetId);
    if (!tr) return 'operação da carteira não encontrada';
    if ((tr.side === 'buy') !== t.amount < 0)
      return tr.side === 'buy'
        ? 'um aporte (compra) é pago por uma saída (valor negativo)'
        : 'um resgate (venda) chega como uma entrada (valor positivo)';
    if ([...operationLinks().values()].some((l) => l.kind === 'trade' && l.target_id === targetId))
      return 'essa operação já tem um lançamento vinculado; desvincule o outro antes';
  }
  return null;
}

function linkOne(txnId: string, kind: TxnOperationKind, targetId: string, name: string): void {
  const t = find('txn', txnId)!;
  operationLinks().set(txnId, {
    kind,
    target_id: targetId,
    label: name,
    previous_kind: t.kind,
    previous_category_id: t.category_id,
  });
  patch('txn', txnId, { kind: 'investment', category_id: null, updated_at: nowUtc() });
}

function bulkArgs(args: Record<string, unknown>) {
  const kind = str(args, 'kind') as TxnOperationKind;
  if (!KINDS.includes(kind)) invalid('tipo de operação inválido');
  const targetId = str(args, 'target_id');
  const raw = Array.isArray(args.txn_ids) ? (args.txn_ids as string[]) : [];
  const ids = [...new Set(raw)];
  if (ids.length === 0) invalid('nenhum lançamento selecionado');
  return { kind, targetId, ids };
}

function preview(args: Record<string, unknown>): OperationLinkPreview {
  const { kind, targetId, ids } = bulkArgs(args);
  const name = label(kind, targetId) ?? notFound('operação');
  let tradeTaken = false;
  const items = ids.map((id) => {
    const t = find('txn', id) ?? notFound('lançamento');
    let reason = blocker(id, kind, targetId);
    if (!reason && kind === 'trade') {
      if (tradeTaken) reason = 'a operação da carteira aceita um lançamento só';
      tradeTaken = true;
    }
    return {
      txn_id: id,
      date: t.date,
      description: t.description,
      account_name: find('account', t.account_id)?.name ?? '?',
      amount: t.amount,
      currency: t.currency,
      ok: reason == null,
      reason,
    };
  });
  const totals: OperationLinkPreview['totals'] = [];
  for (const i of items.filter((x) => x.ok)) {
    const tot = totals.find((x) => x.currency === i.currency);
    if (tot) tot.amount += i.amount;
    else totals.push({ currency: i.currency, amount: i.amount });
  }
  return { label: name, items, linkable: items.filter((i) => i.ok).length, totals };
}

function candidatesMany(ids: string[]): OperationCandidate[] {
  const unique = [...new Set(ids)];
  if (unique.length === 1) return candidates(unique[0]!);
  const merged = new Map<string, { c: OperationCandidate; n: number }>();
  for (const id of unique)
    for (const c of candidates(id)) {
      if (c.kind === 'trade') continue;
      const k = `${c.kind}:${c.target_id}`;
      const m = merged.get(k);
      if (!m) merged.set(k, { c: { ...c }, n: 1 });
      else {
        m.n += 1;
        if (c.score > m.c.score) Object.assign(m.c, { score: c.score, reason: c.reason });
      }
    }
  return [...merged.values()]
    .sort((a, b) => b.n - a.n || b.c.score - a.c.score)
    .map(({ c, n }) =>
      n < unique.length ? { ...c, reason: `${c.reason}; combina com ${n} de ${unique.length}` } : c,
    );
}

registerMockHandlers(
  guarded({
    'transactions.operation_candidates': (args) =>
      candidatesMany([
        ...(Array.isArray(args.txn_ids) ? (args.txn_ids as string[]) : []),
        ...(typeof args.txn_id === 'string' ? [args.txn_id] : []),
      ]),
    'transactions.operation_link_preview': (args) => preview(args),
    'transactions.link_operations': (args) => {
      const { kind, targetId } = bulkArgs(args);
      const p = preview(args);
      if (p.linkable === 0)
        invalid(
          `nenhum dos lançamentos pode ser vinculado a ${p.label} (${p.items[0]?.reason ?? ''})`,
        );
      const rec = write(`${p.items.length} lançamento(s) vinculado(s) a ${p.label}`, () => {
        for (const i of p.items.filter((x) => x.ok)) linkOne(i.txn_id, kind, targetId, p.label);
        return null;
      });
      return {
        label: p.label,
        linked: p.linkable,
        skipped: p.items.length - p.linkable,
        change_set_id: rec.changeSetId,
      };
    },
    'transactions.link_operation': (args) => {
      const txnId = str(args, 'txn_id');
      const kind = str(args, 'kind') as TxnOperationKind;
      if (!KINDS.includes(kind)) invalid('tipo de operação inválido');
      const targetId = str(args, 'target_id');
      const t = find('txn', txnId) ?? notFound('lançamento');
      const name = label(kind, targetId) ?? notFound('operação');
      const why = blocker(txnId, kind, targetId);
      if (why) (why.includes('já') ? conflict : invalid)(why);
      const rec = write(`Lançamento vinculado: ${t.description} → ${name}`, () => {
        linkOne(txnId, kind, targetId, name);
        return null;
      });
      return {
        operation: { kind, target_id: targetId, label: name },
        change_set_id: rec.changeSetId,
      };
    },
    'transactions.unlink_operation': (args) => {
      const txnId = str(args, 'txn_id');
      const t = find('txn', txnId) ?? notFound('lançamento');
      const link = operationLinks().get(txnId);
      if (!link) invalid('o lançamento não está vinculado a uma operação');
      const rec = write(`Vínculo desfeito: ${t.description}`, () => {
        operationLinks().delete(txnId);
        const cat = link.previous_category_id && find('category', link.previous_category_id);
        patch('txn', txnId, {
          kind: link.previous_kind,
          category_id: cat ? cat.id : null,
          updated_at: nowUtc(),
        });
        return null;
      });
      return { change_set_id: rec.changeSetId };
    },
  }),
);
