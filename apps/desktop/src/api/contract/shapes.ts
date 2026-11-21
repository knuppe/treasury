/**
 * Formas das respostas do núcleo usadas pelas telas 02, 03 e 11, presas aos tipos
 * gerados (`src/api/generated/`): se um DTO ganhar ou perder campo, o `tsc` acusa
 * aqui. O mesmo verificador roda contra o núcleo real
 * (`scripts/contract-ledger.mjs`, via Node com remoção de tipos) e contra o mock
 * (`src/api/mock/ledger/contract.test.ts`).
 *
 * Este arquivo não pode usar aliases (`@/`) nem importar código: só tipos.
 */
import type { AccountView } from '../generated/AccountView.ts';
import type { AppInfo } from '../generated/AppInfo.ts';
import type { AppSettings } from '../generated/AppSettings.ts';
import type { AttachmentContent } from '../generated/AttachmentContent.ts';
import type { AttachmentMeta } from '../generated/AttachmentMeta.ts';
import type { AuditEntry } from '../generated/AuditEntry.ts';
import type { Author } from '../generated/Author.ts';
import type { BackupHealth } from '../generated/BackupHealth.ts';
import type { BackupRecord } from '../generated/BackupRecord.ts';
import type { BackupSettings } from '../generated/BackupSettings.ts';
import type { BalancePoint } from '../generated/BalancePoint.ts';
import type { BulkResult } from '../generated/BulkResult.ts';
import type { CardDetails } from '../generated/CardDetails.ts';
import type { Category } from '../generated/Category.ts';
import type { CategoryNode } from '../generated/CategoryNode.ts';
import type { CategoryUsage } from '../generated/CategoryUsage.ts';
import type { ChangeItem } from '../generated/ChangeItem.ts';
import type { ChangeSetDetail } from '../generated/ChangeSetDetail.ts';
import type { ChangeSetSummary } from '../generated/ChangeSetSummary.ts';
import type { Converted } from '../generated/Converted.ts';
import type { CsvMapping } from '../generated/CsvMapping.ts';
import type { CsvPreset } from '../generated/CsvPreset.ts';
import type { CurrencyTotal } from '../generated/CurrencyTotal.ts';
import type { DataSources } from '../generated/DataSources.ts';
import type { DetectResult } from '../generated/DetectResult.ts';
import type { ExportResult } from '../generated/ExportResult.ts';
import type { FxRate } from '../generated/FxRate.ts';
import type { Household } from '../generated/Household.ts';
import type { ImportBatch } from '../generated/ImportBatch.ts';
import type { ImportPreview } from '../generated/ImportPreview.ts';
import type { ImportResult } from '../generated/ImportResult.ts';
import type { InstallmentPlan } from '../generated/InstallmentPlan.ts';
import type { Institution } from '../generated/Institution.ts';
import type { Member } from '../generated/Member.ts';
import type { MonthAmount } from '../generated/MonthAmount.ts';
import type { OfxAccountInfo } from '../generated/OfxAccountInfo.ts';
import type { PayStatementResult } from '../generated/PayStatementResult.ts';
import type { PreviewCounts } from '../generated/PreviewCounts.ts';
import type { PreviewRow } from '../generated/PreviewRow.ts';
import type { ReconcileResult } from '../generated/ReconcileResult.ts';
import type { RecoveryKit } from '../generated/RecoveryKit.ts';
import type { Recurrence } from '../generated/Recurrence.ts';
import type { RecurrenceDeleteResult } from '../generated/RecurrenceDeleteResult.ts';
import type { RecurrencePreview } from '../generated/RecurrencePreview.ts';
import type { Rule } from '../generated/Rule.ts';
import type { RuleActions } from '../generated/RuleActions.ts';
import type { RuleApplyResult } from '../generated/RuleApplyResult.ts';
import type { RuleCondition } from '../generated/RuleCondition.ts';
import type { RuleInput } from '../generated/RuleInput.ts';
import type { RuleSpec } from '../generated/RuleSpec.ts';
import type { Split } from '../generated/Split.ts';
import type { StatementDetail } from '../generated/StatementDetail.ts';
import type { StatementView } from '../generated/StatementView.ts';
import type { Tag } from '../generated/Tag.ts';
import type { TagSummary } from '../generated/TagSummary.ts';
import type { TransferResult } from '../generated/TransferResult.ts';
import type { TxnPage } from '../generated/TxnPage.ts';
import type { TxnView } from '../generated/TxnView.ts';
import type { VaultStatus } from '../generated/VaultStatus.ts';

/**
 * Especificação de um campo: tipos primitivos separados por `|` (`string|null`),
 * `?` na frente para campo opcional (`?string`), uma forma aninhada, ou
 * `[forma]` para lista.
 */
export type Spec = string | ShapeDef | readonly [Spec];
export interface ShapeDef {
  readonly name: string;
  readonly fields: Readonly<Record<string, Spec>>;
  /** Objeto com chaves livres (ex.: `summary`, `counts`). */
  readonly open?: boolean;
}

type Fields<T> = { readonly [K in keyof T]-?: Spec };

function shape<T>(name: string, fields: Fields<T>): ShapeDef {
  return { name, fields: fields };
}

export interface Finding {
  level: 'erro' | 'aviso';
  path: string;
  message: string;
}

function typeOf(v: unknown): string {
  if (v === null) return 'null';
  if (Array.isArray(v)) return 'array';
  return typeof v;
}

/** Confere `value` contra a especificação; devolve os problemas encontrados. */
export function check(value: unknown, spec: Spec, path = '$', out: Finding[] = []): Finding[] {
  if (Array.isArray(spec)) {
    if (!Array.isArray(value)) {
      out.push({ level: 'erro', path, message: `esperava lista, veio ${typeOf(value)}` });
      return out;
    }
    value.slice(0, 25).forEach((item, i) => check(item, spec[0] as Spec, `${path}[${i}]`, out));
    return out;
  }
  if (typeof spec === 'object') {
    const def = spec as ShapeDef;
    if (typeOf(value) !== 'object') {
      out.push({ level: 'erro', path, message: `esperava ${def.name}, veio ${typeOf(value)}` });
      return out;
    }
    const obj = value as Record<string, unknown>;
    for (const [key, field] of Object.entries(def.fields)) {
      const optional = typeof field === 'string' && field.startsWith('?');
      if (!(key in obj)) {
        if (!optional)
          out.push({
            level: 'erro',
            path: `${path}.${key}`,
            message: `campo ausente em ${def.name}`,
          });
        continue;
      }
      if (optional && obj[key] === null && !field.includes('null')) {
        out.push({
          level: 'aviso',
          path: `${path}.${key}`,
          message: `${def.name}.${key} chega como null (o tipo TS diz opcional, sem null)`,
        });
        continue;
      }
      check(obj[key], optional ? field.slice(1) : field, `${path}.${key}`, out);
    }
    if (!def.open) {
      for (const key of Object.keys(obj)) {
        if (!(key in def.fields))
          out.push({
            level: 'aviso',
            path: `${path}.${key}`,
            message: `campo extra em ${def.name} (não está no tipo gerado)`,
          });
      }
    }
    return out;
  }
  const allowed = spec.split('|');
  if (allowed.includes('any')) return out;
  const t = typeOf(value);
  if (!allowed.includes(t))
    out.push({ level: 'erro', path, message: `esperava ${spec}, veio ${t}` });
  return out;
}

const OPEN: ShapeDef = { name: 'objeto', fields: {}, open: true };

export const CardDetailsShape = shape<CardDetails>('CardDetails', {
  credit_limit: 'number|null',
  closing_day: 'number',
  due_day: 'number',
  payment_account_id: 'string|null',
  network: 'string|null',
  last4: 'string|null',
});

export const AccountViewShape = shape<AccountView>('AccountView', {
  id: 'string',
  name: 'string',
  kind: 'string',
  currency: 'string',
  institution_id: 'string|null',
  institution_name: 'string|null',
  member_ids: ['string'],
  opening_balance: 'number',
  opening_date: 'string',
  include_in_net_worth: 'boolean',
  archived: 'boolean',
  color: 'string|null',
  notes: 'string|null',
  sort_order: 'number',
  card: 'any',
  balance: 'number',
  projected_balance: 'number',
  reconciled_balance: 'number',
  balance_base: 'number|null',
  fx_estimated: 'boolean',
  last_txn_date: 'string|null',
  last_import_at: 'string|null',
  staleness_days: 'number|null',
  balance_status: 'string|null',
  missed_month: 'string|null',
  overdraft: 'object|null',
  overdraft_status: 'object|null',
  branch: 'string|null',
  account_number: 'string|null',
  retirement_plan: 'string|null',
  retirement_coverage: 'string|null',
  available_credit: 'number|null',
});

export const BalancePointShape = shape<BalancePoint>('BalancePoint', {
  date: 'string',
  balance: 'number',
});

export const ReconcileResultShape = shape<ReconcileResult>('ReconcileResult', {
  computed_balance: 'number',
  statement_balance: 'number',
  difference: 'number',
  marked_reconciled: 'number',
  change_set_id: 'string|null',
});

export const HouseholdShape = shape<Household>('Household', {
  id: 'string',
  name: 'string',
  base_currency: 'string',
  country: 'string',
});

export const MemberShape = shape<Member>('Member', {
  id: 'string',
  name: 'string',
  color: 'string|null',
  is_primary: 'boolean',
  archived: 'boolean',
  sort_order: 'number',
  profile: 'any',
});

export const InstitutionShape = shape<Institution>('Institution', {
  id: 'string',
  name: 'string',
  kind: 'string',
  code: 'string|null',
  website: 'string|null',
});

const categoryFields = {
  id: 'string',
  parent_id: 'string|null',
  name: 'string',
  kind: 'string',
  icon: 'string|null',
  color: 'string|null',
  is_essential: 'boolean',
  spend_group: 'string|null',
  archived: 'boolean',
  sort_order: 'number',
  path: 'string',
} as const;

export const CategoryShape = shape<Category>('Category', categoryFields);
export const CategoryNodeShape: ShapeDef = shape<CategoryNode>('CategoryNode', {
  ...categoryFields,
  children: ['any'],
});

export const CategoryUsageShape = shape<CategoryUsage>('CategoryUsage', {
  transactions: 'number',
  splits: 'number',
  budget_lines: 'number',
  rules: 'number',
  recurrences: 'number',
  installment_plans: 'number',
  children: 'number',
});

export const SplitShape = shape<Split>('Split', {
  id: 'string',
  category_id: 'string|null',
  member_id: 'string|null',
  amount: 'number',
  notes: 'string|null',
});

export const TagShape = shape<Tag>('Tag', { id: 'string', name: 'string', color: 'string|null' });

export const TxnViewShape = shape<TxnView>('TxnView', {
  id: 'string',
  account_id: 'string',
  account_name: 'string',
  date: 'string',
  amount: 'number',
  currency: 'string',
  kind: 'string',
  status: 'string',
  description: 'string',
  payee: 'string|null',
  notes: 'string|null',
  category_id: 'string|null',
  category_path: 'string|null',
  member_id: 'string|null',
  member_name: 'string|null',
  transfer_id: 'string|null',
  counterpart_account_id: 'string|null',
  counterpart_account_name: 'string|null',
  statement_id: 'string|null',
  statement_due_date: 'string|null',
  installment_plan_id: 'string|null',
  installment_number: 'number|null',
  installment_total: 'number|null',
  recurrence_id: 'string|null',
  import_id: 'string|null',
  external_id: 'string|null',
  splits: [SplitShape],
  tags: [TagShape],
  attachment_count: 'number',
  created_at: 'string',
  updated_at: 'string',
  project_id: 'string|null',
  project_name: 'string|null',
  card_plastic_id: 'string|null',
  card_last4: 'string|null',
  orig_amount: 'number|null',
  orig_currency: 'string|null',
  loan_id: 'string|null',
  loan_name: 'string|null',
  loan_installment_id: 'string|null',
  loan_installment_number: 'number|null',
  operation: 'object|null',
  receipt_id: '?string|null',
});

export const CurrencyTotalShape = shape<CurrencyTotal>('CurrencyTotal', {
  currency: 'string',
  inflow: 'number',
  outflow: 'number',
  net: 'number',
});

export const TxnPageShape = shape<TxnPage>('TxnPage', {
  items: [TxnViewShape],
  total: 'number',
  totals: [CurrencyTotalShape],
});

export const BulkResultShape = shape<BulkResult>('BulkResult', {
  updated: 'number',
  change_set_id: 'string|null',
  skipped: '?number',
});

export const TransferResultShape = shape<TransferResult>('TransferResult', {
  transfer_id: 'string',
  from: TxnViewShape,
  to: TxnViewShape,
});

export const TagSummaryShape = shape<TagSummary>('TagSummary', {
  id: 'string',
  name: 'string',
  color: 'string|null',
  txn_count: 'number',
});

const attachmentFields = {
  id: 'string',
  txn_id: 'string|null',
  filename: 'string',
  mime: 'string',
  size_bytes: 'number',
  sha256: 'string',
  created_at: 'string',
} as const;
export const AttachmentMetaShape = shape<AttachmentMeta>('AttachmentMeta', attachmentFields);
export const AttachmentContentShape = shape<AttachmentContent>('AttachmentContent', {
  ...attachmentFields,
  content_base64: 'string',
});

export const RuleSpecShape = shape<RuleSpec>('RuleSpec', {
  freq: 'string',
  interval: '?number',
  by_month_day: '?number',
  by_day: '?array',
  by_month: '?number',
  count: '?number',
});

export const RecurrenceShape = shape<Recurrence>('Recurrence', {
  id: 'string',
  description: 'string',
  account_id: 'string',
  account_name: 'string',
  amount: 'number',
  currency: 'string',
  kind: 'string',
  category_id: 'string|null',
  member_id: 'string|null',
  to_account_id: 'string|null',
  rrule: 'string',
  spec: RuleSpecShape,
  human: 'string',
  start_date: 'string',
  end_date: 'string|null',
  active: 'boolean',
  next_dates: ['string'],
  monthly_equivalent: 'number',
});

export const RecurrencePreviewShape = shape<RecurrencePreview>('RecurrencePreview', {
  rrule: 'string',
  human: 'string',
  dates: ['string'],
});

export const RecurrenceDeleteResultShape = shape<RecurrenceDeleteResult>('RecurrenceDeleteResult', {
  deleted_planned: 'number',
  change_set_id: 'string|null',
});

export const StatementViewShape = shape<StatementView>('StatementView', {
  id: 'string',
  account_id: 'string',
  account_name: 'string',
  reference_month: 'string',
  period_start: 'string',
  closing_date: 'string',
  due_date: 'string',
  status: 'string',
  total: 'number',
  planned: 'number',
  paid: 'number',
  remaining: 'number',
  informed_total: 'number|null',
  txn_count: 'number',
  currency: 'string',
  dates_source: 'string',
  informed_minimum: 'number|null',
  informed_limit: 'number|null',
  informed_previous_balance: 'number|null',
});

export const StatementDetailShape = shape<StatementDetail>('StatementDetail', {
  statement: StatementViewShape,
  transactions: [TxnViewShape],
  by_plastic: ['array'],
});

export const PayStatementResultShape = shape<PayStatementResult>('PayStatementResult', {
  statement: StatementViewShape,
  transfer: TransferResultShape,
});

export const InstallmentPlanShape = shape<InstallmentPlan>('InstallmentPlan', {
  id: 'string',
  account_id: 'string',
  description: 'string',
  total_amount: 'number',
  currency: 'string',
  installments: 'number',
  purchase_date: 'string',
  category_id: 'string|null',
  member_id: 'string|null',
  paid_installments: 'number',
  remaining_amount: 'number',
});

export const MonthAmountShape = shape<MonthAmount>('MonthAmount', {
  month: 'string',
  amount: 'number',
});

export const RuleConditionShape = shape<RuleCondition>('RuleCondition', {
  field: 'string',
  op: 'string',
  value: 'string|number',
  value2: '?number',
});

export const RuleActionsShape = shape<RuleActions>('RuleActions', {
  category_id: '?string',
  member_id: '?string',
  tags: '?array',
  rename_to: '?string',
  kind: '?string',
});

export const RuleShape = shape<Rule>('Rule', {
  id: 'string',
  name: 'string',
  priority: 'number',
  active: 'boolean',
  match_all: 'boolean',
  conditions: [RuleConditionShape],
  actions: RuleActionsShape,
  hits: 'number',
  last_hit_at: 'string|null',
  created_at: 'string',
  updated_at: 'string',
});

export const RuleInputShape = shape<RuleInput>('RuleInput', {
  name: 'string',
  priority: '?number',
  active: '?boolean',
  match_all: '?boolean',
  conditions: [RuleConditionShape],
  actions: RuleActionsShape,
});

export const RuleApplyResultShape = shape<RuleApplyResult>('RuleApplyResult', {
  matched: 'number',
  updated: 'number',
  sample: [TxnViewShape],
  change_set_id: 'string|null',
});

export const CsvMappingShape = shape<CsvMapping>('CsvMapping', {
  delimiter: '?string',
  has_header: 'boolean',
  skip_rows: '?number',
  date_column: 'string',
  date_format: 'string',
  amount_column: '?string',
  debit_column: '?string',
  credit_column: '?string',
  description_column: 'string',
  payee_column: '?string',
  external_id_column: '?string',
  decimal_comma: 'boolean',
  invert_sign: 'boolean',
  encoding: '?string',
  card_descriptor: '?boolean',
});

export const CsvPresetShape = shape<CsvPreset>('CsvPreset', {
  id: 'string',
  name: 'string',
  mapping: CsvMappingShape,
});

export const OfxAccountInfoShape = shape<OfxAccountInfo>('OfxAccountInfo', {
  bank_id: 'string|null',
  branch_id: 'string|null',
  account_id: 'string|null',
  account_type: 'string|null',
  credit_card: 'boolean',
  currency: 'string|null',
  ledger_balance: 'number|null',
  balance_date: 'string|null',
  date_start: 'string|null',
  date_end: 'string|null',
  statements: 'number',
});

export const DetectResultShape = shape<DetectResult>('DetectResult', {
  format: 'string',
  encoding: 'string',
  delimiter: '?string',
  skip_rows: 'number',
  has_header: 'boolean',
  headers: ['string'],
  sample_rows: ['array'],
  row_count: 'number',
  suggested_preset_id: 'string|null',
  suggested_mapping: 'any',
  ofx: 'any',
  layout: '?any',
});

export const PreviewRowShape = shape<PreviewRow>('PreviewRow', {
  index: 'number',
  line: 'number|null',
  date: 'string|null',
  amount: 'number|null',
  description: 'string',
  payee: 'string|null',
  external_id: 'string|null',
  status: 'string',
  duplicate_of: 'string|null',
  kind: 'string|null',
  suggested_category_id: 'string|null',
  suggested_member_id: 'string|null',
  suggested_tags: ['string'],
  suggested_description: 'string|null',
  rule_id: 'string|null',
  error: 'string|null',
  note: 'string|null',
  card_last4: 'string|null',
  card_plastic_id: 'string|null',
  orig_amount: 'number|null',
  orig_currency: 'string|null',
  planned_id: 'string|null',
});

export const PreviewCountsShape = shape<PreviewCounts>('PreviewCounts', {
  new: 'number',
  duplicate: 'number',
  possible_duplicate: 'number',
  error: 'number',
  ignored: 'number',
});

export const ImportPreviewShape = shape<ImportPreview>('ImportPreview', {
  token: 'string',
  format: 'string',
  encoding: 'string',
  account_id: 'string',
  filename: 'string',
  file_sha256: 'string',
  already_imported: 'boolean',
  already_imported_batch_id: 'string|null',
  currency: 'string',
  rows: [PreviewRowShape],
  counts: PreviewCountsShape,
  ofx: 'any',
  csv_mapping: 'any',
  preset_id: 'string|null',
  warnings: ['string'],
  card_pdf: '?any',
});

export const ImportResultShape = shape<ImportResult>('ImportResult', {
  batch_id: 'string',
  inserted: 'number',
  skipped: 'number',
  installments_paid: 'number',
  transfers_linked: 'number',
  planned_settled: 'number',
  change_set_id: 'string|null',
  backup_id: 'string|null',
});

export const ImportBatchShape = shape<ImportBatch>('ImportBatch', {
  id: 'string',
  filename: 'string',
  format: 'string',
  file_sha256: 'string',
  account_id: 'string',
  account_name: 'string',
  total_rows: 'number',
  inserted: 'number',
  duplicates: 'number',
  errors: ['any'],
  created_at: 'string',
  change_set_id: 'string|null',
});

export const AuthorShape = shape<Author>('Author', { kind: 'string', detail: 'string|null' });

const changeFields = {
  id: 'string',
  created_at: 'string',
  author: AuthorShape,
  reason: 'string',
  status: 'string',
  applied_at: 'string|null',
  undone_at: 'string|null',
  summary: OPEN,
  item_count: 'number',
  proposal_id: '?string',
} as const;

export const ChangeSetSummaryShape = shape<ChangeSetSummary>('ChangeSetSummary', changeFields);

export const ChangeItemShape = shape<ChangeItem>('ChangeItem', {
  seq: 'number',
  table_name: 'string',
  row_id: 'string',
  op: 'string',
  before: 'object|null',
  after: 'object|null',
});

export const ChangeSetDetailShape = shape<ChangeSetDetail>('ChangeSetDetail', {
  ...changeFields,
  items: [ChangeItemShape],
});

export const FxRateShape = shape<FxRate>('FxRate', {
  id: 'string',
  base: 'string',
  quote: 'string',
  date: 'string',
  rate: 'string',
  source: 'string',
  fetched_at: 'string',
});

export const ConvertedShape = shape<Converted>('Converted', {
  amount: 'number',
  currency: 'string',
  rate: 'string',
  rate_date: 'string|null',
  source: 'string|null',
  estimated: 'boolean',
});

export const AppSettingsShape = shape<AppSettings>('AppSettings', {
  auto_lock_minutes: 'number',
  budget_basis: 'string',
  flow_basis: 'string',
  close_to_tray: 'boolean',
  locale: 'string',
  developer_tools: 'boolean',
  watch_downloads: 'boolean',
  downloads_delete_after_import: 'boolean',
  lock_on_sleep: 'boolean',
  allow_screen_capture: 'boolean',
  twr_daily: 'boolean',
  watch_inbox: 'boolean',
  inbox_folder: 'string',
});

export const AppInfoShape = shape<AppInfo>('AppInfo', {
  version: 'string',
  schema_version: 'number',
  db_path: 'string',
  backups_dir: 'string',
  data_root: 'string',
  platform: 'string',
});

export const BackupHealthShape = shape<BackupHealth>('BackupHealth', {
  last_valid_at: 'string|null',
  hours_since_last_valid: 'number|null',
  warning: 'boolean',
  daily_due: 'boolean',
  count: 'number',
  total_bytes: 'number',
});

export const BackupRecordShape = shape<BackupRecord>('BackupRecord', {
  id: 'string',
  created_at: 'string',
  kind: 'string',
  path: 'string',
  destination: 'string',
  key_mode: 'string',
  size_bytes: 'number',
  sha256: 'string|null',
  verified: 'boolean',
  verify_error: 'string|null',
  exists: 'boolean',
  needs_password: 'boolean',
});

export const BackupSettingsShape = shape<BackupSettings>('BackupSettings', {
  daily_hour: 'number',
  keep_daily: 'number',
  keep_weekly: 'number',
  keep_monthly: 'number',
  extra_destinations: ['string'],
  on_close: 'boolean',
  key_mode: 'string',
});

export const AuditEntryShape = shape<AuditEntry>('AuditEntry', {
  id: 'string',
  created_at: 'string',
  client: 'string',
  tool: 'string',
  params: OPEN,
  ok: 'boolean',
  error: 'string|null',
  duration_ms: 'number',
});

export const DataSourcesShape = shape<DataSources>('DataSources', {
  fred_key_set: 'boolean',
  brapi_token_set: 'boolean',
  brapi: 'object|null',
  tiingo_token_set: 'boolean',
  eodhd_token_set: 'boolean',
  last_sync_at: 'string|null',
});

export const VaultStatusShape = shape<VaultStatus>('VaultStatus', {
  state: 'string',
  db_path: 'string',
  auto_lock_minutes: 'number',
  keychain_available: 'boolean',
  keychain_enabled: 'boolean',
  pin_enabled: 'boolean',
  fido2_keys: 'number',
  household_name: 'string|null',
  vault_id: 'string',
  vault_name: 'string',
  vault_count: 'number',
  demo: 'boolean',
  demo_no_password: 'boolean',
  newer_app_version: 'string|null',
});

export const RecoveryKitShape = shape<RecoveryKit>('RecoveryKit', {
  code: 'string',
  created_at: 'string',
  db_path: 'string',
});

export const ExportResultShape = shape<ExportResult>('ExportResult', {
  path: 'string',
  bytes: 'number',
  counts: OPEN,
});

export const McpConfigShape: ShapeDef = {
  name: 'McpClientConfig',
  fields: {
    mcpServers: {
      name: 'mcpServers',
      fields: {
        treasury: {
          name: 'treasury',
          fields: { command: 'string', args: ['string'], env: '?object' },
        },
      },
    },
  },
};
