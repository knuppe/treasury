/**
 * Recuperar de um backup na tela de desbloqueio (docs/backup-recovery.md): a lista vem da
 * pasta de backups do cofre (com o cofre fechado ou quebrado, a tabela de backups não está
 * acessível), o escolhido é restaurado com a senha, e o cofre que não abria vai para o lado,
 * nunca apagado. Daqui também dá para salvar uma cópia de um backup em outra mídia e trazer um
 * backup de fora.
 */
import { useState, type FormEvent } from 'react';

import { isApiError, isTauri } from '@/api/client';
import type { BackupFile } from '@/api/generated/BackupFile';
import {
  useBackupFiles,
  useExportBackupFile,
  useImportBackupFile,
  useRestoreBackupFile,
} from '@/api/vaultQueries';
import { Button, Field, Icon, InstrumentCard, PasswordInput, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { BACKUP_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';
import { chooseSavePath } from '@/lib/saveDialog';

import { FormError } from './FormError';

/** Escolhe um arquivo de backup pelo diálogo "Abrir" do sistema (só no app). */
async function pickBackupFile(title: string): Promise<string | null> {
  if (!isTauri()) return null;
  const { invoke } = await import('@tauri-apps/api/core');
  const path = await invoke<string | string[] | null>('plugin:dialog|open', {
    options: { title, multiple: false, directory: false },
  });
  return typeof path === 'string' ? path : null;
}

/** Salva uma cópia do backup onde a pessoa escolher (pen drive, HD externo, nuvem). */
export function useSaveBackupCopy() {
  const exp = useExportBackupFile();
  const save = async (file: { path: string; file_name: string }) => {
    if (!isTauri()) {
      toast({ tone: 'error', title: t('cofre-backup-copia-so-app') });
      return;
    }
    const dest = await chooseSavePath(t('cofre-backup-copia-titulo'), file.file_name);
    if (!dest) return;
    exp.mutate(
      { path: file.path, dest },
      {
        onSuccess: () =>
          toast({
            tone: 'success',
            title: t('cofre-backup-copia-salva'),
            description: dest,
          }),
        onError: (e) =>
          toast({ tone: 'error', title: t('cofre-backup-copia-erro'), description: e.message }),
      },
    );
  };
  return { save, pending: exp.isPending };
}

/** Traz um backup de fora para a pasta do cofre. */
export function useBringBackup() {
  const imp = useImportBackupFile();
  const bring = async (onDone?: (f: BackupFile) => void) => {
    const source = await pickBackupFile(t('cofre-backup-trazer-titulo'));
    if (!source) return;
    imp.mutate(source, {
      onSuccess: (f) => {
        toast({ tone: 'success', title: t('cofre-backup-trazido'), description: f.file_name });
        onDone?.(f);
      },
      onError: (e) =>
        toast({ tone: 'error', title: t('cofre-backup-trazer-erro'), description: e.message }),
    });
  };
  return { bring, pending: imp.isPending, available: isTauri() };
}

function fileLabel(f: BackupFile): string {
  const kind = f.kind ? BACKUP_KIND[f.kind] : t('cofre-backup-tipo-desconhecido');
  return f.imported ? t('cofre-backup-importado', { tipo: kind }) : kind;
}

export function BackupRecovery({
  reason,
  onBack,
}: {
  /** Por que o cofre não abriu (a mensagem do erro), quando a tela abriu sozinha. */
  reason?: string | null;
  onBack: () => void;
}) {
  const files = useBackupFiles();
  const restore = useRestoreBackupFile();
  const copy = useSaveBackupCopy();
  const bring = useBringBackup();
  const [chosen, setChosen] = useState<string | null>(null);
  const [password, setPassword] = useState('');
  const [backupPassword, setBackupPassword] = useState('');
  const [otherPassword, setOtherPassword] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const list = files.data ?? [];
  const selected = list.find((f) => f.path === chosen) ?? list[0] ?? null;
  const wrong =
    isApiError(restore.error) && restore.error.code === 'wrong_password' ? restore.error : null;

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    setSubmitted(true);
    if (!selected || !password || restore.isPending) return;
    restore.mutate(
      {
        path: selected.path,
        master_password: password,
        ...(otherPassword && backupPassword ? { backup_password: backupPassword } : {}),
      },
      {
        onSuccess: (r) =>
          toast({
            tone: 'success',
            title: t('cofre-backup-restaurado'),
            description: r.moved_to
              ? t('cofre-backup-restaurado-ao-lado', { caminho: r.moved_to })
              : undefined,
          }),
      },
    );
  };

  return (
    <InstrumentCard index="01" title={t('cofre-backup-recuperar-titulo')}>
      <form noValidate onSubmit={onSubmit} className="flex flex-col gap-5">
        {reason ? (
          <p
            role="note"
            className="flex items-start gap-2 rounded-card border border-warning/60 bg-warning/5 px-3 py-2 text-[13px] text-fg"
          >
            <Icon name="alert" size={14} className="mt-0.5 shrink-0 text-warning" />
            <span>{t('cofre-backup-motivo', { erro: reason })}</span>
          </p>
        ) : null}
        <p className="text-[13px] text-dim">{t('cofre-backup-recuperar-texto')}</p>

        {files.isPending ? (
          <p className="text-[13px] text-dim">{t('cofre-backup-procurando')}</p>
        ) : list.length === 0 ? (
          <p className="rounded-card border border-line px-3 py-3 text-[13px] text-dim">
            {t('cofre-backup-nenhum')}
          </p>
        ) : (
          <div
            role="radiogroup"
            aria-label={t('cofre-backup-lista')}
            className="flex max-h-[280px] flex-col gap-1.5 overflow-y-auto"
          >
            {list.map((f) => {
              const on = selected?.path === f.path;
              return (
                <div
                  key={f.path}
                  className={cx(
                    'flex items-center gap-3 rounded-card border px-3 py-2',
                    on ? 'border-accent bg-accent-soft' : 'border-line',
                  )}
                >
                  <label className="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
                    <input
                      type="radio"
                      name="backup"
                      className="size-4 shrink-0"
                      checked={on}
                      onChange={() => setChosen(f.path)}
                    />
                    <span className="flex min-w-0 flex-col">
                      <span className="text-[13.5px] text-fg">
                        {f.created_at ? formatDateTime(f.created_at) : f.file_name}
                      </span>
                      <span className="truncate text-[12px] text-dim">
                        {fileLabel(f)} · {formatBytes(f.size_bytes)}
                      </span>
                    </span>
                  </label>
                  <Button
                    size="sm"
                    variant="ghost"
                    icon={<Icon name="arrowDown" size={13} />}
                    aria-label={t('cofre-backup-salvar-copia-de', {
                      data: f.created_at ? formatDateTime(f.created_at) : f.file_name,
                    })}
                    loading={copy.pending}
                    onClick={() => void copy.save(f)}
                  >
                    {t('cofre-backup-salvar-copia')}
                  </Button>
                </div>
              );
            })}
          </div>
        )}

        {bring.available ? (
          <div>
            <Button
              size="sm"
              variant="secondary"
              icon={<Icon name="arrowUp" size={13} />}
              loading={bring.pending}
              onClick={() => void bring.bring((f) => setChosen(f.path))}
            >
              {t('cofre-backup-trazer')}
            </Button>
          </div>
        ) : null}

        {selected ? (
          <>
            <Field
              label={t('cofre-backup-senha-mestra')}
              hint={t('cofre-backup-senha-mestra-dica')}
              error={
                wrong && !otherPassword
                  ? wrong.message
                  : submitted && !password
                    ? t('cofre-desbloquear-erro-vazia')
                    : undefined
              }
            >
              <PasswordInput
                autoComplete="current-password"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  if (restore.error) restore.reset();
                }}
              />
            </Field>
            <label className="flex items-center gap-2 text-[13px] text-fg">
              <input
                type="checkbox"
                className="size-4"
                checked={otherPassword}
                onChange={(e) => setOtherPassword(e.target.checked)}
              />
              {t('cofre-backup-outra-senha')}
            </label>
            {otherPassword ? (
              <Field
                label={t('cofre-backup-senha-backup')}
                error={wrong && otherPassword ? wrong.message : undefined}
              >
                <PasswordInput
                  autoComplete="off"
                  value={backupPassword}
                  onChange={(e) => {
                    setBackupPassword(e.target.value);
                    if (restore.error) restore.reset();
                  }}
                />
              </Field>
            ) : null}
            <p className="text-[12.5px] text-dim">{t('cofre-backup-ao-lado-aviso')}</p>
          </>
        ) : null}

        <FormError error={wrong ? null : restore.error} />

        <div className="flex items-center justify-between gap-3 border-t border-line pt-4">
          <Button variant="ghost" onClick={onBack}>
            {t('comum-voltar')}
          </Button>
          <Button
            type="submit"
            variant="primary"
            disabled={!selected}
            loading={restore.isPending}
            loadingText={t('cofre-backup-restaurando')}
            icon={<Icon name="restore" size={14} />}
          >
            {t('cofre-backup-restaurar')}
          </Button>
        </div>
      </form>
    </InstrumentCard>
  );
}
