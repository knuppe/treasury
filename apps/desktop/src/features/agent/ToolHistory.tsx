/**
 * Histórico de ferramentas: cada chamada ao MCP do app (agente interno, CLIs locais e
 * clientes externos), do `ai_audit_log`, e os jobs recentes do agente.
 */
import { useMemo, useState } from 'react';

import type { AuditEntry } from '@/api/generated/AuditEntry';
import type { JobInfo } from '@/api/generated/JobInfo';
import { useAuditLog, useJobs } from '@/api/hooks/ai';
import {
  Input,
  InstrumentCard,
  Private,
  Select,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { ErrorBlock, LoadingBlock } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { JOB_STATUS_LABEL, TASK_LABEL, formatDuration } from './format';
import { Mono, Pill, Toggle } from './ui';

const ALL = '__all__';

function paramsText(p: Record<string, unknown>): string {
  const s = JSON.stringify(p);
  return s === '{}' ? '—' : s;
}

const auditColumns = (): TechColumn<AuditEntry>[] => [
  {
    key: 'created_at',
    header: t('agente-col-quando'),
    width: '140px',
    cell: (e) => <span className="num text-[11.5px]">{formatDateTime(e.created_at)}</span>,
    sortValue: (e) => e.created_at,
  },
  { key: 'client', header: t('agente-col-cliente'), width: '22%', sortValue: (e) => e.client },
  {
    key: 'tool',
    header: t('agente-col-ferramenta'),
    width: '18%',
    cell: (e) => <span className="font-mono text-[12px]">{e.tool}</span>,
    sortValue: (e) => e.tool,
  },
  {
    key: 'params',
    header: t('agente-col-parametros'),
    cell: (e) => (
      <Private>
        <Mono className="text-dim">{paramsText(e.params)}</Mono>
      </Private>
    ),
  },
  {
    key: 'ok',
    header: t('agente-col-resultado'),
    width: '150px',
    cell: (e) =>
      e.ok ? (
        <Pill tone="positive">{`✓ ${t('agente-ok')}`}</Pill>
      ) : (
        <span className="flex flex-col gap-0.5">
          <Pill tone="negative">{`× ${t('agente-erro')}`}</Pill>
          {e.error ? <span className="text-[11px] text-negative">{e.error}</span> : null}
        </span>
      ),
    sortValue: (e) => (e.ok ? 1 : 0),
  },
  {
    key: 'duration_ms',
    header: t('agente-col-duracao'),
    numeric: true,
    width: '90px',
    cell: (e) => formatDuration(e.duration_ms),
    sortValue: (e) => e.duration_ms,
  },
];

const jobColumns = (): TechColumn<JobInfo>[] => [
  {
    key: 'created_at',
    header: t('agente-col-inicio'),
    width: '140px',
    cell: (j) => <span className="num text-[11.5px]">{formatDateTime(j.created_at)}</span>,
  },
  {
    key: 'kind',
    header: t('agente-tipo'),
    cell: (j) => (j.kind === 'chat' ? t('agente-aba-conversa') : t('casca-rotina')),
  },
  { key: 'task', header: t('agente-col-tarefa'), cell: (j) => TASK_LABEL[j.task] },
  { key: 'engine_name', header: t('agente-motor'), cell: (j) => j.engine_name ?? '—' },
  {
    key: 'status',
    header: t('agente-estado'),
    cell: (j) => (
      <Pill
        tone={
          j.status === 'done'
            ? 'positive'
            : j.status === 'failed'
              ? 'negative'
              : j.status === 'running'
                ? 'accent'
                : 'dim'
        }
      >
        {JOB_STATUS_LABEL[j.status]}
      </Pill>
    ),
  },
];

export function ToolHistory() {
  const [onlyErrors, setOnlyErrors] = useState(false);
  const [client, setClient] = useState(ALL);
  const [tool, setTool] = useState('');
  const audit = useAuditLog({ limit: 500, ...(onlyErrors ? { only_errors: true } : {}) });
  const jobs = useJobs();

  const entries = audit.data ?? [];
  const clients = useMemo(
    () => [...new Set((audit.data ?? []).map((e) => e.client))].sort(),
    [audit.data],
  );
  const shown = entries.filter(
    (e) =>
      (client === ALL || e.client === client) &&
      (!tool.trim() || e.tool.toLowerCase().includes(tool.trim().toLowerCase())),
  );

  return (
    <div className="flex flex-col gap-4">
      <InstrumentCard
        index="01"
        title={t('agente-ferramentas-titulo')}
        padded={false}
        footer={[
          { label: t('agente-fonte'), value: 'ai_audit_log' },
          { label: t('agente-chamadas'), value: String(shown.length) },
        ]}
      >
        <div className="flex flex-wrap items-center gap-3 border-b border-line px-4 py-2.5">
          <div className="w-[240px]">
            <Select
              aria-label={t('agente-col-cliente')}
              value={client}
              onValueChange={setClient}
              options={[
                { value: ALL, label: t('agente-todos-clientes') },
                ...clients.map((c) => ({ value: c, label: c })),
              ]}
            />
          </div>
          <div className="w-[200px]">
            <Input
              aria-label={t('agente-col-ferramenta')}
              placeholder={t('agente-col-ferramenta')}
              value={tool}
              onChange={(e) => setTool(e.target.value)}
            />
          </div>
          <Toggle label={t('agente-so-erros')} checked={onlyErrors} onChange={setOnlyErrors} />
        </div>
        {audit.isLoading ? (
          <LoadingBlock />
        ) : audit.error ? (
          <ErrorBlock error={audit.error} className="m-3" />
        ) : (
          <TechTable
            label={t('agente-historico-ferramentas')}
            prefsId="Histórico de ferramentas"
            columns={auditColumns()}
            rows={shown}
            getRowId={(e) => e.id}
            defaultSort={{ key: 'created_at', direction: 'desc' }}
            maxHeight="520px"
            dense
            empty={t('agente-nenhuma-chamada')}
          />
        )}
      </InstrumentCard>
      <InstrumentCard index="02" title={t('agente-jobs-titulo')} padded={false}>
        {jobs.isLoading ? (
          <LoadingBlock />
        ) : (
          <TechTable
            label={t('agente-jobs')}
            prefsId="Jobs do agente"
            columns={jobColumns()}
            rows={jobs.data ?? []}
            getRowId={(j) => j.job_id}
            dense
            empty={t('agente-jobs-vazio')}
          />
        )}
      </InstrumentCard>
    </div>
  );
}
