/**
 * Ajustes › Banco de dados: as 100 primeiras linhas de uma tabela, só leitura, para entender o
 * que ela guarda. Chaves, tokens e segredos cifrados vêm ocultos do núcleo; textos longos,
 * cortados; blobs, resumidos (tamanho).
 */
import { useMemo } from 'react';

import { useTableRows } from '@/api/hooks/database';
import { Button, Icon, InstrumentCard, TechTable, type TechColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

type Row = { id: string; cells: (string | number | null)[] };

export function TableViewer({ table, onBack }: { table: string; onBack: () => void }) {
  const q = useTableRows(table);
  const data = q.data;
  const rows = useMemo<Row[]>(
    () => (data?.rows ?? []).map((cells, i) => ({ id: String(i), cells })),
    [data],
  );
  const columns = useMemo<TechColumn<Row>[]>(
    () => [
      {
        key: '#',
        header: '#',
        width: '48px',
        numeric: true,
        cell: (r) => <span className="text-dim">{Number(r.id) + 1}</span>,
      },
      ...(data?.columns ?? []).map((c, i) => ({
        key: `c${i}`,
        header: c.primary_key ? `${c.name} 🔑` : c.name,
        sortValue: (r: Row) => r.cells[i] ?? '',
        cell: (r: Row) => {
          const v = r.cells[i];
          return (
            <span
              title={v == null ? t('ajustes-tabela-nulo') : String(v)}
              className={cx(
                'block max-w-[36ch] truncate font-mono text-[11.5px]',
                v == null || c.masked ? 'text-dim' : 'text-fg',
              )}
            >
              {v == null ? 'NULL' : String(v)}
            </span>
          );
        },
      })),
    ],
    [data],
  );
  return (
    <div className="flex flex-col gap-4">
      <div>
        <Button variant="secondary" icon={<Icon name="arrowLeft" size={13} />} onClick={onBack}>
          {t('comum-voltar')}
        </Button>
      </div>
      <InstrumentCard
        index="12.17"
        title={t('ajustes-tabela-titulo', { tabela: table })}
        padded={false}
      >
        <QueryState loading={q.isPending} error={q.error} label={t('ajustes-tabela-lendo')}>
          {data ? (
            <div className="flex flex-col">
              <p className="px-4 py-3 text-[12.5px] text-dim">
                {[
                  data.group,
                  t('ajustes-tabela-resumo', {
                    linhas: data.total_rows,
                    colunas: data.columns.length,
                    ordem: data.order,
                  }),
                  ...data.notes,
                ]
                  .filter(Boolean)
                  .join(' · ')}
              </p>
              <TechTable
                label={t('ajustes-tabela-primeiras', { tabela: table })}
                prefsId={`Primeiras linhas de ${table}`}
                columns={columns}
                rows={rows}
                getRowId={(r) => r.id}
                minWidth={`${Math.max(720, 160 * data.columns.length)}px`}
                maxHeight="calc(100vh - 320px)"
                dense
                empty={<p className="p-4 text-[13px] text-dim">{t('ajustes-tabela-vazia')}</p>}
              />
            </div>
          ) : null}
        </QueryState>
      </InstrumentCard>
    </div>
  );
}
