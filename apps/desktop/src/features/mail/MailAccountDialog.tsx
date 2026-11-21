/**
 * Assistente da conta do e-mail de ingestão: a pasta local (sem credencial) ou a caixa IMAP
 * com senha de app (o catálogo preenche o servidor; fora dele, o autoconfig do domínio). A
 * senha vai só para o cofre (cifra por campo) e nunca volta; depois de salvar, o Testar
 * mostra as pastas que serão lidas e como a retenção apagaria nesse servidor.
 */
import { useState } from 'react';

import type { LocalRetention } from '@/api/generated/LocalRetention';
import type { MailAccount } from '@/api/generated/MailAccount';
import type { MailAccountInput } from '@/api/generated/MailAccountInput';
import type { MailConnectionTest } from '@/api/generated/MailConnectionTest';
import type { MailSecurity } from '@/api/generated/MailSecurity';
import type { MailboxKind } from '@/api/generated/MailboxKind';
import type { RetentionMode } from '@/api/generated/RetentionMode';
import { isTauri } from '@/api/client';
import {
  pickMailFolder,
  useDiscoverMail,
  useMailProviders,
  useSaveMailAccount,
  useSetMailPassword,
  useTestMailConnection,
} from '@/api/hooks/mail';
import {
  Button,
  Dialog,
  Field,
  Icon,
  Input,
  PasswordInput,
  Select,
  toast,
} from '@/design/components';
import { openExternal } from '@/app/externalLinks';
import {
  FileInput,
  InlineError,
  Segmented,
  Switch,
  TextArea,
} from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

import { RETENTION_LABEL, joinList, splitList } from './labels';

type Kind = 'imap' | 'folder';

const BLOCKED_DEFAULT = '\\Junk, \\Trash, \\Sent, \\Drafts, \\All';

export function MailAccountDialog({
  account,
  onClose,
}: {
  account: MailAccount | null;
  onClose: () => void;
}) {
  const providers = useMailProviders().data ?? [];
  const save = useSaveMailAccount();
  const setPassword = useSetMailPassword();
  const discover = useDiscoverMail();
  const test = useTestMailConnection();
  const [kind, setKind] = useState<Kind>(account?.kind ?? 'imap');
  const [name, setName] = useState(account?.name ?? t('email-conta-nome-padrao'));
  const [folder, setFolder] = useState(account?.folder_path ?? '');
  const [address, setAddress] = useState(account?.username ?? '');
  const [provider, setProvider] = useState(account?.provider ?? 'gmail');
  const [host, setHost] = useState(account?.host ?? '');
  const [port, setPort] = useState(String(account?.port ?? ''));
  const [security, setSecurity] = useState<MailSecurity>(account?.security ?? 'tls');
  const [username, setUsername] = useState(account?.username ?? '');
  const [password, setPw] = useState('');
  const [cert, setCert] = useState<string | null>(null);
  const [mailbox, setMailbox] = useState<MailboxKind>(account?.mailbox_kind ?? 'dedicated');
  const [allowed, setAllowed] = useState(joinList(account?.allowed_folders) || 'INBOX');
  const [blocked, setBlocked] = useState(
    account ? joinList(account.blocked_folders) : BLOCKED_DEFAULT,
  );
  const [allowSenders, setAllowSenders] = useState(joinList(account?.allow_senders));
  const [blockSenders, setBlockSenders] = useState(joinList(account?.block_senders));
  const [family, setFamily] = useState(joinList(account?.family_addresses));
  const [fallback, setFallback] = useState<'ignore' | 'review'>(
    account?.default_action === 'review' ? 'review' : account ? 'ignore' : 'ignore',
  );
  const [interval, setIntervalMin] = useState(String(account?.interval_minutes ?? 15));
  const [keep, setKeep] = useState(account?.keep_original ?? true);
  const [mode, setMode] = useState<RetentionMode>(account?.retention.mode ?? 'never');
  const [days, setDays] = useState(String(account?.retention.days ?? 30));
  const [grace, setGrace] = useState(String(account?.retention.grace_days ?? 7));
  const [local, setLocal] = useState<LocalRetention>(account?.retention.local ?? 'move');
  const [purge, setPurge] = useState(account?.retention.purge_trash ?? false);
  const [result, setResult] = useState<MailConnectionTest | null>(null);
  const [saved, setSaved] = useState<MailAccount | null>(account);

  const prov = providers.find((p) => p.id === provider);
  const gmail = Boolean(prov?.gmail);
  const proton = provider === 'proton';

  const lookup = () =>
    discover.mutate(
      { address },
      {
        onSuccess: (d) => {
          setProvider(d.provider ?? 'manual');
          setHost(d.host);
          setPort(String(d.port));
          setSecurity(d.security);
          setUsername(d.username);
        },
      },
    );

  const input = (): MailAccountInput => ({
    ...(saved ? { id: saved.id } : {}),
    name: name.trim(),
    kind,
    ...(kind === 'folder'
      ? { folder_path: folder.trim() }
      : {
          provider,
          host: host.trim() || undefined,
          port: port.trim() ? Number(port) : undefined,
          security,
          username: username.trim(),
          ...(cert ? { pinned_cert: cert } : {}),
          mailbox_kind: mailbox,
        }),
    allowed_folders: splitList(allowed),
    blocked_folders: splitList(blocked),
    allow_senders: splitList(allowSenders),
    block_senders: splitList(blockSenders),
    family_addresses: splitList(family),
    default_action: fallback,
    interval_minutes: Number(interval) || 15,
    keep_original: keep,
    retention: {
      mode,
      days: Number(days) || 30,
      grace_days: Number(grace) || 0,
      local,
      purge_trash: purge,
    },
  });

  const submit = async () => {
    try {
      const a = await save.mutateAsync(input());
      setSaved(a);
      if (a.kind === 'imap' && password.trim()) {
        const status = await setPassword.mutateAsync({ id: a.id, password: password.trim() });
        setSaved({ ...a, password: status });
        setPw('');
      }
      if (a.kind === 'imap' && (a.password?.set || password.trim())) {
        const r = await test.mutateAsync({ id: a.id });
        if ('reads' in r) setResult(r);
      } else {
        onClose();
      }
      toast({ tone: 'success', title: t('email-conta-salva'), description: a.name });
    } catch {
      /* o erro aparece no formulário */
    }
  };

  const error = save.error ?? setPassword.error ?? test.error ?? discover.error;

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      size="lg"
      resizable
      resizeId="mail-account"
      title={account ? t('email-conta-titulo', { nome: account.name }) : t('email-conta-nova')}
      description={t('email-conta-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {result ? t('comum-fechar') : t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={save.isPending || setPassword.isPending || test.isPending}
            disabled={!name.trim() || (kind === 'folder' ? !folder.trim() : !username.trim())}
            onClick={() => void submit()}
          >
            {kind === 'imap' ? t('email-salvar-testar') : t('comum-salvar')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        {!account ? (
          <Segmented
            label={t('email-tipo-conta')}
            fill
            value={kind}
            onChange={setKind}
            options={[
              { value: 'imap', label: t('email-tipo-imap') },
              { value: 'folder', label: t('email-tipo-pasta') },
            ]}
          />
        ) : null}
        <Field label={t('email-nome')}>
          <Input value={name} onChange={(e) => setName(e.target.value)} />
        </Field>

        {kind === 'folder' ? (
          <Field label={t('email-pasta-vigiada')} hint={t('email-pasta-vigiada-dica')}>
            <div className="flex gap-2">
              <Input
                className="flex-1"
                value={folder}
                placeholder={t('email-pasta-exemplo')}
                onChange={(e) => setFolder(e.target.value)}
              />
              {isTauri() ? (
                <Button
                  variant="secondary"
                  onClick={() => void pickMailFolder().then((p) => p && setFolder(p))}
                >
                  {t('email-escolher')}
                </Button>
              ) : null}
            </div>
          </Field>
        ) : (
          <>
            <div className="grid gap-3 @min-[640px]:grid-cols-[1fr_auto] @min-[640px]:items-end">
              <Field label={t('email-endereco')} hint={t('email-endereco-dica')}>
                <Input
                  value={address}
                  placeholder={t('email-endereco-exemplo')}
                  onChange={(e) => setAddress(e.target.value)}
                />
              </Field>
              <Button
                variant="secondary"
                loading={discover.isPending}
                disabled={!address.includes('@')}
                onClick={lookup}
              >
                {t('email-procurar-config')}
              </Button>
            </div>
            <div className="grid gap-3 @min-[640px]:grid-cols-3">
              <Field label={t('email-provedor')}>
                <Select
                  value={provider}
                  onValueChange={setProvider}
                  options={[
                    ...providers.map((p) => ({ value: p.id, label: p.label })),
                    { value: 'manual', label: t('email-provedor-manual') },
                  ]}
                />
              </Field>
              <Field label={t('email-servidor')}>
                <Input
                  value={host}
                  placeholder={prov?.host ?? t('email-servidor-exemplo')}
                  onChange={(e) => setHost(e.target.value)}
                />
              </Field>
              <Field label={t('email-porta')}>
                <Input
                  value={port}
                  inputMode="numeric"
                  placeholder={String(prov?.port ?? 993)}
                  onChange={(e) => setPort(e.target.value)}
                />
              </Field>
            </div>
            {prov ? (
              <p className="flex items-start gap-2 text-[12.5px] text-dim">
                <Icon name="key" size={12} className="mt-0.5 shrink-0" />
                <span>
                  {prov.help}{' '}
                  <button
                    type="button"
                    className="text-accent underline-offset-2 hover:underline"
                    onClick={() => void openExternal(prov.app_password_url)}
                  >
                    {t('email-onde-criar')}
                  </button>
                </span>
              </p>
            ) : (
              <Field label={t('email-seguranca')} hint={t('email-seguranca-dica')}>
                <Select
                  value={security}
                  onValueChange={(v) => setSecurity(v as MailSecurity)}
                  options={[
                    { value: 'tls', label: 'TLS (993)' },
                    { value: 'starttls', label: t('email-starttls') },
                  ]}
                />
              </Field>
            )}
            <div className="grid gap-3 @min-[640px]:grid-cols-2">
              <Field label={t('email-usuario')}>
                <Input value={username} onChange={(e) => setUsername(e.target.value)} />
              </Field>
              <Field
                label={t('email-senha-app')}
                hint={
                  saved?.password?.set
                    ? t('email-senha-cadastrada', { dica: saved.password.hint ?? '' })
                    : t('email-senha-cifrada')
                }
              >
                <PasswordInput
                  value={password}
                  autoComplete="off"
                  onChange={(e) => setPw(e.target.value)}
                />
              </Field>
            </div>
            <p className="text-[12.5px] text-dim">{t('email-senha-aviso')}</p>
            {proton ? (
              <Field
                label={t('email-certificado')}
                hint={
                  saved?.pinned_cert || cert
                    ? t('email-certificado-fixado')
                    : t('email-certificado-onde')
                }
              >
                <FileInput
                  accept=".pem,application/x-pem-file"
                  onChange={async (e) => {
                    const f = e.target.files?.[0];
                    e.target.value = '';
                    if (f) setCert(await f.text());
                  }}
                />
              </Field>
            ) : null}
            <Segmented
              label={t('email-que-caixa')}
              fill
              value={mailbox}
              onChange={setMailbox}
              options={[
                { value: 'dedicated', label: t('email-caixa-dedicada') },
                { value: 'main', label: t('email-caixa-principal') },
              ]}
            />
            {mailbox === 'main' ? (
              <p className="text-[12.5px] text-warning">
                {t('email-caixa-principal-aviso')} {gmail ? t('email-gmail-lixeira-aviso') : ''}
              </p>
            ) : null}
          </>
        )}

        <details className="rounded-card border border-line px-3 py-2">
          <summary className="cursor-pointer text-[13px] text-fg">
            {t('email-pastas-remetentes')}
          </summary>
          <div className="mt-3 flex flex-col gap-3">
            <Field
              label={t('email-pastas-lidas')}
              hint={
                kind === 'folder'
                  ? t('email-pastas-lidas-dica-pasta')
                  : t('email-pastas-lidas-dica-imap')
              }
            >
              <Input value={allowed} onChange={(e) => setAllowed(e.target.value)} />
            </Field>
            <Field label={t('email-pastas-bloqueadas')} hint={t('email-pastas-bloqueadas-dica')}>
              <Input value={blocked} onChange={(e) => setBlocked(e.target.value)} />
            </Field>
            <Field
              label={t('email-remetentes-permitidos')}
              hint={t('email-remetentes-permitidos-dica')}
            >
              <TextArea
                rows={2}
                value={allowSenders}
                onChange={(e) => setAllowSenders(e.target.value)}
              />
            </Field>
            <Field label={t('email-remetentes-bloqueados')}>
              <TextArea
                rows={2}
                value={blockSenders}
                onChange={(e) => setBlockSenders(e.target.value)}
              />
            </Field>
            <Field label={t('email-enderecos-familia')} hint={t('email-enderecos-familia-dica')}>
              <Input value={family} onChange={(e) => setFamily(e.target.value)} />
            </Field>
            <Field label={t('email-sem-regra-case')}>
              <Select
                value={fallback}
                onValueChange={(v) => setFallback(v as 'ignore' | 'review')}
                options={[
                  { value: 'ignore', label: t('email-padrao-ignorar') },
                  { value: 'review', label: t('email-padrao-revisao') },
                ]}
              />
            </Field>
          </div>
        </details>

        <details className="rounded-card border border-line px-3 py-2">
          <summary className="cursor-pointer text-[13px] text-fg">
            {t('email-buscas-retencao')}
          </summary>
          <div className="mt-3 flex flex-col gap-3">
            <div className="grid gap-3 @min-[640px]:grid-cols-2">
              <Field label={t('email-intervalo')} hint={t('email-intervalo-dica')}>
                <Input
                  value={interval}
                  inputMode="numeric"
                  onChange={(e) => setIntervalMin(e.target.value)}
                />
              </Field>
              <Field label={t('email-apagar-provedor')}>
                <Select
                  value={mode}
                  onValueChange={(v) => setMode(v as RetentionMode)}
                  options={(Object.keys(RETENTION_LABEL) as RetentionMode[]).map((m) => ({
                    value: m,
                    label: RETENTION_LABEL[m],
                  }))}
                />
              </Field>
            </div>
            {mode !== 'never' ? (
              <>
                <div className="grid gap-3 @min-[640px]:grid-cols-2">
                  {mode !== 'after_days' ? (
                    <Field label={t('email-carencia-importada')}>
                      <Input
                        value={grace}
                        inputMode="numeric"
                        onChange={(e) => setGrace(e.target.value)}
                      />
                    </Field>
                  ) : null}
                  {mode !== 'after_success' ? (
                    <Field label={t('email-quantos-dias')} hint={t('email-quantos-dias-dica')}>
                      <Input
                        value={days}
                        inputMode="numeric"
                        onChange={(e) => setDays(e.target.value)}
                      />
                    </Field>
                  ) : null}
                </div>
                {kind === 'folder' ? (
                  <Segmented
                    label={t('email-arquivo-destino')}
                    fill
                    value={local}
                    onChange={setLocal}
                    options={[
                      { value: 'move', label: t('email-mover-processados') },
                      { value: 'trash', label: t('email-mandar-lixeira') },
                    ]}
                  />
                ) : gmail ? (
                  <Switch
                    checked={purge}
                    onChange={setPurge}
                    label={t('email-purgar-gmail')}
                    description={t('email-purgar-gmail-dica')}
                  />
                ) : null}
                <p className="text-[12.5px] text-warning">
                  <Icon name="alert" size={12} className="mr-1 inline" />
                  {t('email-retencao-aviso')}
                </p>
              </>
            ) : null}
            <Switch
              checked={keep}
              onChange={setKeep}
              label={t('email-guardar-original')}
              description={t('email-guardar-original-dica')}
            />
          </div>
        </details>

        {result ? (
          <div className="flex flex-col gap-1 rounded-card border border-positive/50 bg-positive/5 px-3 py-2 text-[12.5px]">
            <p className="text-positive">
              <Icon name="check" size={12} className="mr-1 inline" />
              {t('email-conectou', {
                pastas: joinList(result.reads) || t('email-conectou-nenhuma'),
              })}
            </p>
            <p className="text-dim">{t('email-para-apagar', { como: result.delete_how })}</p>
          </div>
        ) : null}
        <InlineError error={error} />
      </div>
    </Dialog>
  );
}
