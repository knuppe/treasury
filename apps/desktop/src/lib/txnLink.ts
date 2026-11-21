/**
 * Contrato do drill-down: qualquer gráfico ou card abre a tela 04 · Lançamentos já
 * filtrada por uma URL `#/lancamentos?…`. A tela de lançamentos lê com
 * `parseTxnLink`; quem cria o link usa `txnLink`.
 *
 * Parâmetros (todos opcionais; listas separadas por vírgula):
 * `from`, `to` (YYYY-MM-DD) · `basis` (`accrual` | `cash`) · `account` · `category`
 * (inclui subcategorias) · `member` · `kind` · `status` · `tag` · `q` (texto) ·
 * `statement` (fatura) · `import` (lote) · `project` (projeto) · `uncategorized=1`.
 */
import type { TxnFilter } from '@/api/generated/TxnFilter';

const LIST_KEYS = {
  account: 'account_ids',
  category: 'category_ids',
  member: 'member_ids',
  kind: 'kinds',
  status: 'statuses',
  tag: 'tag_ids',
  project: 'project_ids',
} as const;

const SCALAR_KEYS = {
  from: 'date_from',
  to: 'date_to',
  basis: 'basis',
  q: 'text',
  statement: 'statement_id',
  import: 'import_id',
} as const;

export function txnSearch(filter: TxnFilter): string {
  const p = new URLSearchParams();
  for (const [param, key] of Object.entries(SCALAR_KEYS)) {
    const v = filter[key];
    if (v !== undefined && v !== null && v !== '') p.set(param, String(v));
  }
  for (const [param, key] of Object.entries(LIST_KEYS)) {
    const v = filter[key] as readonly string[] | undefined | null;
    if (v && v.length > 0) p.set(param, v.join(','));
  }
  if (filter.uncategorized) p.set('uncategorized', '1');
  return p.toString();
}

/** Caminho para o roteador: `/lancamentos?category=…&from=…`. */
export function txnLink(filter: TxnFilter): string {
  const s = txnSearch(filter);
  return s ? `/lancamentos?${s}` : '/lancamentos';
}

export function parseTxnLink(search: string | URLSearchParams): TxnFilter {
  const p = typeof search === 'string' ? new URLSearchParams(search.replace(/^\?/, '')) : search;
  const f: Record<string, unknown> = {};
  for (const [param, key] of Object.entries(SCALAR_KEYS)) {
    const v = p.get(param);
    if (v) f[key] = v;
  }
  for (const [param, key] of Object.entries(LIST_KEYS)) {
    const v = p.get(param);
    if (v) f[key] = v.split(',').filter(Boolean);
  }
  if (p.get('uncategorized') === '1') f.uncategorized = true;
  return f;
}
