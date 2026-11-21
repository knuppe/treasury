/** 08 · Saúde financeira no painel: nota, faixa e as dimensões (ou indicadores) do modelo. */
import { Link } from 'react-router';

import type { HealthReport } from '@/api/generated/HealthReport';
import { useHealthScore } from '@/api/hooks/health';
import { Icon, InstrumentCard, Measure, Private } from '@/design/components';
import { cx } from '@/design/cx';
import { capitalize } from '@/features/reports/controls';
import { t } from '@/i18n';
import { formatMonthLong } from '@/lib/format';

import { bandTone, formatScore, MODEL_LABEL, STATUS } from './model';
import { PointsBar, PrivateText, ScoreScale, StatusChip } from './parts';

/** Link da seção de detalhes (Relatórios › Saúde), no mês do painel. */
export function healthLink(month?: string, current?: string): string {
  return month && month !== current ? `/relatorios?v=saude&mes=${month}` : '/relatorios?v=saude';
}

/** Card do painel; some com a análise desativada (e enquanto a nota não chega). */
export function HealthCard({
  month,
  current,
  memberId,
}: {
  month: string;
  current: string;
  /** Saúde de um membro (docs/members.md); sem ele, a da família. */
  memberId?: string | null;
}) {
  const q = useHealthScore({ as_of: month, ...(memberId ? { member_id: memberId } : {}) });
  const r = q.data;
  if (!r || !r.enabled) return null;
  return <HealthCardView report={r} link={healthLink(month, current)} />;
}

export function HealthCardView({ report: r, link }: { report: HealthReport; link: string }) {
  const tone = bandTone(r.band);
  const rows =
    r.dimensions.length > 0
      ? r.dimensions.map((d) => ({
          key: d.key,
          label: d.label,
          hint: d.description,
          points: d.score,
          status: d.status,
          text: d.score == null ? null : formatScore(d.score),
        }))
      : r.metrics.map((m) => ({
          key: m.key,
          label: m.label,
          hint: m.explanation,
          points: m.points,
          status: m.status,
          text: m.value_text,
        }));
  return (
    <InstrumentCard
      index="08"
      title={t('saude-titulo')}
      className="@min-[1040px]:col-span-12"
      footer={
        <div className="flex flex-wrap items-center justify-between gap-x-6 gap-y-2">
          <dl className="flex flex-wrap gap-x-6 gap-y-1 text-[12px]">
            <div className="flex items-baseline gap-2">
              <dt className="label-caps">{t('saude-modelo')}</dt>
              <dd className="text-fg">{MODEL_LABEL[r.model]}</dd>
            </div>
            <div className="flex items-baseline gap-2">
              <dt className="label-caps">{t('saude-janela')}</dt>
              <dd className="num text-fg">{t('saude-n-meses', { n: r.months_analyzed })}</dd>
            </div>
            <div className="flex items-baseline gap-2">
              <dt className="label-caps">{t('saude-avaliado')}</dt>
              <dd className="text-fg">{capitalize(formatMonthLong(r.as_of))}</dd>
            </div>
          </dl>
          <Link to={link} className="label-caps inline-flex items-center gap-1.5 hover:text-accent">
            {t('saude-ver-indicadores')} <Icon name="arrowRight" size={12} />
          </Link>
        </div>
      }
    >
      <div className="grid gap-6 @min-[720px]:grid-cols-[minmax(240px,5fr)_7fr]">
        <div className="flex min-w-0 flex-col gap-4">
          <div className="flex flex-wrap items-end gap-x-4 gap-y-2">
            <Measure
              size="lg"
              label={r.model === 'isf' ? t('saude-nota-isf') : t('saude-nota')}
              origin={{
                source: r.model === 'isf' ? t('saude-origem-isf') : t('saude-origem-classicos'),
                date: r.period?.to,
                status: 'estimado',
              }}
              changeKey={r.score ?? -1}
              value={
                r.score == null ? (
                  <span className="text-dim">{'—'}</span>
                ) : (
                  <span>
                    <Private>{formatScore(r.score)}</Private>
                    <span className="ml-1.5 text-[0.4em] text-dim">{'/ 100'}</span>
                  </span>
                )
              }
            />
            {r.band ? (
              <span
                className={cx(
                  'label-caps mb-1 inline-flex items-center gap-1.5 rounded-[3px] border px-2 py-0.5 text-[11px]',
                  STATUS[tone].chip,
                )}
              >
                <span aria-hidden>{STATUS[tone].mark}</span>
                {r.band.label}
              </span>
            ) : null}
          </div>
          <ScoreScale bands={r.bands} band={r.band} score={r.score} reference={r.reference} />
          <PrivateText text={r.summary} className="text-[12.5px] leading-snug text-dim" />
        </div>
        <ul
          className="flex min-w-0 flex-col gap-2.5"
          aria-label={r.dimensions.length > 0 ? t('saude-dimensoes') : t('saude-indicadores')}
        >
          {rows.map((row) => (
            <li key={row.key} className="flex flex-col gap-1">
              <span className="flex items-baseline justify-between gap-3 text-[13px]">
                <span className="flex min-w-0 items-center gap-2">
                  <span className="truncate text-fg" title={row.hint}>
                    {row.label}
                  </span>
                  <StatusChip status={row.status} />
                </span>
                <span className={cx('num shrink-0', STATUS[row.status].text)}>
                  {row.text == null ? '—' : <Private>{row.text}</Private>}
                </span>
              </span>
              <PointsBar points={row.points} status={row.status} label={row.label} />
            </li>
          ))}
        </ul>
      </div>
    </InstrumentCard>
  );
}
