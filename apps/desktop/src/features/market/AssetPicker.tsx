/**
 * Escolha de ativo pela busca de tickers (operação e watchlist): um ativo já cadastrado é
 * escolhido direto; um novo vira um cadastro com os dados da sugestão (ticker, bolsa,
 * classe, moeda, nome), depois de confirmar — e o cadastro pode ser desfeito no histórico.
 */
import { useState } from 'react';

import type { Asset } from '@/api/generated/Asset';
import type { AssetInput } from '@/api/generated/AssetInput';
import type { TickerSuggestion } from '@/api/generated/TickerSuggestion';
import { useCreateAsset } from '@/api/hooks/assets';
import { Button, toast } from '@/design/components';
import { FormError } from '@/vault/FormError';
import { t } from '@/i18n';

import { CLASS_LABEL } from '../portfolio/labels';
import { TickerSearch } from './TickerSearch';

/** Cadastro a partir de uma sugestão; `null` se faltar a classe (bolsa fora do catálogo). */
export function inputFromSuggestion(s: TickerSuggestion): AssetInput | null {
  if (!s.class) return null;
  const input: AssetInput = { ticker: s.ticker, name: s.name, class: s.class };
  if (s.exchange) input.market = s.exchange;
  if (s.currency) input.currency = s.currency;
  if (s.country) input.country = s.country;
  if (s.sector) input.sector = s.sector;
  return input;
}

export function describeSuggestion(s: TickerSuggestion): string {
  return [
    s.symbol ?? s.ticker,
    s.class ? CLASS_LABEL[s.class] : s.type_label,
    s.exchange_name ?? s.exchange,
    s.currency,
  ]
    .filter(Boolean)
    .join(' · ');
}

export function AssetPicker({ onSelect }: { onSelect: (asset: { id: string }) => void }) {
  const [pending, setPending] = useState<TickerSuggestion | null>(null);
  const create = useCreateAsset();
  const pick = (s: TickerSuggestion) => {
    create.reset();
    if (s.asset_id) {
      setPending(null);
      onSelect({ id: s.asset_id });
    } else setPending(s);
  };
  const input = pending ? inputFromSuggestion(pending) : null;
  return (
    <div className="flex flex-col gap-2">
      <TickerSearch onPick={pick} />
      {pending ? (
        <div
          role="group"
          aria-label={t('mercado-ativo-novo')}
          className="flex flex-wrap items-center gap-2 rounded-control border border-line px-2.5 py-2 text-[12.5px]"
        >
          <span className="min-w-0 flex-1">
            <span className="font-medium text-fg">{pending.name}</span>{' '}
            <span className="text-dim">({describeSuggestion(pending)})</span>
            {!input ? (
              <span className="block text-[12px] text-warning">
                {t('mercado-bolsa-fora-catalogo', { bolsa: pending.exchange_name ?? '' })}
              </span>
            ) : null}
          </span>
          <Button size="sm" variant="ghost" onClick={() => setPending(null)}>
            {t('comum-cancelar')}
          </Button>
          {input ? (
            <Button
              size="sm"
              variant="primary"
              loading={create.isPending}
              onClick={() =>
                create.mutate(input, {
                  onSuccess: (a: Asset) => {
                    toast({
                      tone: 'success',
                      title: t('ativos-form-cadastrado'),
                      description: a.ticker ?? a.name,
                    });
                    setPending(null);
                    onSelect(a);
                  },
                })
              }
            >
              {t('mercado-cadastrar-usar')}
            </Button>
          ) : null}
          <FormError error={create.error} />
        </div>
      ) : null}
    </div>
  );
}
