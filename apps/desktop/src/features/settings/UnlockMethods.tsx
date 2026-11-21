/**
 * Ajustes › Segurança: PIN deste computador, recuperação dividida (3 partes, 2 bastam) e
 * chaves físicas FIDO2 (docs/unlock-options.md). Senhas, PIN e partes ficam só no estado do
 * componente e somem ao fechar o diálogo; nada vai para o cache nem para o disco.
 */
import { useEffect, useState } from 'react';

import type { RecoveryShares } from '@/api/generated/RecoveryShares';
import {
  useCreateShares,
  useFido2Remove,
  usePinDisable,
  usePinEnable,
  useRecoveryStatus,
} from '@/api/hooks/vaultExtras';
import { useVaultStatus, vaultKeys } from '@/api/vaultQueries';
import { Button, Dialog, Field, Input, PasswordInput, toast } from '@/design/components';
import { InlineError, Switch } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';
import { copySecret } from '@/lib/osGuard';
import { fido2Present, fido2Register } from '@/vault/nativeUnlock';
import { isTauri } from '@/api/client';
import { useQueryClient } from '@tanstack/react-query';

type PasswordDialogProps = {
  open: boolean;
  title: string;
  description: string;
  confirmLabel: string;
  pending: boolean;
  error: Error | null;
  extra?: React.ReactNode;
  onConfirm: (password: string) => void;
  onClose: () => void;
};

/** A senha some ao fechar: o conteúdo é recriado a cada abertura. */
function PasswordDialog(props: PasswordDialogProps) {
  return <PasswordDialogInner key={props.open ? 'aberto' : 'fechado'} {...props} />;
}

function PasswordDialogInner({
  open,
  title,
  description,
  confirmLabel,
  pending,
  error,
  extra,
  onConfirm,
  onClose,
}: PasswordDialogProps) {
  const [password, setPassword] = useState('');
  return (
    <Dialog
      open={open}
      onOpenChange={(o) => !o && onClose()}
      index="11"
      size="sm"
      title={title}
      description={description}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            disabled={!password}
            loading={pending}
            onClick={() => onConfirm(password)}
          >
            {confirmLabel}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        {extra}
        <Field label={t('ajustes-senha-mestra')}>
          <PasswordInput
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoFocus={!extra}
          />
        </Field>
        <InlineError error={error} />
      </div>
    </Dialog>
  );
}

export function PinCard() {
  const { data: status } = useVaultStatus();
  const enable = usePinEnable();
  const disable = usePinDisable();
  const [ask, setAsk] = useState(false);
  const [pin, setPin] = useState('');
  if (!status) return null;
  return (
    <>
      <Switch
        checked={status.pin_enabled}
        disabled={!status.keychain_available || enable.isPending || disable.isPending}
        onChange={(on) => {
          if (on) setAsk(true);
          else
            disable.mutate({}, { onSuccess: () => toast({ title: t('ajustes-pin-desligado') }) });
        }}
        label={t('ajustes-pin-desbloquear')}
        description={
          status.keychain_available
            ? t('ajustes-pin-desbloquear-dica')
            : t('ajustes-pin-sem-keychain')
        }
      />
      <InlineError error={disable.error} />
      <PasswordDialog
        open={ask}
        title={t('ajustes-pin-ligar-titulo')}
        description={t('ajustes-pin-ligar-dica')}
        confirmLabel={t('ajustes-pin-ligar')}
        pending={enable.isPending}
        error={enable.error}
        extra={
          <Field label={t('ajustes-pin-campo')}>
            <PasswordInput
              autoFocus
              inputMode="numeric"
              maxLength={12}
              value={pin}
              onChange={(e) => setPin(e.target.value.replace(/\D/g, ''))}
            />
          </Field>
        }
        onClose={() => (setAsk(false), setPin(''), enable.reset())}
        onConfirm={(password) =>
          enable.mutate(
            { password, pin },
            {
              onSuccess: () => (
                toast({ tone: 'success', title: t('ajustes-pin-ligado') }),
                setAsk(false),
                setPin('')
              ),
            },
          )
        }
      />
    </>
  );
}

function ShareList({ result }: { result: RecoveryShares }) {
  const copy = (text: string) => {
    void copySecret(text).then(
      () => toast({ title: t('ajustes-partes-copiada') }),
      () => toast({ tone: 'error', title: t('ajustes-nao-copiou') }),
    );
  };
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[13px] text-fg">{t('ajustes-partes-texto')}</p>
      <ol aria-label={t('ajustes-partes')} className="flex flex-col gap-2">
        {result.shares.map((s, i) => (
          <li key={s} className="flex items-center gap-2 rounded-control border border-line p-2">
            <span className="label-caps w-14 shrink-0">
              {t('ajustes-partes-parte', { n: i + 1 })}
            </span>
            <code data-selectable className="num min-w-0 flex-1 text-[12px] break-all text-fg">
              {s}
            </code>
            <Button size="sm" variant="ghost" onClick={() => copy(s)}>
              {t('ajustes-copiar')}
            </Button>
          </li>
        ))}
      </ol>
      {result.fido2_removed > 0 ? (
        <p className="text-[12.5px] text-warning">
          {t('ajustes-partes-fido2-removidas', { n: result.fido2_removed })}
        </p>
      ) : null}
      <Button variant="secondary" onClick={() => window.print()}>
        {t('ajustes-imprimir')}
      </Button>
    </div>
  );
}

export function SharesCard() {
  const recovery = useRecoveryStatus();
  const create = useCreateShares();
  const [ask, setAsk] = useState<'new' | 'rotate' | null>(null);
  const [result, setResult] = useState<RecoveryShares | null>(null);
  const created = recovery.data?.shares_created_at;
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[13px] text-dim">
        {created
          ? t('ajustes-partes-geradas', { data: formatDateTime(created) })
          : t('ajustes-partes-dica')}
      </p>
      <div className="flex flex-wrap gap-2">
        <Button variant="secondary" onClick={() => setAsk('new')}>
          {created ? t('ajustes-partes-novas-copias') : t('ajustes-partes-gerar')}
        </Button>
        {created ? (
          <Button variant="ghost" onClick={() => setAsk('rotate')}>
            {t('ajustes-partes-trocar')}
          </Button>
        ) : null}
      </div>
      <PasswordDialog
        open={ask !== null && !result}
        title={ask === 'rotate' ? t('ajustes-partes-trocar-titulo') : t('ajustes-partes')}
        description={
          ask === 'rotate' ? t('ajustes-partes-trocar-dica') : t('ajustes-partes-confirme')
        }
        confirmLabel={t('ajustes-partes-gerar-botao')}
        pending={create.isPending}
        error={create.error}
        onClose={() => (setAsk(null), create.reset())}
        onConfirm={(password) =>
          create.mutate({ password, rotate: ask === 'rotate' }, { onSuccess: (r) => setResult(r) })
        }
      />
      <Dialog
        open={result !== null}
        onOpenChange={(o) => !o && (setResult(null), setAsk(null))}
        index="11"
        title={t('ajustes-partes')}
        footer={
          <Button variant="primary" onClick={() => (setResult(null), setAsk(null))}>
            {t('ajustes-partes-ja-guardei')}
          </Button>
        }
      >
        {result ? <ShareList result={result} /> : null}
      </Dialog>
    </div>
  );
}

export function Fido2Card() {
  const client = useQueryClient();
  const recovery = useRecoveryStatus();
  const remove = useFido2Remove();
  const [present, setPresent] = useState(false);
  const [adding, setAdding] = useState(false);
  const [label, setLabel] = useState('');
  const [keyPin, setKeyPin] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [removing, setRemoving] = useState<string | null>(null);
  // Só fala com o USB quando o usuário abre o cadastro (nunca ao abrir Ajustes).
  useEffect(() => {
    if (!adding) return;
    let alive = true;
    void fido2Present().then((p) => alive && setPresent(p));
    return () => {
      alive = false;
    };
  }, [adding]);
  const keys = recovery.data?.fido2 ?? [];
  const register = async () => {
    setBusy(true);
    setError(null);
    try {
      await fido2Register(label, keyPin);
      await client.invalidateQueries({ queryKey: vaultKeys.all });
      toast({ tone: 'success', title: t('ajustes-fido2-cadastrada') });
      setAdding(false);
      setLabel('');
      setKeyPin('');
    } catch (e) {
      setError(e instanceof Error ? e : new Error(String(e)));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="flex flex-col gap-3">
      {keys.length ? (
        <ul
          aria-label={t('ajustes-fido2-lista')}
          className="divide-y divide-line rounded-control border border-line"
        >
          {keys.map((k) => (
            <li key={k.id} className="flex items-center justify-between gap-3 px-3 py-2">
              <div>
                <p className="text-[13px] text-fg">{k.label}</p>
                <p className="text-[12px] text-dim">
                  {t('ajustes-fido2-cadastrada-em', { data: formatDateTime(k.created_at) })}
                </p>
              </div>
              <Button size="sm" variant="ghost" onClick={() => setRemoving(k.id)}>
                {t('ajustes-remover')}
              </Button>
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-[13px] text-dim">{t('ajustes-fido2-texto')}</p>
      )}
      <Button
        variant="secondary"
        disabled={!isTauri()}
        title={isTauri() ? undefined : t('ajustes-fido2-so-no-app')}
        onClick={() => setAdding(true)}
      >
        {t('ajustes-fido2-cadastrar')}
      </Button>
      <Dialog
        open={adding}
        onOpenChange={(o) => !o && (setAdding(false), setError(null))}
        index="11"
        size="sm"
        title={t('ajustes-fido2-cadastrar')}
        description={present ? t('ajustes-fido2-toque') : t('ajustes-fido2-conecte')}
        footer={
          <>
            <Button variant="ghost" onClick={() => setAdding(false)}>
              {t('comum-cancelar')}
            </Button>
            <Button variant="primary" loading={busy} onClick={() => void register()}>
              {t('ajustes-fido2-cadastrar-botao')}
            </Button>
          </>
        }
      >
        <div className="flex flex-col gap-4">
          <Field label={t('ajustes-nome')}>
            <Input
              autoFocus
              value={label}
              placeholder={t('ajustes-fido2-placeholder')}
              onChange={(e) => setLabel(e.target.value)}
            />
          </Field>
          <Field label={t('ajustes-fido2-pin')}>
            <PasswordInput value={keyPin} onChange={(e) => setKeyPin(e.target.value)} />
          </Field>
          <InlineError error={error} />
        </div>
      </Dialog>
      <PasswordDialog
        open={removing !== null}
        title={t('ajustes-fido2-remover')}
        description={t('ajustes-fido2-remover-dica')}
        confirmLabel={t('ajustes-remover')}
        pending={remove.isPending}
        error={remove.error}
        onClose={() => (setRemoving(null), remove.reset())}
        onConfirm={(password) =>
          remove.mutate(
            { id: removing!, password },
            {
              onSuccess: () => (toast({ title: t('ajustes-fido2-removida') }), setRemoving(null)),
            },
          )
        }
      />
    </div>
  );
}
