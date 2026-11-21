/**
 * Visão geral do ativo (docs/asset-overview.md §1): o básico num cartão, antes da posição.
 * Nome, classe, setor e segmento, a última cotação com a data e a fonte, os indicadores que mais
 * se olham (cada um leva aos fundamentos), o score de saúde e os documentos e as notícias mais
 * recentes. Tudo vem do que o app já tem; nada é calculado aqui.
 */
import type { SubjectArgs } from '@/api/generated/SubjectArgs';
import { useAssetNews, useDocuments, useIndicators } from '@/api/hooks/research';
import { useAssetScore } from '@/api/hooks/score';
import { InstrumentCard, KeyValue } from '@/design/components';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { formatUnitPrice } from '../portfolio/decimal';
import { sourceName } from '../portfolio/labels';
import { safeLink } from '../research/DocumentsPanel';
import { formatIndicatorValue } from '../research/format';

/** Os indicadores do cartão, na ordem (os que o ativo não tiver ficam de fora). */
const KEY_INDICATORS = ['market_cap', 'pl', 'pvp', 'dy', 'roe', 'net_margin', 'ev_ebitda'];

export function AssetOverview({
  subject,
  onFundamentals,
  onNews,
}: {
  subject: SubjectArgs;
  /** Abre a aba de fundamentos (o cartão some quando não há). */
  onFundamentals?: () => void;
  onNews: () => void;
}) {
  const ind = useIndicators(subject);
  const score = useAssetScore(subject);
  const docs = useDocuments({ ...subject, limit: 3 });
  const news = useAssetNews(subject);
  const v = ind.data;
  const s = v?.subject;
  const keys = (v?.indicators ?? []).filter(
    (i) => KEY_INDICATORS.includes(i.key) && i.value != null,
  );
  keys.sort((a, b) => KEY_INDICATORS.indexOf(a.key) - KEY_INDICATORS.indexOf(b.key));
  const sc = score.data;
  const latest = [
    ...(docs.data ?? []).map((d) => ({
      key: `doc-${d.id}`,
      date: d.delivered_at,
      title: d.subject ?? d.category,
      source: `CVM · ${d.category}`,
      link: safeLink(d.link),
    })),
    ...(news.data?.items ?? []).slice(0, 5).map((n) => ({
      key: n.link,
      date: n.date ?? '',
      title: n.title,
      source: n.source,
      link: safeLink(n.link),
    })),
  ]
    .sort((a, b) => b.date.localeCompare(a.date))
    .slice(0, 6);

  return (
    <section
      aria-label={t('ativo-resumo-do-ativo')}
      className="grid gap-4 @min-[1040px]:grid-cols-3"
    >
      <InstrumentCard index="00" title={t('ativo-visao-geral')}>
        <KeyValue
          items={[
            ...(s?.issuer_name ? [{ label: t('ativo-empresa'), value: s.issuer_name }] : []),
            ...(s?.sector || s?.segment
              ? [
                  {
                    label: t('ativo-setor'),
                    value: [s.sector, s.segment].filter(Boolean).join(' · '),
                  },
                ]
              : []),
            ...(v?.price
              ? [
                  {
                    label: t('ativo-cotacao'),
                    value: (
                      <span>
                        <span className="num text-fg">
                          {formatUnitPrice(v.price.close, s?.currency ?? 'BRL')}
                        </span>{' '}
                        <span className="text-[11.5px] text-dim">
                          {t('ativo-cotacao-em', {
                            data: formatDate(v.price.date),
                            fonte: sourceName(v.price.source),
                          })}
                        </span>
                      </span>
                    ),
                  },
                ]
              : []),
            ...(sc && sc.score != null
              ? [
                  {
                    label: t('ativo-score'),
                    value: `${Math.round(sc.score)} · ${sc.band_label}`,
                  },
                ]
              : []),
          ]}
        />
        {!v && !ind.isPending ? (
          <p className="text-[12.5px] text-dim">{t('ativo-visao-geral-sem-dados')}</p>
        ) : null}
      </InstrumentCard>
      <InstrumentCard index="01" title={t('ativo-indicadores-chave')}>
        {keys.length ? (
          <ul aria-label={t('ativo-indicadores-chave')} className="flex flex-col">
            {keys.map((i) => (
              <li
                key={i.key}
                className="flex items-baseline justify-between gap-3 border-b border-line/60 py-1.5 last:border-0"
              >
                <span className="text-[12.5px] text-dim">{i.label}</span>
                <span className="num text-[13px] text-fg">
                  {formatIndicatorValue(i.value, i.unit, i.currency, i.exact)}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-[12.5px] text-dim">{t('ativo-indicadores-sem')}</p>
        )}
        {onFundamentals && keys.length ? (
          <button
            type="button"
            className="mt-2 text-[12px] text-accent underline-offset-2 hover:underline"
            onClick={onFundamentals}
          >
            {t('ativo-ver-fundamentos')}
          </button>
        ) : null}
      </InstrumentCard>
      <InstrumentCard index="02" title={t('ativo-ultimas')}>
        {latest.length ? (
          <ul aria-label={t('ativo-ultimas')} className="flex flex-col gap-2">
            {latest.map((n) => (
              <li key={n.key} className="flex flex-col">
                {n.link ? (
                  <a
                    href={n.link}
                    target="_blank"
                    rel="noreferrer noopener"
                    className="text-[12.5px] leading-snug text-fg hover:text-accent"
                  >
                    {n.title}
                  </a>
                ) : (
                  <span className="text-[12.5px] leading-snug text-fg">{n.title}</span>
                )}
                <span className="text-[11px] text-dim">
                  {[n.date ? formatDate(n.date) : null, n.source].filter(Boolean).join(' · ')}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-[12.5px] text-dim">
            {news.isPending || docs.isPending
              ? t('controles-carregando')
              : t('ativo-ultimas-vazio')}
          </p>
        )}
        <button
          type="button"
          className="mt-2 text-[12px] text-accent underline-offset-2 hover:underline"
          onClick={onNews}
        >
          {t('ativo-ver-noticias')}
        </button>
      </InstrumentCard>
    </section>
  );
}
