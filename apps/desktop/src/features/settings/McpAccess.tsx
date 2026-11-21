/**
 * Ajustes › Integrações MCP › Agentes externos: chaves de API com escopos (leitura e
 * escrita como proposta), exigir chave e o servidor MCP HTTP local. A chave aparece uma
 * vez, no diálogo de criação; depois só o prefixo.
 */
import { useEffect, useState } from 'react';

import { isTauri } from '@/api/client';
import type { McpApiKey } from '@/api/generated/McpApiKey';
import type { McpKeyCreated } from '@/api/generated/McpKeyCreated';
import type { McpScope } from '@/api/generated/McpScope';
import type { McpServerSettings } from '@/api/generated/McpServerSettings';
import {
  useCreateMcpKey,
  useDeleteMcpKey,
  useMcpKeys,
  useMcpServerSettings,
  useRevokeMcpKey,
  useDismissClientNotices,
  useDismissRequireKeyNotice,
  useSaveMcpServerSettings,
} from '@/api/hooks/mcp';
import {
  Button,
  Dialog,
  Field,
  Input,
  InstrumentCard,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { Checkbox, InlineError, Switch } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { CopyButton } from './GeneralSections';
import { tRich } from './richText';

const SCOPE_LABEL: Record<McpScope, string> = {
  get read() {
    return t('ajustes-mcp-escopo-leitura');
  },
  get write() {
    return t('ajustes-mcp-escopo-escrita');
  },
};

interface HttpStatus {
  running: boolean;
  url: string | null;
  error: string | null;
}

async function httpApply(): Promise<HttpStatus | null> {
  if (!isTauri()) return null;
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<HttpStatus>('mcp_http_apply');
}

function ServerCard() {
  const q = useMcpServerSettings();
  const save = useSaveMcpServerSettings();
  const dismissNotice = useDismissRequireKeyNotice();
  const dismissClients = useDismissClientNotices();
  const [port, setPort] = useState('');
  const [http, setHttp] = useState<HttpStatus | null>(null);
  const s = q.data;
  useEffect(() => {
    void httpApply().then(setHttp);
  }, []);
  if (!s) return null;
  const update = (patch: Partial<McpServerSettings>) =>
    save.mutate(
      { ...s, ...patch },
      {
        onSuccess: () => {
          toast({ tone: 'success', title: t('ajustes-mcp-atualizado') });
          void httpApply().then(setHttp);
        },
      },
    );
  const url = http?.url ?? `http://127.0.0.1:${s.http_port}/mcp`;
  return (
    <InstrumentCard index="12.15" title={t('ajustes-mcp-agentes')}>
      <div className="flex flex-col gap-4">
        {s.require_key_notice ? (
          <div
            role="status"
            className="flex flex-col gap-2 rounded-control border border-accent/50 bg-accent-soft/40 px-3 py-2.5 text-[12.5px] text-fg"
          >
            <p className="font-medium">{t('ajustes-mcp-chave-agora-titulo')}</p>
            <p className="leading-snug">{t('ajustes-mcp-chave-agora-texto')}</p>
            <div>
              <Button size="sm" variant="secondary" onClick={() => dismissNotice.mutate()}>
                {t('ajustes-mcp-chave-agora-entendi')}
              </Button>
            </div>
          </div>
        ) : null}
        {s.new_clients.length > 0 ? (
          <div
            role="status"
            className="flex flex-col gap-2 rounded-control border border-warning/50 bg-warning/5 px-3 py-2.5 text-[12.5px] text-fg"
          >
            <p className="font-medium">{t('ajustes-mcp-cliente-novo-titulo')}</p>
            <ul className="flex flex-col gap-1">
              {s.new_clients.map((c) => (
                <li key={`${c.at}-${c.client}`} className="leading-snug">
                  {t('ajustes-mcp-cliente-novo-item', {
                    cliente: c.client,
                    programa: c.program ?? t('ajustes-mcp-cliente-programa-desconhecido'),
                    pid: c.pid === null ? '?' : String(c.pid),
                    quando: formatDateTime(c.at),
                  })}
                </li>
              ))}
            </ul>
            <p className="leading-snug text-dim">{t('ajustes-mcp-cliente-novo-texto')}</p>
            <div>
              <Button size="sm" variant="secondary" onClick={() => dismissClients.mutate()}>
                {t('ajustes-mcp-chave-agora-entendi')}
              </Button>
            </div>
          </div>
        ) : null}
        <p className="text-[12.5px] leading-snug text-dim">{t('ajustes-mcp-chave-sempre')}</p>
        <Switch
          checked={s.cli_when_closed}
          onChange={(v) => update({ cli_when_closed: v })}
          label={t('ajustes-mcp-app-fechado')}
          description={t('ajustes-mcp-app-fechado-dica')}
        />
        <div className="flex flex-col gap-2 border-t border-line pt-4">
          <Switch
            checked={s.http_enabled}
            onChange={(v) => update({ http_enabled: v })}
            label={t('ajustes-mcp-http')}
            description={t('ajustes-mcp-http-dica')}
          />
          <div className="flex flex-wrap items-end gap-2">
            <Field label={t('ajustes-mcp-porta')} className="w-32">
              <Input
                numeric
                inputMode="numeric"
                value={port || String(s.http_port)}
                onChange={(e) => setPort(e.target.value.replace(/\D/g, ''))}
              />
            </Field>
            <Button
              size="sm"
              variant="secondary"
              disabled={!port || Number(port) === s.http_port}
              onClick={() => (update({ http_port: Number(port) }), setPort(''))}
            >
              {t('ajustes-mcp-salvar-porta')}
            </Button>
          </div>
          {s.http_enabled ? (
            <p className="text-[12.5px] text-dim">
              {http?.error ? (
                <span className="text-negative">
                  {t('ajustes-mcp-nao-subiu', { erro: http.error })}
                </span>
              ) : (
                <>
                  {tRich('ajustes-mcp-endereco', {
                    url: (
                      <code data-selectable className="num text-fg">
                        {url}
                      </code>
                    ),
                  })}
                  {` · ${
                    isTauri()
                      ? http?.running
                        ? t('ajustes-mcp-no-ar')
                        : t('ajustes-mcp-iniciando')
                      : t('ajustes-mcp-so-no-app')
                  }`}
                </>
              )}
            </p>
          ) : null}
        </div>
        <InlineError error={save.error} />
      </div>
    </InstrumentCard>
  );
}

function CreatedKey({ made, port }: { made: McpKeyCreated; port: number }) {
  const stdio = `TREASURY_API_KEY=${made.key}`;
  const http = `Authorization: Bearer ${made.key}\nhttp://127.0.0.1:${port}/mcp`;
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[13px] text-fg">
        {t('ajustes-mcp-copie-agora', {
          escopos: made.info.scopes.map((s) => SCOPE_LABEL[s]).join(', '),
        })}
      </p>
      <code
        data-selectable
        className="num rounded-control border border-line bg-bg p-2 text-[12px] break-all text-fg"
      >
        {made.key}
      </code>
      <CopyButton secret text={made.key} label={t('ajustes-mcp-copiar-chave')} />
      <div className="grid gap-3 border-t border-line pt-3 text-[12.5px] text-dim @min-[400px]:grid-cols-2">
        <div className="flex flex-col gap-1">
          <span className="label-caps">{'stdio (treasury mcp)'}</span>
          <code data-selectable className="num break-all text-fg">
            {stdio}
          </code>
        </div>
        <div className="flex flex-col gap-1">
          <span className="label-caps">{'HTTP'}</span>
          <code data-selectable className="num break-all whitespace-pre-wrap text-fg">
            {http}
          </code>
        </div>
      </div>
    </div>
  );
}

/** Dias que uma chave revogada fica na lista (como `REVOKED_KEEP_DAYS` no núcleo). */
export const REVOKED_KEEP_DAYS = 7;

function leavesListAt(revokedAt: string): string {
  return new Date(Date.parse(revokedAt) + REVOKED_KEEP_DAYS * 86_400_000).toISOString();
}

function KeysCard() {
  const keys = useMcpKeys();
  const settings = useMcpServerSettings();
  const create = useCreateMcpKey();
  const revoke = useRevokeMcpKey();
  const del = useDeleteMcpKey();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState('');
  const [scopes, setScopes] = useState<McpScope[]>(['read']);
  const [days, setDays] = useState('0');
  const [made, setMade] = useState<McpKeyCreated | null>(null);
  const toggle = (s: McpScope, on: boolean) =>
    setScopes((cur) => (on ? [...new Set([...cur, s])] : cur.filter((x) => x !== s)));
  const close = () => {
    setOpen(false);
    setMade(null);
    setName('');
    setScopes(['read']);
    setDays('0');
    create.reset();
  };
  // Revogada: esmaecida até sair da lista (7 dias depois, ou antes, ao excluir).
  const faded = (k: McpApiKey) => (k.revoked_at ? 'opacity-45' : undefined);
  const columns: TechColumn<McpApiKey>[] = [
    {
      key: 'name',
      header: t('ajustes-nome'),
      cell: (k) => (
        <span className={cx('flex flex-col', faded(k))}>
          <span className={k.revoked_at ? 'text-dim line-through' : 'text-fg'}>{k.name}</span>
          <span className="num text-[11.5px] text-dim">{k.prefix}…</span>
        </span>
      ),
    },
    {
      key: 'scopes',
      header: t('ajustes-mcp-escopos'),
      cell: (k) => (
        <span className={faded(k)}>{k.scopes.map((s) => SCOPE_LABEL[s]).join(' · ')}</span>
      ),
    },
    {
      key: 'used',
      header: t('ajustes-mcp-ultimo-uso'),
      width: '140px',
      cell: (k) => (
        <span className={faded(k)}>
          {k.last_used_at ? formatDateTime(k.last_used_at) : t('ajustes-nunca')}
        </span>
      ),
    },
    {
      key: 'state',
      header: t('ajustes-situacao'),
      width: '250px',
      cell: (k) =>
        k.revoked_at ? (
          <span className="flex items-center gap-2">
            <span className="flex flex-col text-dim opacity-70">
              <span>{t('ajustes-mcp-revogada-em', { data: formatDateTime(k.revoked_at) })}</span>
              <span className="text-[11.5px]">
                {t('ajustes-mcp-sai-da-lista', {
                  data: formatDateTime(leavesListAt(k.revoked_at)),
                })}
              </span>
            </span>
            <Button
              size="sm"
              variant="ghost"
              loading={del.isPending && del.variables?.id === k.id}
              onClick={() =>
                del.mutate(
                  { id: k.id },
                  {
                    onSuccess: () =>
                      toast({ title: t('ajustes-mcp-chave-excluida'), description: k.name }),
                  },
                )
              }
            >
              {t('ajustes-mcp-excluir')}
            </Button>
          </span>
        ) : (
          <span className="flex items-center gap-2">
            <span className="text-positive">
              {k.expires_at
                ? t('ajustes-mcp-ate', { data: formatDateTime(k.expires_at) })
                : t('ajustes-mcp-ativa')}
            </span>
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                revoke.mutate(
                  { id: k.id },
                  {
                    onSuccess: () =>
                      toast({ title: t('ajustes-mcp-chave-revogada'), description: k.name }),
                  },
                )
              }
            >
              {t('ajustes-mcp-revogar')}
            </Button>
          </span>
        ),
    },
  ];
  return (
    <InstrumentCard
      index="12.16"
      title={t('ajustes-mcp-chaves')}
      padded={false}
      actions={
        <Button size="sm" variant="primary" onClick={() => setOpen(true)}>
          {t('ajustes-mcp-nova-chave')}
        </Button>
      }
    >
      {(keys.data ?? []).length === 0 ? (
        <p className="px-4 py-6 text-center text-[13px] text-dim">{t('ajustes-mcp-sem-chaves')}</p>
      ) : (
        <TechTable
          label={t('ajustes-mcp-chaves')}
          prefsId="Chaves de API"
          dense
          columns={columns}
          rows={keys.data ?? []}
          getRowId={(k) => k.id}
        />
      )}
      <Dialog
        open={open}
        onOpenChange={(o) => !o && close()}
        index="11"
        title={made ? t('ajustes-mcp-chave-criada') : t('ajustes-mcp-nova-chave-titulo')}
        description={made ? undefined : t('ajustes-mcp-nova-chave-dica')}
        footer={
          made ? (
            <Button variant="primary" onClick={close}>
              {t('ajustes-mcp-ja-copiei')}
            </Button>
          ) : (
            <>
              <Button variant="ghost" onClick={close}>
                {t('comum-cancelar')}
              </Button>
              <Button
                variant="primary"
                disabled={!name.trim()}
                loading={create.isPending}
                onClick={() =>
                  create.mutate(
                    {
                      name: name.trim(),
                      scopes,
                      ...(Number(days) > 0 ? { expires_days: Number(days) } : {}),
                    },
                    { onSuccess: setMade },
                  )
                }
              >
                {t('ajustes-mcp-criar-chave')}
              </Button>
            </>
          )
        }
      >
        {made ? (
          <CreatedKey made={made} port={settings.data?.http_port ?? 7655} />
        ) : (
          <div className="flex flex-col gap-4">
            <Field label={t('ajustes-nome')}>
              <Input
                autoFocus
                value={name}
                placeholder={t('ajustes-mcp-nome-placeholder')}
                onChange={(e) => setName(e.target.value)}
              />
            </Field>
            <fieldset className="flex flex-col gap-2">
              <legend className="label-caps mb-1">{t('ajustes-mcp-escopos')}</legend>
              <Checkbox
                checked
                disabled
                onChange={() => {}}
                label={t('ajustes-mcp-leitura-sempre')}
              />
              <Checkbox
                checked={scopes.includes('write')}
                onChange={(v) => toggle('write', v)}
                label={t('ajustes-mcp-escrita-opcao')}
              />
            </fieldset>
            <Field label={t('ajustes-mcp-validade')}>
              <Select
                value={days}
                onValueChange={setDays}
                options={[
                  { value: '0', label: t('ajustes-mcp-sem-vencimento') },
                  { value: '30', label: t('ajustes-mcp-dias', { n: 30 }) },
                  { value: '90', label: t('ajustes-mcp-dias', { n: 90 }) },
                  { value: '365', label: t('ajustes-mcp-um-ano') },
                ]}
              />
            </Field>
            <InlineError error={create.error} />
          </div>
        )}
      </Dialog>
    </InstrumentCard>
  );
}

export function McpAccess() {
  return (
    <div className="grid gap-5 @min-[1040px]:grid-cols-2">
      <ServerCard />
      <KeysCard />
    </div>
  );
}
