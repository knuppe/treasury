/**
 * Fatos relevantes, comunicados, assembleias e reuniões do emissor (`research.documents`, IPE
 * da CVM), com o link oficial. Assunto e nome são textos da companhia: aparecem como texto
 * puro (nunca HTML). Sem documento, o diagnóstico (`research.documents_status`) diz por quê:
 * classe sem fonte (FIIs, ETFs), emissor não encontrado, coleta pendente ou companhia quieta
 * (docs/design/market-explorer.md, seção 3.5).
 */
import { useState } from 'react';
import { Link } from 'react-router';

import type { CorporateDocument } from '@/api/generated/CorporateDocument';
import type { DocumentsStatus } from '@/api/generated/DocumentsStatus';
import type { SubjectArgs } from '@/api/generated/SubjectArgs';
import { useDocuments, useDocumentsStatus } from '@/api/hooks/research';
import { Button, EmptyState, TechTable, type TechColumn } from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatDateTime, todayIso } from '@/lib/format';

import { Segmented } from '../portfolio/components/Segmented';
import { LoadingBlock } from '../portfolio/components/States';
import { safeDate } from '../portfolio/labels';
import { SubjectError } from './common';

type Cat = 'todos' | 'fato' | 'comunicado' | 'aviso' | 'assembleia';
/** Categorias do IPE como o núcleo filtra (texto da CVM, não traduzido). */
const CATEGORY: Record<Exclude<Cat, 'todos'>, string[]> = {
  fato: ['Fato Relevante'],
  comunicado: ['Comunicado ao Mercado'],
  aviso: ['Aviso aos Acionistas'],
  assembleia: ['Assembleia', 'Reunião da Administração'],
};

type Window = '1a' | 'tudo';

function yearAgo(today: string): string {
  const d = new Date(`${today}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() - 365);
  return d.toISOString().slice(0, 10);
}

/** Por que a lista ficou vazia, com a ação que resolve quando há uma. */
function EmptyReason({
  status,
  wide,
  onWiden,
}: {
  status: DocumentsStatus | undefined;
  wide: boolean;
  onWiden: () => void;
}) {
  const since = status ? formatDate(status.collected_since) : '';
  switch (status?.reason) {
    case 'not_covered':
      return (
        <EmptyState
          title={t('pesquisa-doc-sem-fonte')}
          description={t('pesquisa-doc-sem-fonte-texto')}
        />
      );
    case 'no_issuer':
      return (
        <EmptyState
          title={t('pesquisa-doc-sem-emissor')}
          description={t('pesquisa-doc-sem-emissor-texto')}
          actions={
            status.asset_id ? (
              <Link
                to={`/ativos?ativo=${encodeURIComponent(status.asset_id)}&dialogo=editar`}
                className="font-sans text-[12.5px] text-accent underline-offset-2 hover:underline"
              >
                {t('pesquisa-doc-informar-cnpj')}
              </Link>
            ) : null
          }
        />
      );
    case 'not_collected':
      return (
        <EmptyState
          title={t('pesquisa-doc-sem-coleta')}
          description={t('pesquisa-doc-sem-coleta-texto')}
        />
      );
    case 'quiet':
      return (
        <EmptyState
          title={t('pesquisa-doc-quieta', { data: since })}
          description={t('pesquisa-doc-quieta-texto')}
        />
      );
    default:
      return (
        <EmptyState
          title={t('pesquisa-doc-vazio')}
          description={
            status?.latest
              ? t('pesquisa-doc-ultimo', { data: formatDate(status.latest) })
              : t('pesquisa-doc-vazio-texto')
          }
          actions={
            wide ? null : (
              <Button size="sm" variant="secondary" onClick={onWiden}>
                {t('pesquisa-doc-ampliar', { data: since })}
              </Button>
            )
          }
        />
      );
  }
}

/** Fonte, emissor, coleta e o que está guardado por categoria (para conferir um caso vazio). */
function Diagnosis({ status }: { status: DocumentsStatus }) {
  return (
    <details className="group text-[12px] text-dim">
      <summary className="cursor-pointer select-none hover:text-fg">
        {t('pesquisa-doc-diagnostico')}
      </summary>
      <dl className="mt-2 grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1">
        <dt>{t('pesquisa-doc-diag-fonte')}</dt>
        <dd className="text-fg">{t('pesquisa-doc-diag-fonte-ipe')}</dd>
        <dt>{t('pesquisa-doc-diag-emissor')}</dt>
        <dd className="text-fg">
          {status.issuer_id
            ? `${status.issuer_name ?? '—'} · ${status.issuer_id}`
            : t('pesquisa-doc-diag-sem-emissor')}
        </dd>
        <dt>{t('pesquisa-doc-diag-coleta')}</dt>
        <dd className="text-fg">
          {status.last_sync_at
            ? t('pesquisa-doc-diag-coleta-em', {
                data: formatDateTime(status.last_sync_at),
                desde: formatDate(status.collected_since),
              })
            : t('pesquisa-doc-diag-sem-coleta')}
        </dd>
        <dt>{t('pesquisa-doc-diag-guardados')}</dt>
        <dd className="text-fg">
          {status.by_category.length
            ? status.by_category.map((c) => `${c.category} (${c.count})`).join(' · ')
            : t('pesquisa-doc-diag-nenhum')}
        </dd>
        <dt>{t('pesquisa-doc-diag-categorias')}</dt>
        <dd>{status.kept_categories.join(' · ')}</dd>
      </dl>
    </details>
  );
}

/** Só links https (o núcleo monta o da CVM); qualquer outra coisa vira texto. */
export function safeLink(url: string): string | null {
  return /^https:\/\//i.test(url) ? url : null;
}

const columns = (): TechColumn<CorporateDocument>[] => [
  {
    key: 'delivered',
    header: t('pesquisa-doc-entregue'),
    width: '100px',
    cell: (d) => safeDate(d.delivered_at),
    sortValue: (d) => d.delivered_at,
  },
  {
    key: 'category',
    header: t('pesquisa-doc-categoria'),
    width: '190px',
    cell: (d) => (
      <span className="flex flex-col font-sans leading-tight">
        <span className={d.category === 'Fato Relevante' ? 'text-warning' : undefined}>
          {d.category}
        </span>
        {d.doc_type ? <span className="truncate text-[11.5px] text-dim">{d.doc_type}</span> : null}
      </span>
    ),
  },
  {
    key: 'subject',
    header: t('pesquisa-doc-assunto'),
    cell: (d) => (
      <span className="flex flex-col font-sans leading-snug whitespace-normal">
        <span>{d.subject ?? t('pesquisa-doc-sem-assunto')}</span>
        {d.summary ? <span className="text-[12px] text-dim">{d.summary}</span> : null}
      </span>
    ),
  },
  {
    key: 'link',
    header: t('pesquisa-doc-documento'),
    width: '130px',
    cell: (d) => {
      const href = safeLink(d.link);
      return href ? (
        <a
          href={href}
          target="_blank"
          rel="noreferrer noopener"
          className="font-sans text-accent underline-offset-2 hover:underline"
          aria-label={t('pesquisa-doc-abrir-cvm-rotulo', {
            categoria: d.category,
            data: safeDate(d.delivered_at),
          })}
        >
          {t('pesquisa-doc-abrir-cvm')}
        </a>
      ) : (
        <span className="text-dim">—</span>
      );
    },
  },
];

export function DocumentsPanel({ subject }: { subject: SubjectArgs }) {
  const [cat, setCat] = useState<Cat>('todos');
  const [win, setWin] = useState<Window>('1a');
  const today = todayIso();
  const status = useDocumentsStatus(subject);
  const from = win === 'tudo' && status.data ? status.data.collected_since : yearAgo(today);
  const q = useDocuments({
    ...subject,
    from,
    to: today,
    limit: 200,
    ...(cat === 'todos' ? {} : { categories: CATEGORY[cat] }),
  });
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <Segmented<Cat>
          label={t('pesquisa-doc-categorias')}
          value={cat}
          onChange={setCat}
          options={[
            { value: 'todos', label: t('controles-todos') },
            { value: 'fato', label: t('pesquisa-doc-fatos') },
            { value: 'comunicado', label: t('pesquisa-doc-comunicados') },
            { value: 'aviso', label: t('pesquisa-doc-avisos') },
            { value: 'assembleia', label: t('pesquisa-doc-assembleias') },
          ]}
        />
        <Segmented<Window>
          label={t('pesquisa-doc-janela')}
          value={win}
          onChange={setWin}
          options={[
            { value: '1a', label: t('pesquisa-doc-12-meses') },
            { value: 'tudo', label: t('pesquisa-doc-tudo-coletado') },
          ]}
        />
      </div>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <SubjectError error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyReason status={status.data} wide={win === 'tudo'} onWiden={() => setWin('tudo')} />
      ) : (
        <div className="rounded-card border border-line bg-surface">
          <TechTable
            label={t('pesquisa-doc-tabela')}
            prefsId="Documentos da companhia"
            rows={q.data}
            columns={columns()}
            getRowId={(d) => d.id}
            dense
          />
        </div>
      )}
      <p className="text-[12px] text-dim">{t('pesquisa-doc-rodape')}</p>
      {status.data ? <Diagnosis status={status.data} /> : null}
    </div>
  );
}
