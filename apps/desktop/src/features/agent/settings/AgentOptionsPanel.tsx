/** Ajustes do agente: modo autônomo, histórico, clientes MCP externos e rotinas. */
import type { AiSettingsPatch } from '@/api/generated/AiSettingsPatch';
import { useAiSettings, useSetAiSettings } from '@/api/hooks/ai';
import { Field, InstrumentCard, toast } from '@/design/components';
import { ErrorBlock, LoadingBlock, errorMessage } from '@/features/portfolio/components/States';
import { Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

import { Toggle } from '../ui';

/** Opções do tempo-limite das respostas por CLI (minutos), como no núcleo. */
const CLI_MINUTES = ['15', '30', '60', '120'] as const;
type CliMinutes = (typeof CLI_MINUTES)[number];

export function AgentOptionsPanel({ index = '06' }: { index?: string }) {
  const settings = useAiSettings();
  const set = useSetAiSettings();
  const apply = (patch: AiSettingsPatch) =>
    set.mutate(patch, {
      onError: (e) =>
        toast({ tone: 'error', title: t('agente-nao-salvou'), description: errorMessage(e) }),
    });
  const s = settings.data;
  return (
    <InstrumentCard index={index} title={t('agente-opcoes-titulo')}>
      {settings.isLoading ? (
        <LoadingBlock />
      ) : settings.error ? (
        <ErrorBlock error={settings.error} />
      ) : s ? (
        <div className="grid grid-cols-1 gap-6 @min-[530px]:grid-cols-2">
          <fieldset className="flex flex-col gap-3">
            <legend className="label-caps mb-2">{t('agente-permissoes')}</legend>
            <Toggle
              label={t('agente-autonomo')}
              hint={t('agente-autonomo-dica')}
              checked={s.autonomous}
              disabled={set.isPending}
              onChange={(v) => apply({ autonomous: v })}
            />
            <Toggle
              label={t('agente-mcp-escrita')}
              hint={t('agente-mcp-escrita-dica')}
              checked={s.mcp_allow_writes}
              disabled={set.isPending}
              onChange={(v) => apply({ mcp_allow_writes: v })}
            />
            <Toggle
              label={t('agente-guardar-historico')}
              hint={t('agente-guardar-historico-dica')}
              checked={s.keep_history}
              disabled={set.isPending}
              onChange={(v) => apply({ keep_history: v })}
            />
            <Field label={t('agente-tempo-cli')} hint={t('agente-tempo-cli-dica')}>
              <Segmented<CliMinutes>
                label={t('agente-tempo-cli')}
                fill
                value={String(s.cli_timeout_minutes) as CliMinutes}
                onChange={(v) => apply({ cli_timeout_minutes: Number(v) })}
                options={CLI_MINUTES.map((m) => ({
                  value: m,
                  label: t('agente-tempo-cli-minutos', { n: Number(m) }),
                }))}
              />
            </Field>
          </fieldset>
          <fieldset className="flex flex-col gap-3">
            <legend className="label-caps mb-2">{t('agente-rotinas-legenda')}</legend>
            <Toggle
              label={t('agente-revisao-continua')}
              hint={t('agente-revisao-continua-dica')}
              checked={s.routines.review}
              disabled={set.isPending}
              onChange={(v) => apply({ routines: { review: v } })}
            />
            <Toggle
              label={t('agente-tarefa-fechamento')}
              hint={t('agente-fechamento-dica')}
              checked={s.routines.monthly_close}
              disabled={set.isPending}
              onChange={(v) => apply({ routines: { monthly_close: v } })}
            />
            <Toggle
              label={t('agente-aba-alertas')}
              hint={t('agente-alertas-dica')}
              checked={s.routines.alerts}
              disabled={set.isPending}
              onChange={(v) => apply({ routines: { alerts: v } })}
            />
          </fieldset>
        </div>
      ) : null}
    </InstrumentCard>
  );
}
