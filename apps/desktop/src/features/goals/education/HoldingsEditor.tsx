/**
 * Posições ligadas a uma meta (docs/design/college-goal.md §7): a conta inteira ou um ativo
 * dela, com uma fração ou num pote comum entre as metas. Vale para qualquer meta.
 */
import { useState } from 'react';

import type { HoldingInput } from '@/api/generated/HoldingInput';
import { isApiError } from '@/api/client';
import { useGoalHoldings, useSetGoalHoldings } from '@/api/hooks/goals';
import { usePortfolioSummary } from '@/api/hooks/portfolio';
import { useAccounts } from '@/api/hooks/reports';
import { Button, Field, Icon, Input, Money, Select, toast } from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatPercent } from '@/lib/format';

import { parseNumber } from './model';

const WHOLE = '__conta__';
const errorText = (e: unknown) =>
  isApiError(e) || e instanceof Error ? e.message : t('metas-erro-gravar');

export function HoldingsEditor({
  goalId,
  targetDate,
}: {
  goalId: string;
  targetDate?: string | null;
}) {
  const holdings = useGoalHoldings(goalId);
  const accounts = useAccounts();
  const positions = usePortfolioSummary({ by_account: true });
  const setHoldings = useSetGoalHoldings();
  const [account, setAccount] = useState<string>('');
  const [asset, setAsset] = useState<string>(WHOLE);
  const [share, setShare] = useState('');
  const current: HoldingInput[] = (holdings.data ?? []).map((h) => ({
    account_id: h.account_id,
    asset_id: h.asset_id ?? undefined,
    share: h.share ?? undefined,
    value_mode: h.value_mode,
  }));
  const write = (next: HoldingInput[], done?: () => void) =>
    setHoldings.mutate(
      { goal_id: goalId, holdings: next },
      {
        onSuccess: () => {
          done?.();
          toast({ tone: 'success', title: t('educacao-posicoes-salvas') });
        },
        onError: (e) =>
          toast({ tone: 'error', title: t('educacao-posicoes-erro'), description: errorText(e) }),
      },
    );
  const accountOptions = (accounts.data ?? [])
    .filter((a) => !a.archived && a.kind !== 'credit_card')
    .map((a) => ({ value: a.id, label: a.name }));
  const assetOptions = [
    { value: WHOLE, label: t('educacao-posicao-conta-inteira') },
    ...(positions.data?.positions ?? [])
      .filter((p) => p.account_id === account)
      .map((p) => ({ value: p.asset.id, label: p.asset.ticker ?? p.asset.name })),
  ];
  const pct = parseNumber(share);
  const add = () => {
    if (!account) return;
    write(
      [
        ...current,
        {
          account_id: account,
          asset_id: asset === WHOLE ? undefined : asset,
          share: pct == null ? undefined : pct / 100,
        },
      ],
      () => {
        setAccount('');
        setAsset(WHOLE);
        setShare('');
      },
    );
  };
  return (
    <section aria-label={t('educacao-posicoes')} className="flex flex-col gap-3">
      <p className="text-[12px] text-dim">{t('educacao-posicoes-dica')}</p>
      {(holdings.data ?? []).length ? (
        <ul className="flex flex-col divide-y divide-line rounded-control border border-line">
          {(holdings.data ?? []).map((h, i) => (
            <li
              key={h.id}
              className="flex flex-wrap items-center gap-x-3 gap-y-1 px-3 py-2 text-[12.5px]"
            >
              <span className="min-w-0 flex-1 truncate text-fg">
                {h.account_name}
                {h.asset_name
                  ? ` · ${h.ticker ?? h.asset_name}`
                  : ` · ${t('educacao-posicao-conta-inteira')}`}
              </span>
              <span className="label-caps">
                {h.share != null
                  ? formatPercent(h.share, { decimals: 0 })
                  : t('educacao-pote', { pct: formatPercent(h.pool_share ?? 0, { decimals: 0 }) })}
              </span>
              <span className="num text-fg">
                <Money cents={h.value} currency={h.currency} />
              </span>
              {h.maturity_date ? (
                <span className="w-full text-[11.5px] text-dim">
                  {targetDate
                    ? t('educacao-vencimento-meta', {
                        vence: formatDate(h.maturity_date),
                        meta: formatDate(targetDate),
                      })
                    : t('educacao-vencimento', { vence: formatDate(h.maturity_date) })}
                </span>
              ) : null}
              {h.missing_price ? (
                <span className="w-full text-[11.5px] text-warning">
                  {t('educacao-sem-cotacao')}
                </span>
              ) : null}
              <Button
                variant="ghost"
                size="sm"
                aria-label={t('educacao-posicao-remover', { nome: h.asset_name ?? h.account_name })}
                onClick={() => write(current.filter((_, j) => j !== i))}
              >
                <Icon name="x" size={12} />
              </Button>
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-[12.5px] text-dim">{t('educacao-posicoes-vazio')}</p>
      )}
      <div className="grid items-end gap-3 sm:grid-cols-[1fr_1fr_120px_auto]">
        <Field label={t('educacao-posicao-conta')}>
          <Select
            value={account}
            onValueChange={(v) => {
              setAccount(v);
              setAsset(WHOLE);
            }}
            options={accountOptions}
            placeholder={t('educacao-posicao-escolha-conta')}
          />
        </Field>
        <Field label={t('educacao-posicao-ativo')}>
          <Select value={asset} onValueChange={setAsset} options={assetOptions} />
        </Field>
        <Field label={t('educacao-posicao-fracao')} hint={t('educacao-posicao-fracao-dica')}>
          <Input
            numeric
            inputMode="decimal"
            value={share}
            suffix="%"
            onChange={(e) => setShare(e.target.value)}
          />
        </Field>
        <Button
          variant="secondary"
          size="sm"
          disabled={!account || (pct != null && !(pct >= 1 && pct <= 100))}
          loading={setHoldings.isPending}
          onClick={add}
        >
          {t('educacao-posicao-ligar')}
        </Button>
      </div>
    </section>
  );
}
