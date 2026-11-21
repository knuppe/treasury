/**
 * Cadastro e edição de um motor de IA, com campos por tipo de provider: API (modelo,
 * endpoint opcional, API key só de escrita), CLI local (detecção no PATH e template de
 * comando) e modelo local (URL do servidor neste computador e modelo).
 */
import { useState } from 'react';

import type { Engine } from '@/api/generated/Engine';
import type { EnginePatch } from '@/api/generated/EnginePatch';
import type { Provider } from '@/api/generated/Provider';
import type { ProviderInfo } from '@/api/generated/ProviderInfo';
import {
  useCreateEngine,
  useDeleteSecret,
  useDetectClis,
  useSetSecret,
  useUpdateEngine,
} from '@/api/hooks/ai';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Input,
  PasswordInput,
  Select,
  toast,
} from '@/design/components';
import { LoadingBlock, errorMessage } from '@/features/portfolio/components/States';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';

import { ENGINE_KIND_LABEL } from '../format';
import { GroqGuide } from './GroqGuide';
import { Mono, Pill, Toggle } from '../ui';

function CliDetections({ provider }: { provider: Provider }) {
  const clis = useDetectClis(true);
  if (clis.isLoading)
    return <LoadingBlock label={t('agente-procurando-clis')} className="min-h-0 py-3" />;
  if (clis.error) return <p className="text-[12px] text-negative">{errorMessage(clis.error)}</p>;
  return (
    <div className="rounded-control border border-line">
      <p className="label-caps border-b border-line px-3 py-1.5 text-[10px]">
        {t('agente-clis-encontradas')}
      </p>
      <ul aria-label={t('agente-clis-detectadas')} className="divide-y divide-line">
        {(clis.data ?? []).map((c) => (
          <li
            key={c.provider}
            className={`flex flex-wrap items-center gap-2 px-3 py-1.5 text-[12.5px] ${c.provider === provider ? 'bg-accent-soft' : ''}`}
          >
            <span className="min-w-[140px] text-fg">{c.label}</span>
            {c.path ? (
              <>
                <Pill tone="positive">{t('agente-cli-instalada')}</Pill>
                {c.version ? (
                  <span className="num text-[11px] text-dim">{`v${c.version}`}</span>
                ) : null}
                <span className="font-mono text-[11px] text-dim">{c.path}</span>
                <span className="text-[11px] text-dim">
                  {c.logged_in === true
                    ? t('agente-cli-login-ok')
                    : c.logged_in === false
                      ? t('agente-cli-sem-login')
                      : t('agente-cli-login-incerto')}
                </span>
              </>
            ) : (
              <Pill tone="dim">{t('agente-cli-nao-encontrada')}</Pill>
            )}
            {c.error ? <span className="text-[11px] text-negative">{c.error}</span> : null}
          </li>
        ))}
      </ul>
    </div>
  );
}

export interface EngineDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  providers: readonly ProviderInfo[];
  /** Ausente: cadastro novo. */
  engine?: Engine;
  /** Provider já escolhido no cadastro novo (ex.: o atalho "Usar o Groq"). */
  initialProvider?: Provider;
}

const optText = (v: string) => (v.trim() === '' ? null : v.trim());

export function EngineDialog({
  open,
  onOpenChange,
  providers,
  engine,
  initialProvider,
}: EngineDialogProps) {
  const editing = Boolean(engine);
  const [provider, setProvider] = useState<Provider>(
    engine?.provider ?? initialProvider ?? 'anthropic',
  );
  const info = providers.find((p) => p.provider === provider);
  const kind = info?.kind ?? 'api';
  const [name, setName] = useState(engine?.name ?? '');
  const [model, setModel] = useState(engine?.model ?? info?.default_model ?? '');
  const [endpoint, setEndpoint] = useState(engine?.endpoint ?? '');
  const [template, setTemplate] = useState(engine?.command_template ?? '');
  const [maxTokens, setMaxTokens] = useState(engine?.max_output_tokens?.toString() ?? '');
  const [enabled, setEnabled] = useState(engine?.enabled ?? true);
  const [apiKey, setApiKey] = useState('');
  const [error, setError] = useState<string | null>(null);

  const create = useCreateEngine();
  const update = useUpdateEngine();
  const setSecret = useSetSecret();
  const deleteSecret = useDeleteSecret();
  const busy = create.isPending || update.isPending || setSecret.isPending;

  const changeProvider = (p: Provider) => {
    setProvider(p);
    const next = providers.find((x) => x.provider === p);
    setModel(next?.default_model ?? '');
    setEndpoint('');
    setTemplate('');
  };

  const tokens = maxTokens.trim() === '' ? null : Number(maxTokens);
  const tokensInvalid = tokens != null && (!Number.isInteger(tokens) || tokens <= 0);

  const save = async () => {
    setError(null);
    if (tokensInvalid) return;
    try {
      if (!engine) {
        await create.mutateAsync({
          provider,
          ...(name.trim() ? { name: name.trim() } : {}),
          ...(optText(model) ? { model: model.trim() } : {}),
          ...(optText(endpoint) ? { endpoint: endpoint.trim() } : {}),
          ...(optText(template) ? { command_template: template.trim() } : {}),
          ...(tokens != null ? { max_output_tokens: tokens } : {}),
          ...(apiKey.trim() ? { api_key: apiKey.trim() } : {}),
          enabled,
        });
        toast({ tone: 'success', title: t('agente-motor-cadastrado') });
      } else {
        const patch: EnginePatch = { id: engine.id };
        if (name.trim() && name.trim() !== engine.name) patch.name = name.trim();
        if (optText(model) !== engine.model) patch.model = optText(model);
        if (optText(endpoint) !== engine.endpoint) patch.endpoint = optText(endpoint);
        if (optText(template) !== engine.command_template)
          patch.command_template = optText(template);
        if (tokens !== engine.max_output_tokens) patch.max_output_tokens = tokens;
        if (enabled !== engine.enabled) patch.enabled = enabled;
        if (Object.keys(patch).length > 1) await update.mutateAsync(patch);
        if (apiKey.trim())
          await setSecret.mutateAsync({ engine_id: engine.id, value: apiKey.trim() });
        toast({ tone: 'success', title: t('agente-motor-atualizado') });
      }
      setApiKey('');
      onOpenChange(false);
    } catch (e) {
      setError(errorMessage(e));
    }
  };

  const keyStatus = engine?.secret;

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={
        editing ? t('agente-editar-motor', { nome: engine?.name ?? '' }) : t('agente-novo-motor')
      }
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button variant="primary" loading={busy} onClick={() => void save()}>
            {editing ? t('comum-salvar') : t('agente-cadastrar')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Field
          label={t('agente-provider')}
          hint={
            info
              ? info.needs_key
                ? t('agente-exige-chave', { tipo: ENGINE_KIND_LABEL[info.kind] })
                : ENGINE_KIND_LABEL[info.kind]
              : undefined
          }
        >
          <Select
            value={provider}
            onValueChange={(v) => changeProvider(v as Provider)}
            disabled={editing}
            options={providers.map((p) => ({
              value: p.provider,
              label: p.label,
              hint: ENGINE_KIND_LABEL[p.kind],
            }))}
          />
        </Field>
        <Field label={t('agente-nome')} hint={t('agente-nome-dica')}>
          <Input value={name} placeholder={info?.label} onChange={(e) => setName(e.target.value)} />
        </Field>

        {kind !== 'cli' ? (
          <Field label={t('agente-modelo')} hint={t('agente-modelo-dica')}>
            <Input
              value={model}
              placeholder={info?.default_model ?? t('agente-modelo-placeholder')}
              onChange={(e) => setModel(e.target.value)}
            />
          </Field>
        ) : (
          <Field label={t('agente-modelo-opcional')} hint={t('agente-modelo-opcional-dica')}>
            <Input value={model} onChange={(e) => setModel(e.target.value)} />
          </Field>
        )}

        {kind === 'local' ? (
          <Field label={t('agente-url-servidor')} hint={t('agente-url-servidor-dica')}>
            <Input
              value={endpoint}
              placeholder={info?.default_endpoint ?? 'http://127.0.0.1:11434/v1'}
              onChange={(e) => setEndpoint(e.target.value)}
            />
          </Field>
        ) : kind === 'api' ? (
          <Field
            label={
              provider === 'openai_compatible'
                ? t('agente-endpoint')
                : t('agente-endpoint-opcional')
            }
            hint={
              info?.default_endpoint
                ? t('agente-endpoint-padrao', { url: info.default_endpoint })
                : t('agente-endpoint-dica')
            }
          >
            <Input
              value={endpoint}
              placeholder={info?.default_endpoint ?? 'https://'}
              onChange={(e) => setEndpoint(e.target.value)}
            />
          </Field>
        ) : null}

        <Field
          label={t('agente-max-tokens')}
          hint={t('agente-max-tokens-dica')}
          error={tokensInvalid ? t('agente-max-tokens-erro') : undefined}
        >
          <Input
            numeric
            inputMode="numeric"
            value={maxTokens}
            onChange={(e) => setMaxTokens(e.target.value)}
          />
        </Field>
      </div>

      {provider === 'groq' ? <GroqGuide /> : null}
      {kind === 'api' ? (
        <Field
          label={t('agente-api-key')}
          hint={t('agente-chave-dica')}
          aside={
            keyStatus?.set ? (
              <span className="flex items-center gap-2">
                <Pill tone={keyStatus.needs_reentry ? 'negative' : 'positive'}>
                  {keyStatus.needs_reentry
                    ? t('agente-cadastre-de-novo')
                    : t('agente-chave-cadastrada')}
                </Pill>
                {keyStatus.hint ? (
                  <span className="font-mono text-[11px] text-dim">{keyStatus.hint}</span>
                ) : null}
              </span>
            ) : editing ? (
              <Pill tone="warning">{t('agente-sem-chave')}</Pill>
            ) : null
          }
        >
          <PasswordInput
            value={apiKey}
            autoComplete="off"
            placeholder={keyStatus?.set ? t('agente-substituir-chave') : t('agente-cole-chave')}
            onChange={(e) => setApiKey(e.target.value)}
          />
        </Field>
      ) : null}
      {kind === 'api' && engine && keyStatus?.set ? (
        <div>
          <Button
            size="sm"
            variant="ghost"
            loading={deleteSecret.isPending}
            onClick={() =>
              deleteSecret.mutate(engine.id, {
                onSuccess: () => toast({ tone: 'success', title: t('agente-chave-apagada') }),
                onError: (e) => setError(errorMessage(e)),
              })
            }
          >
            {t('agente-apagar-chave')}
          </Button>
        </div>
      ) : null}

      {kind === 'cli' ? (
        <>
          <CliDetections provider={provider} />
          <Field label={t('agente-template')} hint={t('agente-template-dica')}>
            <textarea
              rows={3}
              value={template}
              placeholder={info?.default_template ?? t('agente-template-placeholder')}
              onChange={(e) => setTemplate(e.target.value)}
              className="w-full rounded-control border border-line-strong bg-bg px-2.5 py-2 font-mono text-[12px] text-fg outline-none focus:border-accent"
            />
          </Field>
          {info?.default_template ? (
            <p className="text-[12px] text-dim">
              {tRich('agente-template-padrao', {
                template: <Mono>{info.default_template}</Mono>,
              })}
            </p>
          ) : null}
        </>
      ) : null}

      <Toggle label={t('agente-motor-ligado')} checked={enabled} onChange={setEnabled} />
      {error ? (
        <p role="alert" className="text-[12.5px] text-negative">
          {error}
        </p>
      ) : null}
    </Dialog>
  );
}
