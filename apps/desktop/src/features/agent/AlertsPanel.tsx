/**
 * Caixa de alertas do agente e das rotinas: estado (novo, visto, resolvido), filtros por
 * estado, gravidade e tipo, e as rotinas do gestor com a última execução.
 */
import { useMemo, useState } from 'react';
import { Link } from 'react-router';

import type { Alert } from '@/api/generated/Alert';
import type { AlertScreen } from '@/api/generated/AlertScreen';
import type { AlertTarget } from '@/api/generated/AlertTarget';
import type { AlertSeverity } from '@/api/generated/AlertSeverity';
import type { AlertStatus } from '@/api/generated/AlertStatus';
import {
  useAlerts,
  useDeleteAlerts,
  useRoutines,
  useRunRoutine,
  useSetAlertStatus,
} from '@/api/hooks/ai';
import {
  Button,
  Icon,
  InstrumentCard,
  Markdown,
  Private,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { ErrorBlock, LoadingBlock, errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';

import { ALERT_STATUS_LABEL, SEVERITY_LABEL, labels } from './format';
import { Pill } from './ui';

type StatusFilter = 'open' | AlertStatus;

const STATUS_LABEL: Record<StatusFilter, string> = labels({
  open: 'agente-filtro-abertos',
  new: 'agente-filtro-novos',
  seen: 'agente-filtro-vistos',
  resolved: 'agente-filtro-resolvidos',
});
const STATUS_FILTERS: readonly StatusFilter[] = ['open', 'new', 'seen', 'resolved'];

const SEVERITY_TONE = { info: 'accent', warning: 'warning', critical: 'negative' } as const;
const SEVERITY_ICON = { info: 'info', warning: 'alert', critical: 'alert' } as const;

const OPEN_LABEL: Record<AlertScreen, string> = labels({
  asset: 'agente-alerta-abrir-asset',
  account: 'agente-alerta-abrir-account',
  budget: 'agente-alerta-abrir-budget',
  transactions: 'agente-alerta-abrir-transactions',
  card_statement: 'agente-alerta-abrir-card-statement',
  physical_asset: 'agente-alerta-abrir-physical-asset',
  allocation: 'agente-alerta-abrir-allocation',
  reports: 'agente-alerta-abrir-reports',
  sources: 'agente-alerta-abrir-sources',
  asset_data: 'agente-alerta-abrir-asset-data',
  import: 'agente-alerta-abrir-import',
});

/** Caminho da tela de origem do alerta (o ativo, a conta, a categoria do mês…). */
export function alertHref(t: AlertTarget): string | null {
  const q = (p: Record<string, string | null | undefined>) => {
    const s = new URLSearchParams();
    for (const [k, v] of Object.entries(p)) if (v) s.set(k, v);
    const str = s.toString();
    return str ? `?${str}` : '';
  };
  switch (t.screen) {
    case 'asset':
      return t.id ? `/ativos${q({ ativo: t.id })}` : null;
    case 'account':
      return t.id ? `/contas${q({ conta: t.id })}` : null;
    case 'budget':
      return `/orcamento${q({ mes: t.month })}`;
    case 'transactions':
      return t.account_id
        ? txnLink({
            account_ids: [t.account_id],
            ...(t.date ? { date_from: t.date, date_to: t.date } : {}),
          })
        : null;
    case 'card_statement':
      return t.id ? txnLink({ statement_id: t.id }) : null;
    case 'physical_asset':
      return t.id ? `/carteira${q({ aba: 'bens', bem: t.id })}` : null;
    case 'allocation':
      return '/carteira?aba=alocacao';
    case 'reports':
      return '/relatorios';
    case 'sources':
      return '/ajustes?secao=fontes';
    case 'asset_data':
      return '/carteira?aba=saude&secao=dados';
    case 'import':
      return `/lancamentos${q({ importar: '1', conta: t.account_id })}`;
  }
}

/**
 * Título colado no fim do texto anterior ("…do mês.## Fechamento"), como alguns motores
 * entregam ao juntar as falas entre as ferramentas: volta para a própria linha. O núcleo faz o
 * mesmo ao gravar; aqui vale para os alertas já gravados.
 */
export function tidyAlertBody(text: string): string {
  return text.replace(/([^\n#|])[ \t]*(#{2,6} )(?=\p{Lu})/gu, '$1\n\n$2');
}

/** Corpo do alerta em Markdown (títulos, listas, tabelas, negrito, itálico, links). */
function AlertBody({ text }: { text: string }) {
  return <Markdown text={tidyAlertBody(text)} tone="dim" className="text-[13px]" />;
}

/** Ferramentas de onde vieram os números (`data.sources`): só para quem quiser ver. */
function AlertSources({ data }: { data: Alert['data'] }) {
  const sources = Array.isArray(data.sources)
    ? data.sources.filter((s): s is string => typeof s === 'string')
    : [];
  if (!sources.length) return null;
  return (
    <details className="text-[11.5px] text-dim">
      <summary className="cursor-pointer select-none">{t('agente-alerta-avancado')}</summary>
      <p className="mt-1 font-mono">{t('agente-alerta-fontes', { lista: sources.join(', ') })}</p>
    </details>
  );
}

function AlertRow({ alert }: { alert: Alert }) {
  const href = alert.target ? alertHref(alert.target) : null;
  const setStatus = useSetAlertStatus();
  const del = useDeleteAlerts();
  const act = (status: AlertStatus) =>
    setStatus.mutate(
      { ids: [alert.id], status },
      {
        onError: (e) =>
          toast({
            tone: 'error',
            title: t('agente-nao-atualizou'),
            description: errorMessage(e),
          }),
      },
    );
  return (
    <li
      aria-label={t('agente-alerta-rotulo', { titulo: alert.title })}
      className={cx(
        'flex flex-col gap-1.5 px-4 py-3',
        alert.status === 'new' && 'border-l-2 border-l-accent',
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <span
          className={cx(
            alert.severity === 'critical'
              ? 'text-negative'
              : alert.severity === 'warning'
                ? 'text-warning'
                : 'text-accent',
          )}
        >
          <Icon name={SEVERITY_ICON[alert.severity]} size={13} />
        </span>
        <Pill tone={SEVERITY_TONE[alert.severity]}>{SEVERITY_LABEL[alert.severity]}</Pill>
        <Pill tone={alert.status === 'new' ? 'accent' : 'dim'}>
          {ALERT_STATUS_LABEL[alert.status]}
        </Pill>
        <h3 className="min-w-0 flex-1 text-[13.5px] font-medium text-fg">{alert.title}</h3>
        <span className="num text-[11px] text-dim">{formatDateTime(alert.created_at)}</span>
      </div>
      <Private>
        <AlertBody text={alert.body} />
      </Private>
      <AlertSources data={alert.data ?? {}} />
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-[11.5px] text-dim">
          {`${alert.kind_label || alert.kind} · ${alert.source}`}
        </span>
        <span className="flex-1" />
        {href && alert.target ? (
          <Link
            to={href}
            className="inline-flex items-center gap-1 rounded-control px-2 py-1 text-[12px] text-accent hover:bg-accent/10"
          >
            <Icon name="arrowRight" size={12} />
            {OPEN_LABEL[alert.target.screen]}
          </Link>
        ) : null}
        {alert.status === 'new' ? (
          <Button size="sm" variant="ghost" onClick={() => act('seen')}>
            {t('agente-marcar-visto')}
          </Button>
        ) : null}
        {alert.status !== 'resolved' ? (
          <Button
            size="sm"
            variant="secondary"
            onClick={() => act('resolved')}
            icon={<Icon name="check" size={12} />}
          >
            {t('agente-resolver')}
          </Button>
        ) : (
          <Button size="sm" variant="ghost" onClick={() => act('seen')}>
            {t('agente-reabrir')}
          </Button>
        )}
        <Button
          size="sm"
          variant="ghost"
          aria-label={t('agente-apagar-alerta', { titulo: alert.title })}
          onClick={() => del.mutate([alert.id])}
        >
          {t('comum-apagar')}
        </Button>
      </div>
    </li>
  );
}

function RoutinesCard() {
  const routines = useRoutines();
  const run = useRunRoutine();
  return (
    <InstrumentCard index="02" title={t('agente-rotinas')} padded={false}>
      {routines.isLoading ? (
        <LoadingBlock />
      ) : routines.error ? (
        <ErrorBlock error={routines.error} className="m-3" />
      ) : (
        <ul className="divide-y divide-line">
          {(routines.data ?? []).map((r) => (
            <li key={r.routine} className="flex flex-wrap items-center gap-3 px-4 py-2.5">
              <span className="min-w-0 flex-1">
                <span className="block text-[13px] text-fg">{r.label}</span>
                <span className="text-[11.5px] text-dim">
                  {[
                    r.enabled ? t('agente-rotina-ligada') : t('agente-rotina-desligada'),
                    r.engine_routed ? t('agente-rotina-com-motor') : t('agente-rotina-local'),
                    t('agente-rotina-ultima', {
                      quando: r.last_run
                        ? r.last_run.length > 7
                          ? formatDateTime(r.last_run)
                          : r.last_run
                        : t('agente-rotina-nunca'),
                    }),
                  ].join(' · ')}
                </span>
              </span>
              <Button
                size="sm"
                variant="ghost"
                disabled={r.running}
                loading={run.isPending && run.variables === r.routine}
                onClick={() =>
                  run.mutate(r.routine, {
                    onSuccess: () =>
                      toast({
                        tone: 'info',
                        title: t('agente-rotina-andamento', { rotina: r.label }),
                      }),
                    onError: (e) =>
                      toast({
                        tone: 'error',
                        title: t('agente-nao-rodou'),
                        description: errorMessage(e),
                      }),
                  })
                }
              >
                {r.running ? t('agente-rodando') : t('agente-rodar-agora')}
              </Button>
            </li>
          ))}
        </ul>
      )}
    </InstrumentCard>
  );
}

export function AlertsPanel() {
  const [status, setStatus] = useState<StatusFilter>('open');
  const [severity, setSeverity] = useState<'all' | AlertSeverity>('all');
  const [kind, setKind] = useState('all');
  const alerts = useAlerts(status === 'open' ? { limit: 200 } : { status, limit: 200 });
  const setMany = useSetAlertStatus();

  const all = alerts.data ?? [];
  const kinds = useMemo(() => {
    const m = new Map<string, string>();
    for (const a of alerts.data ?? []) m.set(a.kind, a.kind_label || a.kind);
    return [...m.entries()].sort((a, b) => a[1].localeCompare(b[1]));
  }, [alerts.data]);
  const shown = all.filter(
    (a) => (severity === 'all' || a.severity === severity) && (kind === 'all' || a.kind === kind),
  );
  const fresh = shown.filter((a) => a.status === 'new').map((a) => a.id);

  return (
    <div className="grid grid-cols-1 gap-4 @min-[1040px]:grid-cols-[minmax(0,1fr)_340px]">
      <InstrumentCard
        index="01"
        title={t('agente-caixa-alertas')}
        padded={false}
        actions={
          fresh.length > 0 ? (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setMany.mutate({ ids: fresh, status: 'seen' })}
            >
              {t('agente-marcar-todos-vistos')}
            </Button>
          ) : null
        }
      >
        <div className="flex flex-wrap gap-2 border-b border-line px-4 py-2.5">
          <div className="w-[220px]">
            <Select
              aria-label={t('agente-estado')}
              value={status}
              onValueChange={(v) => setStatus(v as StatusFilter)}
              options={STATUS_FILTERS.map((v) => ({ value: v, label: STATUS_LABEL[v] }))}
            />
          </div>
          <div className="w-[170px]">
            <Select
              aria-label={t('agente-gravidade')}
              value={severity}
              onValueChange={(v) => setSeverity(v as 'all' | AlertSeverity)}
              options={[
                { value: 'all', label: t('agente-toda-gravidade') },
                ...(['critical', 'warning', 'info'] as const).map((s) => ({
                  value: s,
                  label: SEVERITY_LABEL[s],
                })),
              ]}
            />
          </div>
          <div className="w-[200px]">
            <Select
              aria-label={t('agente-tipo')}
              value={kind}
              onValueChange={setKind}
              options={[
                { value: 'all', label: t('agente-todos-tipos') },
                ...kinds.map(([k, label]) => ({ value: k, label })),
              ]}
            />
          </div>
        </div>
        {alerts.isLoading ? (
          <LoadingBlock />
        ) : alerts.error ? (
          <ErrorBlock error={alerts.error} className="m-3" onRetry={() => void alerts.refetch()} />
        ) : shown.length === 0 ? (
          <p className="px-4 py-8 text-center text-[13px] text-dim">{t('agente-sem-alertas')}</p>
        ) : (
          <ul aria-label={t('agente-aba-alertas')} className="divide-y divide-line">
            {shown.map((a) => (
              <AlertRow key={a.id} alert={a} />
            ))}
          </ul>
        )}
      </InstrumentCard>
      <RoutinesCard />
    </div>
  );
}
