/**
 * Proventos do ativo na aba Proventos (ações e FIIs da B3): os anunciados pelo Fundamentus,
 * os que a família tinha direito e ainda não lançou (para confirmar) e o histórico lançado.
 */
import type { Asset } from '@/api/generated/Asset';
import { useFundamentus, useSyncFundamentusIncome } from '@/api/hooks/dataSources';
import {
  Button,
  Icon,
  InstrumentCard,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import type { FundamentusDividend } from '@/api/generated/FundamentusDividend';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';

import { formatDecimal } from '../portfolio/decimal';
import { PendingIncomeBlock } from './PendingIncome';

const COLUMNS: TechColumn<FundamentusDividend>[] = [
  {
    key: 'com',
    get header() {
      return t('ativos-anunciados-data-com');
    },
    cell: (d) => formatDate(d.date_com),
  },
  {
    key: 'kind',
    get header() {
      return t('ativos-termos-tipo');
    },
    cell: (d) => d.kind,
  },
  {
    key: 'value',
    get header() {
      return t('ativos-anunciados-valor-cota');
    },
    numeric: true,
    cell: (d) =>
      `R$ ${formatDecimal((d.value / Math.max(1, d.per_shares)).toFixed(8), { minDecimals: 2, maxDecimals: 8 })}`,
  },
  {
    key: 'pay',
    get header() {
      return t('ativos-pagamento');
    },
    cell: (d) => (d.pay_date ? formatDate(d.pay_date) : '—'),
  },
];

export function AnnouncedIncome({ asset }: { asset: Pick<Asset, 'id' | 'ticker' | 'class'> }) {
  const ticker = asset.ticker ?? '';
  const q = useFundamentus(ticker || null);
  const sync = useSyncFundamentusIncome();
  const list = q.data?.dividends ?? [];
  return (
    <div className="flex flex-col gap-4">
      <PendingIncomeBlock assetId={asset.id} />
      <InstrumentCard
        index="08.P"
        title={asset.class === 'fii' ? t('ativos-anunciados-fii') : t('ativos-anunciados-titulo')}
        actions={
          <Button
            size="sm"
            variant="secondary"
            loading={sync.isPending}
            icon={<Icon name="refresh" size={13} />}
            onClick={() =>
              sync.mutate(
                { asset_id: asset.id, refresh: true },
                {
                  onSuccess: (r) =>
                    toast({
                      tone: 'success',
                      title: t('ativos-anunciados-novos', { n: r.events_added }),
                      description: t('ativos-anunciados-novos-dica'),
                    }),
                },
              )
            }
          >
            {t('ativos-anunciados-buscar')}
          </Button>
        }
        footer={[{ label: t('ativos-fonte'), value: t('ativos-anunciados-fonte') }]}
      >
        <QueryState loading={q.isLoading} error={q.error}>
          {list.length ? (
            <TechTable
              label={t('ativos-anunciados-tabela', { ticker })}
              prefsId={`Proventos anunciados de ${ticker}`}
              columns={COLUMNS}
              rows={list.slice(0, 36)}
              getRowId={(d) => `${d.date_com}-${d.kind}-${d.value}`}
            />
          ) : (
            <p className="text-[13px] text-dim">{t('ativos-anunciados-nenhum')}</p>
          )}
        </QueryState>
        <InlineError error={sync.error} />
      </InstrumentCard>
    </div>
  );
}
