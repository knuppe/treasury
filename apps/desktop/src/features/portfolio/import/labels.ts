/** Rótulos das importações de investimentos (B3 e notas de corretagem), pelo catálogo `importacao-b3`. */
import type { B3IgnoreRule } from '@/api/generated/B3IgnoreRule';
import type { B3PositionStatus } from '@/api/generated/B3PositionStatus';
import type { B3ReportKind } from '@/api/generated/B3ReportKind';
import type { B3RowStatus } from '@/api/generated/B3RowStatus';
import type { B3TradeOrigin } from '@/api/generated/B3TradeOrigin';
import type { NoteTradeStatus } from '@/api/generated/NoteTradeStatus';
import { t } from '@/i18n';
import type { MessageKey, Messages } from '@/i18n/keys';

import type { Tone } from './shared';

/** Chaves sem argumentos. */
type Key = { [K in MessageKey]: [Messages[K]] extends [never] ? K : never }[MessageKey];

/** Objeto cujos textos vêm do catálogo na hora da leitura (o app remonta ao trocar o idioma). */
function labels<K extends string>(keys: Record<K, Key>): Record<K, string> {
  const out = {} as Record<K, string>;
  for (const k of Object.keys(keys) as K[]) {
    Object.defineProperty(out, k, { enumerable: true, get: () => t(keys[k]) });
  }
  return out;
}

interface StatusDef {
  label: string;
  tone: Tone;
  hint: string;
}

function status(label: Key, tone: Tone, hint: Key): StatusDef {
  return {
    tone,
    get label() {
      return t(label);
    },
    get hint() {
      return t(hint);
    },
  };
}

export const B3_KIND_LABEL: Record<B3ReportKind, string> = labels({
  trades: 'b3-relatorio-negociacao',
  movements: 'b3-relatorio-movimentacao',
  position: 'b3-relatorio-posicao',
});

export const B3_STATUS: Record<B3RowStatus, StatusDef> = {
  new: status('b3-status-nova', 'positive', 'b3-status-nova-dica'),
  duplicate: status('b3-status-duplicada', 'dim', 'b3-status-duplicada-dica'),
  possible_duplicate: status(
    'b3-status-possivel-duplicada',
    'warning',
    'b3-status-possivel-duplicada-dica',
  ),
  no_account: status('b3-status-sem-conta', 'dim', 'b3-status-sem-conta-dica'),
  review: status('b3-status-revisar', 'accent', 'b3-status-revisar-dica'),
  error: status('b3-status-erro', 'negative', 'b3-status-erro-dica'),
};

export const B3_STATUS_ORDER: readonly B3RowStatus[] = [
  'new',
  'review',
  'possible_duplicate',
  'duplicate',
  'no_account',
  'error',
];

export const B3_ORIGIN_LABEL: Record<B3TradeOrigin, string> = labels({
  trades: 'b3-relatorio-negociacao',
  settlement: 'b3-origem-liquidacao',
  movement: 'b3-relatorio-movimentacao',
  fraction_auction: 'b3-origem-leilao',
  subscription: 'b3-origem-subscricao',
  custody_update: 'b3-origem-atualizacao',
  opening: 'b3-origem-abertura',
});

export const B3_RULE_LABEL: Record<B3IgnoreRule, string> = labels({
  option: 'b3-regra-opcao',
  forward: 'b3-regra-termo',
  future: 'b3-regra-futuro',
  unsupported_market: 'b3-regra-mercado',
  lending: 'b3-regra-aluguel',
  custody_transfer: 'b3-regra-custodia',
  settlement_duplicate: 'b3-regra-liquidacao',
  subscription_right: 'b3-regra-subscricao',
  balance_update: 'b3-regra-saldo',
  custody_snapshot: 'b3-regra-retrato',
  manual_event: 'b3-regra-manual',
  forwarded_income: 'b3-regra-repassado',
  income_reversal: 'b3-regra-estorno',
  no_value: 'b3-regra-sem-valor',
  fraction: 'b3-regra-fracao',
  unknown_movement: 'b3-regra-desconhecida',
  invalid: 'b3-regra-invalida',
});

function positionStatus(label: Key, tone: Tone): { label: string; tone: Tone } {
  return {
    tone,
    get label() {
      return t(label);
    },
  };
}

export const B3_POSITION_STATUS: Record<B3PositionStatus, { label: string; tone: Tone }> = {
  match: positionStatus('b3-posicao-confere', 'positive'),
  divergent: positionStatus('b3-posicao-diverge', 'negative'),
  no_account: positionStatus('b3-status-sem-conta', 'dim'),
  missing_in_b3: positionStatus('b3-posicao-fora', 'warning'),
};

export const NOTE_STATUS: Record<NoteTradeStatus, StatusDef> = {
  new: status('b3-status-nova', 'positive', 'b3-nota-nova-dica'),
  fill_costs: status('b3-nota-completar', 'accent', 'b3-nota-completar-dica'),
  duplicate: status('b3-status-duplicada', 'dim', 'b3-nota-duplicada-dica'),
  unresolved: status('b3-nota-sem-ativo', 'warning', 'b3-nota-sem-ativo-dica'),
};

export const NOTE_STATUS_ORDER: readonly NoteTradeStatus[] = [
  'new',
  'fill_costs',
  'unresolved',
  'duplicate',
];
