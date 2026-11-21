/**
 * 11.15 · Banco de dados: tamanho do arquivo cifrado, maiores tabelas, rotinas de
 * manutenção (integridade, estatísticas, otimização, compactação) e expurgo do que não é
 * dado da família, sempre com prévia e backup verificado antes de apagar.
 */
import { useMemo, useState } from 'react';

import type { MaintenanceAction } from '@/api/generated/MaintenanceAction';
import type { MaintenanceResult } from '@/api/generated/MaintenanceResult';
import type { PurgeKind } from '@/api/generated/PurgeKind';
import type { PurgePreview } from '@/api/generated/PurgePreview';
import type { TableSize } from '@/api/generated/TableSize';
import { purgePreview, useDatabaseInfo, useMaintenance, usePurge } from '@/api/hooks/database';
import {
  Button,
  InstrumentCard,
  KeyValue,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDateTime, formatNumber } from '@/lib/format';
import { ConfirmDialog, InlineError, QueryState } from '@/features/transactions/ui/controls';
import { formatBytes } from '@/features/transactions/ui/hooks';

import { TableViewer } from './TableViewer';

const ACTIONS: { action: MaintenanceAction; readonly label: string; readonly hint: string }[] = [
  {
    action: 'integrity_check',
    get label() {
      return t('ajustes-banco-integridade');
    },
    get hint() {
      return t('ajustes-banco-integridade-dica');
    },
  },
  {
    action: 'analyze',
    get label() {
      return t('ajustes-banco-estatisticas');
    },
    get hint() {
      return t('ajustes-banco-estatisticas-dica');
    },
  },
  {
    action: 'optimize',
    get label() {
      return t('ajustes-banco-otimizar');
    },
    get hint() {
      return t('ajustes-banco-otimizar-dica');
    },
  },
  {
    action: 'vacuum',
    get label() {
      return t('ajustes-banco-compactar');
    },
    get hint() {
      return t('ajustes-banco-compactar-dica');
    },
  },
];

const PURGES: { kind: PurgeKind; readonly label: string; days: boolean }[] = [
  {
    kind: 'change_history',
    get label() {
      return t('ajustes-banco-expurgo-historico');
    },
    days: true,
  },
  {
    kind: 'ai_audit',
    get label() {
      return t('ajustes-banco-expurgo-auditoria');
    },
    days: true,
  },
  {
    kind: 'ai_conversations',
    get label() {
      return t('ajustes-banco-expurgo-conversas');
    },
    days: true,
  },
  {
    kind: 'market_cache',
    get label() {
      return t('ajustes-banco-expurgo-caches');
    },
    days: true,
  },
  {
    kind: 'research_data',
    get label() {
      return t('ajustes-banco-expurgo-analise');
    },
    days: false,
  },
];

const DAYS = [30, 90, 180, 365, 730];

/** Colunas da tabela das maiores tabelas (os títulos saem no idioma atual). */
const tableColumns = (): TechColumn<TableSize>[] => [
  {
    key: 'name',
    header: t('ajustes-banco-col-tabela'),
    cell: (row) => <span className="font-mono">{row.name}</span>,
  },
  { key: 'group', header: t('ajustes-banco-col-guarda'), cell: (row) => row.group || '—' },
  {
    key: 'rows',
    header: t('ajustes-linhas'),
    numeric: true,
    sortValue: (row) => row.rows,
    cell: (row) => formatNumber(row.rows),
  },
  {
    key: 'bytes',
    header: t('ajustes-tamanho'),
    numeric: true,
    sortValue: (row) => row.bytes,
    cell: (row) => formatBytes(row.bytes),
  },
  {
    key: 'index',
    header: t('ajustes-banco-col-indices'),
    numeric: true,
    sortValue: (row) => row.index_bytes,
    cell: (row) => formatBytes(row.index_bytes),
  },
];

function MaintenanceReport({ r }: { r: MaintenanceResult }) {
  const label = ACTIONS.find((a) => a.action === r.action)?.label ?? r.action;
  return (
    <div
      role="status"
      className={
        'rounded-control border px-3 py-2 text-[12.5px] ' +
        (r.ok ? 'border-positive/40 text-fg' : 'border-negative/50 text-negative')
      }
    >
      <p>
        {t(r.ok ? 'ajustes-banco-relatorio-ok' : 'ajustes-banco-relatorio-problemas', {
          acao: label,
          segundos: r.millis / 1000,
          tamanho:
            r.file_bytes_after !== r.file_bytes_before
              ? ` · ${formatBytes(r.file_bytes_before)} → ${formatBytes(r.file_bytes_after)}`
              : '',
        })}
      </p>
      {!r.ok && r.messages.length ? (
        <ul className="mt-1 list-disc pl-5">
          {r.messages.slice(0, 8).map((m) => (
            <li key={m}>{m}</li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}

function PurgeRow({ kind, label, days }: { kind: PurgeKind; label: string; days: boolean }) {
  const [olderThan, setOlderThan] = useState(180);
  const [preview, setPreview] = useState<PurgePreview | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [confirm, setConfirm] = useState(false);
  const purge = usePurge();
  const args = { kind, ...(days ? { older_than_days: olderThan } : {}) };
  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      setPreview(await purgePreview(args));
    } catch (e) {
      setError(e);
    } finally {
      setLoading(false);
    }
  };
  return (
    <li className="flex flex-col gap-2 border-t border-line pt-3">
      <div className="flex flex-wrap items-center gap-3">
        <span className="min-w-[16rem] flex-1 text-[13.5px] text-fg">{label}</span>
        {days ? (
          <div className="w-44">
            <Select
              aria-label={t('ajustes-banco-idade', { item: label })}
              value={String(olderThan)}
              onValueChange={(v) => {
                setOlderThan(Number(v));
                setPreview(null);
              }}
              options={DAYS.map((d) => ({
                value: String(d),
                label: t('ajustes-banco-mais-de-dias', { n: d }),
              }))}
            />
          </div>
        ) : null}
        <Button size="sm" variant="ghost" loading={loading} onClick={() => void load()}>
          {t('ajustes-banco-previa')}
        </Button>
      </div>
      {preview ? (
        <div className="flex flex-wrap items-center gap-3 pl-1 text-[12.5px] text-dim">
          <span>
            {preview.rows === 0
              ? t('ajustes-banco-nada')
              : t('ajustes-banco-previa-texto', {
                  n: preview.rows,
                  tamanho: formatBytes(preview.bytes),
                  consequencia: preview.consequence,
                })}
          </span>
          {preview.rows > 0 ? (
            <Button size="sm" variant="danger" onClick={() => setConfirm(true)}>
              {t('comum-apagar')}
            </Button>
          ) : null}
        </div>
      ) : null}
      <InlineError error={error} />
      <ConfirmDialog
        open={confirm}
        onOpenChange={setConfirm}
        title={t('ajustes-banco-apagar-titulo', { item: label })}
        description={
          preview
            ? t('ajustes-banco-apagar-texto', {
                n: preview.rows,
                consequencia: preview.consequence,
              })
            : undefined
        }
        confirmLabel={t('ajustes-banco-apagar-confirmar')}
        pending={purge.isPending}
        error={purge.error}
        onConfirm={() =>
          purge.mutate(args, {
            onSuccess: (r) => {
              setConfirm(false);
              setPreview(null);
              toast({
                tone: 'success',
                title: t('ajustes-banco-apagados', { n: r.rows }),
                description: t('ajustes-banco-apagados-dica', {
                  antes: formatBytes(r.file_bytes_before),
                  depois: formatBytes(r.file_bytes_after),
                }),
              });
            },
          })
        }
      />
    </li>
  );
}

export function DatabaseSection() {
  const { data, isLoading, error } = useDatabaseInfo();
  const run = useMaintenance();
  const [report, setReport] = useState<MaintenanceResult | null>(null);
  const [all, setAll] = useState(false);
  /** Tabela aberta com clique duplo (as primeiras linhas, só leitura). */
  const [viewing, setViewing] = useState<string | null>(null);
  const tables = data?.tables ?? [];
  const columns = useMemo(() => tableColumns(), []);
  if (viewing) return <TableViewer table={viewing} onBack={() => setViewing(null)} />;
  return (
    <div className="grid gap-5 @min-[1040px]:grid-cols-2">
      <InstrumentCard index="12.15" title={t('ajustes-secao-banco')}>
        <QueryState loading={isLoading} error={error}>
          {data ? (
            <div className="flex flex-col gap-4">
              <KeyValue
                items={[
                  { label: t('ajustes-arquivo'), value: formatBytes(data.file_bytes) },
                  {
                    label: t('ajustes-banco-espaco-livre'),
                    value: `${formatBytes(data.free_bytes)} (${data.pages ? Math.round((100 * data.free_bytes) / (data.pages * data.page_size)) : 0}%)`,
                  },
                  {
                    label: t('ajustes-banco-cifra'),
                    value: data.cipher_version ? `SQLCipher ${data.cipher_version}` : '—',
                  },
                  { label: 'SQLite', value: data.sqlite_version },
                  { label: t('ajustes-banco-versao-esquema'), value: `V${data.schema_version}` },
                  {
                    label: t('ajustes-secao-historico'),
                    value: data.oldest_change
                      ? t('ajustes-banco-conjuntos-desde', {
                          n: data.change_sets,
                          data: formatDateTime(data.oldest_change),
                        })
                      : t('ajustes-banco-conjuntos', { n: data.change_sets }),
                  },
                  {
                    label: t('ajustes-banco-ultima-manutencao'),
                    value: data.last_maintenance_at
                      ? formatDateTime(data.last_maintenance_at)
                      : t('ajustes-nunca'),
                  },
                ]}
              />
              <p className="break-all font-mono text-[11.5px] text-dim">{data.path}</p>
            </div>
          ) : null}
        </QueryState>
      </InstrumentCard>
      <InstrumentCard index="12.16" title={t('ajustes-banco-manutencao')}>
        <div className="flex flex-col gap-3">
          <ul className="flex flex-col gap-2">
            {ACTIONS.map((a) => (
              <li key={a.action} className="flex items-center justify-between gap-4">
                <div className="flex min-w-0 flex-col">
                  <span className="text-[13.5px] text-fg">{a.label}</span>
                  <span className="text-[12px] text-dim">{a.hint}</span>
                </div>
                <Button
                  size="sm"
                  variant="secondary"
                  loading={run.isPending && run.variables?.action === a.action}
                  disabled={run.isPending}
                  onClick={() => run.mutate({ action: a.action }, { onSuccess: setReport })}
                >
                  {t('ajustes-banco-rodar')}
                </Button>
              </li>
            ))}
          </ul>
          {report ? <MaintenanceReport r={report} /> : null}
          <InlineError error={run.error} />
        </div>
      </InstrumentCard>
      <div className="@min-[1040px]:col-span-2">
        <InstrumentCard
          index="12.17"
          title={t('ajustes-banco-maiores')}
          actions={
            tables.length > 12 ? (
              <Button size="sm" variant="ghost" onClick={() => setAll((v) => !v)}>
                {all
                  ? t('ajustes-banco-so-maiores')
                  : t('ajustes-banco-todas', { n: tables.length })}
              </Button>
            ) : undefined
          }
        >
          <TechTable
            label={t('ajustes-banco-maiores-rotulo')}
            prefsId="Maiores tabelas do banco"
            columns={columns}
            rows={all ? tables : tables.slice(0, 12)}
            getRowId={(t) => t.name}
            onActivate={(t) => setViewing(t.name)}
          />
          <p className="px-1 pt-2 text-[12px] text-dim">{t('ajustes-banco-clique-duplo')}</p>
        </InstrumentCard>
      </div>
      <div className="@min-[1040px]:col-span-2">
        <InstrumentCard index="12.18" title={t('ajustes-banco-expurgo')}>
          <div className="flex flex-col gap-2">
            <p className="max-w-[80ch] text-[13px] leading-relaxed text-dim">
              {t('ajustes-banco-expurgo-texto')}
            </p>
            <ul className="flex flex-col gap-3">
              {PURGES.map((p) => (
                <PurgeRow key={p.kind} kind={p.kind} label={p.label} days={p.days} />
              ))}
            </ul>
          </div>
        </InstrumentCard>
      </div>
    </div>
  );
}
