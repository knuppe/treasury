/**
 * Limites de gasto por provider de API (diário, mensal e o aviso antes de tarefas caras),
 * com o gasto de hoje e do mês ao lado. CLIs e modelos locais não têm custo no app.
 */
import { useState } from 'react';

import type { Provider } from '@/api/generated/Provider';
import type { ProviderSpend } from '@/api/generated/ProviderSpend';
import type { SpendLimit } from '@/api/generated/SpendLimit';
import { useEngines, useLimits, useProviders, useSetLimit, useUsageSummary } from '@/api/hooks/ai';
import { Button, Field, Input, InstrumentCard, Private, Select, toast } from '@/design/components';
import { ErrorBlock, LoadingBlock, errorMessage } from '@/features/portfolio/components/States';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';

import { costInputValue, formatCost, parseCost } from '../format';

const ADD = '__add__';

function LimitRow({
  provider,
  label,
  limit,
  spend,
}: {
  provider: Provider;
  label: string;
  limit: SpendLimit | undefined;
  spend: ProviderSpend | undefined;
}) {
  const set = useSetLimit();
  const [daily, setDaily] = useState(costInputValue(limit?.daily_micros));
  const [monthly, setMonthly] = useState(costInputValue(limit?.monthly_micros));
  const [confirm, setConfirm] = useState(costInputValue(limit?.confirm_above_micros));
  const parse = (text: string) => (text.trim() === '' ? null : parseCost(text));
  const values = { daily: parse(daily), monthly: parse(monthly), confirm: parse(confirm) };
  const invalid = (text: string, v: number | null) => text.trim() !== '' && v == null;
  const anyInvalid =
    invalid(daily, values.daily) ||
    invalid(monthly, values.monthly) ||
    invalid(confirm, values.confirm);
  const save = () =>
    set.mutate(
      {
        provider,
        daily_micros: values.daily,
        monthly_micros: values.monthly,
        confirm_above_micros: values.confirm,
      },
      {
        onSuccess: () =>
          toast({ tone: 'success', title: t('agente-limites-salvos', { provider: label }) }),
        onError: (e) =>
          toast({ tone: 'error', title: t('agente-nao-salvou'), description: errorMessage(e) }),
      },
    );
  return (
    <li
      aria-label={t('agente-limites-rotulo', { provider: label })}
      className="flex flex-col gap-2 px-4 py-3"
    >
      <div className="flex flex-wrap items-baseline gap-3">
        <h3 className="text-[13.5px] font-medium text-fg">{label}</h3>
        {spend ? (
          <span className="num text-[11.5px] text-dim">
            {tRich('agente-gasto', {
              hoje: <Private>{formatCost(spend.today_micros)}</Private>,
              mes: <Private>{formatCost(spend.month_micros)}</Private>,
            })}
          </span>
        ) : null}
      </div>
      <div className="flex flex-wrap items-end gap-3">
        <Field
          label={t('agente-por-dia')}
          className="w-[140px]"
          error={invalid(daily, values.daily) ? t('agente-valor-invalido') : undefined}
        >
          <Input
            numeric
            value={daily}
            placeholder={t('agente-sem-limite')}
            onChange={(e) => setDaily(e.target.value)}
          />
        </Field>
        <Field
          label={t('agente-por-mes')}
          className="w-[140px]"
          error={invalid(monthly, values.monthly) ? t('agente-valor-invalido') : undefined}
        >
          <Input
            numeric
            value={monthly}
            placeholder={t('agente-sem-limite')}
            onChange={(e) => setMonthly(e.target.value)}
          />
        </Field>
        <Field
          label={t('agente-confirmar-acima')}
          className="w-[190px]"
          error={invalid(confirm, values.confirm) ? t('agente-valor-invalido') : undefined}
        >
          <Input
            numeric
            value={confirm}
            placeholder={t('agente-nunca-perguntar')}
            onChange={(e) => setConfirm(e.target.value)}
          />
        </Field>
        <Button variant="secondary" loading={set.isPending} disabled={anyInvalid} onClick={save}>
          {t('comum-salvar')}
        </Button>
      </div>
    </li>
  );
}

export function LimitsPanel({ index = '03' }: { index?: string }) {
  const limits = useLimits();
  const providers = useProviders();
  const engines = useEngines();
  const usage = useUsageSummary({});
  const [extra, setExtra] = useState<Provider[]>([]);

  const apiProviders = (providers.data ?? []).filter((p) => p.kind === 'api');
  const shown = apiProviders.filter(
    (p) =>
      extra.includes(p.provider) ||
      (limits.data ?? []).some((l) => l.provider === p.provider) ||
      (engines.data ?? []).some((e) => e.provider === p.provider),
  );
  const addable = apiProviders.filter((p) => !shown.includes(p));

  return (
    <InstrumentCard
      index={index}
      title={t('agente-limites-titulo')}
      padded={false}
      footer={<p className="text-[12px] text-dim">{t('agente-limites-rodape')}</p>}
    >
      {limits.isLoading || providers.isLoading ? (
        <LoadingBlock />
      ) : limits.error ? (
        <ErrorBlock error={limits.error} className="m-3" />
      ) : (
        <>
          {shown.length === 0 ? (
            <p className="px-4 py-6 text-center text-[13px] text-dim">{t('agente-sem-provider')}</p>
          ) : (
            <ul aria-label={t('agente-limites-por-provider')} className="divide-y divide-line">
              {shown.map((p) => (
                <LimitRow
                  key={p.provider}
                  provider={p.provider}
                  label={p.label}
                  limit={(limits.data ?? []).find((l) => l.provider === p.provider)}
                  spend={usage.data?.spend.find((s) => s.provider === p.provider)}
                />
              ))}
            </ul>
          )}
          {addable.length > 0 ? (
            <div className="w-[280px] border-t border-line px-4 py-3">
              <Select
                aria-label={t('agente-limitar-outro')}
                value={ADD}
                onValueChange={(v) => {
                  if (v !== ADD) setExtra((x) => [...x, v as Provider]);
                }}
                options={[
                  { value: ADD, label: t('agente-limitar-outro-mais'), disabled: true },
                  ...addable.map((p) => ({ value: p.provider, label: p.label })),
                ]}
              />
            </div>
          ) : null}
        </>
      )}
    </InstrumentCard>
  );
}
