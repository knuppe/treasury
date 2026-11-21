/**
 * Base dos hooks TanStack Query dos módulos do núcleo.
 *
 * - Chave de consulta: `[modulo, acao, args]` (outras telas usam a mesma
 *   convenção e compartilham o cache). Sem argumentos, `args` é `{}`.
 * - Mutações invalidam os módulos afetados por prefixo (`['accounts']` pega
 *   `accounts.list`, `accounts.get`…), sempre incluindo o histórico.
 */
import {
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
  type UseQueryOptions,
} from '@tanstack/react-query';

import { call, type ApiError } from '../client';

export type CoreKey = readonly [module: string, action: string, args: unknown];

export function coreKey(module: string, action: string, args?: unknown): CoreKey {
  return [module, action, args ?? {}];
}

type QueryOpts<T> = Omit<UseQueryOptions<T, ApiError, T, CoreKey>, 'queryKey' | 'queryFn'>;

/** Consulta `modulo.acao` com `args`. */
export function useCore<T>(module: string, action: string, args?: unknown, options?: QueryOpts<T>) {
  const key = coreKey(module, action, args);
  return useQuery<T, ApiError, T, CoreKey>({
    queryKey: key,
    queryFn: () => call<T>(`${module}.${action}`, args ?? {}),
    ...options,
  });
}

/**
 * Grupos de módulos afetados por cada tipo de escrita. Um lançamento muda
 * saldos, faturas, relatórios, orçamento, metas e o histórico.
 */
export const AFFECTS = {
  ledger: [
    'transactions',
    'accounts',
    'cards',
    'recurrences',
    'rules',
    'import',
    'reports',
    'budget',
    'goals',
    'portfolio',
    'categories',
    'projects',
    'physical',
    'tax_burden',
    // Finanças por membro (household.members_overview).
    'household',
    // Previsão de saldo, quanto posso gastar hoje e assinaturas detectadas.
    'forecast',
    'subscriptions',
    'installments',
    'scenarios',
    'income',
    'fgts',
    'food',
  ],
  accounts: [
    'accounts',
    'transactions',
    'cards',
    'reports',
    'goals',
    'portfolio',
    'budget',
    'tax_burden',
    'forecast',
    // Titulares da conta mudam as visões por membro; arquivar muda cenários e assinaturas.
    'household',
    'scenarios',
    'subscriptions',
    'debt',
    'fgts',
  ],
  categories: [
    'categories',
    'transactions',
    'rules',
    'recurrences',
    'cards',
    'reports',
    'budget',
    'tax_burden',
  ],
  household: [
    'household',
    'accounts',
    'transactions',
    'reports',
    'tax_burden',
    'income',
    // Nascimento e sexo do membro mudam a linha da vida e os padrões da aposentadoria.
    'longevity',
    'goals',
  ],
  rules: ['rules', 'transactions', 'reports', 'budget'],
  fx: [
    'fx',
    'accounts',
    'reports',
    'goals',
    'portfolio',
    'cards',
    'tax_burden',
    'forecast',
    'budget',
    'household',
  ],
  settings: ['settings', 'reports', 'budget', 'tax_burden', 'mcp'],
  backup: ['backup'],
  market: [
    'market',
    'indices',
    'fx',
    'accounts',
    'reports',
    'portfolio',
    'assets',
    'physical',
    'pension',
    'tax_burden',
  ],
  /** Contratos de dívida: amortizações e pagamentos podem gerar lançamentos. */
  debt: [
    'debt',
    'fgts',
    // Fim das dívidas é marco da linha da vida.
    'longevity',
    'transactions',
    'accounts',
    'reports',
    'budget',
    'physical',
    'tax_burden',
    'forecast',
  ],
  /** Patrimônio físico: o valor dos bens entra no patrimônio líquido (e no IPVA estimado). */
  physical: ['physical', 'reports', 'projects', 'tax_burden'],
  /** Projetos: o gasto deles muda o valor das benfeitorias ligadas. */
  projects: ['projects', 'transactions', 'physical', 'reports', 'tax_burden'],
  /** DARF pago pode gerar a despesa de IR. */
  tax: ['tax', 'transactions', 'accounts', 'reports', 'budget', 'tax_burden'],
  /** Fontes externas ligadas/desligadas: as consultas que dependem delas mudam. */
  sources: ['sources', 'research'],
  /** Manutenção do banco: só os números do próprio banco mudam. */
  database: ['database'],
  /** Assinatura dispensada ou trazida de volta. */
  subscriptions: ['subscriptions'],
  /** Série de parcelas ignorada ou trazida de volta: muda a previsão e os compromissos. */
  installments: ['installments', 'forecast', 'cards'],
  /** Rendas e holerites: mudam a previsão, os impostos pagos e as visões por membro. */
  income: ['income', 'tax_burden', 'household', 'scenarios'],
  /** Cenários "e se" salvos. */
  scenarios: ['scenarios'],
  /** Preferências de notificação. */
  notify: ['notify'],
  /** Modelo da saúde financeira: a nota muda de forma. */
  health: ['health'],
  /** Estimativa de impostos: país e estado, grupos das categorias e informes de rendimentos. */
  tax_burden: ['tax_burden'],
  /** Cupons fiscais: mudam o que a família compra e o imposto da estimativa. */
  receipts: ['receipts', 'tax_burden', 'food'],
  /** Alimentação: produtos, categorias de produto e os pagamentos do cupom. */
  food: ['food', 'receipts'],
  /** E-mail de ingestão: contas, regras, a fila e a auditoria. */
  mail: ['mail'],
  /** Lembrete de apoio ao Treasury. */
  support: ['support'],
  /** Aceite dos termos de uso. */
  terms: ['terms'],
  /** Perfil (nível e tom): muda o menu, o painel e como o agente fala. O questionário da
   * família pode ligar a linha da vida. */
  profile: ['profile', 'ai', 'household', 'longevity', 'goals'],
  /** Linha da vida: o quadro da família, os padrões da aposentadoria e as ferramentas do agente. */
  longevity: ['longevity', 'goals', 'mcp'],
} as const;

export type AffectGroup = keyof typeof AFFECTS;

/**
 * Invalida os módulos de um ou mais grupos (e sempre o histórico de alterações e a fila de
 * propostas do agente).
 */
export function invalidateGroups(
  client: QueryClient,
  groups: readonly AffectGroup[],
): Promise<void> {
  const modules = new Set<string>(['changes']);
  for (const g of groups) for (const m of AFFECTS[g]) modules.add(m);
  // Toda escrita pode retirar propostas do agente que mexiam nos mesmos dados.
  const keys: string[][] = [
    ...[...modules].map((m) => [m]),
    ['ai', 'proposals'],
    ['ai', 'proposal'],
    ['ai', 'pending_count'],
    // "Para resolver" lê quase tudo (vencimentos, contas, orçamento, propostas).
    ['attention'],
  ];
  return Promise.all(keys.map((k) => client.invalidateQueries({ queryKey: k }))).then(
    () => undefined,
  );
}

/** Tudo menos o cofre (desfazer/refazer podem tocar em qualquer tabela; restauração troca o banco). */
export function invalidateAll(client: QueryClient): Promise<void> {
  return client.invalidateQueries({ predicate: (q) => q.queryKey[0] !== 'vault' });
}

export interface CoreMutationOptions<TResult, TArgs> {
  /** Grupos invalidados no sucesso; `all` invalida tudo menos o cofre. */
  affects: readonly AffectGroup[] | 'all';
  onSuccess?: (result: TResult, args: TArgs) => void;
  /** Mutações com senha: não guardam variáveis no cache. */
  sensitive?: boolean;
}

/** Mutação `modulo.acao` que invalida o cache dos módulos afetados. */
export function useCoreMutation<TArgs = unknown, TResult = unknown>(
  cmd: string,
  { affects, onSuccess, sensitive }: CoreMutationOptions<TResult, TArgs>,
) {
  const client = useQueryClient();
  return useMutation<TResult, ApiError, TArgs>({
    mutationFn: (args: TArgs) => call<TResult>(cmd, args ?? {}),
    ...(sensitive ? { gcTime: 0 } : {}),
    onSuccess: async (result, args) => {
      await (affects === 'all' ? invalidateAll(client) : invalidateGroups(client, affects));
      onSuccess?.(result, args);
    },
  });
}
