/**
 * Ajustes › Atualizações (docs/auto-update.md): o que o app faz sozinho com uma versão nova
 * (baixar e conferir, só avisar, ou não procurar), procurar agora, baixar a versão avisada e
 * instalar a versão já baixada e conferida, sempre com confirmação (o app reinicia e o cofre
 * pede o desbloqueio de novo).
 */
import { useEffect, useState } from 'react';

import type { UpdateMode } from '@/api/generated/UpdateMode';
import { useSaveUpdateSettings, useUpdateSettings } from '@/api/hooks/settings';
import { Button, InstrumentCard, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { listenApp } from '@/features/agent/tauriBridge';
import { ConfirmDialog, InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';
import { checkForUpdate, installUpdate, updateStatus, type UpdateStatus } from '@/lib/updates';

const MODES: readonly UpdateMode[] = ['download', 'notify', 'off'];

const MODE_LABEL: Record<UpdateMode, () => string> = {
  download: () => t('ajustes-atualizacao-modo-baixar'),
  notify: () => t('ajustes-atualizacao-modo-avisar'),
  off: () => t('ajustes-atualizacao-modo-desligado'),
};

const MODE_HINT: Record<UpdateMode, () => string> = {
  download: () => t('ajustes-atualizacao-modo-baixar-dica'),
  notify: () => t('ajustes-atualizacao-modo-avisar-dica'),
  off: () => t('ajustes-atualizacao-modo-desligado-dica'),
};

export function UpdatesSection() {
  const settings = useUpdateSettings();
  const save = useSaveUpdateSettings();
  const [status, setStatus] = useState<UpdateStatus | null>(null);
  const [busy, setBusy] = useState<'check' | 'download' | null>(null);
  const [ask, setAsk] = useState(false);
  const [installError, setInstallError] = useState<unknown>(null);
  useEffect(() => {
    void updateStatus().then(setStatus);
    const refresh = () => void updateStatus().then(setStatus);
    const offReady = listenApp('update://ready', refresh);
    const offAvailable = listenApp('update://available', refresh);
    return () => {
      offReady();
      offAvailable();
    };
  }, []);
  const mode = settings.data?.mode ?? 'download';
  const configured = status?.configured === true;
  const ready = status?.ready === true ? status.available : null;
  // Avisada e ainda não baixada (modo "só avisar", ou procura manual nesse modo).
  const waiting = !ready && status?.available ? status.available : null;
  const run = (kind: 'check' | 'download') => {
    setBusy(kind);
    void checkForUpdate(kind === 'download' ? true : undefined)
      .then((s) => {
        setStatus(s);
        if (kind === 'check' && s && !s.error && !s.available)
          toast({ tone: 'success', title: t('ajustes-atualizacao-em-dia') });
      })
      .finally(() => setBusy(null));
  };
  const notes = (ready ?? waiting)?.notes;
  return (
    <InstrumentCard index="12.U" title={t('ajustes-atualizacao-titulo')}>
      <div className="flex flex-col gap-3">
        <div
          role="radiogroup"
          aria-label={t('ajustes-atualizacao-modo')}
          className="flex flex-col gap-2"
        >
          {MODES.map((m) => {
            const on = mode === m;
            return (
              <button
                key={m}
                type="button"
                role="radio"
                aria-checked={on}
                disabled={!settings.data || save.isPending}
                onClick={() =>
                  !on &&
                  save.mutate(
                    { mode: m },
                    {
                      onSuccess: () =>
                        toast({
                          tone: m === 'off' ? 'warning' : 'success',
                          title: t('ajustes-atualizacao-modo-salvo', { modo: MODE_LABEL[m]() }),
                        }),
                    },
                  )
                }
                className={cx(
                  'flex flex-col gap-0.5 rounded-control border px-3.5 py-2.5 text-left transition-colors focus-visible:outline-2 focus-visible:outline-accent',
                  on ? 'border-accent bg-accent-soft' : 'border-line hover:bg-surface-2',
                )}
              >
                <span className={cx('text-[13.5px]', on ? 'text-accent' : 'text-fg')}>
                  {MODE_LABEL[m]()}
                </span>
                <span className="text-[12px] leading-snug text-dim">{MODE_HINT[m]()}</span>
              </button>
            );
          })}
        </div>
        <p className="text-[12.5px] leading-snug text-dim">
          {t('ajustes-atualizacao-privacidade')}
        </p>
        {configured ? (
          <p className="text-[12.5px] text-dim">
            {t('ajustes-atualizacao-versao', { versao: status.current })}
            {status.last_check
              ? ` · ${t('ajustes-atualizacao-ultima', { quando: formatDateTime(status.last_check) })}`
              : ''}
          </p>
        ) : (
          <p className="text-[12.5px] text-dim">{t('ajustes-atualizacao-indisponivel')}</p>
        )}
        {status?.error ? (
          <p className="text-[12.5px] text-negative">
            {t('ajustes-atualizacao-erro', { erro: status.error })}
          </p>
        ) : null}
        {ready || waiting ? (
          <div
            role="status"
            className="flex flex-col gap-2 rounded-control border border-accent/50 bg-accent-soft/40 px-3 py-2.5 text-[12.5px] text-fg"
          >
            <p className="font-medium">
              {ready
                ? t('ajustes-atualizacao-pronta', { versao: ready.version })
                : t('ajustes-atualizacao-disponivel', { versao: waiting?.version ?? '' })}
            </p>
            {notes ? <p className="leading-snug whitespace-pre-line">{notes}</p> : null}
            <div>
              {ready ? (
                <Button size="sm" variant="primary" onClick={() => setAsk(true)}>
                  {t('ajustes-atualizacao-instalar')}
                </Button>
              ) : (
                <Button
                  size="sm"
                  variant="primary"
                  disabled={status?.checking}
                  loading={busy === 'download'}
                  onClick={() => run('download')}
                >
                  {t('ajustes-atualizacao-baixar')}
                </Button>
              )}
            </div>
          </div>
        ) : null}
        <div>
          <Button
            size="sm"
            variant="secondary"
            disabled={!configured || status.checking || busy !== null}
            loading={busy === 'check'}
            onClick={() => run('check')}
          >
            {t('ajustes-atualizacao-procurar')}
          </Button>
        </div>
        <ConfirmDialog
          open={ask}
          onOpenChange={setAsk}
          tone="primary"
          title={t('ajustes-atualizacao-confirmar-titulo', { versao: ready?.version ?? '' })}
          description={t('ajustes-atualizacao-confirmar-texto')}
          confirmLabel={t('ajustes-atualizacao-instalar')}
          error={installError}
          onConfirm={() => {
            setInstallError(null);
            void installUpdate().catch(setInstallError);
          }}
        />
        <InlineError error={save.error} />
      </div>
    </InstrumentCard>
  );
}
