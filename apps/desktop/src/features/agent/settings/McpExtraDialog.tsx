/**
 * Cadastro e edição de um MCP extra do cofre: rótulo, nome (prefixo das ferramentas), tipo
 * (comando local ou endereço HTTP), comando e argumentos ou URL, e as variáveis de ambiente
 * ou cabeçalhos. Um valor secreto já cadastrado nunca volta: aparece mascarado (`••••…a1b2`)
 * e o campo em branco mantém o que está gravado.
 */
import { useState } from 'react';

import type { McpServer } from '@/api/generated/McpServer';
import type { McpServerInput } from '@/api/generated/McpServerInput';
import type { McpTransport } from '@/api/generated/McpTransport';
import type { McpVar } from '@/api/generated/McpVar';
import type { McpVarInput } from '@/api/generated/McpVarInput';
import { useSaveMcpExtra } from '@/api/hooks/ai';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Icon,
  Input,
  PasswordInput,
  Select,
  toast,
  useFieldControl,
} from '@/design/components';
import { errorMessage } from '@/features/portfolio/components/States';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';

import { Mono, Toggle } from '../ui';

/** Mesma regra do núcleo (`mcp_extra::validate`). */
const NAME_RE = /^[a-z][a-z0-9_]{1,23}$/;
const RESERVED = 'treasury';

/** Exemplo de cabeçalho de acesso (texto técnico, igual em todos os idiomas). */
const AUTH_EXAMPLE = 'Authorization: Bearer …';

export const validMcpName = (name: string) =>
  NAME_RE.test(name) && !name.includes('__') && name !== RESERVED;

/** Valor secreto já cadastrado, mascarado (só os 4 últimos caracteres). */
export const maskedSecret = (hint: string | null) => `••••${hint ?? ''}`;

interface VarRow {
  /** Chave estável da linha (React). */
  uid: number;
  key: string;
  value: string;
  secret: boolean;
  /** Secreto já gravado (e a dica dele). */
  saved: { hint: string | null } | null;
}

let rowSeq = 0;
const rowsOf = (vars: readonly McpVar[]): VarRow[] =>
  vars.map((v) => ({
    uid: ++rowSeq,
    key: v.key,
    value: v.secret ? '' : (v.value ?? ''),
    secret: v.secret,
    saved: v.secret && v.set ? { hint: v.hint } : null,
  }));

/** Linhas vazias saem; o secreto em branco vai sem valor (o núcleo mantém o gravado). */
function toInput(rows: readonly VarRow[]): McpVarInput[] {
  return rows
    .filter((r) => r.key.trim() !== '')
    .map((r) =>
      r.secret && r.value === ''
        ? { key: r.key.trim(), secret: true }
        : { key: r.key.trim(), value: r.value, secret: r.secret },
    );
}

/** Variáveis do servidor como cadastro, sem tocar nos segredos gravados. */
export function varsToInput(vars: readonly McpVar[]): McpVarInput[] {
  return vars.map((v) =>
    v.secret ? { key: v.key, secret: true } : { key: v.key, value: v.value ?? '', secret: false },
  );
}

/** O servidor como está, pronto para salvar de novo com uma mudança. */
export function serverToInput(s: McpServer): McpServerInput {
  return {
    id: s.id,
    name: s.name,
    label: s.label,
    transport: s.transport,
    ...(s.command != null ? { command: s.command } : {}),
    args: s.args,
    ...(s.url != null ? { url: s.url } : {}),
    env: varsToInput(s.env),
    headers: varsToInput(s.headers),
    enabled: s.enabled,
    auto_approve: s.auto_approve,
  };
}

function TextArea({
  value,
  onChange,
  rows = 3,
}: {
  value: string;
  onChange: (v: string) => void;
  rows?: number;
}) {
  const control = useFieldControl({});
  return (
    <textarea
      id={control.id}
      aria-describedby={control['aria-describedby']}
      rows={rows}
      value={value}
      spellCheck={false}
      onChange={(e) => onChange(e.target.value)}
      className="w-full rounded-control border border-line-strong bg-bg px-2.5 py-2 font-mono text-[12px] text-fg outline-none focus:border-accent"
    />
  );
}

function VarEditor({
  rows,
  onChange,
  rowLabel,
  addLabel,
  keyPlaceholder,
}: {
  rows: VarRow[];
  onChange: (rows: VarRow[]) => void;
  rowLabel: (n: number) => string;
  addLabel: string;
  keyPlaceholder: string;
}) {
  const patch = (uid: number, p: Partial<VarRow>) =>
    onChange(rows.map((r) => (r.uid === uid ? { ...r, ...p } : r)));
  return (
    <div className="flex flex-col gap-2">
      {rows.map((r, i) => (
        <div
          key={r.uid}
          role="group"
          aria-label={rowLabel(i + 1)}
          className="flex flex-wrap items-center gap-2"
        >
          <Input
            aria-label={t('agente-mcpx-chave')}
            value={r.key}
            placeholder={keyPlaceholder}
            spellCheck={false}
            autoComplete="off"
            wrapperClassName="w-[180px]"
            className="font-mono text-[12px]"
            onChange={(e) => patch(r.uid, { key: e.target.value })}
          />
          {r.secret ? (
            <PasswordInput
              aria-label={t('agente-mcpx-valor')}
              value={r.value}
              autoComplete="off"
              placeholder={r.saved ? t('agente-mcpx-manter-segredo') : undefined}
              wrapperClassName="min-w-[160px] flex-1"
              onChange={(e) => patch(r.uid, { value: e.target.value })}
            />
          ) : (
            <Input
              aria-label={t('agente-mcpx-valor')}
              value={r.value}
              spellCheck={false}
              autoComplete="off"
              wrapperClassName="min-w-[160px] flex-1"
              className="font-mono text-[12px]"
              onChange={(e) => patch(r.uid, { value: e.target.value })}
            />
          )}
          <Toggle
            label={t('agente-mcpx-secreta')}
            checked={r.secret}
            onChange={(secret) => patch(r.uid, { secret, value: '' })}
          />
          <Button
            size="sm"
            variant="ghost"
            icon={<Icon name="x" size={12} />}
            onClick={() => onChange(rows.filter((x) => x.uid !== r.uid))}
          >
            {t('agente-mcpx-remover')}
          </Button>
          {r.secret && r.saved ? (
            <span className="basis-full font-mono text-[11px] text-dim">
              {`${maskedSecret(r.saved.hint)} · ${t('agente-mcpx-manter-segredo')}`}
            </span>
          ) : null}
        </div>
      ))}
      <div>
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="plus" size={12} />}
          onClick={() =>
            onChange([...rows, { uid: ++rowSeq, key: '', value: '', secret: false, saved: null }])
          }
        >
          {addLabel}
        </Button>
      </div>
    </div>
  );
}

export interface McpExtraDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Ausente: cadastro novo. */
  server?: McpServer;
}

export function McpExtraDialog({ open, onOpenChange, server }: McpExtraDialogProps) {
  const editing = Boolean(server);
  const [label, setLabel] = useState(server?.label ?? '');
  const [name, setName] = useState(server?.name ?? '');
  const [transport, setTransport] = useState<McpTransport>(server?.transport ?? 'stdio');
  const [command, setCommand] = useState(server?.command ?? '');
  const [args, setArgs] = useState((server?.args ?? []).join('\n'));
  const [url, setUrl] = useState(server?.url ?? '');
  const [env, setEnv] = useState<VarRow[]>(() => rowsOf(server?.env ?? []));
  const [headers, setHeaders] = useState<VarRow[]>(() => rowsOf(server?.headers ?? []));
  const [enabled, setEnabled] = useState(server?.enabled ?? true);
  const [autoApprove, setAutoApprove] = useState(server?.auto_approve ?? false);
  const [tried, setTried] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const save = useSaveMcpExtra();

  const stdio = transport === 'stdio';
  const nameError = validMcpName(name) ? null : t('agente-mcpx-nome-erro');
  const labelError =
    label.trim() !== '' && label.trim().length <= 60 ? null : t('agente-mcpx-rotulo-erro');
  const commandError = stdio && command.trim() === '' ? t('agente-mcpx-comando-erro') : null;
  const urlError =
    !stdio && !/^(https:\/\/|http:\/\/(localhost|127\.0\.0\.1|\[::1\]))/.test(url.trim())
      ? t('agente-mcpx-url-erro')
      : null;
  const invalid = Boolean(nameError ?? labelError ?? commandError ?? urlError);

  const submit = async () => {
    setTried(true);
    setError(null);
    if (invalid) return;
    const input: McpServerInput = {
      ...(server ? { id: server.id } : {}),
      name,
      label: label.trim(),
      transport,
      ...(stdio ? { command: command.trim() } : { url: url.trim() }),
      args: stdio
        ? args
            .split('\n')
            .map((a) => a.trim())
            .filter(Boolean)
        : [],
      env: stdio ? toInput(env) : [],
      headers: stdio ? [] : toInput(headers),
      enabled,
      auto_approve: autoApprove,
    };
    try {
      await save.mutateAsync(input);
      toast({ tone: 'success', title: t('agente-mcpx-salvo') });
      onOpenChange(false);
    } catch (e) {
      setError(errorMessage(e));
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={
        editing ? t('agente-mcpx-editar', { nome: server?.label ?? '' }) : t('agente-mcpx-novo')
      }
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button variant="primary" loading={save.isPending} onClick={() => void submit()}>
            {editing ? t('comum-salvar') : t('agente-cadastrar')}
          </Button>
        </>
      }
    >
      <p className="flex items-start gap-2 text-[12px] text-warning" role="note">
        <Icon name="alert" size={12} className="mt-0.5 shrink-0" />
        {t('agente-mcpx-seguranca')}
      </p>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Field
          label={t('agente-mcpx-rotulo')}
          hint={t('agente-mcpx-rotulo-dica')}
          error={tried ? labelError : undefined}
        >
          <Input value={label} onChange={(e) => setLabel(e.target.value)} />
        </Field>
        <Field
          label={t('agente-mcpx-nome')}
          hint={t('agente-mcpx-nome-dica', { exemplo: `${name || 'celular'}-abrir_app` })}
          error={tried || name !== '' ? (nameError ?? undefined) : undefined}
        >
          <Input
            value={name}
            spellCheck={false}
            autoComplete="off"
            className="font-mono text-[12.5px]"
            placeholder="celular"
            onChange={(e) => setName(e.target.value)}
          />
        </Field>
        <Field label={t('agente-mcpx-tipo')}>
          <Select
            value={transport}
            onValueChange={(v) => setTransport(v as McpTransport)}
            options={[
              { value: 'stdio', label: t('agente-mcpx-tipo-stdio') },
              { value: 'http', label: t('agente-mcpx-tipo-http') },
            ]}
          />
        </Field>
        {stdio ? (
          <Field
            label={t('agente-mcpx-comando')}
            hint={t('agente-mcpx-comando-dica')}
            error={tried ? (commandError ?? undefined) : undefined}
          >
            <Input
              value={command}
              spellCheck={false}
              autoComplete="off"
              className="font-mono text-[12.5px]"
              placeholder="npx"
              onChange={(e) => setCommand(e.target.value)}
            />
          </Field>
        ) : (
          <Field
            label={t('agente-mcpx-url')}
            hint={t('agente-mcpx-url-dica')}
            error={tried ? (urlError ?? undefined) : undefined}
          >
            <Input
              value={url}
              spellCheck={false}
              autoComplete="off"
              className="font-mono text-[12.5px]"
              placeholder="https://"
              onChange={(e) => setUrl(e.target.value)}
            />
          </Field>
        )}
      </div>
      {stdio ? (
        <>
          <Field label={t('agente-mcpx-argumentos')} hint={t('agente-mcpx-argumentos-dica')}>
            <TextArea value={args} onChange={setArgs} />
          </Field>
          <fieldset className="flex flex-col gap-2">
            <legend className="label-caps mb-1">{t('agente-mcpx-variaveis')}</legend>
            <VarEditor
              rows={env}
              onChange={setEnv}
              rowLabel={(n) => t('agente-mcpx-variavel-n', { n })}
              addLabel={t('agente-mcpx-adicionar-variavel')}
              keyPlaceholder="API_TOKEN"
            />
          </fieldset>
        </>
      ) : (
        <fieldset className="flex flex-col gap-2">
          <legend className="label-caps mb-1">{t('agente-mcpx-cabecalhos')}</legend>
          <p className="text-[12px] text-dim">
            {tRich('agente-mcpx-cabecalhos-dica', {
              exemplo: <Mono>{AUTH_EXAMPLE}</Mono>,
            })}
          </p>
          <VarEditor
            rows={headers}
            onChange={setHeaders}
            rowLabel={(n) => t('agente-mcpx-cabecalho-n', { n })}
            addLabel={t('agente-mcpx-adicionar-cabecalho')}
            keyPlaceholder="Authorization"
          />
        </fieldset>
      )}
      <div className="flex flex-col gap-3">
        <Toggle label={t('agente-mcpx-ligado')} checked={enabled} onChange={setEnabled} />
        <Toggle
          label={t('agente-mcpx-permitir-sem-perguntar')}
          hint={t('agente-mcpx-permitir-dica')}
          checked={autoApprove}
          onChange={setAutoApprove}
        />
        <p className="text-[12px] text-dim">{t('agente-mcpx-explica')}</p>
      </div>
      {error ? (
        <p role="alert" className="text-[12.5px] text-negative">
          {error}
        </p>
      ) : null}
    </Dialog>
  );
}
