/**
 * MCPs extras deste cofre: servidores MCP de fora (um comando local ou um endereço HTTP)
 * que o agente deste cofre pode usar. Cada servidor liga e desliga, pode permitir as
 * chamadas sem perguntar, ser testado (conecta e lista as ferramentas), editado e excluído.
 * Os valores secretos nunca vêm do núcleo: só "cadastrado" e os 4 últimos caracteres.
 */
import { useState } from 'react';

import type { McpServer } from '@/api/generated/McpServer';
import type { McpTestResult } from '@/api/generated/McpTestResult';
import type { McpVar } from '@/api/generated/McpVar';
import { useDeleteMcpExtra, useMcpExtras, useSaveMcpExtra, useTestMcpExtra } from '@/api/hooks/ai';
import { Button, Icon, InstrumentCard, toast } from '@/design/components';
import {
  ConfirmButton,
  ErrorBlock,
  LoadingBlock,
  errorMessage,
} from '@/features/portfolio/components/States';
import { t } from '@/i18n';

import { Mono, Pill, Spec, Toggle } from '../ui';
import { McpExtraDialog, maskedSecret, serverToInput } from './McpExtraDialog';

type TestState = { ok: true; result: McpTestResult } | { ok: false; error: string };

const varText = (v: McpVar) =>
  `${v.key}=${v.secret ? (v.set ? maskedSecret(v.hint) : '—') : (v.value ?? '')}`;

function TestOutcome({ state }: { state: TestState }) {
  if (!state.ok)
    return (
      <p role="status" className="text-[12px] text-negative">
        {`× ${state.error}`}
      </p>
    );
  const { tools, server_info: info } = state.result;
  return (
    <div role="status" className="flex flex-col gap-1.5">
      <p className="text-[12px] text-positive">
        {tools.length === 0
          ? t('agente-mcpx-sem-ferramentas')
          : t('agente-mcpx-ferramentas-encontradas', { n: tools.length })}
        {info ? <span className="text-dim">{` · ${info}`}</span> : null}
      </p>
      {tools.length > 0 ? (
        <ul
          aria-label={t('agente-mcpx-ferramentas')}
          className="flex flex-col gap-1 rounded-control border border-line px-3 py-2"
        >
          {tools.map((tool) => (
            <li key={tool.name} className="flex flex-wrap items-baseline gap-x-2 text-[12px]">
              <Mono>{tool.name}</Mono>
              {tool.description ? <span className="text-dim">{tool.description}</span> : null}
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}

function ServerRow({ server, onEdit }: { server: McpServer; onEdit: () => void }) {
  const save = useSaveMcpExtra();
  const del = useDeleteMcpExtra();
  const test = useTestMcpExtra();
  const [outcome, setOutcome] = useState<TestState | null>(null);
  const vars = server.transport === 'stdio' ? server.env : server.headers;
  const toggle = (patch: { enabled?: boolean; auto_approve?: boolean }) =>
    save.mutate(
      { ...serverToInput(server), ...patch },
      {
        onError: (e) =>
          toast({ tone: 'error', title: t('agente-nao-alterou'), description: errorMessage(e) }),
      },
    );
  return (
    <li
      aria-label={t('agente-mcpx-linha', { nome: server.label })}
      className="flex flex-col gap-2 px-4 py-3"
    >
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-[14px] font-medium text-fg">{server.label}</span>
        <Pill tone={server.transport === 'stdio' ? 'accent' : 'warning'}>
          {server.transport === 'stdio' ? t('agente-mcpx-tipo-stdio') : t('agente-mcpx-tipo-http')}
        </Pill>
        <Spec label={t('agente-mcpx-ferramentas-prefixo')}>{`${server.name}-*`}</Spec>
        <span className="flex-1" />
        <Toggle
          label={t('controles-ligado')}
          checked={server.enabled}
          disabled={save.isPending}
          onChange={(enabled) => toggle({ enabled })}
        />
        <Toggle
          label={t('agente-mcpx-permitir-sem-perguntar')}
          checked={server.auto_approve}
          disabled={save.isPending}
          onChange={(auto_approve) => toggle({ auto_approve })}
        />
      </div>
      <Mono className="text-dim">
        {server.transport === 'stdio'
          ? [server.command ?? '', ...server.args].join(' ')
          : (server.url ?? '')}
      </Mono>
      {vars.length > 0 ? (
        <ul className="flex flex-wrap gap-x-4 gap-y-1">
          {vars.map((v) => (
            <li key={v.key}>
              <Mono className="text-dim">{varText(v)}</Mono>
            </li>
          ))}
        </ul>
      ) : null}
      <div className="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant="secondary"
          loading={test.isPending}
          icon={<Icon name="reticle" size={12} />}
          onClick={() => {
            setOutcome(null);
            test.mutate(server.id, {
              onSuccess: (result) => setOutcome({ ok: true, result }),
              onError: (e) => setOutcome({ ok: false, error: errorMessage(e) }),
            });
          }}
        >
          {t('agente-testar')}
        </Button>
        <Button size="sm" variant="ghost" onClick={onEdit}>
          {t('comum-editar')}
        </Button>
        <ConfirmButton
          label={t('agente-excluir')}
          title={t('agente-mcpx-excluir', { nome: server.label })}
          description={t('agente-mcpx-excluir-texto')}
          confirmLabel={t('agente-excluir')}
          pending={del.isPending}
          onConfirm={() =>
            del.mutateAsync(server.id).catch((e: unknown) =>
              toast({
                tone: 'error',
                title: t('agente-mcpx-nao-excluiu'),
                description: errorMessage(e),
              }),
            )
          }
        />
        {test.isPending ? (
          <span className="text-[12px] text-dim">{t('agente-mcpx-testando')}</span>
        ) : null}
      </div>
      {outcome && !test.isPending ? <TestOutcome state={outcome} /> : null}
    </li>
  );
}

export function McpExtrasPanel({ index = '07' }: { index?: string }) {
  const servers = useMcpExtras();
  const [dialog, setDialog] = useState<{ server?: McpServer } | null>(null);
  const list = servers.data ?? [];
  return (
    <InstrumentCard
      index={index}
      title={t('agente-mcpx-titulo')}
      padded={false}
      actions={
        <Button
          size="sm"
          variant="secondary"
          icon={<Icon name="plus" size={12} />}
          onClick={() => setDialog({})}
        >
          {t('agente-mcpx-adicionar')}
        </Button>
      }
      footer={
        <div className="flex flex-col gap-1.5">
          <p className="text-[12px] text-dim">{t('agente-mcpx-explica')}</p>
          <p className="flex items-start gap-2 text-[12px] text-warning" role="note">
            <Icon name="alert" size={12} className="mt-0.5 shrink-0" />
            {t('agente-mcpx-seguranca')}
          </p>
        </div>
      }
    >
      {servers.isLoading ? (
        <LoadingBlock />
      ) : servers.error ? (
        <ErrorBlock error={servers.error} className="m-3" onRetry={() => void servers.refetch()} />
      ) : list.length === 0 ? (
        <p className="px-4 py-6 text-center text-[13px] text-dim">{t('agente-mcpx-vazio')}</p>
      ) : (
        <ul aria-label={t('agente-mcpx-lista')} className="divide-y divide-line">
          {list.map((s) => (
            <ServerRow key={s.id} server={s} onEdit={() => setDialog({ server: s })} />
          ))}
        </ul>
      )}
      {dialog ? (
        <McpExtraDialog
          key={dialog.server?.id ?? 'new'}
          open
          onOpenChange={(o) => {
            if (!o) setDialog(null);
          }}
          {...(dialog.server ? { server: dialog.server } : {})}
        />
      ) : null}
    </InstrumentCard>
  );
}
