/**
 * Tabela de preços por modelo (USD por milhão de tokens), editável porque os preços
 * mudam. Um modelo vale como prefixo (`claude-sonnet` cobre `claude-sonnet-4-5`) e `*` é
 * o padrão do provider.
 */
import { useState } from 'react';

import type { ModelPrice } from '@/api/generated/ModelPrice';
import type { Provider } from '@/api/generated/Provider';
import { useDeletePrice, usePrices, useProviders, useSetPrice } from '@/api/hooks/ai';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Icon,
  Input,
  InstrumentCard,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { ErrorBlock, LoadingBlock, errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';

import { costInputValue, formatCost, parseCost } from '../format';
import { Pill } from '../ui';

function PriceDialog({ price, onClose }: { price: ModelPrice | null; onClose: () => void }) {
  const providers = useProviders();
  const set = useSetPrice();
  const apiProviders = (providers.data ?? []).filter((p) => p.kind === 'api');
  const [provider, setProvider] = useState<Provider>(price?.provider ?? 'anthropic');
  const [model, setModel] = useState(price?.model ?? '');
  const [input, setInput] = useState(costInputValue(price?.input_micros_per_mtok));
  const [output, setOutput] = useState(costInputValue(price?.output_micros_per_mtok));
  const [cached, setCached] = useState(costInputValue(price?.cached_micros_per_mtok));
  const vIn = parseCost(input);
  const vOut = parseCost(output);
  const vCached = cached.trim() === '' ? null : parseCost(cached);
  const cachedInvalid = cached.trim() !== '' && vCached == null;
  const valid = model.trim() !== '' && vIn != null && vOut != null && !cachedInvalid;
  return (
    <Dialog
      open
      onOpenChange={(o) => {
        if (!o) onClose();
      }}
      title={price ? t('agente-preco-de', { modelo: price.model }) : t('agente-novo-preco')}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="primary"
            disabled={!valid}
            loading={set.isPending}
            onClick={() =>
              set.mutate(
                {
                  provider,
                  model: model.trim(),
                  input_micros_per_mtok: vIn!,
                  output_micros_per_mtok: vOut!,
                  ...(vCached != null ? { cached_micros_per_mtok: vCached } : {}),
                },
                {
                  onSuccess: () => {
                    toast({ tone: 'success', title: t('agente-preco-salvo') });
                    onClose();
                  },
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('agente-nao-salvou'),
                      description: errorMessage(e),
                    }),
                },
              )
            }
          >
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Field label={t('agente-provider')}>
          <Select
            value={provider}
            disabled={Boolean(price)}
            onValueChange={(v) => setProvider(v as Provider)}
            options={apiProviders.map((p) => ({ value: p.provider, label: p.label }))}
          />
        </Field>
        <Field label={t('agente-modelo-prefixo')} hint={t('agente-modelo-prefixo-dica')}>
          <Input
            value={model}
            disabled={Boolean(price)}
            onChange={(e) => setModel(e.target.value)}
          />
        </Field>
        <Field
          label={t('agente-entrada-mtok')}
          error={input.trim() && vIn == null ? t('agente-valor-invalido') : undefined}
        >
          <Input numeric value={input} onChange={(e) => setInput(e.target.value)} />
        </Field>
        <Field
          label={t('agente-saida-mtok')}
          error={output.trim() && vOut == null ? t('agente-valor-invalido') : undefined}
        >
          <Input numeric value={output} onChange={(e) => setOutput(e.target.value)} />
        </Field>
        <Field
          label={t('agente-cache-mtok')}
          hint={t('agente-opcional')}
          error={cachedInvalid ? t('agente-valor-invalido') : undefined}
        >
          <Input numeric value={cached} onChange={(e) => setCached(e.target.value)} />
        </Field>
      </div>
    </Dialog>
  );
}

export function PricesPanel({ index = '05' }: { index?: string }) {
  const prices = usePrices();
  const providers = useProviders();
  const del = useDeletePrice();
  const [editing, setEditing] = useState<ModelPrice | 'new' | null>(null);
  const label = (p: Provider) => providers.data?.find((x) => x.provider === p)?.label ?? p;

  const columns: TechColumn<ModelPrice>[] = [
    {
      key: 'provider',
      header: t('agente-provider'),
      cell: (p) => label(p.provider),
      sortValue: (p) => p.provider,
    },
    {
      key: 'model',
      header: t('agente-modelo'),
      cell: (p) => <span className="font-mono text-[12px]">{p.model}</span>,
      sortValue: (p) => p.model,
    },
    {
      key: 'in',
      header: t('agente-entrada'),
      numeric: true,
      cell: (p) => formatCost(p.input_micros_per_mtok),
    },
    {
      key: 'out',
      header: t('agente-saida'),
      numeric: true,
      cell: (p) => formatCost(p.output_micros_per_mtok),
    },
    {
      key: 'cached',
      header: t('agente-cache'),
      numeric: true,
      cell: (p) => (p.cached_micros_per_mtok == null ? '—' : formatCost(p.cached_micros_per_mtok)),
    },
    {
      key: 'source',
      header: t('agente-origem'),
      cell: (p) => (
        <Pill tone={p.source === 'user' ? 'accent' : 'dim'}>
          {p.source === 'user' ? t('agente-editado') : t('agente-padrao')}
        </Pill>
      ),
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('agente-acoes')}</span>,
      align: 'right',
      cell: (p) => (
        <span className="flex justify-end gap-1">
          <Button size="sm" variant="ghost" onClick={() => setEditing(p)}>
            {t('comum-editar')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            aria-label={t('agente-apagar-preco', { modelo: p.model })}
            onClick={() =>
              del.mutate(
                { provider: p.provider, model: p.model },
                {
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('agente-nao-apagou'),
                      description: errorMessage(e),
                    }),
                },
              )
            }
          >
            {t('comum-apagar')}
          </Button>
        </span>
      ),
    },
  ];

  return (
    <InstrumentCard
      index={index}
      title={t('agente-precos-titulo')}
      padded={false}
      actions={
        <Button
          size="sm"
          variant="secondary"
          icon={<Icon name="plus" size={12} />}
          onClick={() => setEditing('new')}
        >
          {t('agente-novo-preco')}
        </Button>
      }
    >
      {prices.isLoading ? (
        <LoadingBlock />
      ) : prices.error ? (
        <ErrorBlock error={prices.error} className="m-3" />
      ) : (
        <TechTable
          label={t('agente-precos')}
          prefsId="Preços por modelo"
          columns={columns}
          rows={prices.data ?? []}
          getRowId={(p) => `${p.provider}:${p.model}`}
          dense
          empty={t('agente-precos-vazio')}
        />
      )}
      {editing ? (
        <PriceDialog price={editing === 'new' ? null : editing} onClose={() => setEditing(null)} />
      ) : null}
    </InstrumentCard>
  );
}
