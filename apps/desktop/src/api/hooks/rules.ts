/** Hooks de `rules.*`: regras de categorização, teste, aplicação e sugestão. */
import type { LearnedRule } from '../generated/LearnedRule';
import type { Rule } from '../generated/Rule';
import type { RuleApplyArgs } from '../generated/RuleApplyArgs';
import type { RuleApplyResult } from '../generated/RuleApplyResult';
import type { RuleInput } from '../generated/RuleInput';
import type { RuleTestArgs } from '../generated/RuleTestArgs';
import type { RuleUpdate } from '../generated/RuleUpdate';
import type { TxnView } from '../generated/TxnView';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const useRules = () => useCore<Rule[]>('rules', 'list');

export const useCreateRule = () =>
  useCoreMutation<RuleInput, Rule>('rules.create', { affects: ['rules'] });

export const useUpdateRule = () =>
  useCoreMutation<RuleUpdate, Rule>('rules.update', { affects: ['rules'] });

export const useDeleteRule = () =>
  useCoreMutation<{ id: string }, null>('rules.delete', { affects: ['rules'] });

/** Aplica (ou simula com `dry_run`). A simulação não grava, mas invalidar é barato. */
export const useApplyRules = () =>
  useCoreMutation<RuleApplyArgs, RuleApplyResult>('rules.apply', { affects: ['ledger'] });

/** Simula uma regra ainda não salva contra os lançamentos existentes. */
export const testRule = (args: RuleTestArgs) => call<TxnView[]>('rules.test', args);

/** Regra sugerida a partir de um lançamento ("descrição contém …" + a categoria dele). */
export const suggestRule = (txnId: string) =>
  call<RuleInput>('rules.suggest_from_txn', { txn_id: txnId });

/** Regras que o app pode oferecer: a mesma categoria três vezes no mesmo estabelecimento. */
export const useLearnedRules = () => useCore<LearnedRule[]>('rules', 'learned');

/** Não oferecer mais a regra para este trecho. */
export const useIgnoreLearnedRule = () =>
  useCoreMutation<{ key: string }, null>('rules.learned_ignore', { affects: ['rules'] });
