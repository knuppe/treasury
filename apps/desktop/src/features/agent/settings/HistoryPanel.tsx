/**
 * Retenção do que o agente guarda no cofre: anexos (padrão e recomendado: não guardar, só o
 * nome e o tamanho; guardar engorda o banco cifrado) e conversas (faxina diária das não
 * fixadas sem uso há N dias, com `VACUUM` depois de limpezas grandes). Mostra o tamanho
 * atual e tem "Limpar agora".
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { AiSettingsPatch } from '@/api/generated/AiSettingsPatch';
import type { CleanupReport } from '@/api/generated/CleanupReport';
import type { HistoryStats } from '@/api/generated/HistoryStats';
import type { Retention } from '@/api/generated/Retention';
import { aiKeys, useAiSettings, useSetAiSettings } from '@/api/hooks/ai';
import { Button, Field, Icon, InstrumentCard, KeyValue, toast } from '@/design/components';
import { ErrorBlock, LoadingBlock, errorMessage } from '@/features/portfolio/components/States';
import { Segmented } from '@/features/transactions/ui/controls';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { t } from '@/i18n';
import { formatDateTime, formatNumber } from '@/lib/format';

import { labels } from '../format';

export const historyStatsKey = ['ai', 'history_stats'] as const;

const RETENTION_HINT: Record<Retention, string> = labels({
  none: 'agente-ret-nenhum-dica',
  days7: 'agente-ret-7-dica',
  always: 'agente-ret-sempre-dica',
});

const DAYS = ['30', '90', '180', '365', '0'] as const;
type Days = (typeof DAYS)[number];

function daysLabel(d: Days): string {
  if (d === '365') return t('agente-1-ano');
  if (d === '0') return t('agente-nunca');
  return t('agente-n-dias', { n: Number(d) });
}

function asDays(n: number): Days {
  return (DAYS as readonly string[]).includes(String(n)) ? (String(n) as Days) : '90';
}

export function HistoryPanel({ index = '07' }: { index?: string }) {
  const settings = useAiSettings();
  const set = useSetAiSettings();
  const client = useQueryClient();
  const stats = useQuery({
    queryKey: historyStatsKey,
    queryFn: () => call<HistoryStats>('ai.history.stats'),
  });
  const cleanup = useMutation({
    mutationFn: () => call<CleanupReport>('ai.history.cleanup'),
    onSuccess: (r) => {
      void client.invalidateQueries({ queryKey: historyStatsKey });
      void client.invalidateQueries({ queryKey: aiKeys.conversations });
      const freed = Math.max(0, r.db_bytes_before - r.db_bytes_after);
      const counts = { conversas: r.conversations_deleted, anexos: r.attachments_expired };
      toast({
        tone: 'success',
        title: t('agente-faxina-feita'),
        description: r.vacuumed
          ? t('agente-faxina-texto-encolheu', { ...counts, tamanho: formatBytes(freed) })
          : t('agente-faxina-texto', counts),
      });
    },
    onError: (e) =>
      toast({ tone: 'error', title: t('agente-nao-limpou'), description: errorMessage(e) }),
  });
  const apply = (patch: AiSettingsPatch) =>
    set.mutate(patch, {
      onSuccess: () => void client.invalidateQueries({ queryKey: historyStatsKey }),
      onError: (e) =>
        toast({ tone: 'error', title: t('agente-nao-salvou'), description: errorMessage(e) }),
    });
  const s = settings.data;
  const st = stats.data;

  return (
    <InstrumentCard index={index} title={t('agente-hist-titulo')} bodyClassName="@container">
      {settings.isLoading ? (
        <LoadingBlock />
      ) : settings.error ? (
        <ErrorBlock error={settings.error} />
      ) : s ? (
        <div className="grid grid-cols-1 gap-6 @min-[680px]:grid-cols-2">
          <div className="flex flex-col gap-4">
            <Field
              label={t('agente-anexos-enviados')}
              hint={RETENTION_HINT[s.attachment_retention]}
            >
              <Segmented<Retention>
                label={t('agente-guardar-anexos')}
                fill
                value={s.attachment_retention}
                onChange={(v) => apply({ attachment_retention: v })}
                options={[
                  { value: 'none', label: t('agente-nao-guardar') },
                  { value: 'days7', label: t('agente-7-dias') },
                  { value: 'always', label: t('agente-sempre') },
                ]}
              />
            </Field>
            {s.attachment_retention === 'none' ? (
              <p className="flex items-start gap-1.5 text-[12px] text-positive">
                <Icon name="check" size={12} className="mt-0.5 shrink-0" />
                {t('agente-recomendado')}
              </p>
            ) : (
              <p role="alert" className="flex items-start gap-1.5 text-[12px] text-warning">
                <Icon name="alert" size={12} className="mt-0.5 shrink-0" />
                {st
                  ? t('agente-nao-recomendado-hoje', { tamanho: formatBytes(st.attachment_bytes) })
                  : t('agente-nao-recomendado')}
              </p>
            )}
            <Field
              label={t('agente-apagar-sem-uso')}
              hint={
                s.history_retention_days > 0 ? t('agente-faxina-diaria') : t('agente-faxina-nunca')
              }
            >
              <Segmented<Days>
                label={t('agente-retencao-conversas')}
                fill
                value={asDays(s.history_retention_days)}
                onChange={(v) => apply({ history_retention_days: Number(v) })}
                options={DAYS.map((d) => ({ value: d, label: daysLabel(d) }))}
              />
            </Field>
          </div>
          <div className="flex flex-col gap-3">
            {stats.error ? <ErrorBlock error={stats.error} /> : null}
            {st ? (
              <KeyValue
                items={[
                  {
                    label: t('agente-conversas'),
                    value: st.pinned
                      ? t('agente-conversas-fixadas', { n: st.conversations, fixadas: st.pinned })
                      : formatNumber(st.conversations),
                  },
                  { label: t('agente-mensagens'), value: formatNumber(st.messages) },
                  {
                    label: t('agente-anexos'),
                    value: t('agente-anexos-guardados', {
                      n: st.attachments,
                      guardados: st.stored_attachments,
                    }),
                  },
                  { label: t('agente-historico'), value: formatBytes(st.history_bytes) },
                  {
                    label: t('agente-cofre'),
                    value:
                      st.free_bytes > 0
                        ? t('agente-cofre-livres', {
                            tamanho: formatBytes(st.db_bytes),
                            livres: formatBytes(st.free_bytes),
                          })
                        : formatBytes(st.db_bytes),
                  },
                  {
                    label: t('agente-ultima-faxina'),
                    value: st.last_cleanup
                      ? formatDateTime(st.last_cleanup)
                      : t('agente-ainda-nao'),
                  },
                ]}
              />
            ) : stats.isLoading ? (
              <LoadingBlock />
            ) : null}
            <div className="flex flex-wrap items-center gap-2">
              <Button
                variant="secondary"
                loading={cleanup.isPending}
                icon={<Icon name="refresh" size={12} />}
                onClick={() => cleanup.mutate()}
              >
                {t('agente-limpar-agora')}
              </Button>
              <span className="text-[11.5px] text-dim">{t('agente-limpar-agora-dica')}</span>
            </div>
          </div>
        </div>
      ) : null}
    </InstrumentCard>
  );
}
