/**
 * Aba Notícias do ativo (docs/asset-overview.md §2): os documentos oficiais da CVM e as
 * manchetes dos portais (filtradas no computador), da mais nova para a mais antiga. Só título,
 * veículo, data e link. A busca do Google Notícias é só um link que abre no navegador: o app não
 * a consulta (docs/data-sources.md).
 */
import type { SubjectArgs } from '@/api/generated/SubjectArgs';
import { useAssetNews, useDocuments } from '@/api/hooks/research';
import { Icon } from '@/design/components';
import { t } from '@/i18n';
import type { MessageKey } from '@/i18n/keys';
import { formatDate } from '@/lib/format';

import { ErrorBlock, LoadingBlock, Notice } from '../portfolio/components/States';
import { safeLink } from '../research/DocumentsPanel';

type Origin = 'cvm' | 'feed';

interface Row {
  key: string;
  date: string;
  title: string;
  source: string;
  link: string | null;
  origin: Origin;
}

const ORIGIN_LABEL = {
  cvm: 'ativo-noticia-origem-cvm',
  feed: 'ativo-noticia-origem-portal',
} as const satisfies Record<Origin, MessageKey>;

export function AssetNews({ subject }: { subject: SubjectArgs }) {
  const news = useAssetNews(subject);
  const docs = useDocuments({ ...subject, limit: 20 });
  if (news.isPending && docs.isPending) return <LoadingBlock />;
  const rows: Row[] = [
    ...(docs.data ?? []).map((d): Row => ({
      key: `doc-${d.id}`,
      date: d.delivered_at,
      title: d.subject ?? d.category,
      source: d.category,
      link: safeLink(d.link),
      origin: 'cvm',
    })),
    ...(news.data?.items ?? []).map((n): Row => ({
      key: n.link,
      date: n.date ?? '',
      title: n.title,
      source: n.source,
      link: safeLink(n.link),
      origin: 'feed',
    })),
  ].sort((a, b) => b.date.localeCompare(a.date));
  const searchUrl = news.data?.search_url ? safeLink(news.data.search_url) : null;
  const failed = (news.data?.sources ?? []).filter((s) => !s.ok);
  return (
    <section aria-label={t('ativo-aba-noticias')} className="flex flex-col gap-4">
      <p className="text-[12.5px] text-dim">{t('ativo-noticias-texto')}</p>
      {searchUrl ? (
        <div
          role="note"
          className="flex flex-wrap items-center gap-3 rounded-card border border-line bg-surface px-3 py-2 text-[12.5px]"
        >
          <Icon name="info" size={13} className="text-accent" />
          <span className="flex-1 text-fg">{t('ativo-noticias-google-dica')}</span>
          <a
            href={searchUrl}
            target="_blank"
            rel="noreferrer noopener"
            className="text-accent hover:underline"
          >
            {t('ativo-noticias-abrir-google')}
          </a>
        </div>
      ) : null}
      {news.isError ? <ErrorBlock error={news.error} onRetry={() => void news.refetch()} /> : null}
      {rows.length === 0 ? (
        <Notice title={t('ativo-noticias-vazio')}>{t('ativo-noticias-vazio-texto')}</Notice>
      ) : (
        <ul
          aria-label={t('ativo-aba-noticias')}
          className="divide-y divide-line rounded-control border border-line"
        >
          {rows.map((r) => (
            <li key={r.key} className="flex flex-col gap-0.5 px-3 py-2">
              {r.link ? (
                <a
                  href={r.link}
                  target="_blank"
                  rel="noreferrer noopener"
                  className="text-[13px] text-fg hover:text-accent"
                >
                  {r.title}
                </a>
              ) : (
                <span className="text-[13px] text-fg">{r.title}</span>
              )}
              <span className="text-[11.5px] text-dim">
                {[r.date ? formatDate(r.date) : null, r.source, t(ORIGIN_LABEL[r.origin])]
                  .filter(Boolean)
                  .join(' · ')}
              </span>
            </li>
          ))}
        </ul>
      )}
      {failed.length ? (
        <p className="text-[11.5px] text-dim">
          {t('ativo-noticias-falhou', { fontes: failed.map((f) => f.source).join(', ') })}
        </p>
      ) : null}
    </section>
  );
}
