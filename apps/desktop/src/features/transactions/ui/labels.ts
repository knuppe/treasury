/**
 * Rótulos dos enums do núcleo usados em várias telas (02, 03, 04, 11…), no idioma do app
 * (catálogo `controles.ftl`). Cada tabela lê o catálogo na hora (getter), então pode ser
 * importada e guardada em constante sem congelar o idioma.
 */
import type { AccountKind } from '@/api/generated/AccountKind';
import type { Author } from '@/api/generated/Author';
import type { AuthorKind } from '@/api/generated/AuthorKind';
import type { BackupKind } from '@/api/generated/BackupKind';
import type { InstitutionKind } from '@/api/generated/InstitutionKind';
import type { RowStatus } from '@/api/generated/RowStatus';
import type { RuleField } from '@/api/generated/RuleField';
import type { RuleOp } from '@/api/generated/RuleOp';
import type { StatementStatus } from '@/api/generated/StatementStatus';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import { t } from '@/i18n';
import type { MessageKey, Messages } from '@/i18n/keys';

/** Chaves sem argumentos (as dos rótulos). */
type PlainKey = { [K in MessageKey]: [Messages[K]] extends [never] ? K : never }[MessageKey];

/** Tabela de rótulos que lê o catálogo a cada acesso (enumerável, como um objeto comum). */
function labels<V extends string>(keys: Record<V, PlainKey>): Record<V, string> {
  const out = {} as Record<V, string>;
  for (const [value, key] of Object.entries(keys) as [V, PlainKey][]) {
    Object.defineProperty(out, value, { enumerable: true, get: () => t(key) });
  }
  return out;
}

export const ACCOUNT_KIND: Record<AccountKind, string> = labels({
  checking: 'controles-conta-corrente',
  savings: 'controles-conta-poupanca',
  cash: 'controles-conta-dinheiro',
  credit_card: 'controles-conta-cartao',
  brokerage: 'controles-conta-corretora',
  foreign: 'controles-conta-exterior',
  benefit: 'controles-conta-beneficio',
  fgts: 'controles-conta-fgts',
  retirement: 'controles-conta-aposentadoria',
  other: 'controles-conta-outra',
});

const GROUP_TITLE: Record<AccountKind, string> = labels({
  checking: 'controles-grupo-correntes',
  savings: 'controles-grupo-poupanca',
  cash: 'controles-grupo-dinheiro',
  credit_card: 'controles-grupo-cartoes',
  brokerage: 'controles-grupo-corretoras',
  foreign: 'controles-grupo-exterior',
  benefit: 'controles-grupo-beneficio',
  fgts: 'controles-grupo-fgts',
  retirement: 'controles-grupo-aposentadoria',
  other: 'controles-grupo-outras',
});

/** Grupos da tela de contas, na ordem de exibição. */
export const ACCOUNT_GROUPS: { kind: AccountKind; readonly title: string }[] = (
  [
    'checking',
    'savings',
    'cash',
    'credit_card',
    'brokerage',
    'foreign',
    'benefit',
    'fgts',
    'retirement',
    'other',
  ] as const
).map((kind) => ({
  kind,
  get title() {
    return GROUP_TITLE[kind];
  },
}));

export const INSTITUTION_KIND: Record<InstitutionKind, string> = labels({
  bank: 'controles-instituicao-banco',
  broker: 'controles-instituicao-corretora',
  issuer: 'controles-instituicao-emissor',
  other: 'controles-instituicao-outra',
});

export const TXN_KIND: Record<TxnKind, string> = labels({
  income: 'controles-tipo-receita',
  expense: 'controles-tipo-despesa',
  transfer: 'controles-tipo-transferencia',
  card_payment: 'controles-tipo-pagamento-fatura',
  investment: 'controles-tipo-aporte',
  adjustment: 'controles-tipo-ajuste',
});

export const TXN_STATUS: Record<TxnStatus, string> = labels({
  planned: 'controles-status-previsto',
  cleared: 'controles-status-efetivado',
  reconciled: 'controles-status-conciliado',
});

export const STATEMENT_STATUS: Record<StatementStatus, string> = labels({
  open: 'controles-fatura-aberta',
  closed: 'controles-fatura-fechada',
  paid: 'controles-fatura-paga',
  partial: 'controles-fatura-parcial',
});

export const ROW_STATUS: Record<RowStatus, string> = labels({
  new: 'controles-linha-nova',
  duplicate: 'controles-linha-duplicada',
  possible_duplicate: 'controles-linha-possivel-duplicada',
  error: 'controles-linha-erro',
  ignored: 'controles-linha-ignorada',
});

export const BACKUP_KIND: Record<BackupKind, string> = labels({
  auto_daily: 'controles-backup-diario',
  manual: 'controles-backup-manual',
  pre_migration: 'controles-backup-migracao',
  pre_import: 'controles-backup-importacao',
  pre_ai: 'controles-backup-ia',
  pre_restore: 'controles-backup-restauracao',
  pre_rekey: 'controles-backup-senha',
  on_close: 'controles-backup-fechar',
});

export const AUTHOR_KIND: Record<AuthorKind, string> = labels({
  user: 'controles-autor-voce',
  import: 'controles-autor-importacao',
  ai: 'controles-autor-ia',
  system: 'controles-autor-sistema',
});

export function authorLabel(a: Author): string {
  return a.detail ? `${AUTHOR_KIND[a.kind]} · ${a.detail}` : AUTHOR_KIND[a.kind];
}

export const RULE_FIELD: Record<RuleField, string> = labels({
  description: 'controles-campo-descricao',
  payee: 'controles-campo-favorecido',
  notes: 'controles-campo-notas',
  amount: 'controles-campo-valor',
  account_id: 'controles-campo-conta',
  kind: 'controles-campo-tipo',
});

export const RULE_OP: Record<RuleOp, string> = labels({
  contains: 'controles-op-contem',
  equals: 'controles-op-igual',
  starts_with: 'controles-op-comeca',
  ends_with: 'controles-op-termina',
  regex: 'controles-op-regex',
  between: 'controles-op-entre',
  gte: 'controles-op-maior-igual',
  lte: 'controles-op-menor-igual',
});

/** Nomes das tabelas do núcleo no histórico de alterações. */
export const TABLE_LABEL: Record<string, string> = labels({
  household: 'controles-tabela-familia',
  member: 'controles-tabela-membro',
  institution: 'controles-tabela-instituicao',
  account: 'controles-tabela-conta',
  account_member: 'controles-tabela-membro-conta',
  credit_card: 'controles-tabela-cartao',
  card_statement: 'controles-tabela-fatura',
  category: 'controles-tabela-categoria',
  tag: 'controles-tabela-tag',
  txn: 'controles-tabela-lancamento',
  txn_split: 'controles-tabela-parte-divisao',
  txn_tag: 'controles-tabela-tag-lancamento',
  attachment: 'controles-tabela-anexo',
  installment_plan: 'controles-tabela-parcelamento',
  recurrence: 'controles-tabela-recorrencia',
  rule: 'controles-tabela-regra',
  import_batch: 'controles-tabela-lote-importacao',
  fx_rate: 'controles-tabela-cambio',
  budget: 'controles-tabela-orcamento',
  budget_line: 'controles-tabela-linha-orcamento',
  goal: 'controles-tabela-meta',
  goal_account: 'controles-tabela-conta-meta',
  project: 'controles-tabela-projeto',
  asset: 'controles-tabela-ativo',
  trade: 'controles-tabela-operacao',
  screen: 'controles-tabela-tela-screener',
});

const TABLE_COUNT = {
  household: 'controles-n-familia',
  member: 'controles-n-membro',
  institution: 'controles-n-instituicao',
  account: 'controles-n-conta',
  account_member: 'controles-n-membro-conta',
  credit_card: 'controles-n-cartao',
  card_statement: 'controles-n-fatura',
  category: 'controles-n-categoria',
  tag: 'controles-n-tag',
  txn: 'controles-n-lancamento',
  txn_split: 'controles-n-parte-divisao',
  txn_tag: 'controles-n-tag-lancamento',
  attachment: 'controles-n-anexo',
  installment_plan: 'controles-n-parcelamento',
  recurrence: 'controles-n-recorrencia',
  rule: 'controles-n-regra',
  import_batch: 'controles-n-lote-importacao',
  fx_rate: 'controles-n-cambio',
  budget: 'controles-n-orcamento',
  budget_line: 'controles-n-linha-orcamento',
  goal: 'controles-n-meta',
  goal_account: 'controles-n-conta-meta',
  project: 'controles-n-projeto',
  asset: 'controles-n-ativo',
  trade: 'controles-n-operacao',
  screen: 'controles-n-tela-screener',
} as const;

/** "3 contas", "1 lançamento": a quantidade com o nome da tabela no plural do idioma. */
export function tableCount(table: string, n: number): string {
  const key = (TABLE_COUNT as Record<string, (typeof TABLE_COUNT)[keyof typeof TABLE_COUNT]>)[
    table
  ];
  return key ? t(key, { n }) : `${n} ${table}`;
}
