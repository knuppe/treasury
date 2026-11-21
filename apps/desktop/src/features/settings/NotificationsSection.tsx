/**
 * Ajustes › Notificações (docs/notifications.md): quais lembretes viram alerta e
 * notificação do sistema, com quantos dias de antecedência as contas avisam e se o texto
 * mostra valores (aparece na tela bloqueada). Os alertas ficam na caixa do Agente.
 */
import { useState } from 'react';

import { isTauri } from '@/api/client';
import type { NotifyPrefs } from '@/api/generated/NotifyPrefs';
import type { AccountKind } from '@/api/generated/AccountKind';
import type { StalenessSetting } from '@/api/generated/StalenessSetting';
import { useSetStalenessLimits, useStalenessLimits } from '@/api/hooks/accounts';
import { runRemindersNow, useNotifyPrefs, useSetNotifyPrefs } from '@/api/hooks/planning';
import { Button, Field, Input, InstrumentCard, toast } from '@/design/components';
import { InlineError, QueryState, Switch } from '@/features/transactions/ui/controls';
import { ACCOUNT_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';
import { formatMoney } from '@/lib/format';

type Flag = Exclude<keyof NotifyPrefs, 'bills_days' | 'system' | 'show_amounts'>;

/** Lembretes que viram alerta e notificação (rótulos no idioma atual). */
const KINDS: { key: Flag; readonly label: string; readonly description: string }[] = [
  {
    key: 'bills',
    get label() {
      return t('ajustes-notif-contas');
    },
    get description() {
      return t('ajustes-notif-contas-dica');
    },
  },
  {
    key: 'negative_balance',
    get label() {
      return t('ajustes-notif-saldo');
    },
    get description() {
      return t('ajustes-notif-saldo-dica');
    },
  },
  {
    key: 'budget',
    get label() {
      return t('ajustes-notif-orcamento');
    },
    get description() {
      return t('ajustes-notif-orcamento-dica');
    },
  },
  {
    key: 'subscriptions',
    get label() {
      return t('ajustes-notif-assinaturas');
    },
    get description() {
      return t('ajustes-notif-assinaturas-dica');
    },
  },
  {
    key: 'dividends',
    get label() {
      return t('ajustes-notif-proventos');
    },
    get description() {
      return t('ajustes-notif-proventos-dica');
    },
  },
  {
    key: 'price_targets',
    get label() {
      return t('ajustes-notif-watchlist');
    },
    get description() {
      return t('ajustes-notif-watchlist-dica');
    },
  },
  {
    key: 'stale_accounts',
    get label() {
      return t('ajustes-notif-contas-paradas');
    },
    get description() {
      return t('ajustes-notif-contas-paradas-dica');
    },
  },
  {
    key: 'scores',
    get label() {
      return t('ajustes-notif-scores');
    },
    get description() {
      return t('ajustes-notif-scores-dica');
    },
  },
  {
    key: 'agent',
    get label() {
      return t('ajustes-notif-agente');
    },
    get description() {
      return t('ajustes-notif-agente-dica');
    },
  },
];

async function testNotification(showAmounts: boolean) {
  if (!isTauri()) {
    toast({ tone: 'info', title: t('ajustes-notif-so-no-app') });
    return;
  }
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('system_notify', {
    title: t('ajustes-notif-teste-titulo'),
    body: showAmounts
      ? t('ajustes-notif-teste-valores', { valor: formatMoney(123456) })
      : t('ajustes-notif-teste-sem-valores'),
  });
}

function Form({ prefs }: { prefs: NotifyPrefs }) {
  const save = useSetNotifyPrefs();
  const [days, setDays] = useState(String(prefs.bills_days));
  const set = (patch: Partial<NotifyPrefs>) => save.mutate({ ...prefs, ...patch });
  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3">
        <Switch
          checked={prefs.system}
          onChange={(v) => set({ system: v })}
          label={t('ajustes-notif-sistema')}
          description={t('ajustes-notif-sistema-dica')}
        />
        <Switch
          checked={prefs.show_amounts}
          disabled={!prefs.system}
          onChange={(v) => set({ show_amounts: v })}
          label={t('ajustes-notif-valores')}
          description={t('ajustes-notif-valores-dica')}
        />
      </div>
      <div className="flex flex-col gap-3 border-t border-line pt-4">
        <h3 className="label-caps text-fg">{t('ajustes-notif-lembretes')}</h3>
        {KINDS.map((k) => (
          <Switch
            key={k.key}
            checked={prefs[k.key]}
            onChange={(v) => set({ [k.key]: v })}
            label={k.label}
            description={k.description}
          />
        ))}
        <Field label={t('ajustes-notif-antecedencia')} hint={t('ajustes-notif-antecedencia-dica')}>
          <Input
            numeric
            inputMode="numeric"
            className="w-24"
            value={days}
            disabled={!prefs.bills}
            onChange={(e) => setDays(e.target.value.replace(/\D/g, ''))}
            onBlur={() => {
              const n = Math.min(Number(days || 0), 15);
              setDays(String(n));
              if (n !== prefs.bills_days) set({ bills_days: n });
            }}
            suffix={t('ajustes-notif-dias')}
          />
        </Field>
      </div>
      <InlineError error={save.error} />
      <div className="flex flex-wrap gap-2 border-t border-line pt-4">
        <Button
          variant="secondary"
          disabled={!prefs.system}
          onClick={() =>
            void testNotification(prefs.show_amounts).catch((e: unknown) =>
              toast({
                tone: 'error',
                title: t('ajustes-notif-teste-falhou'),
                description:
                  e instanceof Error
                    ? e.message
                    : String((e as { message?: string })?.message ?? e),
              }),
            )
          }
        >
          {t('ajustes-notif-testar')}
        </Button>
        <Button
          variant="ghost"
          onClick={() =>
            void runRemindersNow().then((r) =>
              toast({
                tone: 'success',
                title: r.created
                  ? t('ajustes-notif-novos', { n: r.created })
                  : t('ajustes-notif-nenhum'),
                description: t('ajustes-notif-caixa'),
              }),
            )
          }
        >
          {t('ajustes-notif-conferir')}
        </Button>
      </div>
    </div>
  );
}

/** Uma linha dos limites: "atualizado há" e "possivelmente incompleto", em dias. */
function LimitRow({ s }: { s: StalenessSetting }) {
  const save = useSetStalenessLimits();
  const [aging, setAging] = useState(String(s.limits.aging_days));
  const [incomplete, setIncomplete] = useState(String(s.limits.incomplete_days));
  const commit = (kind: AccountKind) => {
    const a = Number(aging || 0);
    const i = Number(incomplete || 0);
    if (a === s.limits.aging_days && i === s.limits.incomplete_days) return;
    save.mutate(
      { kind, limits: { aging_days: a, incomplete_days: i } },
      {
        onError: () => {
          setAging(String(s.limits.aging_days));
          setIncomplete(String(s.limits.incomplete_days));
        },
      },
    );
  };
  const label = ACCOUNT_KIND[s.kind];
  return (
    <div className="grid grid-cols-[1fr_6rem_6rem_auto] items-center gap-2">
      <span className="text-[13px] text-fg">{label}</span>
      {(
        [
          [aging, setAging, 'ajustes-contas-paradas-envelhece'],
          [incomplete, setIncomplete, 'ajustes-contas-paradas-incompleto'],
        ] as const
      ).map(([value, setValue, key]) => (
        <Input
          key={key}
          numeric
          inputMode="numeric"
          aria-label={t(key, { tipo: label })}
          value={value}
          onChange={(e) => setValue(e.target.value.replace(/\D/g, ''))}
          onBlur={() => commit(s.kind)}
          suffix={t('ajustes-notif-dias')}
        />
      ))}
      <span className="min-w-16">
        {s.custom ? (
          <Button
            variant="ghost"
            size="sm"
            onClick={() =>
              save.mutate(
                { kind: s.kind },
                {
                  onSuccess: () => {
                    setAging(String(s.default_limits.aging_days));
                    setIncomplete(String(s.default_limits.incomplete_days));
                  },
                },
              )
            }
          >
            {t('ajustes-contas-paradas-padrao')}
          </Button>
        ) : null}
      </span>
      {save.error ? (
        <span className="col-span-4">
          <InlineError error={save.error} />
        </span>
      ) : null}
    </div>
  );
}

/** Ajustes › Notificações › Contas sem extrato recente: os limites de cada tipo de conta. */
function StalenessCard() {
  const q = useStalenessLimits();
  return (
    <InstrumentCard index="12.N2" title={t('ajustes-contas-paradas-titulo')}>
      <p className="mb-3 text-[12.5px] text-dim">{t('ajustes-contas-paradas-dica')}</p>
      <QueryState loading={q.isPending} error={q.error}>
        <div className="flex flex-col gap-2">
          <div
            className="grid grid-cols-[1fr_6rem_6rem_auto] gap-2 label-caps text-dim"
            aria-hidden
          >
            <span>{t('ajustes-contas-paradas-tipo')}</span>
            <span>{t('ajustes-contas-paradas-col-envelhece')}</span>
            <span>{t('ajustes-contas-paradas-col-incompleto')}</span>
            <span className="min-w-16" />
          </div>
          {(q.data ?? []).map((s) => (
            <LimitRow key={`${s.kind}:${s.limits.aging_days}:${s.limits.incomplete_days}`} s={s} />
          ))}
        </div>
      </QueryState>
    </InstrumentCard>
  );
}

export function NotificationsSection() {
  const q = useNotifyPrefs();
  return (
    <div className="flex flex-col gap-5">
      <InstrumentCard index="12.N" title={t('ajustes-notif-titulo')}>
        <QueryState loading={q.isPending} error={q.error}>
          {q.data ? <Form prefs={q.data} /> : null}
        </QueryState>
      </InstrumentCard>
      <StalenessCard />
    </div>
  );
}
