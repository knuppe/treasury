import { useEffect, useRef, useState, type FormEvent } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { flushSync } from 'react-dom';

import { isApiError, transportKind } from '@/api/client';
import { MOCK_RECOVERY_CODE } from '@/api/mockFixtures';
import { VAULT_ERRORS, type VaultStatus } from '@/api/vault';
import { useFido2Slots, useUnlockPin, useUnlockShares } from '@/api/hooks/vaultExtras';
import { useUnlockDemo, useUnlockRecovery, useUnlockVault, vaultKeys } from '@/api/vaultQueries';
import {
  Button,
  Field,
  Icon,
  Input,
  InstrumentCard,
  PasswordInput,
  Select,
  toast,
} from '@/design/components';
import { Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { estimatePasswordStrength } from '@/lib/passwordStrength';

import { BackupRecovery } from './BackupRecovery';
import { FormError } from './FormError';
import {
  biometricStatus,
  fido2Unlock,
  unlockBiometric,
  type BiometricStatus,
} from './nativeUnlock';
import { StrengthMeter } from './StrengthMeter';
import { validateNewPassword } from './validation';
import { VaultLayout } from './VaultLayout';
import { OtherVaults } from './VaultSwitcher';

type Mode = 'password' | 'pin' | 'recovery' | 'backup';

/**
 * Erros ao abrir que não são só a senha (arquivo danificado, migração, E/S, criptografia): a
 * recuperação por backup abre direto (docs/backup-recovery.md).
 */
const DAMAGED_CODES = new Set(['db', 'io', 'crypto', 'migration', 'other']);

/** Cofre bloqueado: senha mestra, PIN ou Touch ID; em último caso, a recuperação. */
export function UnlockVault({ status }: { status: VaultStatus }) {
  if (status.demo_no_password) return <UnlockDemo status={status} />;
  return <UnlockWithPassword status={status} />;
}

/**
 * Demonstração sem senha (docs/demo.md): um botão abre pela chave fixa do app. O aviso lembra
 * que qualquer pessoa com o app abre esse cofre, por isso só serve para dados fictícios.
 */
function UnlockDemo({ status }: { status: VaultStatus }) {
  const unlock = useUnlockDemo();
  return (
    <VaultLayout
      kicker={t('cofre-demo-kicker', { nome: status.vault_name })}
      heading={status.household_name ?? status.vault_name}
      lead={t('cofre-demo-lead')}
      dbPath={status.db_path}
      footer={<OtherVaults count={status.vault_count} />}
    >
      <div className="flex flex-col gap-4">
        <p
          role="note"
          className="flex items-start gap-2 rounded-card border border-warning/60 bg-warning/5 px-3 py-2 text-[13px] text-warning"
        >
          <Icon name="alert" size={14} className="mt-0.5 shrink-0" />
          <span>{t('cofre-demo-aviso')}</span>
        </p>
        <Button
          variant="primary"
          loading={unlock.isPending}
          onClick={() => unlock.mutate()}
          autoFocus
        >
          {t('cofre-demo-abrir')}
        </Button>
        <FormError error={unlock.error} />
      </div>
    </VaultLayout>
  );
}

function UnlockWithPassword({ status }: { status: VaultStatus }) {
  const [mode, setMode] = useState<Mode>('password');
  const [reason, setReason] = useState<string | null>(null);
  const household = status.household_name ?? t('cofre-desbloquear-cofre-padrao');
  const lead = {
    password: t('cofre-desbloquear-lead-senha'),
    pin: t('cofre-desbloquear-lead-pin'),
    recovery: t('cofre-desbloquear-lead-recuperacao'),
    backup: t('cofre-desbloquear-lead-backup'),
  }[mode];
  const toBackup = (why: string | null) => {
    setReason(why);
    setMode('backup');
  };
  return (
    <VaultLayout
      kicker={
        status.vault_count > 1
          ? t('cofre-desbloquear-kicker-nomeado', { nome: status.vault_name })
          : t('cofre-desbloquear-kicker')
      }
      heading={household}
      lead={lead}
      dbPath={status.db_path}
      footer={<OtherVaults count={status.vault_count} />}
    >
      {mode === 'password' ? (
        <PasswordForm
          status={status}
          onRecovery={() => setMode('recovery')}
          onPin={() => setMode('pin')}
          onBackup={toBackup}
        />
      ) : mode === 'pin' ? (
        <PinForm onBack={() => setMode('password')} />
      ) : mode === 'backup' ? (
        <BackupRecovery reason={reason} onBack={() => setMode('password')} />
      ) : (
        <RecoveryForm status={status} onBack={() => setMode('password')} />
      )}
    </VaultLayout>
  );
}

function useBiometric(enabled: boolean): BiometricStatus {
  const [bio, setBio] = useState<BiometricStatus>({ available: false, kind: null });
  useEffect(() => {
    if (!enabled) return;
    let alive = true;
    void biometricStatus().then((b) => alive && setBio(b));
    return () => {
      alive = false;
    };
  }, [enabled]);
  return bio;
}

/** Touch ID (ou senha do Mac) e PIN, quando ligados. */
function QuickUnlock({ status, onPin }: { status: VaultStatus; onPin: () => void }) {
  const client = useQueryClient();
  const bio = useBiometric(status.keychain_enabled);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  if (!(status.keychain_enabled && bio.available) && !status.pin_enabled) return null;
  const touch = async () => {
    setBusy(true);
    setError(null);
    try {
      client.setQueryData(vaultKeys.status, await unlockBiometric());
    } catch (e) {
      setError(e instanceof Error ? e : new Error(String(e)));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="flex flex-col gap-2 border-t border-line pt-4">
      <div className="flex flex-wrap gap-2">
        {status.keychain_enabled && bio.available ? (
          <Button
            variant="secondary"
            loading={busy}
            icon={<Icon name="key" size={13} />}
            onClick={() => void touch()}
          >
            {bio.kind === 'touch_id'
              ? t('cofre-desbloquear-touch-id')
              : t('cofre-desbloquear-senha-mac')}
          </Button>
        ) : null}
        {status.pin_enabled ? (
          <Button variant="secondary" icon={<Icon name="lock" size={13} />} onClick={onPin}>
            {t('cofre-usar-pin')}
          </Button>
        ) : null}
      </div>
      <FormError error={error} />
    </div>
  );
}

function PinForm({ onBack }: { onBack: () => void }) {
  const unlock = useUnlockPin();
  const inputRef = useRef<HTMLInputElement>(null);
  const [pin, setPin] = useState('');
  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!pin || unlock.isPending) return;
    unlock.mutate(
      { pin },
      {
        onError: () => {
          flushSync(() => setPin(''));
          inputRef.current?.focus();
        },
      },
    );
  };
  return (
    <InstrumentCard index="01" title={t('cofre-pin-titulo')}>
      <form noValidate onSubmit={onSubmit} className="flex flex-col gap-5">
        <Field label={t('cofre-pin-campo')} error={unlock.error ? unlock.error.message : undefined}>
          <PasswordInput
            ref={inputRef}
            autoFocus
            inputMode="numeric"
            autoComplete="off"
            maxLength={12}
            value={pin}
            onChange={(e) => {
              setPin(e.target.value.replace(/\D/g, ''));
              if (unlock.error) unlock.reset();
            }}
          />
        </Field>
        <Button
          type="submit"
          variant="primary"
          size="lg"
          fullWidth
          loading={unlock.isPending}
          loadingText={t('cofre-pin-conferindo')}
          icon={<Icon name="unlock" size={15} />}
        >
          {t('cofre-desbloquear')}
        </Button>
        <div className="border-t border-line pt-4">
          <Button
            size="sm"
            variant="ghost"
            icon={<Icon name="chevronLeft" size={13} />}
            onClick={onBack}
            className="-ml-2.5"
          >
            {t('cofre-usar-senha-mestra')}
          </Button>
        </div>
      </form>
    </InstrumentCard>
  );
}

function PasswordForm({
  status,
  onRecovery,
  onPin,
  onBackup,
}: {
  status: VaultStatus;
  onRecovery: () => void;
  onPin: () => void;
  /** Abre a recuperação por backup, com o motivo quando o cofre deu erro ao abrir. */
  onBackup: (reason: string | null) => void;
}) {
  const unlock = useUnlockVault();
  const inputRef = useRef<HTMLInputElement>(null);
  const [password, setPassword] = useState('');
  const [emptyError, setEmptyError] = useState(false);

  const wrongPassword =
    isApiError(unlock.error) && unlock.error.code === VAULT_ERRORS.wrongPassword
      ? unlock.error
      : null;

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (unlock.isPending) return;
    if (!password) {
      setEmptyError(true);
      inputRef.current?.focus();
      return;
    }
    unlock.mutate(
      { password },
      {
        onError: (e) => {
          if (isApiError(e) && DAMAGED_CODES.has(e.code)) {
            onBackup(e.message);
            return;
          }
          flushSync(() => setPassword(''));
          inputRef.current?.focus();
        },
      },
    );
  };
  const damaged =
    isApiError(unlock.error) && unlock.error.code === VAULT_ERRORS.vaultDamaged
      ? unlock.error
      : null;

  return (
    <InstrumentCard
      index="01"
      title={t('cofre-desbloquear')}
      footer={[
        {
          label: t('cofre-auto-bloqueio'),
          value:
            status.auto_lock_minutes > 0
              ? t('cofre-auto-bloqueio-minutos', { minutos: status.auto_lock_minutes })
              : t('cofre-desligado'),
        },
        {
          label: t('cofre-keychain'),
          value: status.keychain_enabled ? t('cofre-ligado') : t('cofre-desligado'),
        },
      ]}
    >
      <form noValidate onSubmit={onSubmit} className="flex flex-col gap-5">
        <Field
          label={t('cofre-senha-mestra')}
          error={
            wrongPassword
              ? wrongPassword.message
              : emptyError && !password
                ? t('cofre-desbloquear-erro-vazia')
                : undefined
          }
        >
          <PasswordInput
            ref={inputRef}
            autoFocus
            autoComplete="current-password"
            value={password}
            onChange={(e) => {
              setPassword(e.target.value);
              setEmptyError(false);
              if (unlock.error) unlock.reset();
            }}
          />
        </Field>

        <FormError error={wrongPassword ? null : unlock.error} />
        {damaged ? (
          <Button
            variant="secondary"
            icon={<Icon name="restore" size={14} />}
            onClick={() => onBackup(null)}
          >
            {t('cofre-backup-abrir-recuperacao')}
          </Button>
        ) : null}

        <Button
          type="submit"
          variant="primary"
          size="lg"
          fullWidth
          loading={unlock.isPending}
          loadingText={t('cofre-desbloquear-derivando')}
          icon={<Icon name="unlock" size={15} />}
        >
          {t('cofre-desbloquear')}
        </Button>

        <QuickUnlock status={status} onPin={onPin} />

        <div className="flex items-center justify-between border-t border-line pt-4">
          <span className="text-[12.5px] text-dim">{t('cofre-desbloquear-esqueceu')}</span>
          <Button
            size="sm"
            variant="ghost"
            icon={<Icon name="key" size={13} />}
            onClick={onRecovery}
          >
            {t('cofre-recuperar')}
          </Button>
        </div>
        <div className="-mt-2 flex items-center justify-between">
          <span className="text-[12.5px] text-dim">{t('cofre-desbloquear-nao-abre')}</span>
          <Button
            size="sm"
            variant="ghost"
            icon={<Icon name="restore" size={13} />}
            onClick={() => onBackup(null)}
          >
            {t('cofre-backup-abrir-recuperacao')}
          </Button>
        </div>
      </form>
    </InstrumentCard>
  );
}

type RecoveryMethod = 'kit' | 'shares' | 'fido2';

function RecoveryForm({ status, onBack }: { status: VaultStatus; onBack: () => void }) {
  const client = useQueryClient();
  const recover = useUnlockRecovery();
  const shares = useUnlockShares();
  const slots = useFido2Slots(status.fido2_keys > 0);
  const formRef = useRef<HTMLFormElement>(null);
  const [method, setMethod] = useState<RecoveryMethod>('kit');
  const [code, setCode] = useState('');
  const [partA, setPartA] = useState('');
  const [partB, setPartB] = useState('');
  const [slotId, setSlotId] = useState('');
  const [keyPin, setKeyPin] = useState('');
  const [fidoBusy, setFidoBusy] = useState(false);
  const [fidoError, setFidoError] = useState<Error | null>(null);
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [submitted, setSubmitted] = useState(false);

  const slotList = slots.data ?? [];
  const chosenSlot = slotId || slotList[0]?.id || '';
  const strength = estimatePasswordStrength(password, [status.household_name ?? '']);
  const errors = {
    code: method === 'kit' && !code.trim() ? t('cofre-recuperar-erro-codigo') : undefined,
    parts:
      method === 'shares' && (!partA.trim() || !partB.trim())
        ? t('cofre-recuperar-erro-partes')
        : undefined,
    slot: method === 'fido2' && !chosenSlot ? t('cofre-recuperar-erro-sem-chave') : undefined,
    ...validateNewPassword(password, confirm, strength),
  };
  const valid = Object.values(errors).every((e) => !e);
  const show = (error: string | undefined) => (submitted ? error : undefined);
  const wrongCode =
    isApiError(recover.error) && recover.error.code === VAULT_ERRORS.wrongRecoveryCode
      ? recover.error
      : null;
  const pending = recover.isPending || shares.isPending || fidoBusy;

  const done = () =>
    toast({
      tone: 'success',
      title: t('cofre-recuperar-toast-titulo'),
      description: t('cofre-recuperar-toast-descricao'),
    });

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (pending) return;
    if (!valid) {
      flushSync(() => setSubmitted(true));
      formRef.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
      return;
    }
    if (method === 'kit') {
      recover.mutate({ code: code.trim(), new_password: password }, { onSuccess: done });
    } else if (method === 'shares') {
      shares.mutate(
        { shares: [partA.trim(), partB.trim()], new_password: password },
        { onSuccess: done },
      );
    } else {
      setFidoBusy(true);
      setFidoError(null);
      void fido2Unlock(chosenSlot, password, keyPin)
        .then((st) => {
          client.setQueryData(vaultKeys.status, st);
          done();
        })
        .catch((e: unknown) => setFidoError(e instanceof Error ? e : new Error(String(e))))
        .finally(() => setFidoBusy(false));
    }
  };

  const methods: { value: RecoveryMethod; label: string }[] = [
    { value: 'kit', label: t('cofre-recuperar-metodo-kit') },
    { value: 'shares', label: t('cofre-recuperar-metodo-partes') },
    ...(status.fido2_keys > 0
      ? [{ value: 'fido2' as const, label: t('cofre-recuperar-metodo-chave') }]
      : []),
  ];

  return (
    <InstrumentCard index="01" title={t('cofre-recuperar')}>
      <form ref={formRef} noValidate onSubmit={onSubmit} className="flex flex-col gap-5">
        <Segmented<RecoveryMethod>
          label={t('cofre-recuperar-como')}
          value={method}
          onChange={setMethod}
          options={methods}
        />

        {method === 'kit' ? (
          <Field
            label={t('cofre-recuperar-codigo')}
            hint={
              import.meta.env.DEV && transportKind() === 'mock'
                ? t('cofre-recuperar-codigo-mock', { codigo: MOCK_RECOVERY_CODE })
                : t('cofre-recuperar-codigo-dica')
            }
            error={wrongCode ? wrongCode.message : show(errors.code)}
          >
            <Input
              autoFocus
              autoComplete="off"
              autoCapitalize="characters"
              spellCheck={false}
              placeholder="XXXX-XXXX-XXXX-XXXX"
              className="num tracking-[0.08em] uppercase"
              value={code}
              onChange={(e) => setCode(e.target.value)}
            />
          </Field>
        ) : method === 'shares' ? (
          <div className="flex flex-col gap-3">
            {[
              [t('cofre-recuperar-primeira-parte'), partA, setPartA] as const,
              [t('cofre-recuperar-segunda-parte'), partB, setPartB] as const,
            ].map(([label, value, set], i) => (
              <Field
                key={label}
                label={label}
                hint={i === 0 ? t('cofre-recuperar-partes-dica') : undefined}
                error={i === 1 ? show(errors.parts) : undefined}
              >
                <Input
                  autoComplete="off"
                  autoCapitalize="characters"
                  spellCheck={false}
                  placeholder="TRSP-XXXX-XXXX-…"
                  className="num tracking-[0.06em] uppercase"
                  value={value}
                  onChange={(e) => set(e.target.value)}
                />
              </Field>
            ))}
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            <Field label={t('cofre-recuperar-chave')} error={show(errors.slot)}>
              <Select
                value={chosenSlot}
                onValueChange={setSlotId}
                options={slotList.map((s) => ({ value: s.id, label: s.label }))}
              />
            </Field>
            <Field
              label={t('cofre-recuperar-pin-chave')}
              hint={t('cofre-recuperar-pin-chave-dica')}
            >
              <PasswordInput
                autoComplete="off"
                value={keyPin}
                onChange={(e) => setKeyPin(e.target.value)}
              />
            </Field>
          </div>
        )}

        <div className="flex flex-col gap-2">
          <Field label={t('cofre-nova-senha-mestra')} error={show(errors.password)}>
            <PasswordInput
              autoComplete="new-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </Field>
          <StrengthMeter strength={strength} />
        </div>

        <Field label={t('cofre-confirmar-nova-senha')} error={show(errors.confirm)}>
          <PasswordInput
            autoComplete="new-password"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
          />
        </Field>

        <FormError
          error={
            method === 'kit'
              ? wrongCode
                ? null
                : recover.error
              : method === 'shares'
                ? shares.error
                : fidoError
          }
        />

        <Button
          type="submit"
          variant="primary"
          size="lg"
          fullWidth
          loading={pending}
          loadingText={
            method === 'fido2' ? t('cofre-recuperar-toque-chave') : t('cofre-recuperar-recifrando')
          }
          icon={<Icon name="key" size={15} />}
        >
          {t('cofre-recuperar-botao')}
        </Button>

        <div className="border-t border-line pt-4">
          <Button
            size="sm"
            variant="ghost"
            icon={<Icon name="chevronLeft" size={13} />}
            onClick={onBack}
            className="-ml-2.5"
          >
            {t('cofre-recuperar-voltar')}
          </Button>
        </div>
      </form>
    </InstrumentCard>
  );
}
