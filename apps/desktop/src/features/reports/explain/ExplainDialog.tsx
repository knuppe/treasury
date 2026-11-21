import { useNavigate } from 'react-router';

import { useExplain } from '@/api/hooks/reports';
import type { ValueSource } from '@/api/generated/ValueSource';
import { Button, Dialog, Icon, Money, TechTable, type TechColumn } from '@/design/components';
import { formatDate, formatPercent } from '@/lib/format';
import { t } from '@/i18n';

import { Loading, QueryError } from '../controls';
import { useExplainStore } from './store';

const KIND_LABEL = {
  txn: 'relatorios-fonte-lancamento',
  account: 'relatorios-fonte-conta',
  fx_rate: 'relatorios-fonte-cambio',
  opening: 'relatorios-fonte-saldo-inicial',
  index: 'relatorios-fonte-indice',
  portfolio: 'relatorios-fonte-carteira',
  loan: 'relatorios-fonte-divida',
  physical: 'relatorios-fonte-bem-fisico',
} as const satisfies Record<ValueSource['kind'], string>;

type SourceRow = ValueSource & { _i: number };

const columns = (): TechColumn<SourceRow>[] => [
  {
    key: 'kind',
    header: t('componentes-fonte'),
    width: '110px',
    cell: (s) => <span className="label-caps">{t(KIND_LABEL[s.kind])}</span>,
  },
  {
    key: 'label',
    header: t('relatorios-descricao'),
    cell: (s) => (
      <span className={s.estimated ? 'italic' : undefined}>
        {s.estimated ? '≈ ' : ''}
        {s.label}
      </span>
    ),
  },
  {
    key: 'date',
    header: t('componentes-data'),
    width: '96px',
    numeric: true,
    cell: (s) => (s.date ? formatDate(s.date) : '—'),
  },
  {
    key: 'amount',
    header: t('graficos-valor'),
    width: '140px',
    numeric: true,
    cell: (s) =>
      s.kind === 'fx_rate' ? (
        <span className="text-dim">{t('relatorios-taxa')}</span>
      ) : (
        <Money cents={s.amount} currency={s.currency} />
      ),
  },
];

/**
 * Painel "medido, nunca inventado": fórmula em português, valor, fontes (lançamentos,
 * contas, saldo inicial, câmbio) e avisos de estimado e de moedas sem câmbio.
 */
export function ExplainDialog() {
  const target = useExplainStore((s) => s.target);
  const close = useExplainStore((s) => s.close);
  const navigate = useNavigate();
  const query = useExplain(target?.args ?? null);
  const data = query.data;
  return (
    <Dialog
      open={target !== null}
      onOpenChange={(o) => {
        if (!o) close();
      }}
      index="≡"
      size="lg"
      title={
        target
          ? t('relatorios-explicacao-de', { titulo: target.title })
          : t('relatorios-explicacao')
      }
      footer={
        target?.link ? (
          <Button
            variant="primary"
            iconRight={<Icon name="arrowRight" size={13} />}
            onClick={() => {
              close();
              void navigate(target.link!);
            }}
          >
            {t('relatorios-ver-lancamentos')}
          </Button>
        ) : undefined
      }
    >
      {query.error ? (
        <QueryError error={query.error} onRetry={() => void query.refetch()} />
      ) : !data ? (
        <Loading label={t('relatorios-calculando-origem')} />
      ) : (
        <div className="flex flex-col gap-4">
          <div className="flex flex-wrap items-end justify-between gap-4">
            <div className="flex flex-col gap-1">
              <span className="label-caps">{t('graficos-valor')}</span>
              <span className="num text-[28px] leading-none font-medium text-fg">
                <Money cents={data.value} currency={data.currency} estimated={data.estimated} />
              </span>
              {data.ratio != null ? (
                <span className="num text-[13px] text-dim">{formatPercent(data.ratio)}</span>
              ) : null}
            </div>
            <div className="flex flex-col items-end gap-1 text-right">
              <span className="label-caps">{t('relatorios-fontes')}</span>
              <span className="num text-[13px] text-fg">
                {t('relatorios-itens', { n: data.count })}
                {data.truncated
                  ? ` · ${t('relatorios-mostrando', { n: data.sources.length })}`
                  : ''}
              </span>
            </div>
          </div>
          <section
            aria-label={t('relatorios-formula')}
            className="rounded-card border border-line bg-bg px-3.5 py-3"
          >
            <span className="label-caps text-accent">{t('relatorios-formula')}</span>
            <p className="mt-1 text-[13.5px] leading-relaxed text-fg">{data.formula}</p>
          </section>
          {data.estimated ? (
            <p className="flex items-center gap-2 text-[12.5px] text-warning italic">
              <Icon name="reticleDashed" size={13} /> {t('relatorios-inclui-estimado')}
            </p>
          ) : null}
          {data.missing_fx.length > 0 ? (
            <p className="flex items-center gap-2 text-[12.5px] text-warning">
              <Icon name="alert" size={13} />{' '}
              {t('relatorios-sem-cambio', { moedas: data.missing_fx.join(', ') })}
            </p>
          ) : null}
          <TechTable
            label={t('relatorios-fontes-valor')}
            prefsId="Fontes do valor"
            columns={columns()}
            rows={data.sources.map((s, i) => ({ ...s, _i: i }))}
            getRowId={(s) => String(s._i)}
            maxHeight="320px"
            dense
            empty={t('relatorios-nenhuma-fonte')}
          />
          {data.truncated ? (
            <p className="text-[12px] text-dim">
              {t('relatorios-lista-recentes', { n: data.sources.length })}
            </p>
          ) : null}
        </div>
      )}
    </Dialog>
  );
}
