/**
 * Aba "Saúde" da página do ativo: score de saúde do ativo (`score.get`), de 0 a 100, com a
 * faixa e o significado para quem compra e segura, confiança, sub-scores Saúde e
 * Atratividade, segmento da metodologia (trocável), por que a nota, dimensões e métricas,
 * modelos clássicos (Piotroski, Altman, Beneish, Magic Formula), conferência com o
 * Fundamentus, histórico e "discordo".
 *
 * Não é a saúde financeira da família (`features/health`).
 */
import type { ReactNode } from 'react';

import type { AssetScore } from '@/api/generated/AssetScore';
import type { AssetScoreArgs } from '@/api/generated/AssetScoreArgs';
import type { AssetScoreReconciliation } from '@/api/generated/AssetScoreReconciliation';
import type { ScoreFramework } from '@/api/generated/ScoreFramework';
import { useAssetScore, useScoreMethodology, useSetScoreSegment } from '@/api/hooks/score';
import {
  InstrumentCard,
  Measure,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatNumber, formatPercent } from '@/lib/format';

import { LoadingBlock, Notice } from '../portfolio/components/States';
import { safeDate } from '../portfolio/labels';
import { SubjectError } from '../research/common';
import { DimensionsList } from './Dimensions';
import { SecContactFix } from './SecContactFix';
import {
  BAND,
  bandKey,
  bandOfScore,
  flagEffect,
  formatConfidence,
  formatDelta,
  formatMetricValue,
  formatScore,
  SEVERITY,
} from './format';
import { BandChip, BandScale, Tag } from './parts';
import { ScoreEventsSection } from './ScoreEvents';
import { ScoreFeedbackSection } from './ScoreFeedback';
import { ScoreHistory } from './ScoreHistory';

const AUTO = 'auto';

/** Segmento da metodologia com a troca manual (só pela interface; `null` volta ao automático). */
function SegmentControl({ s }: { s: AssetScore }) {
  const methodology = useScoreMethodology();
  const set = useSetScoreSegment();
  const options = (methodology.data?.segments ?? []).filter((x) => x.class === s.asset_class);
  const ticker = s.ticker;
  if (!ticker || options.length === 0) {
    return <span className="text-[13px] text-fg">{s.segment_label || '—'}</span>;
  }
  const override = methodology.data?.overrides[ticker] ?? null;
  return (
    <div className="flex flex-col gap-1">
      <Select
        aria-label={t('score-segmento-metodologia')}
        className="h-8 text-[12.5px]"
        value={override ?? AUTO}
        disabled={set.isPending}
        onValueChange={(v) =>
          set.mutate(
            { ticker, class: s.asset_class, segment: v === AUTO ? null : v },
            {
              onSuccess: () =>
                toast({
                  tone: 'success',
                  title: v === AUTO ? t('score-segmento-automatico') : t('score-segmento-trocado'),
                  description: t('score-segmento-refeito', { ticker }),
                }),
              onError: (e) =>
                toast({ tone: 'error', title: t('score-segmento'), description: e.message }),
            },
          )
        }
        options={[
          { value: AUTO, label: t('score-segmento-auto-setor') },
          ...options.map((o) => ({ value: o.key, label: o.label })),
        ]}
      />
      <span className="text-[11.5px] leading-snug text-dim">
        {override ? t('score-escolhido-por-voce') : ''}
        {s.segment_label}
      </span>
    </div>
  );
}

function ScoreHeader({ s }: { s: AssetScore }) {
  const partial = s.status === 'partial';
  const insufficient = s.status === 'insufficient_data';
  const change = s.change;
  return (
    <InstrumentCard
      index="01"
      title={t('score-titulo')}
      headingLevel={3}
      footer={[
        {
          label: t('score-metodologia'),
          value: `v${s.methodology_version} · ${s.profile_label || '—'}`,
        },
        { label: t('score-avaliado-em'), value: safeDate(s.as_of) },
        {
          label: t('score-dados-de'),
          value: (
            <span className="inline-flex items-center gap-1.5">
              {safeDate(s.data_date)}
              {s.stale ? (
                <Tag tone="warning" title={t('score-defasado-dica')}>
                  {t('score-defasado')}
                </Tag>
              ) : null}
            </span>
          ),
        },
      ]}
    >
      <div className="grid gap-6 @min-[790px]:grid-cols-[minmax(240px,5fr)_7fr]">
        <div className="flex min-w-0 flex-col gap-3">
          <div className="flex flex-wrap items-end gap-x-4 gap-y-2">
            <Measure
              size="lg"
              label={partial ? t('score-nota-parcial') : t('score-nota')}
              changeKey={s.score ?? -1}
              value={
                s.score == null ? (
                  <span className="text-dim">—</span>
                ) : (
                  <span className={cx(partial && 'italic')}>
                    {formatScore(s.score)}
                    <span className="ml-1.5 text-[0.4em] text-dim not-italic">{'/ 100'}</span>
                  </span>
                )
              }
            />
            <BandChip
              size="md"
              className="mb-1"
              band={partial ? 'dados_insuficientes' : s.band}
              label={partial ? t('score-dados-insuficientes') : s.band_label}
              partial={partial}
            />
          </div>
          {s.score != null ? <BandScale score={s.score} band={partial ? '' : s.band} /> : null}
          {s.band_meaning && !insufficient ? (
            <p className="text-[12.5px] leading-snug text-dim">{s.band_meaning}</p>
          ) : null}
          {change ? (
            <p className="text-[12.5px] text-dim">
              {withSlots(
                change.band_changed
                  ? t('score-mudanca-faixa', {
                      pontos: slot(0),
                      data: safeDate(change.previous_as_of),
                      nota: formatScore(change.previous_score),
                      faixa: BAND[bandKey(change.previous_band)].label,
                    })
                  : t('score-mudanca', {
                      pontos: slot(0),
                      data: safeDate(change.previous_as_of),
                      nota: formatScore(change.previous_score),
                    }),
                [
                  <span
                    key="pontos"
                    className={cx(
                      'num',
                      (change.delta ?? 0) > 0
                        ? 'text-positive'
                        : (change.delta ?? 0) < 0
                          ? 'text-negative'
                          : 'text-fg',
                    )}
                  >
                    {(change.delta ?? 0) > 0 ? '▲ ' : (change.delta ?? 0) < 0 ? '▼ ' : ''}
                    {t('score-pontos', { delta: formatDelta(change.delta) })}
                  </span>,
                ],
              )}
            </p>
          ) : null}
        </div>
        <div className="flex min-w-0 flex-col gap-4">
          <div className="grid grid-cols-3 gap-4">
            <Measure
              size="md"
              label={t('score-confianca')}
              value={formatConfidence(s.confidence)}
            />
            <Measure
              size="md"
              label={t('score-saude')}
              hint={t('score-saude-dica')}
              value={
                <span className={BAND[bandOfScore(s.health)].text}>{formatScore(s.health)}</span>
              }
            />
            <Measure
              size="md"
              label={t('score-atratividade')}
              hint={t('score-atratividade-dica')}
              value={
                <span className={BAND[bandOfScore(s.attractiveness)].text}>
                  {formatScore(s.attractiveness)}
                </span>
              }
            />
          </div>
          <div className="grid gap-4 @min-[530px]:grid-cols-2">
            <div className="flex min-w-0 flex-col gap-1.5">
              <span className="label-caps">{t('score-segmento')}</span>
              <SegmentControl s={s} />
            </div>
            <div className="flex min-w-0 flex-col gap-1.5">
              <span className="label-caps">{t('score-pares')}</span>
              <span className="text-[12.5px] leading-snug text-fg">
                {s.peers
                  ? s.peers.sector
                    ? t('score-pares-setor', {
                        setor: s.peers.sector_size,
                        nome: s.peers.sector,
                        segmento: s.peers.segment_size,
                        classe: s.peers.class_size,
                      })
                    : t('score-pares-resumo', {
                        setor: s.peers.sector_size,
                        segmento: s.peers.segment_size,
                        classe: s.peers.class_size,
                      })
                  : '—'}
              </span>
            </div>
          </div>
          {change && change.alerts.length > 0 ? (
            <ul aria-label={t('score-alertas')} className="flex flex-col gap-1">
              {change.alerts.map((a) => (
                <li key={a} className="flex items-start gap-1.5 text-[12.5px] text-warning">
                  <span aria-hidden>!</span>
                  {a}
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      </div>
    </InstrumentCard>
  );
}

function InsufficientNotice({ s }: { s: AssetScore }) {
  const critical = s.missing.filter((m) => m.critical);
  const others = s.missing.filter((m) => !m.critical);
  return (
    <Notice tone="warning" title={t('score-nao-calculado')}>
      <div className="flex flex-col gap-2">
        <p className="text-fg">{s.band_meaning}</p>
        <ul aria-label={t('score-criticos-ausentes')} className="flex flex-col gap-0.5">
          {critical.map((m) => (
            <li key={m.key}>
              <span className="font-medium text-fg">{m.label}</span>{' '}
              <Tag tone="warning">{t('score-critica')}</Tag> · {m.dimension}: {m.reason}
            </li>
          ))}
        </ul>
        {others.length > 0 ? (
          <p>
            {t('score-tambem-sem-dado', {
              lista: others.map((m) => `${m.label} (${m.reason})`).join('; '),
            })}
          </p>
        ) : null}
        {critical.some((m) => m.key === 'sec_contact') ? <SecContactFix /> : null}
        <p>{t('score-sem-metricas')}</p>
      </div>
    </Notice>
  );
}

function ReasonList({
  title,
  items,
  mark,
  tone,
}: {
  title: string;
  items: readonly string[];
  mark: string;
  tone: string;
}) {
  if (items.length === 0) return null;
  return (
    <div className="flex min-w-0 flex-col gap-1.5">
      <h4 className="label-caps">{title}</h4>
      <ul aria-label={title} className="flex flex-col gap-1">
        {items.map((item) => (
          <li key={item} className="flex items-start gap-2 text-[13px] leading-snug text-fg">
            <span aria-hidden className={cx('num mt-px text-[11px]', tone)}>
              {mark}
            </span>
            {item}
          </li>
        ))}
      </ul>
    </div>
  );
}

function WhyCard({ s, index }: { s: AssetScore; index: string }) {
  const optional = s.missing.filter((m) => !m.critical);
  const scored = s.status !== 'insufficient_data';
  const drivers = scored ? s.top_drivers : [];
  const detractors = scored ? s.top_detractors : [];
  return (
    <InstrumentCard index={index} title={t('score-por-que')} headingLevel={3}>
      <div className="flex flex-col gap-5">
        <div
          className={cx(
            'grid gap-5 @min-[790px]:grid-cols-2',
            !drivers.length && !detractors.length && 'hidden',
          )}
        >
          <ReasonList title={t('score-a-favor')} items={drivers} mark="▲" tone="text-positive" />
          <ReasonList title={t('score-contra')} items={detractors} mark="▼" tone="text-negative" />
        </div>
        {s.flags.length > 0 ? (
          <div className="flex flex-col gap-1.5">
            <h4 className="label-caps">{t('score-red-flags')}</h4>
            <ul aria-label={t('score-red-flags')} className="flex flex-col gap-2">
              {s.flags.map((f) => {
                const effect = flagEffect(f);
                return (
                  <li
                    key={f.key}
                    className="flex flex-col gap-0.5 rounded-control border border-line px-3 py-2"
                  >
                    <span className="flex flex-wrap items-center gap-2 text-[13px]">
                      <span className="font-medium text-fg">{f.label}</span>
                      <span className={cx('label-caps text-[10px]', SEVERITY[f.severity].text)}>
                        {t('score-gravidade', {
                          nivel: SEVERITY[f.severity].label.toLowerCase(),
                        })}
                      </span>
                      {effect ? <span className="num text-negative">{effect}</span> : null}
                    </span>
                    <span className="text-[12.5px] text-dim">{f.detail}</span>
                  </li>
                );
              })}
            </ul>
            {s.raw_score != null && s.score != null && s.raw_score !== s.score ? (
              <p className="text-[12px] text-dim">
                {t('score-antes-flags', { nota: formatScore(s.raw_score) })}
              </p>
            ) : null}
          </div>
        ) : null}
        {optional.length > 0 && s.status !== 'insufficient_data' ? (
          <p className="text-[12px] leading-snug text-dim">
            {t('score-sem-dado-redistribuido', {
              lista: optional.map((m) => `${m.label} (${m.reason})`).join('; '),
            })}
          </p>
        ) : null}
      </div>
    </InstrumentCard>
  );
}

function frameworkValue(f: ScoreFramework): string {
  if (f.value == null) return '—';
  if (f.key === 'piotroski') {
    return t('score-piotroski', { valor: formatNumber(f.value, 0), total: f.items.length || 9 });
  }
  return formatNumber(f.value, 2);
}

function FrameworkCard({ f }: { f: ScoreFramework }) {
  return (
    <div
      className="flex min-w-0 flex-col gap-2 rounded-control border border-line px-3 py-2.5"
      data-framework={f.key}
    >
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <h4 className="label-caps text-fg">{f.label}</h4>
        <span className="flex items-baseline gap-2">
          <span className="num text-[15px] text-fg">{frameworkValue(f)}</span>
          {f.reading ? <span className="text-[12px] text-dim">{f.reading}</span> : null}
        </span>
      </div>
      {f.items.length > 0 ? (
        <ul
          aria-label={t('score-itens', { modelo: f.label })}
          className="flex flex-col gap-0.5 text-[12.5px]"
        >
          {f.items.map((it) => (
            <li key={it.label} className="flex items-baseline justify-between gap-3">
              <span className="min-w-0 text-fg">{it.label}</span>
              {it.passed != null ? (
                <span
                  className={cx('num shrink-0', it.passed ? 'text-positive' : 'text-negative')}
                  aria-label={it.passed ? t('score-atende') : t('score-nao-atende')}
                >
                  {it.passed ? '✓' : '✗'}
                </span>
              ) : (
                <span className="num shrink-0 text-dim">
                  {it.value == null ? '—' : formatNumber(it.value, 2)}
                </span>
              )}
            </li>
          ))}
        </ul>
      ) : null}
      {f.note ? <p className="text-[12px] text-dim">{f.note}</p> : null}
      <p className="text-[11px] text-dim">{f.reference}</p>
    </div>
  );
}

function reconciliationColumns(s: AssetScore): TechColumn<AssetScoreReconciliation>[] {
  const units = new Map(s.dimensions.flatMap((d) => d.metrics.map((m) => [m.key, m.unit])));
  const fmt = (r: AssetScoreReconciliation, v: number) => {
    const unit = units.get(r.metric);
    // Proventos por ação (conferência com a B3) têm valores pequenos.
    return unit ? formatMetricValue(v, unit) : formatNumber(v, Math.abs(v) < 1 ? 4 : 2);
  };
  return [
    { key: 'label', header: t('score-metrica'), hideable: false },
    { key: 'ours', header: 'Treasury', numeric: true, cell: (r) => fmt(r, r.ours) },
    { key: 'theirs', header: t('score-fonte'), numeric: true, cell: (r) => fmt(r, r.theirs) },
    {
      key: 'diff',
      header: t('score-diferenca'),
      numeric: true,
      cell: (r) => (
        <span className={r.diff > 0.1 ? 'text-warning' : 'text-dim'}>
          {formatPercent(r.diff, { decimals: 1 })}
        </span>
      ),
    },
    { key: 'source', header: t('score-conferido-com') },
  ];
}

function Section({ children }: { children: ReactNode }) {
  return <div className="flex flex-col gap-5">{children}</div>;
}

/** Fontes dos dados do score de cripto, com o link que cada uma pede na atribuição. */
const CRYPTO_SOURCES = [
  ['CoinGecko', 'https://www.coingecko.com/'],
  ['Coin Metrics (CC BY-NC 4.0)', 'https://coinmetrics.io/'],
  ['DefiLlama', 'https://defillama.com/'],
  ['mempool.space', 'https://mempool.space/'],
] as const;

export function AssetScorePanel({ subject }: { subject: AssetScoreArgs }) {
  const q = useAssetScore(subject);
  if (q.isPending) return <LoadingBlock label={t('score-calculando')} />;
  if (q.isError) return <SubjectError error={q.error} onRetry={() => void q.refetch()} />;
  const s = q.data;
  const footer = (
    <footer className="flex flex-col gap-1.5 border-t border-line pt-3 text-[12px] leading-snug text-dim">
      {s.notes.length > 0 ? (
        <ul aria-label={t('score-notas-calculo')} className="list-inside list-disc">
          {s.notes.map((n) => (
            <li key={n}>{n}</li>
          ))}
        </ul>
      ) : null}
      {s.asset_class === 'crypto' ? (
        // Atribuição com link ao lado do dado, como pedem a CoinGecko e a Coin Metrics (CC BY-NC).
        <p aria-label={t('score-cripto-fontes')}>
          {t('score-cripto-fontes-texto')}{' '}
          {CRYPTO_SOURCES.map(([name, href], i) => (
            <span key={href}>
              {i > 0 ? ' · ' : null}
              <a
                href={href}
                target="_blank"
                rel="noreferrer noopener"
                className="text-accent underline-offset-2 hover:underline"
              >
                {name}
              </a>
            </span>
          ))}
        </p>
      ) : null}
      <p>{s.disclaimer}</p>
    </footer>
  );

  if (s.status === 'not_supported') {
    return (
      <Section>
        <Notice title={t('score-indisponivel', { classe: s.class_label.toLowerCase() })}>
          {s.band_meaning}
        </Notice>
        {footer}
      </Section>
    );
  }

  let n = 1;
  const next = () => String(++n).padStart(2, '0');
  // Sem score não há "por que essa nota" (só as red flags, se houver).
  const hasWhy =
    s.flags.length > 0 ||
    (s.status !== 'insufficient_data' &&
      (s.top_drivers.length > 0 || s.top_detractors.length > 0 || s.missing.length > 0));
  return (
    <Section>
      <ScoreHeader s={s} />
      {s.asset_class === 'crypto' ? (
        <Notice title={t('score-cripto-titulo')}>{t('score-cripto-texto')}</Notice>
      ) : null}
      {s.status === 'insufficient_data' ? <InsufficientNotice s={s} /> : null}
      {s.status === 'partial' ? (
        <Notice tone="warning" title={t('score-parcial-titulo')}>
          {t('score-parcial-texto', { pct: Math.round(s.imputed_share * 100) })}
        </Notice>
      ) : null}
      {hasWhy ? <WhyCard s={s} index={next()} /> : null}
      {s.dimensions.length > 0 ? (
        <InstrumentCard index={next()} title={t('score-dimensoes-titulo')} headingLevel={3}>
          <div className="flex flex-col gap-3">
            <p className="text-[12.5px] leading-snug text-dim">{t('score-dimensoes-ajuda')}</p>
            <DimensionsList dimensions={s.dimensions} />
          </div>
        </InstrumentCard>
      ) : null}
      {s.frameworks.length > 0 ? (
        <InstrumentCard index={next()} title={t('score-modelos')} headingLevel={3}>
          <div className="grid gap-3 @min-[790px]:grid-cols-2">
            {s.frameworks.map((f) => (
              <FrameworkCard key={f.key} f={f} />
            ))}
          </div>
        </InstrumentCard>
      ) : null}
      {s.reconciliation.length > 0 ? (
        <InstrumentCard index={next()} title={t('score-conferencia')} headingLevel={3}>
          <div className="flex flex-col gap-2">
            <p className="text-[12.5px] text-dim">{t('score-conferencia-ajuda')}</p>
            <TechTable
              label={t('score-conferencia')}
              prefsId="Conferência com outras fontes"
              columns={reconciliationColumns(s)}
              rows={s.reconciliation}
              getRowId={(r) => r.metric}
              customizable={false}
              dense
            />
          </div>
        </InstrumentCard>
      ) : null}
      <ScoreEventsSection subject={subject} score={s} index={next()} />
      <ScoreHistory subject={subject} index={next()} />
      <ScoreFeedbackSection subject={subject} score={s} />
      {footer}
    </Section>
  );
}
