/**
 * Rótulos do e-mail de ingestão (docs/email-ingest.md), no idioma do app (catálogo `email.ftl`).
 * Cada tabela lê o catálogo na hora (getter), então pode ser importada e guardada em constante.
 */
import type { MailAiMode } from '@/api/generated/MailAiMode';
import type { MailAuditEntry } from '@/api/generated/MailAuditEntry';
import type { MailRuleAction } from '@/api/generated/MailRuleAction';
import type { MailStatus } from '@/api/generated/MailStatus';
import type { RetentionMode } from '@/api/generated/RetentionMode';
import type { RetentionState } from '@/api/generated/RetentionState';
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

export const KIND_LABEL: Record<string, string> = labels({
  statement: 'email-tipo-extrato',
  card_statement: 'email-tipo-fatura-pdf',
  brokerage_note: 'email-tipo-nota',
  b3_report: 'email-tipo-b3',
  pension_statement: 'email-tipo-previdencia',
  payslip: 'email-tipo-holerite',
  receipt: 'email-tipo-cupom',
  image: 'email-tipo-imagem',
  purchase: 'email-tipo-compra',
  pix: 'email-tipo-pix',
  template: 'email-tipo-modelo',
  unknown: 'email-tipo-desconhecido',
});

export const STATUS_LABEL: Record<MailStatus, string> = labels({
  awaiting_review: 'email-status-revisar',
  imported: 'email-status-importada',
  duplicate: 'email-status-repetida',
  failed: 'email-status-falhou',
  unrecognized: 'email-status-nao-reconhecida',
  ignored: 'email-status-ignorada',
  ai_pending: 'email-status-ia-lendo',
});

export const STATUS_TONE: Record<MailStatus, string> = {
  awaiting_review: 'text-accent',
  imported: 'text-positive',
  duplicate: 'text-dim',
  failed: 'text-negative',
  unrecognized: 'text-warning',
  ignored: 'text-dim',
  ai_pending: 'text-accent',
};

export const ACTION_LABEL: Record<MailRuleAction, string> = labels({
  review: 'email-acao-revisar',
  auto: 'email-acao-auto',
  ignore: 'email-acao-ignorar',
});

export const AI_LABEL: Record<MailAiMode, string> = labels({
  never: 'email-ia-nunca',
  on_request: 'email-ia-a-pedido',
  auto: 'email-ia-auto',
});

export const RETENTION_LABEL: Record<RetentionMode, string> = labels({
  never: 'email-retencao-nunca',
  after_success: 'email-retencao-sucesso',
  after_days: 'email-retencao-dias',
  both: 'email-retencao-ambos',
});

export const STATE_LABEL: Record<RetentionState, string> = labels({
  off: 'email-estado-desligado',
  testing: 'email-estado-teste',
  awaiting_confirmation: 'email-estado-confirmacao',
  active: 'email-estado-ativo',
});

const AUDIT_LABEL: Record<string, string> = labels({
  sync: 'email-auditoria-busca',
  received: 'email-auditoria-recebida',
  imported: 'email-auditoria-importada',
  duplicate: 'email-auditoria-repetida',
  deleted: 'email-auditoria-apagada',
  would_delete: 'email-auditoria-apagaria',
  skipped_delete: 'email-auditoria-nao-apagou',
  ai: 'email-auditoria-ia',
  reviewed: 'email-auditoria-revisao',
  error: 'email-auditoria-erro',
});

export function auditLabel(e: MailAuditEntry): string {
  return AUDIT_LABEL[e.action] ?? e.action;
}

/** "a, b\nc" → ["a", "b", "c"]. */
export function splitList(s: string): string[] {
  return s
    .split(/[\n,;]+/)
    .map((x) => x.trim())
    .filter(Boolean);
}

export const joinList = (v: readonly string[] | undefined | null) => (v ?? []).join(', ');
