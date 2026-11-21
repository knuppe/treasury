/**
 * Mock de `database.*`: tamanho e maiores tabelas a partir do livro-caixa do mock,
 * manutenção (sempre íntegra) e expurgo com prévia.
 */
import type { DatabaseInfo } from '@/api/generated/DatabaseInfo';
import type { MaintenanceResult } from '@/api/generated/MaintenanceResult';
import type { PurgeKind } from '@/api/generated/PurgeKind';
import type { PurgePreview } from '@/api/generated/PurgePreview';
import type { PurgeResult } from '@/api/generated/PurgeResult';
import type { TableRows } from '@/api/generated/TableRows';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { ready } from './ledger/boot';
import { invalid, notFound, str } from './ledger/errors';
import { db, rows } from './ledger/store';

const LABEL: Record<PurgeKind, [string, string, number]> = {
  change_history: [
    'Histórico de alterações antigo',
    'As alterações apagadas não podem mais ser desfeitas; propostas pendentes ficam.',
    240,
  ],
  ai_audit: [
    'Auditoria do agente e do MCP',
    'Some o registro de quem consultou ou propôs o quê nesse período.',
    1_830,
  ],
  ai_conversations: [
    'Conversas antigas do agente',
    'Conversas não fixadas sem uso no período (e os anexos delas) somem.',
    12,
  ],
  market_cache: [
    'Caches de mercado',
    'Cotações intradiárias antigas e o cache do Fundamentus; voltam na próxima consulta.',
    52_400,
  ],
  research_data: [
    'Dados da análise de ativos',
    'Fundamentos, cotações do universo e documentos baixados da CVM, B3 e SEC; a próxima coleta baixa tudo de novo (cerca de 230 MB pela rede).',
    412_000,
  ],
};

const purged = new Set<PurgeKind>();
let fileBytes = 186_400_000;
let lastMaintenance: string | null = null;
onMockReset(() => {
  purged.clear();
  fileBytes = 186_400_000;
  lastMaintenance = null;
});

function info(): DatabaseInfo {
  const txns = rows('txn').length;
  const tables = [
    ['fundamental', 'análise de ativos (CVM, B3, SEC)', 412_000, 92_000_000, 18_000_000],
    ['research_quote', 'análise de ativos (CVM, B3, SEC)', 380_000, 41_000_000, 9_000_000],
    ['intraday_price', 'cotações, índices e referências', 52_400, 6_100_000, 1_300_000],
    ['change_item', 'histórico de alterações', 9_800, 4_900_000, 600_000],
    ['price', 'cotações, índices e referências', 31_000, 3_200_000, 800_000],
    ['txn', 'lançamentos', txns, txns * 420, txns * 120],
    ['ai_message', 'agente', 340, 900_000, 40_000],
    ['category', '', rows('category').length, 32_768, 8_192],
  ] as const;
  return {
    path: '~/Library/Application Support/Treasury/treasury.db',
    file_bytes: fileBytes,
    page_size: 4096,
    pages: Math.round(fileBytes / 4096),
    free_bytes: 7_340_032,
    sqlite_version: '3.50.4',
    cipher_version: '4.10.0 community',
    schema_version: 96,
    change_sets: purged.has('change_history') ? 60 : 300,
    oldest_change: purged.has('change_history') ? '2026-04-18T10:00:00Z' : '2025-01-02T12:00:00Z',
    tables: tables
      .map(([name, group, r, bytes, index_bytes]) => ({ name, group, rows: r, bytes, index_bytes }))
      .sort((a, b) => b.bytes - a.bytes),
    last_maintenance_at: lastMaintenance,
  };
}

function kindOf(args: Record<string, unknown>): PurgeKind {
  const k = str(args, 'kind') as PurgeKind;
  if (!(k in LABEL)) invalid(`argumentos inválidos: unknown variant \`${k}\``);
  return k;
}

function preview(args: Record<string, unknown>): PurgePreview {
  const kind = kindOf(args);
  const [label, consequence, n] = LABEL[kind];
  const days = typeof args.older_than_days === 'number' ? args.older_than_days : 180;
  const rowsGone = purged.has(kind) ? 0 : Math.round(n * Math.min(1, 180 / Math.max(days, 1)));
  return {
    kind,
    label,
    consequence,
    cutoff:
      kind === 'research_data' ? null : new Date(Date.now() - days * 86_400_000).toISOString(),
    rows: rowsGone,
    bytes: rowsGone * 220,
  };
}

/** As primeiras linhas de uma tabela do livro-caixa do mock (as outras vêm vazias). */
function tableRows(args: Record<string, unknown>): TableRows {
  const table = str(args, 'table');
  const known = info().tables.find((t) => t.name === table);
  const store = (
    db.t as unknown as Record<string, Map<string, Record<string, unknown>> | undefined>
  )[table];
  if (!known && !store) notFound(`tabela ${table}`);
  const limit = Math.min(Math.max(Number(args.limit ?? 100), 1), 100);
  const all = store ? [...store.values()] : [];
  const names = all[0] ? Object.keys(all[0]) : [];
  const show = (v: unknown): string | number | null =>
    v == null
      ? null
      : typeof v === 'number'
        ? v
        : typeof v === 'boolean'
          ? Number(v)
          : typeof v === 'string'
            ? v.length > 300
              ? `${v.slice(0, 300)}…`
              : v
            : JSON.stringify(v);
  return {
    table,
    group: known?.group ?? '',
    columns: names.map((name) => ({
      name,
      decl_type: '',
      primary_key: name === 'id',
      masked: false,
    })),
    rows: all.slice(0, limit).map((r) => names.map((n) => show(r[n]))),
    total_rows: store ? all.length : 0,
    order: 'rowid',
    notes: [
      ...(all.length > limit ? [`mostrando as ${limit} primeiras de ${all.length} linhas`] : []),
      ...(store ? [] : ['o modo de demonstração não guarda as linhas desta tabela']),
    ],
  };
}

registerMockHandlers({
  'database.table_rows': (args) => {
    ready();
    return tableRows(args);
  },
  'database.info': (): DatabaseInfo => {
    ready();
    return info();
  },
  'database.purge_preview': (args) => {
    ready();
    return preview(args);
  },
  'database.purge': (args): PurgeResult => {
    ready();
    const p = preview(args);
    const before = fileBytes;
    purged.add(p.kind);
    fileBytes = Math.max(40_000_000, fileBytes - p.bytes);
    lastMaintenance = new Date().toISOString();
    return {
      kind: p.kind,
      rows: p.rows,
      backup_id: 'bkp-mock-purge',
      file_bytes_before: before,
      file_bytes_after: fileBytes,
    };
  },
  'database.run': (args): MaintenanceResult => {
    ready();
    const action = str(args, 'action') as MaintenanceResult['action'];
    const before = fileBytes;
    if (action === 'vacuum') fileBytes -= 7_340_032;
    lastMaintenance = new Date().toISOString();
    return {
      action,
      ok: true,
      messages: action === 'integrity_check' ? ['ok'] : [],
      file_bytes_before: before,
      file_bytes_after: fileBytes,
      millis: 420,
    };
  },
});
