/** Motores de IA cadastrados: estado, chave (só "cadastrada" e a dica), teste e edição. */
import { useState } from 'react';

import type { Engine } from '@/api/generated/Engine';
import type { Provider } from '@/api/generated/Provider';
import type { EngineTest } from '@/api/generated/EngineTest';
import {
  useDeleteEngine,
  useEngines,
  useProviders,
  useTestEngine,
  useUpdateEngine,
} from '@/api/hooks/ai';
import { Button, Icon, InstrumentCard, toast } from '@/design/components';
import {
  ConfirmButton,
  ErrorBlock,
  LoadingBlock,
  errorMessage,
} from '@/features/portfolio/components/States';
import { t } from '@/i18n';

import { engineProblem } from '../engines';
import { ENGINE_KIND_DESTINATION, ENGINE_KIND_LABEL, formatDuration } from '../format';
import { Mono, Pill, Spec, Toggle } from '../ui';
import { EngineDialog } from './EngineDialog';

function TestResult({ result }: { result: EngineTest }) {
  return result.ok ? (
    <p role="status" className="text-[12px] text-positive">
      {t('agente-respondeu-em', { tempo: formatDuration(result.latency_ms) })}
      {result.reply ? (
        <span className="text-dim">
          {` · ${t('agente-aspas', { texto: result.reply.slice(0, 80) })}`}
        </span>
      ) : null}
    </p>
  ) : (
    <p role="status" className="text-[12px] text-negative">
      {`× ${result.error ?? t('agente-falhou')}`}
    </p>
  );
}

function EngineRow({ engine, onEdit }: { engine: Engine; onEdit: () => void }) {
  const update = useUpdateEngine();
  const del = useDeleteEngine();
  const test = useTestEngine();
  const [result, setResult] = useState<EngineTest | null>(null);
  const problem = engineProblem(engine);
  return (
    <li
      aria-label={t('agente-motor-rotulo', { nome: engine.name })}
      className="flex flex-col gap-2 px-4 py-3"
    >
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-[14px] font-medium text-fg">{engine.name}</span>
        <Pill
          tone={engine.kind === 'local' ? 'positive' : engine.kind === 'cli' ? 'accent' : 'warning'}
        >
          {ENGINE_KIND_LABEL[engine.kind]}
        </Pill>
        <span className="text-[12px] text-dim">{engine.provider_label}</span>
        {problem ? (
          <Pill tone="negative">{problem}</Pill>
        ) : (
          <Pill tone="positive">{t('agente-pronto')}</Pill>
        )}
        <span className="flex-1" />
        <Toggle
          label={t('controles-ligado')}
          checked={engine.enabled}
          disabled={update.isPending}
          onChange={(enabled) =>
            update.mutate(
              { id: engine.id, enabled },
              {
                onError: (e) =>
                  toast({
                    tone: 'error',
                    title: t('agente-nao-alterou'),
                    description: errorMessage(e),
                  }),
              },
            )
          }
        />
      </div>
      <div className="flex flex-wrap gap-x-5 gap-y-1">
        {engine.model ? <Spec label={t('agente-modelo')}>{engine.model}</Spec> : null}
        {engine.kind !== 'cli' && engine.effective_endpoint ? (
          <Spec label={engine.kind === 'local' ? t('agente-url-local') : t('agente-endpoint')}>
            {engine.effective_endpoint}
          </Spec>
        ) : null}
        {engine.needs_key || engine.secret.set ? (
          <Spec label={t('agente-api-key')}>
            {engine.secret.set
              ? engine.secret.hint
                ? t('agente-chave-cadastrada-dica', { dica: engine.secret.hint })
                : t('agente-chave-cadastrada')
              : t('agente-chave-nao-cadastrada')}
          </Spec>
        ) : null}
        <Spec label={t('agente-dados')}>{ENGINE_KIND_DESTINATION[engine.kind]}</Spec>
      </div>
      {engine.kind === 'cli' && engine.effective_template ? (
        <Mono className="text-dim">{engine.effective_template}</Mono>
      ) : null}
      {engine.kind === 'cli' && engine.provider !== 'claude_code' ? (
        <p className="text-[12px] text-dim">{t('agente-cli-historico-fora')}</p>
      ) : null}
      <div className="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant="secondary"
          loading={test.isPending}
          icon={<Icon name="reticle" size={12} />}
          onClick={() =>
            test.mutate(engine.id, {
              onSuccess: setResult,
              onError: (e) =>
                setResult({
                  engine_id: engine.id,
                  ok: false,
                  latency_ms: 0,
                  reply: null,
                  error: errorMessage(e),
                  input_tokens: 0,
                  output_tokens: 0,
                }),
            })
          }
        >
          {t('agente-testar')}
        </Button>
        <Button size="sm" variant="ghost" onClick={onEdit}>
          {t('comum-editar')}
        </Button>
        <ConfirmButton
          label={t('agente-excluir')}
          title={t('agente-excluir-motor', { nome: engine.name })}
          description={t('agente-excluir-motor-texto')}
          confirmLabel={t('agente-excluir')}
          pending={del.isPending}
          onConfirm={() =>
            del.mutateAsync(engine.id).catch((e: unknown) =>
              toast({
                tone: 'error',
                title: t('agente-nao-excluiu'),
                description: errorMessage(e),
              }),
            )
          }
        />
        {result ? <TestResult result={result} /> : null}
      </div>
    </li>
  );
}

export function EnginesPanel({ index = '01' }: { index?: string }) {
  const engines = useEngines();
  const providers = useProviders();
  const [dialog, setDialog] = useState<{ engine?: Engine; provider?: Provider } | null>(null);
  const hasGroq = (engines.data ?? []).some((e) => e.provider === 'groq');
  return (
    <InstrumentCard
      index={index}
      title={t('agente-motores-titulo')}
      padded={false}
      actions={
        <span className="flex items-center gap-2">
          {!hasGroq && (engines.data ?? []).length > 0 ? (
            <Button
              size="sm"
              variant="ghost"
              disabled={!providers.data}
              onClick={() => setDialog({ provider: 'groq' })}
            >
              {t('groq-usar-gratuito')}
            </Button>
          ) : null}
          <Button
            size="sm"
            variant="secondary"
            icon={<Icon name="plus" size={12} />}
            disabled={!providers.data}
            onClick={() => setDialog({})}
          >
            {t('agente-adicionar-motor')}
          </Button>
        </span>
      }
      footer={<p className="text-[12px] text-dim">{t('agente-motores-rodape')}</p>}
    >
      {engines.isLoading ? (
        <LoadingBlock />
      ) : engines.error ? (
        <ErrorBlock error={engines.error} className="m-3" onRetry={() => void engines.refetch()} />
      ) : (engines.data ?? []).length === 0 ? (
        <div className="flex flex-col items-center gap-3 px-4 py-6 text-center">
          <p className="text-[13px] text-dim">{t('agente-motores-vazio')}</p>
          <Button
            size="sm"
            variant="primary"
            disabled={!providers.data}
            onClick={() => setDialog({ provider: 'groq' })}
          >
            {t('groq-usar-gratuito')}
          </Button>
        </div>
      ) : (
        <ul aria-label={t('agente-motores')} className="divide-y divide-line">
          {(engines.data ?? []).map((e) => (
            <EngineRow key={e.id} engine={e} onEdit={() => setDialog({ engine: e })} />
          ))}
        </ul>
      )}
      {dialog && providers.data ? (
        <EngineDialog
          key={dialog.engine?.id ?? 'new'}
          open
          onOpenChange={(o) => {
            if (!o) setDialog(null);
          }}
          providers={providers.data}
          engine={dialog.engine}
          {...(dialog.provider ? { initialProvider: dialog.provider } : {})}
        />
      ) : null}
    </InstrumentCard>
  );
}
