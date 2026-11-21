import { useState } from 'react';

import type { BackupFile } from '@/api/generated/BackupFile';
import type { BackupRecord } from '@/api/generated/BackupRecord';
import type { BackupSettings } from '@/api/generated/BackupSettings';
import {
  useBackupHealth,
  useBackups,
  useBackupSettings,
  useCreateBackup,
  useRestoreBackup,
  useSaveBackupSettings,
  useSetBackupPassword,
  useVerifyBackup,
} from '@/api/hooks/backup';
import { isApiError } from '@/api/client';
import { useBackupFiles, useRestoreBackupFile } from '@/api/vaultQueries';
import { useBringBackup, useSaveBackupCopy } from '@/vault/BackupRecovery';
import {
  Button,
  Dialog,
  Field,
  Icon,
  InstrumentCard,
  Input,
  LiveIndicator,
  Measure,
  PasswordInput,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { formatDateTime } from '@/lib/format';
import { InlineError, QueryState, Segmented, Switch } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { BACKUP_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';

const HOURS = Array.from({ length: 24 }, (_, h) => ({
  value: String(h),
  label: `${String(h).padStart(2, '0')}:00`,
}));

/** Backup a restaurar: um da lista (com `id`) ou um arquivo trazido de fora (só o caminho). */
type Restorable = Pick<BackupRecord, 'created_at' | 'kind' | 'key_mode'> & {
  id?: string;
  path: string;
};

/** Restaurar: pede a senha mestra atual e, quando o backup foi feito com outra, a do backup. */
export function RestoreDialog({
  backup,
  onOpenChange,
}: {
  backup: Restorable | null;
  onOpenChange: (open: boolean) => void;
}) {
  const byId = useRestoreBackup();
  const byFile = useRestoreBackupFile();
  const restore = backup?.id ? byId : byFile;
  const [master, setMaster] = useState('');
  const [backupPassword, setBackupPassword] = useState('');
  const [needsBackupPassword, setNeedsBackupPassword] = useState(false);
  const [lastId, setLastId] = useState<string | null>(null);
  if ((backup?.path ?? null) !== lastId) {
    setLastId(backup?.path ?? null);
    setMaster('');
    setBackupPassword('');
    setNeedsBackupPassword(backup?.key_mode === 'separate');
    restore.reset();
  }
  if (!backup) return null;
  const done = {
    onSuccess: () => {
      toast({
        tone: 'success' as const,
        title: t('ajustes-backup-restaurado'),
        description: backup.created_at
          ? t('ajustes-backup-restaurado-dica', { data: formatDateTime(backup.created_at) })
          : undefined,
      });
      onOpenChange(false);
    },
    onError: (e: Error) => {
      // O núcleo pede a senha do backup (o texto vem do catálogo, no idioma do app).
      if (
        e.message === t('nucleo-geral-erro-backup-senha') ||
        e.message === t('nucleo-geral-erro-backup-senha-antiga') ||
        (isApiError(e) && e.code === 'wrong_password' && !backup.id)
      )
        setNeedsBackupPassword(true);
    },
  };
  const extra = backupPassword ? { backup_password: backupPassword } : {};
  const submit = () =>
    backup.id
      ? byId.mutate({ id: backup.id, master_password: master, ...extra }, done)
      : byFile.mutate({ path: backup.path, master_password: master, ...extra }, done);
  return (
    <Dialog
      open
      onOpenChange={onOpenChange}
      index="11"
      size="md"
      title={t('ajustes-backup-restaurar-titulo')}
      description={t('ajustes-backup-restaurar-texto', {
        data: backup.created_at ? formatDateTime(backup.created_at) : '—',
        tipo: backup.kind ? BACKUP_KIND[backup.kind].toLowerCase() : '',
      })}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="danger"
            loading={restore.isPending}
            loadingText={t('ajustes-backup-restaurando')}
            disabled={!master}
            onClick={submit}
          >
            {t('ajustes-backup-restaurar')}
          </Button>
        </>
      }
    >
      <Field label={t('ajustes-backup-senha-atual')}>
        <PasswordInput
          value={master}
          onChange={(e) => setMaster(e.target.value)}
          autoComplete="current-password"
          autoFocus
        />
      </Field>
      {needsBackupPassword ? (
        <Field label={t('ajustes-backup-senha')} hint={t('ajustes-backup-senha-dica')}>
          <PasswordInput
            value={backupPassword}
            onChange={(e) => setBackupPassword(e.target.value)}
          />
        </Field>
      ) : (
        <button
          type="button"
          className="label-caps self-start hover:text-fg"
          onClick={() => setNeedsBackupPassword(true)}
        >
          {t('ajustes-backup-outra-senha')}
        </button>
      )}
      <InlineError error={restore.error} />
    </Dialog>
  );
}

function VerifyDialog({
  backup,
  onOpenChange,
}: {
  backup: BackupRecord | null;
  onOpenChange: (o: boolean) => void;
}) {
  const verify = useVerifyBackup();
  const [password, setPassword] = useState('');
  if (!backup) return null;
  return (
    <Dialog
      open
      onOpenChange={onOpenChange}
      index="11"
      size="sm"
      title={t('ajustes-backup-verificar-titulo')}
      description={t('ajustes-backup-verificar-texto')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={verify.isPending}
            disabled={!password}
            onClick={() =>
              verify.mutate(
                { id: backup.id, backup_password: password },
                {
                  onSuccess: (r) => {
                    toast(
                      r.verified
                        ? { tone: 'success', title: t('ajustes-backup-integro') }
                        : {
                            tone: 'error',
                            title: t('ajustes-backup-problema'),
                            description: errorText(r.verify_error),
                          },
                    );
                    onOpenChange(false);
                  },
                },
              )
            }
          >
            {t('ajustes-backup-verificar')}
          </Button>
        </>
      }
    >
      <Field label={t('ajustes-backup-senha')}>
        <PasswordInput value={password} onChange={(e) => setPassword(e.target.value)} autoFocus />
      </Field>
      <InlineError error={verify.error} />
    </Dialog>
  );
}

function SettingsForm({ settings }: { settings: BackupSettings }) {
  const save = useSaveBackupSettings();
  const setPassword = useSetBackupPassword();
  const [form, setForm] = useState(settings);
  const [destination, setDestination] = useState('');
  const [separate, setSeparate] = useState('');
  const dirty = JSON.stringify(form) !== JSON.stringify(settings);
  const set = (p: Partial<BackupSettings>) => setForm((f) => ({ ...f, ...p }));
  return (
    <div className="flex flex-col gap-4">
      <div className="grid gap-3 @min-[400px]:grid-cols-4">
        <Field label={t('ajustes-backup-hora')}>
          <Select
            value={String(form.daily_hour)}
            onValueChange={(v) => set({ daily_hour: Number(v) })}
            options={HOURS}
          />
        </Field>
        <Field label={t('ajustes-backup-diarios')}>
          <Input
            type="number"
            min={0}
            numeric
            value={form.keep_daily}
            onChange={(e) => set({ keep_daily: Number(e.target.value) })}
          />
        </Field>
        <Field label={t('ajustes-backup-semanais')}>
          <Input
            type="number"
            min={0}
            numeric
            value={form.keep_weekly}
            onChange={(e) => set({ keep_weekly: Number(e.target.value) })}
          />
        </Field>
        <Field label={t('ajustes-backup-mensais')}>
          <Input
            type="number"
            min={0}
            numeric
            value={form.keep_monthly}
            onChange={(e) => set({ keep_monthly: Number(e.target.value) })}
          />
        </Field>
      </div>
      <Switch
        checked={form.on_close}
        onChange={(on_close) => set({ on_close })}
        label={t('ajustes-backup-ao-fechar')}
        description={t('ajustes-backup-ao-fechar-dica')}
      />
      <fieldset className="flex flex-col gap-2">
        <legend className="label-caps mb-1">{t('ajustes-backup-destinos')}</legend>
        {form.extra_destinations.map((d) => (
          <div key={d} className="flex items-center gap-2 text-[13px]">
            <span className="num min-w-0 flex-1 truncate">{d}</span>
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                set({ extra_destinations: form.extra_destinations.filter((x) => x !== d) })
              }
            >
              {t('ajustes-remover')}
            </Button>
          </div>
        ))}
        <div className="flex gap-2">
          <Input
            aria-label={t('ajustes-backup-destino-novo')}
            placeholder={t('ajustes-backup-destino-placeholder')}
            value={destination}
            onChange={(e) => setDestination(e.target.value)}
          />
          <Button
            variant="secondary"
            disabled={!destination.trim()}
            onClick={() => {
              set({
                extra_destinations: [...new Set([...form.extra_destinations, destination.trim()])],
              });
              setDestination('');
            }}
          >
            {t('ajustes-adicionar')}
          </Button>
        </div>
      </fieldset>
      <fieldset className="flex flex-col gap-2">
        <legend className="label-caps mb-1">{t('ajustes-backup-chave')}</legend>
        <Segmented
          label={t('ajustes-backup-chave')}
          value={form.key_mode}
          onChange={(key_mode) => set({ key_mode })}
          options={[
            { value: 'same', label: t('ajustes-backup-chave-mestra') },
            { value: 'separate', label: t('ajustes-backup-chave-propria') },
          ]}
        />
        {form.key_mode === 'separate' ? (
          <div className="flex flex-wrap items-end gap-2">
            <Field
              label={t('ajustes-backup-senha-propria')}
              className="w-72"
              hint={t('ajustes-backup-senha-propria-dica')}
            >
              <PasswordInput
                value={separate}
                onChange={(e) => setSeparate(e.target.value)}
                autoComplete="new-password"
              />
            </Field>
            <Button
              variant="secondary"
              loading={setPassword.isPending}
              disabled={separate.length === 0}
              onClick={() =>
                setPassword.mutate(
                  { password: separate },
                  {
                    onSuccess: () => (
                      toast({ tone: 'success', title: t('ajustes-backup-senha-definida') }),
                      setSeparate('')
                    ),
                  },
                )
              }
            >
              {t('ajustes-backup-definir-senha')}
            </Button>
          </div>
        ) : null}
      </fieldset>
      <InlineError error={save.error ?? setPassword.error} />
      <div className="flex gap-2">
        <Button
          variant="primary"
          disabled={!dirty}
          loading={save.isPending}
          onClick={() =>
            save.mutate(form, {
              onSuccess: () =>
                toast({ tone: 'success', title: t('ajustes-backup-ajustes-salvos') }),
            })
          }
        >
          {t('ajustes-backup-salvar-ajustes')}
        </Button>
        {dirty ? (
          <Button variant="ghost" onClick={() => setForm(settings)}>
            {t('ajustes-backup-descartar')}
          </Button>
        ) : null}
      </div>
    </div>
  );
}

/** 11.4 · Backups: saúde (alerta de 48 h), lista, criar, verificar, restaurar e ajustes. */
export function BackupsSection() {
  const health = useBackupHealth();
  const list = useBackups();
  const settings = useBackupSettings();
  const create = useCreateBackup();
  const verify = useVerifyBackup();
  const [restoring, setRestoring] = useState<Restorable | null>(null);
  const [verifying, setVerifying] = useState<BackupRecord | null>(null);
  const copy = useSaveBackupCopy();
  const bring = useBringBackup();
  const h = health.data;

  const columns: TechColumn<BackupRecord>[] = [
    {
      key: 'created_at',
      header: t('ajustes-quando'),
      numeric: true,
      align: 'left',
      width: '140px',
      sortValue: (b) => b.created_at,
      cell: (b) => formatDateTime(b.created_at),
    },
    {
      key: 'kind',
      header: t('ajustes-backup-col-tipo'),
      width: '210px',
      cell: (b) => <span className="block truncate">{BACKUP_KIND[b.kind]}</span>,
    },
    {
      key: 'destination',
      header: t('ajustes-backup-col-onde'),
      cell: (b) => (
        <span className="block truncate text-dim" title={b.path}>
          {b.destination === 'primary' ? t('ajustes-backup-pasta-app') : b.destination}
        </span>
      ),
    },
    {
      key: 'size',
      header: t('ajustes-tamanho'),
      numeric: true,
      width: '96px',
      cell: (b) => formatBytes(b.size_bytes),
    },
    {
      key: 'state',
      header: t('ajustes-situacao'),
      width: '200px',
      cell: (b) =>
        !b.exists ? (
          <span className="text-negative">{t('ajustes-backup-sumiu')}</span>
        ) : b.verified ? (
          <span className="text-positive">
            {b.key_mode === 'separate'
              ? t('ajustes-backup-verificado-propria')
              : t('ajustes-backup-verificado')}
          </span>
        ) : (
          <span className="block truncate text-warning" title={b.verify_error ?? undefined}>
            {b.verify_error ? errorText(b.verify_error) : t('ajustes-backup-nao-verificado')}
          </span>
        ),
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('ajustes-backup-col-acoes')}</span>,
      align: 'right',
      width: '300px',
      cell: (b) => (
        <span className="inline-flex gap-1" onClick={(e) => e.stopPropagation()}>
          <Button
            size="sm"
            variant="ghost"
            disabled={!b.exists}
            onClick={() =>
              verify.mutate(
                { id: b.id },
                {
                  onSuccess: (r) => {
                    if (r.verified)
                      toast({
                        tone: 'success',
                        title: t('ajustes-backup-integro'),
                        description: formatDateTime(r.created_at),
                      });
                    else if (r.needs_password) setVerifying(r);
                    else
                      toast({
                        tone: 'error',
                        title: t('ajustes-backup-problema'),
                        description: errorText(r.verify_error),
                      });
                  },
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('ajustes-backup-nao-verificou'),
                      description: errorText(e),
                    }),
                },
              )
            }
          >
            {t('ajustes-backup-verificar')}
          </Button>
          <Button size="sm" variant="ghost" disabled={!b.exists} onClick={() => setRestoring(b)}>
            {t('ajustes-backup-restaurar')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={!b.exists}
            loading={copy.pending}
            onClick={() => void copy.save({ path: b.path, file_name: fileNameOf(b.path) })}
          >
            {t('ajustes-backup-salvar-copia')}
          </Button>
        </span>
      ),
    },
  ];

  return (
    <div className="flex flex-col gap-5">
      <InstrumentCard
        index="12.4"
        title={t('ajustes-backup-saude')}
        live={create.isPending ? t('ajustes-backup-fazendo') : undefined}
        actions={
          <Button
            size="sm"
            variant="secondary"
            loading={create.isPending}
            icon={<Icon name="shield" size={12} />}
            onClick={() =>
              create.mutate(
                {},
                {
                  onSuccess: (b) =>
                    toast({
                      tone: 'success',
                      title: t('ajustes-backup-feito'),
                      description: t('ajustes-backup-feito-dica', {
                        tamanho: formatBytes(b.size_bytes),
                      }),
                    }),
                },
              )
            }
          >
            {t('ajustes-backup-fazer')}
          </Button>
        }
      >
        <QueryState loading={health.isLoading} error={health.error}>
          {h ? (
            <div className="flex flex-col gap-4">
              {h.warning ? (
                <p
                  role="alert"
                  className="flex items-center gap-2 rounded-card border border-negative/60 bg-negative/5 px-3 py-2 text-[13px] text-negative"
                >
                  <Icon name="alert" size={14} />
                  {h.last_valid_at
                    ? t('ajustes-backup-atrasado', { horas: h.hours_since_last_valid ?? 0 })
                    : t('ajustes-backup-nenhum')}
                </p>
              ) : null}
              <div className="grid gap-5 @min-[400px]:grid-cols-4">
                <Measure
                  size="md"
                  label={t('ajustes-backup-ultimo')}
                  value={
                    <span className="text-[22px]">
                      {h.last_valid_at ? formatDateTime(h.last_valid_at) : '—'}
                    </span>
                  }
                />
                <Measure
                  size="md"
                  label={t('ajustes-backup-ha')}
                  value={
                    <span
                      className={cx('text-[22px]', h.warning ? 'text-negative' : 'text-positive')}
                    >
                      {h.hours_since_last_valid != null
                        ? t('ajustes-backup-horas', { horas: h.hours_since_last_valid })
                        : '—'}
                    </span>
                  }
                />
                <Measure
                  size="md"
                  label={t('ajustes-backup-no-disco')}
                  value={<span className="text-[22px]">{h.count}</span>}
                />
                <Measure
                  size="md"
                  label={t('ajustes-backup-espaco')}
                  value={<span className="text-[22px]">{formatBytes(h.total_bytes)}</span>}
                />
              </div>
              {h.daily_due ? (
                <LiveIndicator tone="warning" active={false} label={t('ajustes-backup-pendente')} />
              ) : null}
            </div>
          ) : null}
        </QueryState>
        <InlineError error={create.error} />
      </InstrumentCard>

      <InstrumentCard
        index="12.5"
        title={t('ajustes-secao-backups')}
        padded={false}
        actions={
          bring.available ? (
            <Button
              size="sm"
              variant="secondary"
              loading={bring.pending}
              icon={<Icon name="arrowUp" size={12} />}
              onClick={() => void bring.bring()}
            >
              {t('ajustes-backup-trazer')}
            </Button>
          ) : undefined
        }
      >
        <QueryState loading={list.isLoading} error={list.error}>
          <TechTable
            label={t('ajustes-secao-backups')}
            prefsId="Backups"
            dense
            columns={columns}
            rows={list.data ?? []}
            getRowId={(b) => b.id}
            maxHeight="420px"
            className="[&_table]:table-fixed"
          />
        </QueryState>
      </InstrumentCard>

      <ImportedBackups onRestore={setRestoring} />

      <InstrumentCard index="12.6" title={t('ajustes-backup-ajustes')}>
        <QueryState loading={settings.isLoading} error={settings.error}>
          {settings.data ? (
            <SettingsForm key={JSON.stringify(settings.data)} settings={settings.data} />
          ) : null}
        </QueryState>
      </InstrumentCard>

      <RestoreDialog backup={restoring} onOpenChange={(o) => !o && setRestoring(null)} />
      <VerifyDialog backup={verifying} onOpenChange={(o) => !o && setVerifying(null)} />
    </div>
  );
}

function fileNameOf(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/** Backups trazidos de fora (`importados/`): fora da lista e da rotação, restauráveis daqui. */
function ImportedBackups({ onRestore }: { onRestore: (b: Restorable) => void }) {
  const files = useBackupFiles();
  const copy = useSaveBackupCopy();
  const imported = (files.data ?? []).filter((f: BackupFile) => f.imported);
  if (!imported.length) return null;
  return (
    <InstrumentCard index="12.5b" title={t('ajustes-backup-trazidos')}>
      <ul className="flex flex-col divide-y divide-line">
        {imported.map((f) => (
          <li key={f.path} className="flex flex-wrap items-center gap-3 py-2 text-[13px]">
            <span className="min-w-0 flex-1">
              <span className="block text-fg">{f.file_name}</span>
              <span className="block text-dim">
                {f.created_at ? formatDateTime(f.created_at) : '—'} · {formatBytes(f.size_bytes)}
              </span>
            </span>
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                onRestore({
                  path: f.path,
                  created_at: f.created_at ?? '',
                  kind: f.kind ?? 'manual',
                  key_mode: 'same',
                })
              }
            >
              {t('ajustes-backup-restaurar')}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              loading={copy.pending}
              onClick={() => void copy.save(f)}
            >
              {t('ajustes-backup-salvar-copia')}
            </Button>
          </li>
        ))}
      </ul>
      <p className="mt-2 text-[12.5px] text-dim">{t('ajustes-backup-trazidos-dica')}</p>
    </InstrumentCard>
  );
}
