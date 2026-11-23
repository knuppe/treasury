/**
 * Assinaturas e cobranças recorrentes (docs/subscriptions.md): o que se repete todo mês ou
 * todo ano nos lançamentos, com aumento de preço, cobrança nova e cobrança atrasada; cria a
 * recorrência sugerida ou dispensa a detecção ("não é assinatura").
 */
import { useMemo } from 'react';

import type { Subscription } from '@/api/generated/Subscription';
import {
  useDismissSubscription,
  useRestoreSubscription,
  useSubscriptions,
} from '@/api/hooks/planning';
import { useCreateRecurrence } from '@/api/hooks/recurrences';
import {
  Button,
  EmptyState,
  Icon,
  InstrumentCard,
  Money,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { tRich } from '@/features/settings/richText';
import { QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatLocale } from '@/lib/format';
import { daysBetween } from '@/lib/months';

const STATUS: Record<Subscription['status'], string> = {
  get active() {
    return t('planejamento-assinaturas-em-dia');
  },
  get new() {
    return t('planejamento-assinaturas-nova');
  },
  get late() {
    return t('planejamento-assinaturas-nao-veio');
  },
};

function pct(v: number) {
  const locale = formatLocale() === 'xx' ? 'pt-BR' : formatLocale();
  const n = (v * 100).toLocaleString(locale, { maximumFractionDigits: 1 });
  return `${v > 0 ? '+' : ''}${n}%`;
}

function Actions({ s }: { s: Subscription }) {
  const create = useCreateRecurrence();
  const dismiss = useDismissSubscription();
  return (
    <span className="flex justify-end gap-1">
      {s.suggested_recurrence ? (
        <Button
          size="sm"
          variant="ghost"
          loading={create.isPending}
          onClick={() =>
            create.mutate(s.suggested_recurrence!, {
              onSuccess: () =>
                toast({
                  tone: 'success',
                  title: t('planejamento-assinaturas-recorrencia-criada'),
                  description: t('planejamento-assinaturas-recorrencia-criada-descricao', {
                    nome: s.name,
                  }),
                }),
            })
          }
        >
          {t('planejamento-assinaturas-criar-recorrencia')}
        </Button>
      ) : (
        <span className="label-caps self-center px-2 text-[10px] text-dim">
          {t('planejamento-assinaturas-recorrencia-ok')}
        </span>
      )}
      <Button
        size="sm"
        variant="ghost"
        loading={dismiss.isPending}
        onClick={() => dismiss.mutate({ key: s.key })}
      >
        {t('planejamento-assinaturas-nao-e')}
      </Button>
    </span>
  );
}

function subscriptionColumns(): TechColumn<Subscription>[] {
  return [
    {
      key: 'name',
      header: t('planejamento-assinaturas-assinatura'),
      cell: (s) => (
        <span className="flex min-w-0 flex-col">
          <span className="truncate text-fg">{s.name}</span>
          <span className="truncate text-[11.5px] text-dim">
            {s.account_name}
            {s.category ? ` · ${s.category}` : ''}
            {` · ${
              s.kind === 'fixed_bill'
                ? t('planejamento-assinaturas-conta-fixa')
                : t('planejamento-assinaturas-tipo-assinatura')
            }`}
          </span>
        </span>
      ),
    },
    {
      key: 'cadence',
      header: t('planejamento-assinaturas-frequencia'),
      width: '92px',
      cell: (s) =>
        s.cadence === 'monthly'
          ? t('planejamento-assinaturas-mensal')
          : t('planejamento-assinaturas-anual'),
    },
    {
      key: 'amount',
      header: t('planejamento-valor'),
      numeric: true,
      // Ordena pelo custo por mês (anual e mensal na mesma régua).
      sortValue: (s) => s.monthly_cost,
      cell: (s) => (
        <span className="flex flex-col items-end">
          <Money cents={s.amount} currency={s.currency} />
          {s.cadence !== 'monthly' ? (
            <span className="text-[11px] text-dim">
              {tRich('planejamento-assinaturas-por-mes-valor', {
                valor: <Money cents={s.monthly_cost} currency={s.currency} />,
              })}
            </span>
          ) : null}
          {s.change_pct != null ? (
            <span
              className={cx(
                'num text-[11px]',
                s.change_pct > 0 ? 'text-negative' : 'text-positive',
              )}
            >
              {t('planejamento-assinaturas-desde-anterior', { pct: pct(s.change_pct) })}
            </span>
          ) : null}
        </span>
      ),
    },
    {
      key: 'next_expected',
      header: t('planejamento-assinaturas-proxima'),
      width: '100px',
      cell: (s) => formatDate(s.next_expected),
    },
    {
      key: 'status',
      header: t('planejamento-assinaturas-situacao'),
      width: '90px',
      cell: (s) => (
        <span
          className={cx(
            'label-caps text-[10.5px]',
            s.status === 'new' ? 'text-accent' : s.status === 'late' ? 'text-warning' : 'text-dim',
          )}
        >
          {STATUS[s.status]}
        </span>
      ),
    },
    { key: 'actions', header: '', cell: (s) => <Actions s={s} /> },
  ];
}

export function SubscriptionsPanel() {
  const q = useSubscriptions();
  const restore = useRestoreSubscription();
  const r = q.data;
  const columns = useMemo(() => subscriptionColumns(), []);
  // Avisos só do que mudou há pouco (a tabela mostra a variação de todas).
  const recent = (d: string) => r !== undefined && daysBetween(d, r.as_of) <= 60;
  const increases = r?.items.filter((s) => (s.change_pct ?? 0) > 0 && recent(s.last_date)) ?? [];
  const fresh = r?.items.filter((s) => s.status === 'new') ?? [];
  const late = r?.items.filter((s) => s.status === 'late') ?? [];
  return (
    <QueryState loading={q.isPending} error={q.error} onRetry={() => void q.refetch()}>
      {r ? (
        r.items.length === 0 && r.dismissed.length === 0 ? (
          <EmptyState
            kicker={t('planejamento-assinaturas-vazio')}
            title={t('planejamento-assinaturas-vazio-titulo')}
            description={t('planejamento-assinaturas-vazio-descricao')}
          />
        ) : (
          <div className="flex flex-col gap-4">
            <InstrumentCard
              index="05.A"
              title={t('planejamento-assinaturas-titulo')}
              padded={false}
              footer={[
                {
                  label: t('planejamento-por-mes'),
                  value: <Money cents={r.monthly_total} currency={r.currency} />,
                },
                {
                  label: t('planejamento-assinaturas-por-ano'),
                  value: <Money cents={r.annual_total} currency={r.currency} />,
                },
                {
                  label: t('planejamento-assinaturas-contas-fixas-mes'),
                  value: <Money cents={r.fixed_monthly_total} currency={r.currency} />,
                },
                { label: t('planejamento-assinaturas-detectadas'), value: String(r.items.length) },
              ]}
            >
              {increases.length || fresh.length || late.length ? (
                <ul className="flex flex-col gap-1 border-b border-line px-4 py-3 text-[13px]">
                  {increases.map((s) => (
                    <li key={`up:${s.key}`} className="flex items-center gap-2 text-negative">
                      <Icon name="arrowUp" size={13} />
                      <span>
                        {tRich('planejamento-assinaturas-subiu', {
                          nome: s.name,
                          pct: pct(s.change_pct!),
                          antes: <Money cents={s.previous_amount ?? 0} currency={s.currency} />,
                          agora: <Money cents={s.amount} currency={s.currency} />,
                          data: formatDate(s.last_date),
                        })}
                      </span>
                    </li>
                  ))}
                  {fresh.map((s) => (
                    <li key={`new:${s.key}`} className="flex items-center gap-2 text-accent">
                      <Icon name="info" size={13} />
                      <span>
                        {tRich('planejamento-assinaturas-cobranca-nova', {
                          nome: s.name,
                          valor: <Money cents={s.amount} currency={s.currency} />,
                          data: formatDate(s.first_date),
                        })}
                      </span>
                    </li>
                  ))}
                  {late.map((s) => (
                    <li key={`late:${s.key}`} className="flex items-center gap-2 text-warning">
                      <Icon name="alert" size={13} />
                      <span>
                        {t('planejamento-assinaturas-atrasada', {
                          nome: s.name,
                          data: formatDate(s.next_expected),
                        })}
                      </span>
                    </li>
                  ))}
                </ul>
              ) : null}
              <TechTable
                label={t('planejamento-assinaturas-tabela')}
                prefsId="Assinaturas detectadas"
                columns={columns}
                rows={r.items}
                getRowId={(s) => s.key}
                minWidth="880px"
                dense
              />
            </InstrumentCard>
            {r.dismissed.length ? (
              <details className="text-[12.5px] text-dim">
                <summary className="cursor-pointer">
                  {t('planejamento-assinaturas-dispensadas', { n: r.dismissed.length })}
                </summary>
                <ul className="mt-2 flex flex-col gap-1">
                  {r.dismissed.map((k) => (
                    <li key={k} className="flex items-center gap-2">
                      <span className="truncate">{k.split('|')[0]}</span>
                      <Button size="sm" variant="ghost" onClick={() => restore.mutate({ key: k })}>
                        {t('planejamento-assinaturas-trazer')}
                      </Button>
                    </li>
                  ))}
                </ul>
              </details>
            ) : null}
            <p className="text-[12px] text-dim">{t('planejamento-assinaturas-fonte')}</p>
          </div>
        )
      ) : null}
    </QueryState>
  );
}
