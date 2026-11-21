import { useNavigate } from 'react-router';

import type { ImportBatch } from '@/api/generated/ImportBatch';
import { useUndo } from '@/api/hooks/changes';
import { useImportBatches } from '@/api/hooks/import';
import {
  Button,
  EmptyState,
  Icon,
  InstrumentCard,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { formatDateTime } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';
import { QueryState } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { t } from '@/i18n';

/** Aba "Importações": lotes importados, com atalho para os lançamentos e desfazer. */
export function ImportsPanel({ onImport }: { onImport: () => void }) {
  const { data = [], isLoading, error, refetch } = useImportBatches({ limit: 100 });
  const undo = useUndo();
  const navigate = useNavigate();

  const columns: TechColumn<ImportBatch>[] = [
    {
      key: 'created_at',
      header: t('importacao-col-quando'),
      width: '140px',
      numeric: true,
      align: 'left',
      sortValue: (b) => b.created_at,
      cell: (b) => formatDateTime(b.created_at),
    },
    {
      key: 'filename',
      header: t('importacao-col-arquivo'),
      sortValue: (b) => b.filename,
      cell: (b) => (
        <span>
          {b.filename} <span className="label-caps ml-1">{b.format}</span>
        </span>
      ),
    },
    {
      key: 'account',
      header: t('importacao-col-conta'),
      cell: (b) => <span className="text-dim">{b.account_name}</span>,
    },
    {
      key: 'inserted',
      header: t('importacao-col-inseridos'),
      numeric: true,
      sortValue: (b) => b.inserted,
      cell: (b) => b.inserted,
    },
    {
      key: 'duplicates',
      header: t('importacao-col-duplicados'),
      numeric: true,
      cell: (b) => b.duplicates,
    },
    {
      key: 'errors',
      header: t('importacao-col-erros'),
      numeric: true,
      cell: (b) =>
        b.errors.length ? (
          <span
            className="text-negative"
            title={b.errors
              .map((e) =>
                t('importacao-erro-linha', {
                  linha: String(e.line ?? e.index),
                  mensagem: e.message,
                }),
              )
              .join('\n')}
          >
            {b.errors.length}
          </span>
        ) : (
          0
        ),
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('importacao-col-acoes')}</span>,
      align: 'right',
      width: '210px',
      cell: (b) => (
        <span className="inline-flex gap-1" onClick={(e) => e.stopPropagation()}>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => void navigate(txnLink({ import_id: b.id }))}
          >
            {t('importacao-ver-lancamentos')}
          </Button>
          {b.change_set_id ? (
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                undo.mutate(
                  { id: b.change_set_id! },
                  {
                    onSuccess: () =>
                      toast({
                        tone: 'success',
                        title: t('importacao-desfeita'),
                        description: b.filename,
                      }),
                    onError: (e) =>
                      toast({
                        tone: 'error',
                        title: t('importacao-erro-desfazer'),
                        description: errorText(e),
                      }),
                  },
                )
              }
            >
              {t('importacao-desfazer')}
            </Button>
          ) : null}
        </span>
      ),
    },
  ];

  return (
    <InstrumentCard
      index="04.4"
      title={t('importacao-importacoes')}
      padded={false}
      actions={
        <Button
          size="sm"
          variant="secondary"
          icon={<Icon name="arrowDown" size={12} />}
          onClick={onImport}
        >
          {t('importacao-importar-extrato')}
        </Button>
      }
    >
      <QueryState loading={isLoading} error={error} onRetry={() => void refetch()}>
        {data.length === 0 ? (
          <div className="p-4">
            <EmptyState
              kicker={t('importacao-vazio-kicker')}
              title={t('importacao-vazio-titulo')}
              description={t('importacao-vazio-descricao')}
              actions={
                <Button variant="primary" onClick={onImport}>
                  {t('importacao-importar-extrato')}
                </Button>
              }
            />
          </div>
        ) : (
          <TechTable
            label={t('importacao-importacoes')}
            prefsId="Importações"
            columns={columns}
            rows={data}
            getRowId={(b) => b.id}
            dense
            defaultSort={{ key: 'created_at', direction: 'desc' }}
          />
        )}
      </QueryState>
    </InstrumentCard>
  );
}
