/**
 * Relatórios › Saúde financeira: a nota, as dimensões e cada indicador com valor × meta,
 * como foi calculado, os valores usados e as sugestões. Mês em `?mes=` (padrão: o atual).
 */
import { Link, useSearchParams } from 'react-router';

import type { HealthMetric } from '@/api/generated/HealthMetric';
import type { HealthReport } from '@/api/generated/HealthReport';
import { useHealthScore } from '@/api/hooks/health';
import { EmptyState, Icon, InstrumentCard, Money, Private } from '@/design/components';
import { cx } from '@/design/cx';
import { usePageContext } from '@/features/companion/pageContext';
import { capitalize, MonthPicker, QueryView } from '@/features/reports/controls';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, todayIso } from '@/lib/format';
import { monthOf } from '@/lib/months';

import { bandTone, formatScore, MODEL_LABEL, SOURCE_URL, STATUS } from './model';
import { PointsBar, PrivateText, ScoreScale, StatusChip } from './parts';

const SETTINGS = '/ajustes?secao=saude';

export function HealthPanel() {
  const [params, setParams] = useSearchParams();
  const current = monthOf(todayIso());
  const m = params.get('mes');
  const month = m && /^\d{4}-\d{2}$/.test(m) && m <= current ? m : current;
  const q = useHealthScore({ as_of: month });
  const setMonth = (next: string) => {
    const p = new URLSearchParams(params);
    if (next === current) p.delete('mes');
    else p.set('mes', next);
    setParams(p, { replace: true });
  };
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <MonthPicker month={month} max={current} onChange={setMonth} />
        <Link
          to={SETTINGS}
          className="label-caps inline-flex items-center gap-1.5 text-dim hover:text-accent"
        >
          <Icon name="sliders" size={12} /> {t('saude-trocar-modelo')}
        </Link>
      </div>
      <QueryView query={q} loadingLabel={t('saude-carregando')}>
        {(r) => (r.enabled ? <HealthDetail report={r} /> : <HealthOff />)}
      </QueryView>
    </div>
  );
}

function HealthOff() {
  return (
    <EmptyState
      kicker={t('saude-desativada')}
      title={t('saude-desligada-titulo')}
      description={t('saude-desligada-descricao')}
      actions={
        <Link
          to={SETTINGS}
          className="label-caps inline-flex items-center gap-1.5 text-accent hover:underline"
        >
          {t('saude-abrir-ajustes')} <Icon name="arrowRight" size={12} />
        </Link>
      }
    />
  );
}

export function HealthDetail({ report: r }: { report: HealthReport }) {
  usePageContext({
    screen: 'relatorios',
    title: `09 · ${t('casca-tela-relatorios')}`,
    view: 'saude',
    period: r.period ? { from: r.period.from, to: r.period.to } : undefined,
    figures: {
      model: r.model,
      score: r.score,
      band: r.band?.label ?? null,
      summary: r.summary,
    },
    items: r.metrics.map((m) => ({
      metric: m.label,
      status: m.status,
      value: m.value_text,
      target: m.target,
    })),
  });
  const tone = bandTone(r.band);
  const groups =
    r.dimensions.length > 0
      ? r.dimensions.map((d) => ({
          key: d.key,
          title: d.label,
          description: d.description,
          score: d.score,
          status: d.status,
          metrics: d.metrics
            .map((k) => r.metrics.find((m) => m.key === k))
            .filter((m): m is HealthMetric => m !== undefined),
        }))
      : [
          {
            key: 'indicadores',
            title: t('saude-indicadores'),
            description: t('saude-indicadores-descricao'),
            score: r.score,
            status: tone,
            metrics: r.metrics,
          },
        ];
  return (
    <div className="flex flex-col gap-5">
      <InstrumentCard
        index="SF"
        title={t('saude-titulo-modelo', { modelo: MODEL_LABEL[r.model] })}
        footer={[
          {
            label: t('saude-janela'),
            value: r.period
              ? t('saude-periodo', {
                  de: formatDate(r.period.from),
                  ate: formatDate(r.period.to),
                })
              : t('saude-sem-lancamentos'),
          },
          { label: t('saude-meses'), value: String(r.months_analyzed) },
          { label: t('saude-avaliado'), value: capitalize(formatMonthLong(r.as_of)) },
          ...(r.stale_inputs.length > 0
            ? [
                {
                  label: t('saude-contas-paradas'),
                  value: t('saude-contas-paradas-valor', { n: r.stale_inputs.length }),
                },
              ]
            : r.estimated
              ? [{ label: t('saude-cambio'), value: t('saude-cambio-estimado') }]
              : []),
        ]}
      >
        <div className="grid gap-6 @min-[720px]:grid-cols-[minmax(220px,4fr)_8fr]">
          <div className="flex flex-col gap-3">
            <span className="label-caps">{t('saude-nota-0-100')}</span>
            <div className="flex flex-wrap items-baseline gap-3">
              <span className="num text-[length:var(--measure-lg)] leading-none font-medium text-fg">
                {r.score == null ? (
                  <span className="text-dim">{'—'}</span>
                ) : (
                  <Private>{formatScore(r.score)}</Private>
                )}
              </span>
              {r.band ? (
                <span
                  className={cx(
                    'label-caps inline-flex items-center gap-1.5 rounded-[3px] border px-2 py-0.5 text-[11px]',
                    STATUS[tone].chip,
                  )}
                >
                  <span aria-hidden>{STATUS[tone].mark}</span>
                  {r.band.label}
                </span>
              ) : null}
            </div>
            <ScoreScale bands={r.bands} band={r.band} score={r.score} reference={r.reference} />
          </div>
          <div className="flex flex-col gap-3">
            <PrivateText text={r.summary} className="text-[14px] leading-snug text-fg" />
            {r.bands.length > 0 ? (
              <ul
                className="flex flex-wrap gap-x-4 gap-y-1 text-[11.5px] text-dim"
                aria-label={t('saude-faixas')}
              >
                {r.bands.map((b) => (
                  <li
                    key={b.key}
                    className={cx('num', r.band?.key === b.key && 'font-medium text-fg')}
                  >
                    {b.label} {b.min}–{b.max}
                  </li>
                ))}
              </ul>
            ) : null}
            <ul className="flex flex-col gap-1 text-[12px] leading-snug text-dim">
              {r.notes.map((n) => (
                <li key={n} className="flex gap-2">
                  <Icon name="info" size={12} className="mt-0.5 shrink-0" />
                  <span>{n}</span>
                </li>
              ))}
            </ul>
            {r.model === 'isf' ? (
              <a
                href={r.reference?.source_url ?? SOURCE_URL}
                target="_blank"
                rel="noreferrer"
                className="text-[12px] text-accent hover:underline"
              >
                {t('saude-sobre-isf')}
              </a>
            ) : null}
          </div>
        </div>
      </InstrumentCard>
      {groups.map((g, i) => (
        <section
          key={g.key}
          aria-label={g.title}
          className="flex flex-col gap-3 rounded-card border border-line/70 p-4"
        >
          <header className="flex flex-wrap items-baseline justify-between gap-3">
            <div className="flex min-w-0 flex-col gap-0.5">
              <h3 className="flex items-center gap-2 text-[14px] font-medium text-fg">
                <span className="num text-[11px] text-accent">
                  {String(i + 1).padStart(2, '0')}
                </span>
                {g.title}
                <StatusChip status={g.status} />
              </h3>
              <p className="text-[12.5px] text-dim">{g.description}</p>
            </div>
            <span className={cx('num text-[18px]', STATUS[g.status].text)}>
              {g.score == null ? '—' : <Private>{formatScore(g.score)}</Private>}
            </span>
          </header>
          <div className="grid gap-3 @min-[900px]:grid-cols-2">
            {g.metrics.map((m) => (
              <MetricCard key={m.key} metric={m} showWeight={r.dimensions.length === 0} />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

function MetricCard({ metric: m, showWeight }: { metric: HealthMetric; showWeight: boolean }) {
  return (
    <article
      aria-label={m.label}
      className="flex min-w-0 flex-col gap-2.5 rounded-control border border-line bg-surface-2/30 p-3.5"
    >
      <header className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-0.5">
          <h4 className="text-[13.5px] font-medium text-fg">{m.label}</h4>
          <p className="text-[12px] leading-snug text-dim">{m.explanation}</p>
        </div>
        <StatusChip status={m.status} />
      </header>
      <dl className="grid grid-cols-[auto_1fr] items-baseline gap-x-4 gap-y-1 text-[12.5px]">
        <dt className="label-caps">{t('saude-medido')}</dt>
        <dd className={cx('num text-[15px]', STATUS[m.status].text)}>
          {m.value_text == null ? t('saude-sem-dados') : <Private>{m.value_text}</Private>}
        </dd>
        <dt className="label-caps">{t('saude-meta')}</dt>
        <dd className="text-fg">{m.target}</dd>
        <dt className="label-caps">{t('saude-pontos')}</dt>
        <dd className="num text-fg">
          {m.points == null ? '—' : <Private>{formatScore(m.points)}</Private>}
          <span className="text-dim">
            {' '}
            {showWeight
              ? t('saude-peso-nota', { peso: Math.round(m.weight * 100) })
              : t('saude-peso-dimensao', { peso: Math.round(m.weight * 100) })}
          </span>
        </dd>
      </dl>
      <PointsBar points={m.points} status={m.status} label={m.label} />
      <p className="text-[12px] leading-snug text-dim">
        <span className="label-caps mr-1.5">{t('saude-calculo')}</span>
        {m.computation}
      </p>
      {m.amounts.length > 0 ? (
        <dl className="flex flex-wrap gap-x-5 gap-y-1 text-[12px]">
          {m.amounts.map((a) => (
            <div key={a.label} className="flex items-baseline gap-1.5">
              <dt className="text-dim">{a.label}</dt>
              <dd className="num text-fg">
                <Money cents={a.amount} currency={a.currency} />
              </dd>
            </div>
          ))}
        </dl>
      ) : null}
      {m.suggestions.length > 0 ? (
        <div className="flex flex-col gap-1 border-t border-line/70 pt-2">
          <span className="label-caps">
            {m.status === 'unknown' ? t('saude-para-medir') : t('saude-sugestoes')}
          </span>
          <ul className="flex flex-col gap-1 text-[12.5px] leading-snug text-fg">
            {m.suggestions.map((s) => (
              <li key={s} className="flex gap-2">
                <span aria-hidden className="text-accent">
                  {'→'}
                </span>
                <span>{s}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </article>
  );
}
