/**
 * Apoio dos mocks de importação de investimentos (`b3.*` e `notes.*`): chamam os
 * mocks da carteira e dos ativos pelo registro (sem importar o estado deles) e
 * gravam cada importação no histórico como um conjunto de alterações externo, com
 * desfazer/refazer de verdade (apaga ou recria o que a importação gravou).
 */
import type { AccountView } from '@/api/generated/AccountView';
import type { PositionView } from '@/api/generated/PositionView';

import { extraMockHandler, mockFail } from '../../mockRegistry';
import { author, recordExternal } from './store';

/** Chama outro comando do mock (síncrono). */
export function invoke<T>(cmd: string, args: Record<string, unknown> = {}): T {
  const handler = extraMockHandler(cmd);
  if (!handler) mockFail('unknown_command', `Comando desconhecido: ${cmd}.`);
  return handler(args) as T;
}

/** Sem acentos e em minúsculas (como `import::text::fold` do núcleo). */
export function fold(s: string): string {
  return s.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase().trim();
}

export function brokerageAccounts(): AccountView[] {
  return invoke<AccountView[]>('accounts.list').filter(
    (a) => !a.archived && (a.kind === 'brokerage' || a.kind === 'foreign'),
  );
}

/** Accounts that can hold assets (the same list as the core's B3 import and the dialog). */
export function custodyAccounts(): AccountView[] {
  const kinds: AccountView['kind'][] = [
    'brokerage',
    'foreign',
    'checking',
    'savings',
    'retirement',
    'other',
  ];
  return invoke<AccountView[]>('accounts.list').filter(
    (a) => !a.archived && kinds.includes(a.kind),
  );
}

/** Quantidade na conta na data, pelo motor da carteira do mock. */
export function heldQuantity(assetId: string, accountId: string, asOf: string): number {
  try {
    const p = invoke<PositionView | undefined>('portfolio.position', {
      asset_id: assetId,
      account_id: accountId,
      as_of: asOf,
    });
    return p ? Number(p.quantity) : 0;
  } catch {
    return 0;
  }
}

export const cents = (v: number) => Math.round(v * 100);

/** O que uma importação gravou (para desfazer). */
export interface WriteLog {
  assets: string[];
  trades: string[];
  incomes: string[];
  actions: string[];
  /** Accounts created by the import (B3: "create a new account" on the accounts step). */
  accounts: string[];
  /** Operações que ganharam custos: os campos de antes. */
  updates: { id: string; before: Record<string, unknown> }[];
}

export const emptyLog = (): WriteLog => ({
  assets: [],
  trades: [],
  incomes: [],
  actions: [],
  accounts: [],
  updates: [],
});

function undoLog(log: WriteLog): void {
  for (const id of [...log.trades].reverse()) invoke('portfolio.delete_trade', { id });
  for (const id of [...log.incomes].reverse()) invoke('portfolio.delete_income', { id });
  for (const id of [...log.actions].reverse()) invoke('portfolio.delete_action', { id });
  for (const u of [...log.updates].reverse())
    invoke('portfolio.update_trade', { id: u.id, ...u.before });
  for (const id of [...log.assets].reverse()) invoke('assets.delete', { id });
  for (const id of [...log.accounts].reverse()) invoke('accounts.delete', { id });
}

function summaryOf(log: WriteLog): Record<string, Record<string, number>> {
  const insert: Record<string, number> = {};
  if (log.assets.length) insert.asset = log.assets.length;
  if (log.trades.length) insert.trade = log.trades.length;
  if (log.incomes.length) insert.income_event = log.incomes.length;
  if (log.actions.length) insert.corporate_action = log.actions.length;
  if (log.accounts.length) insert.account = log.accounts.length;
  return {
    ...(Object.keys(insert).length ? { insert } : {}),
    ...(log.updates.length ? { update: { trade: log.updates.length } } : {}),
  };
}

const isEmpty = (log: WriteLog) =>
  log.assets.length +
    log.trades.length +
    log.incomes.length +
    log.actions.length +
    log.accounts.length +
    log.updates.length ===
  0;

/**
 * Roda a gravação; se ela lançar, desfaz o que já foi gravado (transação). Se gravou
 * algo, registra no histórico com autor "importação" e devolve o id do conjunto.
 */
export function recordImport(
  reason: string,
  detail: string,
  run: (log: WriteLog) => void,
): { changeSetId: string | null; log: WriteLog } {
  const attempt = (): WriteLog => {
    const log = emptyLog();
    try {
      run(log);
    } catch (e) {
      undoLog(log);
      throw e;
    }
    return log;
  };
  let log = attempt();
  if (isEmpty(log)) return { changeSetId: null, log };
  const first = log;
  const changeSetId = recordExternal(reason, author('import', detail), {
    summary: summaryOf(log),
    undo: () => undoLog(log),
    redo: () => {
      log = attempt();
    },
  });
  return { changeSetId, log: first };
}
