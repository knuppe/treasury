/**
 * MCPs das suas CLIs: os servidores MCP que as CLIs instaladas carregariam por conta própria
 * (do Claude, do Cursor, do Grok, do Codex, do Gemini e do OpenCode). Por padrão nenhum entra
 * nas conversas do agente deste cofre, só o Treasury e os MCPs extras; a pessoa libera um a um.
 * Os liberados são chamados pela CLI direto, fora da confirmação e da auditoria do Treasury.
 */
import type { CliUserMcp } from '@/api/generated/CliUserMcp';
import { useCliMcps, useProviders, useSetCliMcps } from '@/api/hooks/ai';
import { Icon, InstrumentCard, toast } from '@/design/components';
import { ErrorBlock, LoadingBlock, errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';

import { Mono, Toggle } from '../ui';

export function CliMcpsPanel({ index }: { index: string }) {
  const q = useCliMcps();
  const providers = useProviders();
  const set = useSetCliMcps();
  const label = (p: string) => providers.data?.find((x) => x.provider === p)?.label ?? p;
  const list = q.data ?? [];
  const toggle = (m: CliUserMcp, allowed: boolean) => {
    const next = list.filter((x) => (x.name === m.name ? allowed : x.allowed)).map((x) => x.name);
    set.mutate(next, {
      onError: (e) =>
        toast({ tone: 'error', title: t('agente-nao-alterou'), description: errorMessage(e) }),
    });
  };
  return (
    <InstrumentCard
      index={index}
      title={t('agente-climcp-titulo')}
      padded={false}
      footer={
        <div className="flex flex-col gap-1.5">
          <p className="text-[12px] text-dim">{t('agente-climcp-explica')}</p>
          <p className="flex items-start gap-2 text-[12px] text-warning" role="note">
            <Icon name="alert" size={12} className="mt-0.5 shrink-0" />
            {t('agente-climcp-seguranca')}
          </p>
        </div>
      }
    >
      {q.isLoading ? (
        <LoadingBlock />
      ) : q.error ? (
        <ErrorBlock error={q.error} className="m-3" onRetry={() => void q.refetch()} />
      ) : list.length === 0 ? (
        <p className="px-4 py-6 text-center text-[13px] text-dim">{t('agente-climcp-vazio')}</p>
      ) : (
        <ul aria-label={t('agente-climcp-lista')} className="divide-y divide-line">
          {list.map((m) => (
            <li key={m.name} className="flex flex-wrap items-center gap-3 px-4 py-2.5">
              <Mono>{m.name}</Mono>
              <span className="text-[12px] text-dim">
                {m.clis.length ? m.clis.map(label).join(', ') : t('agente-climcp-nao-encontrado')}
              </span>
              <span className="flex-1" />
              <Toggle
                label={t('agente-climcp-liberar', { nome: m.name })}
                checked={m.allowed}
                disabled={set.isPending}
                onChange={(on) => toggle(m, on)}
              />
            </li>
          ))}
        </ul>
      )}
    </InstrumentCard>
  );
}
