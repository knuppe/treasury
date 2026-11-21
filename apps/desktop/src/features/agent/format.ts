/**
 * Formatação e rótulos do agente: custos em micro-dólares, tokens, tarefas, níveis,
 * estados e tipos de motor, no idioma do app (catálogo `agente.ftl`). As tabelas leem o
 * catálogo na hora (getter), então podem ficar em constante sem congelar o idioma.
 */
import type { AiTask } from '@/api/generated/AiTask';
import type { AlertSeverity } from '@/api/generated/AlertSeverity';
import type { AlertStatus } from '@/api/generated/AlertStatus';
import type { EngineKind } from '@/api/generated/EngineKind';
import type { JobStatus } from '@/api/generated/JobStatus';
import type { MemoryKind } from '@/api/generated/MemoryKind';
import type { ProcedureTopic } from '@/api/generated/ProcedureTopic';
import type { ProposalLevel } from '@/api/generated/ProposalLevel';
import type { ProposalStatus } from '@/api/generated/ProposalStatus';
import { t } from '@/i18n';
import type { MessageKey, Messages } from '@/i18n/keys';
import { countLabel, currencyPrefix, formatLocale, formatNumber, MINUS } from '@/lib/format';

const MICROS = 1_000_000;

/** Chaves sem argumentos (as dos rótulos). */
export type PlainKey = { [K in MessageKey]: [Messages[K]] extends [never] ? K : never }[MessageKey];

/** Tabela de rótulos que lê o catálogo a cada acesso (enumerável, como um objeto comum). */
export function labels<V extends string>(keys: Record<V, PlainKey>): Record<V, string> {
  const out = {} as Record<V, string>;
  for (const [value, key] of Object.entries(keys) as [V, PlainKey][]) {
    Object.defineProperty(out, value, { enumerable: true, get: () => t(key) });
  }
  return out;
}

/** Código para o `Intl` no idioma ativo (o pseudo-idioma formata como pt-BR). */
export function intlLocale(): string {
  const code = formatLocale();
  return code === 'xx' ? 'pt-BR' : code;
}

/**
 * Custo em micro-unidades (USD × 10⁻⁶) no formato do idioma: `USD 1,23` em pt-BR. Abaixo
 * de dez centavos mostra 4 casas (`USD 0,0042`), para que chamadas baratas não virem zero.
 */
export function formatCost(micros: number, currency = 'USD'): string {
  if (!Number.isFinite(micros)) return '—';
  const abs = Math.abs(micros) / MICROS;
  const decimals = abs > 0 && abs < 0.1 ? 4 : 2;
  const body = formatNumber(abs, decimals);
  const prefix = currencyPrefix(currency);
  // Inglês: símbolo colado ao número (`$1.23`); código ISO sempre com espaço.
  const glued = formatLocale() === 'en' && prefix.length === 1;
  return `${micros < 0 ? MINUS : ''}${prefix}${glued ? '' : ' '}${body}`;
}

/** Valor em dólares digitado (`1,50`, `0.25`, `2`) → micro-dólares; `null` se inválido. */
export function parseCost(text: string): number | null {
  const raw = text
    .trim()
    .replace(/^US\$|^USD|^\$/i, '')
    .trim();
  if (raw === '') return null;
  // Inglês: vírgula é milhar e ponto é decimal. Nos outros idiomas, ponto é milhar e
  // vírgula é decimal; sem vírgula, um ponto vale como decimal.
  const normalized =
    formatLocale() === 'en'
      ? raw.replace(/,/g, '')
      : raw.includes(',')
        ? raw.replace(/\./g, '').replace(',', '.')
        : raw;
  if (!/^\d+(\.\d{0,6})?$/.test(normalized)) return null;
  return Math.round(Number(normalized) * MICROS);
}

/** Micro-dólares → texto editável (`1,5`), sem moeda. */
export function costInputValue(micros: number | null | undefined): string {
  if (micros == null) return '';
  return new Intl.NumberFormat(intlLocale(), {
    maximumFractionDigits: 6,
    useGrouping: false,
  }).format(micros / MICROS);
}

/** Tokens compactos: `950`, `12,3 mil`, `1,2 mi`. */
export function formatTokens(n: number): string {
  const abs = Math.abs(n);
  if (abs >= 1_000_000) return t('agente-tokens-mi', { n: n / 1_000_000 });
  if (abs >= 10_000) return t('agente-tokens-mil', { n: n / 1_000 });
  return formatNumber(n);
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms} ms`;
  const s = new Intl.NumberFormat(intlLocale(), { maximumFractionDigits: 1 }).format(ms / 1000);
  return `${s} s`;
}

export const TASK_LABEL: Record<AiTask, string> = labels({
  chat: 'agente-tarefa-chat',
  categorize: 'agente-tarefa-categorizacao',
  pdf: 'agente-tarefa-pdf',
  monthly_close: 'agente-tarefa-fechamento',
  alerts: 'agente-tarefa-alertas',
  study: 'agente-tarefa-estudo',
});

export const ENGINE_KIND_LABEL: Record<EngineKind, string> = labels({
  api: 'agente-tipo-motor-api',
  cli: 'agente-tipo-motor-cli',
  local: 'agente-tipo-motor-local',
});

/** Para onde os dados vão, por tipo de motor (o indicador permanente). */
export const ENGINE_KIND_DESTINATION: Record<EngineKind, string> = labels({
  api: 'agente-destino-api',
  cli: 'agente-destino-cli',
  local: 'agente-destino-local',
});

export const LEVEL_LABEL: Record<ProposalLevel, string> = labels({
  reversible: 'agente-nivel-reversivel',
  sensitive: 'agente-nivel-sensivel',
});

export const PROPOSAL_STATUS_LABEL: Record<ProposalStatus, string> = labels({
  pending: 'agente-proposta-pendente',
  approved: 'agente-proposta-aprovada',
  rejected: 'agente-proposta-rejeitada',
  failed: 'agente-proposta-falhou',
  withdrawn: 'agente-proposta-retirada',
});

export const ALERT_STATUS_LABEL: Record<AlertStatus, string> = labels({
  new: 'agente-alerta-novo',
  seen: 'agente-alerta-visto',
  resolved: 'agente-alerta-resolvido',
});

export const SEVERITY_LABEL: Record<AlertSeverity, string> = labels({
  info: 'agente-gravidade-info',
  warning: 'agente-gravidade-atencao',
  critical: 'agente-gravidade-critico',
});

export const JOB_STATUS_LABEL: Record<JobStatus, string> = labels({
  running: 'agente-job-andamento',
  done: 'agente-job-concluido',
  failed: 'agente-job-falhou',
  cancelled: 'agente-job-cancelado',
});

export const MEMORY_KIND_LABEL: Record<MemoryKind, string> = labels({
  profile: 'agente-memoria-perfil',
  family_rule: 'agente-memoria-regra',
  note: 'agente-memoria-nota',
});

/** Rótulo do comando de escrita proposto (`transactions.set_category` → texto). */
const COMMAND_LABEL: Record<string, string> = labels({
  'transactions.create': 'agente-comando-criar-lancamento',
  'transactions.update': 'agente-comando-alterar-lancamento',
  'transactions.set_category': 'agente-comando-categorizar',
  'transactions.split': 'agente-comando-dividir',
  'transactions.create_transfer': 'agente-comando-criar-transferencia',
  'transactions.delete': 'agente-comando-apagar-lancamento',
  'devfeedback.delete': 'agente-comando-apagar-feedback',
  'devfeedback.discard': 'agente-comando-descartar-feedback',
  'transactions.create_tag': 'agente-comando-criar-etiqueta',
  'categories.create': 'agente-comando-criar-categoria',
  'categories.update': 'agente-comando-alterar-categoria',
  'household.create_institution': 'agente-comando-cadastrar-instituicao',
  'household.update_institution': 'agente-comando-alterar-instituicao',
  'accounts.create': 'agente-comando-cadastrar-conta',
  'accounts.update': 'agente-comando-alterar-conta',
  'recurrences.update': 'agente-comando-alterar-recorrencia',
  'assets.update': 'agente-comando-alterar-ativo',
  'physical.create': 'agente-comando-cadastrar-bem',
  'physical.update': 'agente-comando-alterar-bem',
  'physical.add_valuation': 'agente-comando-avaliacao-bem',
  'physical.add_improvement': 'agente-comando-benfeitoria',
  'rules.create': 'agente-comando-criar-regra',
  'rules.update': 'agente-comando-alterar-regra',
  'rules.apply': 'agente-comando-aplicar-regra',
  'recurrences.create': 'agente-comando-criar-recorrencia',
  'budget.set_lines': 'agente-comando-definir-orcamento',
  'budget.copy': 'agente-comando-copiar-orcamento',
  'goals.create': 'agente-comando-criar-meta',
  'goals.update': 'agente-comando-alterar-meta',
  'assets.create': 'agente-comando-cadastrar-ativo',
  'portfolio.watch': 'agente-comando-acompanhar-ativo',
  'portfolio.create_trade': 'agente-comando-registrar-operacao',
  'portfolio.create_income': 'agente-comando-registrar-provento',
  'portfolio.quick_income': 'agente-comando-proventos-ticker',
  'portfolio.create_action': 'agente-comando-evento-corporativo',
  'import.commit': 'agente-comando-importar-extrato',
  'import.rows': 'agente-comando-importar-lidos',
  'portfolio.bulk': 'agente-comando-investimentos-lote',
  'b3.commit': 'agente-comando-importar-b3',
  'ai.remember': 'agente-comando-gravar-memoria',
});

/** Assuntos da memória de procedimentos do agente. */
export const PROCEDURE_TOPIC_LABEL: Record<ProcedureTopic, string> = labels({
  statement: 'agente-assunto-extrato',
  card_bill: 'agente-assunto-fatura',
  investments: 'agente-assunto-investimentos',
  income: 'agente-assunto-proventos',
  prices: 'agente-assunto-cotacoes',
  tax: 'agente-assunto-imposto',
  registration: 'agente-assunto-cadastro',
  other: 'agente-assunto-outro',
});

export function commandLabel(cmd: string): string {
  return COMMAND_LABEL[cmd] ?? cmd;
}

/** Tabelas do banco nas linhas do diff: o rótulo e a contagem ("3 lançamentos"). */
const TABLES = {
  txn: ['agente-tabela-lancamento', 'agente-n-lancamento'],
  txn_split: ['agente-tabela-divisao', 'agente-n-divisao'],
  txn_tag: ['agente-tabela-etiqueta-lancamento', 'agente-n-etiqueta-lancamento'],
  tag: ['agente-tabela-etiqueta', 'agente-n-etiqueta'],
  category: ['agente-tabela-categoria', 'agente-n-categoria'],
  rule: ['agente-tabela-regra', 'agente-n-regra'],
  rule_hit: ['agente-tabela-aplicacao-regra', 'agente-n-aplicacao-regra'],
  recurrence: ['agente-tabela-recorrencia', 'agente-n-recorrencia'],
  budget: ['agente-tabela-orcamento', 'agente-n-orcamento'],
  budget_line: ['agente-tabela-linha-orcamento', 'agente-n-linha-orcamento'],
  goal: ['agente-tabela-meta', 'agente-n-meta'],
  goal_account: ['agente-tabela-conta-meta', 'agente-n-conta-meta'],
  project: ['agente-tabela-projeto', 'agente-n-projeto'],
  asset: ['agente-tabela-ativo', 'agente-n-ativo'],
  trade: ['agente-tabela-operacao', 'agente-n-operacao'],
  income_event: ['agente-tabela-provento', 'agente-n-provento'],
  corporate_action: ['agente-tabela-evento-corporativo', 'agente-n-evento-corporativo'],
  watchlist_item: ['agente-tabela-acompanhamento', 'agente-n-acompanhamento'],
  import_batch: ['agente-tabela-lote-importacao', 'agente-n-lote-importacao'],
  ai_memory: ['agente-tabela-memoria', 'agente-n-memoria'],
  ai_procedure: ['agente-tabela-procedimento', 'agente-n-procedimento'],
  dev_feedback: ['agente-tabela-feedback', 'agente-n-feedback'],
  dev_feedback_note: ['agente-tabela-nota-feedback', 'agente-n-nota-feedback'],
  dev_feedback_link: ['agente-tabela-vinculo-feedback', 'agente-n-vinculo-feedback'],
  account: ['agente-tabela-conta', 'agente-n-conta'],
  account_member: ['agente-tabela-membro-conta', 'agente-n-membro-conta'],
  credit_card: ['agente-tabela-cartao', 'agente-n-cartao'],
  institution: ['agente-tabela-instituicao', 'agente-n-instituicao'],
  fixed_income_terms: ['agente-tabela-renda-fixa', 'agente-n-renda-fixa'],
  physical_asset: ['agente-tabela-bem', 'agente-n-bem'],
  physical_asset_member: ['agente-tabela-dono-bem', 'agente-n-dono-bem'],
  physical_acquisition_cost: ['agente-tabela-custo-aquisicao', 'agente-n-custo-aquisicao'],
  physical_improvement: ['agente-tabela-benfeitoria', 'agente-n-benfeitoria'],
  physical_valuation: ['agente-tabela-avaliacao-bem', 'agente-n-avaliacao-bem'],
  physical_asset_loan: ['agente-tabela-financiamento-bem', 'agente-n-financiamento-bem'],
} as const;

type TableKeys = (typeof TABLES)[keyof typeof TABLES];

function tableKeys(table: string): TableKeys | undefined {
  return (TABLES as Record<string, TableKeys>)[table];
}

/** Rótulo das tabelas do banco nas linhas do diff. */
export function tableLabel(table: string): string {
  const keys = tableKeys(table);
  return keys ? t(keys[0]) : table;
}

/** "3 lançamentos", "1 regra": a quantidade com o nome da tabela (em minúsculas). */
export function tableCount(table: string, n: number): string {
  const keys = tableKeys(table);
  return keys ? t(keys[1], { n }) : countLabel(n, table);
}

/** Resumo do change_set (`{txn: {update: 3}}`) → `altera 3 lançamentos`. */
export function summaryText(summary: Record<string, Record<string, number>>): string {
  const parts: string[] = [];
  for (const [table, ops] of Object.entries(summary)) {
    for (const [o, n] of Object.entries(ops)) {
      if (n <= 0) continue;
      const itens = tableCount(table, n);
      parts.push(
        o === 'insert'
          ? t('agente-resumo-cria', { itens })
          : o === 'update'
            ? t('agente-resumo-altera', { itens })
            : o === 'delete'
              ? t('agente-resumo-apaga', { itens })
              : `${o} ${itens}`,
      );
    }
  }
  return parts.join(' · ');
}
