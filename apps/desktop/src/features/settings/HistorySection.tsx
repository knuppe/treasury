import { useState } from 'react';

import type { AuthorKind } from '@/api/generated/AuthorKind';
import type { ChangeSetSummary } from '@/api/generated/ChangeSetSummary';
import { useChange, useChanges, useRedo, useUndo } from '@/api/hooks/changes';
import { Button, InstrumentCard, TechTable, toast, type TechColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { formatDateTime } from '@/lib/format';
import { ChangeReview } from '@/features/agent/ChangeReview';
import { InlineError, QueryState, Segmented } from '@/features/transactions/ui/controls';
import { errorCode, errorText } from '@/features/transactions/ui/errors';
import { AUTHOR_KIND, authorLabel, tableCount } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';

const PAGE = 50;

const STATUS_LABEL: Record<ChangeSetSummary['status'], string> = {
  get applied() {
    return t('ajustes-hist-aplicada');
  },
  get undone() {
    return t('ajustes-hist-desfeita');
  },
  get pending() {
    return t('ajustes-hist-pendente');
  },
  get rejected() {
    return t('ajustes-hist-rejeitada');
  },
};

const OPS = [
  ['insert', 'ajustes-hist-criou'],
  ['update', 'ajustes-hist-alterou'],
  ['delete', 'ajustes-hist-apagou'],
] as const;

function summaryText(c: ChangeSetSummary): string {
  const parts: string[] = [];
  for (const [op, key] of OPS) {
    const tables = c.summary[op];
    if (!tables) continue;
    const items = Object.entries(tables).map(([table, n]) => tableCount(table, n));
    parts.push(t(key, { itens: items.join(', ') }));
  }
  return parts.join(' · ');
}

function ChangeDetail({ id, onDone }: { id: string; onDone: () => void }) {
  const { data, isLoading, error } = useChange(id);
  const undo = useUndo();
  const redo = useRedo();
  const actionError = undo.error ?? redo.error;
  const items = data?.items ?? [];
  return (
    <QueryState loading={isLoading} error={error}>
      {data ? (
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="flex min-w-0 flex-col gap-1">
              <h3 className="text-[15px] font-medium text-fg">{data.reason}</h3>
              <p className="label-caps">
                {[
                  formatDateTime(data.created_at),
                  authorLabel(data.author),
                  data.undone_at
                    ? t('ajustes-hist-situacao-em', {
                        situacao: STATUS_LABEL[data.status],
                        data: formatDateTime(data.undone_at),
                      })
                    : STATUS_LABEL[data.status],
                ].join(' · ')}
              </p>
              <p className="text-[12.5px] text-dim">{summaryText(data)}</p>
            </div>
            {data.status === 'applied' ? (
              <Button
                variant="secondary"
                loading={undo.isPending}
                onClick={() =>
                  undo.mutate(
                    { id },
                    {
                      onSuccess: () => (
                        toast({
                          tone: 'success',
                          title: t('ajustes-hist-desfeito'),
                          description: data.reason,
                        }),
                        onDone()
                      ),
                    },
                  )
                }
              >
                {t('ajustes-hist-desfazer')}
              </Button>
            ) : data.status === 'undone' ? (
              <Button
                variant="secondary"
                loading={redo.isPending}
                onClick={() =>
                  redo.mutate(
                    { id },
                    {
                      onSuccess: () => (
                        toast({
                          tone: 'success',
                          title: t('ajustes-hist-refeito'),
                          description: data.reason,
                        }),
                        onDone()
                      ),
                    },
                  )
                }
              >
                {t('ajustes-hist-refazer')}
              </Button>
            ) : null}
          </div>
          {actionError ? (
            errorCode(actionError) === 'conflict' ? (
              <div
                role="alert"
                className="rounded-card border border-warning/60 bg-warning/5 px-3 py-2 text-[12.5px] text-warning"
              >
                <p className="font-medium">
                  {undo.error
                    ? t('ajustes-hist-conflito-desfazer')
                    : t('ajustes-hist-conflito-refazer')}
                </p>
                <p className="mt-0.5">{`${errorText(actionError)} ${t('ajustes-hist-conflito-dica')}`}</p>
              </div>
            ) : (
              <InlineError error={actionError} />
            )
          ) : null}
          <div className="max-h-[60vh] overflow-auto">
            <ChangeReview items={items} maxRows={12} showSummary={false} />
          </div>
        </div>
      ) : null}
    </QueryState>
  );
}

/** 11.7 · Histórico de alterações: quem mudou o quê, com diff e desfazer/refazer. */
export function HistorySection() {
  const [author, setAuthor] = useState<'all' | AuthorKind>('all');
  const [page, setPage] = useState(0);
  const [selected, setSelected] = useState<string | null>(null);
  const list = useChanges({
    limit: PAGE,
    offset: page * PAGE,
    ...(author !== 'all' ? { author_kind: author } : {}),
  });
  const rows = list.data ?? [];

  const columns: TechColumn<ChangeSetSummary>[] = [
    {
      key: 'when',
      header: t('ajustes-quando'),
      numeric: true,
      align: 'left',
      width: '132px',
      cell: (c) => formatDateTime(c.created_at),
    },
    {
      key: 'reason',
      header: t('ajustes-hist-col-alteracao'),
      cell: (c) => (
        <div className="flex min-w-0 flex-col py-1">
          <span className={cx('truncate', c.status === 'undone' && 'text-dim line-through')}>
            {c.reason}
          </span>
          <span className="truncate text-[11.5px] text-dim">{summaryText(c)}</span>
        </div>
      ),
    },
    {
      key: 'author',
      header: t('ajustes-hist-col-autor'),
      width: '22%',
      cell: (c) => <span className="block truncate text-dim">{authorLabel(c.author)}</span>,
    },
    {
      key: 'items',
      header: t('ajustes-linhas'),
      numeric: true,
      width: '64px',
      cell: (c) => c.item_count,
    },
    {
      key: 'status',
      header: t('ajustes-situacao'),
      width: '92px',
      cell: (c) => (
        <span className={c.status === 'undone' ? 'text-dim' : 'text-fg'}>
          {STATUS_LABEL[c.status]}
        </span>
      ),
    },
  ];

  return (
    <div className="grid gap-5 @min-[1300px]:grid-cols-[minmax(0,7fr)_minmax(0,6fr)]">
      <InstrumentCard
        index="12.7"
        title={t('ajustes-secao-historico')}
        padded={false}
        actions={
          <Segmented<'all' | AuthorKind>
            label={t('ajustes-hist-col-autor')}
            size="sm"
            value={author}
            onChange={(v) => (setAuthor(v), setPage(0), setSelected(null))}
            options={[
              { value: 'all', label: t('ajustes-hist-todos') },
              ...(['user', 'import', 'ai', 'system'] as const).map((k) => ({
                value: k,
                label: AUTHOR_KIND[k],
              })),
            ]}
          />
        }
      >
        <QueryState loading={list.isLoading} error={list.error}>
          <TechTable
            label={t('ajustes-hist-alteracoes')}
            prefsId="Alterações"
            dense
            columns={columns}
            rows={rows}
            getRowId={(c) => c.id}
            selectedId={selected}
            onSelectedChange={(id) => setSelected(id)}
            onActivate={(c) => setSelected(c.id)}
            className="[&_table]:table-fixed"
            maxHeight="48vh"
            empty={t('ajustes-hist-vazio')}
          />
          <nav
            aria-label={t('ajustes-hist-paginas')}
            className="flex items-center justify-end gap-2 border-t border-line px-3 py-2"
          >
            <Button
              size="sm"
              variant="ghost"
              disabled={page === 0}
              onClick={() => setPage(page - 1)}
            >
              {t('ajustes-hist-recentes')}
            </Button>
            <span className="num text-[12px] text-dim">
              {t('ajustes-hist-pagina', { n: page + 1 })}
            </span>
            <Button
              size="sm"
              variant="ghost"
              disabled={rows.length < PAGE}
              onClick={() => setPage(page + 1)}
            >
              {t('ajustes-hist-antigas')}
            </Button>
          </nav>
        </QueryState>
      </InstrumentCard>
      <InstrumentCard index="12.8" title={t('ajustes-hist-detalhe')}>
        {selected ? (
          <ChangeDetail key={selected} id={selected} onDone={() => void list.refetch()} />
        ) : (
          <p className="text-[13px] text-dim">{t('ajustes-hist-escolha')}</p>
        )}
      </InstrumentCard>
    </div>
  );
}
