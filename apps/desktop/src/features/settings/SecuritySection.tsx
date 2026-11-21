import { useState, type FormEvent } from 'react';

import type { RecoveryKit } from '@/api/generated/RecoveryKit';
import { useAppSettings, useSaveAppSettings } from '@/api/hooks/settings';
import {
  useChangePassword,
  useKeychainDisable,
  useKeychainEnable,
  useRecoveryKit,
  useRecoveryStatus,
} from '@/api/hooks/vaultExtras';
import { useVaultStatus } from '@/api/vaultQueries';
import {
  Button,
  Dialog,
  Field,
  InstrumentCard,
  KeyValue,
  PasswordInput,
  Select,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { estimatePasswordStrength } from '@/lib/passwordStrength';
import { formatDateTime } from '@/lib/format';
import { StrengthMeter } from '@/vault/StrengthMeter';
import { validateNewPassword } from '@/vault/validation';
import { ConfirmDialog, InlineError, Switch } from '@/features/transactions/ui/controls';

import { tRich } from './richText';
import { Fido2Card, PinCard, SharesCard } from './UnlockMethods';

/** Opções do bloqueio automático, em minutos (rótulos no idioma atual). */
const autoLock = () => [
  { value: '0', label: t('ajustes-seg-nunca') },
  ...[1, 5, 15, 30].map((n) => ({ value: String(n), label: t('ajustes-seg-minutos', { n }) })),
  { value: '60', label: t('ajustes-seg-horas', { n: 1 }) },
  { value: '240', label: t('ajustes-seg-horas', { n: 4 }) },
];

function ChangePassword() {
  const change = useChangePassword();
  const [current, setCurrent] = useState('');
  const [next, setNext] = useState('');
  const [confirm, setConfirm] = useState('');
  const [touched, setTouched] = useState(false);
  const strength = estimatePasswordStrength(next);
  const errors = validateNewPassword(next, confirm, strength);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setTouched(true);
    if (!current || errors.password || errors.confirm) return;
    change.mutate(
      { current, new: next },
      {
        onSuccess: () => {
          toast({
            tone: 'success',
            title: t('ajustes-seg-trocada'),
            description: t('ajustes-seg-trocada-dica'),
          });
          setCurrent('');
          setNext('');
          setConfirm('');
          setTouched(false);
        },
      },
    );
  };
  return (
    <form
      onSubmit={submit}
      className="flex max-w-md flex-col gap-3"
      aria-label={t('ajustes-seg-trocar-form')}
    >
      <Field
        label={t('ajustes-seg-senha-atual')}
        error={touched && !current ? t('ajustes-seg-informe-atual') : undefined}
      >
        <PasswordInput
          value={current}
          onChange={(e) => setCurrent(e.target.value)}
          autoComplete="current-password"
        />
      </Field>
      <Field label={t('ajustes-seg-nova')} error={touched ? errors.password : undefined}>
        <PasswordInput
          value={next}
          onChange={(e) => setNext(e.target.value)}
          autoComplete="new-password"
        />
      </Field>
      {next ? <StrengthMeter strength={strength} /> : null}
      <Field label={t('ajustes-seg-confirmar')} error={touched ? errors.confirm : undefined}>
        <PasswordInput
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
          autoComplete="new-password"
        />
      </Field>
      <InlineError error={change.error} />
      <Button
        type="submit"
        variant="primary"
        className="self-start"
        loading={change.isPending}
        loadingText={t('ajustes-seg-recifrando')}
      >
        {t('ajustes-seg-trocar')}
      </Button>
    </form>
  );
}

function RecoveryKitCard() {
  const kit = useRecoveryKit();
  const recovery = useRecoveryStatus();
  const [ask, setAsk] = useState(false);
  const [password, setPassword] = useState('');
  const [shown, setShown] = useState<RecoveryKit | null>(null);
  const last = recovery.data?.kit_created_at ?? null;
  return (
    <div className="flex flex-col gap-3">
      <p className="max-w-[64ch] text-[13px] leading-relaxed text-dim">
        {tRich('ajustes-seg-kit-texto', {}, (content) => (
          <strong className="text-warning">{content}</strong>
        ))}
      </p>
      {recovery.data?.kit_outdated ? (
        <p role="status" className="max-w-[64ch] text-[13px] leading-relaxed text-warning">
          {t('ajustes-seg-kit-velho')}
        </p>
      ) : last ? (
        <p className="text-[12.5px] text-dim">
          {t('ajustes-seg-kit-ultimo', { data: formatDateTime(last) })}
        </p>
      ) : (
        <p role="status" className="max-w-[64ch] text-[13px] leading-relaxed text-warning">
          {t('ajustes-seg-kit-nenhum')}
        </p>
      )}
      <Button className="self-start" onClick={() => setAsk(true)}>
        {t('ajustes-seg-kit-gerar')}
      </Button>
      <Dialog
        open={ask}
        onOpenChange={(o) => {
          setAsk(o);
          if (!o) {
            setPassword('');
            setShown(null);
            kit.reset();
          }
        }}
        index="11"
        title={shown ? t('ajustes-seg-kit') : t('ajustes-seg-kit-confirme')}
        size="md"
        footer={
          shown ? (
            <>
              <Button variant="ghost" onClick={() => setAsk(false)}>
                {t('ajustes-seg-kit-fechar')}
              </Button>
              <Button variant="primary" onClick={() => window.print()}>
                {t('ajustes-imprimir')}
              </Button>
            </>
          ) : (
            <>
              <Button variant="ghost" onClick={() => setAsk(false)}>
                {t('comum-cancelar')}
              </Button>
              <Button
                variant="primary"
                loading={kit.isPending}
                disabled={!password}
                onClick={() =>
                  kit.mutate({ password }, { onSuccess: (k) => (setShown(k), setPassword('')) })
                }
              >
                {t('ajustes-seg-kit-mostrar')}
              </Button>
            </>
          )
        }
      >
        {shown ? (
          <div className="recovery-kit flex flex-col gap-4" data-print-area>
            {/* Na impressão, só o kit aparece (fundo branco, texto preto). */}
            <style>{`@media print {
              body * { visibility: hidden !important; }
              [data-print-area], [data-print-area] * { visibility: visible !important; color: #000 !important; background: #fff !important; border-color: #000 !important; }
              [data-print-area] { position: fixed; inset: 0; padding: 32px; }
            }`}</style>
            <p
              role="alert"
              className="rounded-card border border-warning/60 bg-warning/5 px-3 py-2 text-[13px] text-warning"
            >
              {t('ajustes-seg-kit-alerta')}
            </p>
            <p className="num rounded-card border border-line-strong bg-bg px-4 py-5 text-center text-[22px] tracking-[0.12em] text-fg select-all">
              {shown.code}
            </p>
            <KeyValue
              layout="stacked"
              items={[
                { label: t('ajustes-cofre'), value: shown.db_path },
                { label: t('ajustes-seg-kit-gerado'), value: formatDateTime(shown.created_at) },
                { label: t('ajustes-seg-kit-como'), value: t('ajustes-seg-kit-como-valor') },
              ]}
            />
          </div>
        ) : (
          <>
            <Field label={t('ajustes-senha-mestra')}>
              <PasswordInput
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoComplete="current-password"
                autoFocus
              />
            </Field>
            <InlineError error={kit.error} />
          </>
        )}
      </Dialog>
    </div>
  );
}

function KeychainCard() {
  const { data: status } = useVaultStatus();
  const enable = useKeychainEnable();
  const disable = useKeychainDisable();
  const [ask, setAsk] = useState(false);
  const [password, setPassword] = useState('');
  if (!status) return null;
  return (
    <div className="flex flex-col gap-3">
      <Switch
        checked={status.keychain_enabled}
        disabled={!status.keychain_available || enable.isPending || disable.isPending}
        onChange={(on) => {
          if (on) setAsk(true);
          else
            disable.mutate(
              {},
              {
                onSuccess: () =>
                  toast({ tone: 'success', title: t('ajustes-seg-keychain-desligado') }),
              },
            );
        }}
        label={t('ajustes-seg-touch-id')}
        description={
          status.keychain_available
            ? t('ajustes-seg-touch-id-dica')
            : t('ajustes-seg-keychain-indisponivel')
        }
      />
      <InlineError error={disable.error} />
      <Dialog
        open={ask}
        onOpenChange={(o) => (setAsk(o), o || (setPassword(''), enable.reset()))}
        index="11"
        size="sm"
        title={t('ajustes-seg-keychain-ligar')}
        description={t('ajustes-seg-keychain-ligar-dica')}
        footer={
          <>
            <Button variant="ghost" onClick={() => setAsk(false)}>
              {t('comum-cancelar')}
            </Button>
            <Button
              variant="primary"
              disabled={!password}
              loading={enable.isPending}
              onClick={() =>
                enable.mutate(
                  { password },
                  {
                    onSuccess: () => (
                      toast({ tone: 'success', title: t('ajustes-seg-keychain-ligado') }),
                      setAsk(false),
                      setPassword('')
                    ),
                  },
                )
              }
            >
              {t('ajustes-seg-ligar')}
            </Button>
          </>
        }
      >
        <Field label={t('ajustes-senha-mestra')}>
          <PasswordInput value={password} onChange={(e) => setPassword(e.target.value)} autoFocus />
        </Field>
        <InlineError error={enable.error} />
      </Dialog>
    </div>
  );
}

function AutoLockCard() {
  const { data: settings } = useAppSettings();
  const save = useSaveAppSettings();
  if (!settings) return null;
  const value = String(settings.auto_lock_minutes);
  const base = autoLock();
  const options = base.some((o) => o.value === value)
    ? base
    : [...base, { value, label: t('ajustes-seg-minutos', { n: settings.auto_lock_minutes }) }];
  return (
    <div className="flex flex-col gap-4">
      <Field label={t('ajustes-seg-bloqueio')} hint={t('ajustes-seg-bloqueio-dica')}>
        <Select
          value={value}
          onValueChange={(v) =>
            save.mutate(
              { auto_lock_minutes: Number(v) },
              {
                onSuccess: () =>
                  toast({ tone: 'success', title: t('ajustes-seg-bloqueio-ajustado') }),
              },
            )
          }
          options={options}
          className="max-w-64"
        />
      </Field>
      <Switch
        checked={settings.lock_on_sleep}
        onChange={(v) =>
          save.mutate(
            { lock_on_sleep: v },
            {
              onSuccess: () =>
                toast({ tone: 'success', title: t('ajustes-seg-bloqueio-ajustado') }),
            },
          )
        }
        label={t('ajustes-seg-dormir')}
        description={t('ajustes-seg-dormir-dica')}
      />
    </div>
  );
}

/** Capturas de tela: bloqueadas com o modo privacidade, salvo se a pessoa liberar. */
function ScreenCaptureCard() {
  const { data: settings } = useAppSettings();
  const save = useSaveAppSettings();
  const [ask, setAsk] = useState(false);
  if (!settings) return null;
  const set = (allow: boolean) =>
    save.mutate(
      { allow_screen_capture: allow },
      {
        onSuccess: () => {
          setAsk(false);
          toast({
            tone: allow ? 'warning' : 'success',
            title: allow
              ? t('ajustes-seg-capturas-liberadas')
              : t('ajustes-seg-capturas-bloqueadas'),
          });
        },
      },
    );
  return (
    <>
      <Switch
        checked={settings.allow_screen_capture}
        // Liberar pede confirmação, com o risco; voltar a proteger é na hora.
        onChange={(v) => (v ? setAsk(true) : set(false))}
        label={t('ajustes-seg-capturas')}
        description={
          settings.allow_screen_capture
            ? t('ajustes-seg-capturas-dica-liberadas')
            : t('ajustes-seg-capturas-dica')
        }
      />
      <ConfirmDialog
        open={ask}
        onOpenChange={setAsk}
        title={t('ajustes-seg-capturas-confirmar-titulo')}
        description={t('ajustes-seg-capturas-confirmar-texto')}
        confirmLabel={t('ajustes-seg-capturas-confirmar')}
        pending={save.isPending}
        error={save.error}
        onConfirm={() => set(true)}
      />
    </>
  );
}

/** 11.1 · Segurança: senha mestra, kits de recuperação, Touch ID, PIN, chaves físicas e bloqueio. */
export function SecuritySection() {
  return (
    <div className="grid gap-5 @min-[1040px]:grid-cols-2">
      <InstrumentCard index="12.1" title={t('ajustes-senha-mestra')}>
        <ChangePassword />
      </InstrumentCard>
      <div className="flex flex-col gap-5">
        <InstrumentCard index="12.2" title={t('ajustes-seg-kit')}>
          <RecoveryKitCard />
        </InstrumentCard>
        <InstrumentCard index="12.3" title={t('ajustes-seg-desbloqueio')}>
          <div className="flex flex-col gap-5">
            <KeychainCard />
            <PinCard />
            <AutoLockCard />
          </div>
        </InstrumentCard>
      </div>
      <InstrumentCard index="12.4" title={t('ajustes-seg-dividida')}>
        <SharesCard />
      </InstrumentCard>
      <InstrumentCard index="12.5" title={t('ajustes-seg-fido2')}>
        <Fido2Card />
      </InstrumentCard>
      <InstrumentCard index="12.6" title={t('ajustes-seg-capturas-titulo')}>
        <ScreenCaptureCard />
      </InstrumentCard>
    </div>
  );
}
