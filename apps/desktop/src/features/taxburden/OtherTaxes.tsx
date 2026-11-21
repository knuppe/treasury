/**
 * O que não é consumo: impostos lançados como despesa (IR, IOF, IPTU, IPVA…), a renda do
 * trabalho (informada ou estimada a partir do líquido, com a base legal) e os informes de
 * rendimentos, o patrimônio (IPVA estimado pelo valor do veículo) e o IR retido em proventos.
 */
import { Link } from 'react-router';

import type { DirectLine } from '@/api/generated/DirectLine';
import type { PayrollLine } from '@/api/generated/PayrollLine';
import type { PropertyLine } from '@/api/generated/PropertyLine';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import type { TaxEstimate } from '@/api/generated/TaxEstimate';
import { useCountryInfo } from '@/api/hooks/country';
import { InstrumentCard, Measure, Money, TechTable } from '@/design/components';
import { cx } from '@/design/cx';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

import { IncomeReportsPanel } from './IncomeReports';
import { formatRate, sourcesOf } from './model';

export function DirectCard({ estimate: e }: { estimate: TaxEstimate }) {
  const total = e.direct.reduce((s, d) => s + d.amount, 0);
  return (
    <InstrumentCard
      index="10.7"
      title={t('impostos-lancados')}
      padded={false}
      className="@min-[1040px]:col-span-6 @min-[1040px]:row-span-2"
      footer={<p className="text-[12px] leading-snug text-dim">{t('impostos-lancados-nota')}</p>}
    >
      <TechTable<DirectLine>
        label={t('impostos-lancados')}
        prefsId="taxburden.direct"
        dense
        customizable={false}
        rows={e.direct}
        getRowId={(d) => d.category_id ?? d.path}
        empty={<p className="px-4 py-6 text-[13px] text-dim">{t('impostos-lancados-vazio')}</p>}
        footer={
          e.direct.length > 0
            ? {
                path: <span className="label-caps">{t('impostos-total')}</span>,
                amount: <Money cents={total} currency={e.currency} />,
              }
            : undefined
        }
        columns={[
          { key: 'path', header: t('impostos-categoria'), hideable: false },
          { key: 'label', header: t('impostos-tipo') },
          {
            key: 'amount',
            header: t('impostos-valor'),
            numeric: true,
            width: '130px',
            cell: (d) => <Money cents={d.amount} currency={e.currency} />,
          },
        ]}
      />
    </InstrumentCard>
  );
}

function OriginChip({ origin }: { origin: string }) {
  const reported = origin === 'reported' || origin === 'payslips';
  return (
    <span
      className={cx(
        'label-caps inline-flex items-center rounded-[3px] border px-1.5 py-px text-[10px] whitespace-nowrap',
        reported ? 'border-accent/50 text-accent' : 'border-line-strong text-dim',
      )}
    >
      {origin === 'payslips'
        ? t('impostos-origem-holerites')
        : reported
          ? t('impostos-origem-informado')
          : t('impostos-origem-estimado')}
    </span>
  );
}

export function PayrollCard({ estimate: e }: { estimate: TaxEstimate }) {
  const bases = [...new Set(e.payroll.flatMap((p) => p.legal_basis))];
  const country = useCountryInfo();
  const brazil = (country?.code ?? 'BR') === 'BR';
  return (
    <InstrumentCard
      index="10.10"
      title={t('impostos-renda-trabalho')}
      className="@min-[1040px]:col-span-12"
      footer={
        <p className="text-[12px] leading-snug text-dim">
          {brazil ? t('impostos-renda-trabalho-nota') : t('impostos-renda-trabalho-nota-exterior')}
        </p>
      }
    >
      <div className="flex flex-col gap-5">
        <TechTable<PayrollLine>
          label={t('impostos-renda-trabalho')}
          prefsId="taxburden.payroll"
          dense
          rows={e.payroll}
          getRowId={(p) => `${p.member_id ?? 'casa'}:${p.origin}`}
          minWidth="720px"
          empty={
            <p className="px-4 py-6 text-[13px] text-dim">{t('impostos-renda-trabalho-vazio')}</p>
          }
          columns={[
            {
              key: 'member',
              header: t('impostos-membro'),
              hideable: false,
              sortValue: (p) => p.member_name ?? '',
              cell: (p) => p.member_name ?? t('impostos-sem-membro'),
            },
            {
              key: 'origin',
              header: t('impostos-origem'),
              width: '120px',
              cell: (p) => <OriginChip origin={p.origin} />,
            },
            { key: 'months', header: t('impostos-meses'), numeric: true, width: '76px' },
            {
              key: 'net',
              header: t('impostos-liquido-recebido'),
              numeric: true,
              cell: (p) => <Money cents={p.net} currency={e.currency} />,
            },
            {
              key: 'gross',
              header: t('impostos-bruto'),
              numeric: true,
              cell: (p) => (
                <Money cents={p.gross} currency={e.currency} estimated={p.origin === 'estimated'} />
              ),
            },
            {
              key: 'social_security',
              header: country?.payroll.contribution ?? 'INSS',
              numeric: true,
              cell: (p) => (
                <Money
                  cents={p.social_security}
                  currency={e.currency}
                  estimated={p.origin === 'estimated'}
                />
              ),
            },
            {
              key: 'income_tax',
              header: t('impostos-ir-retido'),
              numeric: true,
              cell: (p) => (
                <Money
                  cents={p.income_tax}
                  currency={e.currency}
                  estimated={p.origin === 'estimated'}
                />
              ),
            },
          ]}
        />
        {bases.length > 0 ? (
          <div className="flex flex-col gap-1">
            <h3 className="label-caps text-fg">{t('impostos-regras-usadas')}</h3>
            <ul className="flex flex-col gap-0.5 text-[12px] leading-snug text-dim">
              {bases.map((b) => (
                <li key={b}>{b}</li>
              ))}
            </ul>
          </div>
        ) : null}
        {/* O informe de rendimentos (com 13º) é o documento do Brasil. */}
        {brazil ? <IncomeReportsPanel year={e.year} currency={e.currency} /> : null}
      </div>
    </InstrumentCard>
  );
}

export function PropertyCard({
  estimate: e,
  dataset,
}: {
  estimate: TaxEstimate;
  dataset: TaxDataset | undefined;
}) {
  const recorded = e.direct.filter(
    (d) => d.group_key === 'tax_property_iptu' || d.group_key === 'tax_property_ipva',
  );
  return (
    <InstrumentCard
      index="10.8"
      title={t('impostos-parte-patrimonio')}
      className="@min-[1040px]:col-span-6"
      footer={<p className="text-[12px] leading-snug text-dim">{t('impostos-patrimonio-nota')}</p>}
    >
      {e.property.length === 0 ? (
        <p className="text-[13px] leading-relaxed text-dim">
          {recorded.length > 0 ? t('impostos-patrimonio-lancados') : t('impostos-patrimonio-vazio')}
        </p>
      ) : (
        <ul className="flex flex-col divide-y divide-line/70">
          {e.property.map((p) => (
            <PropertyItem
              key={`${p.tax}:${p.asset_id}`}
              line={p}
              currency={e.currency}
              dataset={dataset}
            />
          ))}
        </ul>
      )}
    </InstrumentCard>
  );
}

function PropertyItem({
  line: p,
  currency,
  dataset,
}: {
  line: PropertyLine;
  currency: string;
  dataset: TaxDataset | undefined;
}) {
  const sources = sourcesOf(dataset, p.source_ids);
  return (
    <li className="flex flex-col gap-1 py-2 first:pt-0 last:pb-0">
      <div className="flex flex-wrap items-baseline justify-between gap-3">
        <span className="text-[13px] text-fg">
          {p.tax === 'iptu'
            ? t('iptu-impostos-linha', { imovel: p.name })
            : t('impostos-imposto-estimado-de', { imposto: p.tax.toUpperCase(), bem: p.name })}
        </span>
        <Money cents={p.amount} currency={currency} estimated className="text-[14px]" />
      </div>
      <p className="text-[12px] text-dim">
        {withSlots(
          t(p.tax === 'iptu' ? 'impostos-valor-venal-vezes' : 'impostos-valor-vezes', {
            valor: slot(0),
            aliquota: formatRate(p.rate),
          }),
          [<Money key="v" cents={p.value} currency={currency} />],
        )}
        {sources.length > 0 ? (
          <>
            {' · '}
            {sources.map((s, i) => (
              <span key={s.id}>
                {i > 0 ? ', ' : ''}
                <a
                  href={s.url}
                  target="_blank"
                  rel="noreferrer"
                  className="text-accent underline-offset-2 hover:underline"
                >
                  {s.publisher}
                </a>
              </span>
            ))}
          </>
        ) : null}
      </p>
    </li>
  );
}

export function InvestmentsCard({ estimate: e }: { estimate: TaxEstimate }) {
  return (
    <InstrumentCard
      index="10.9"
      title={t('impostos-ir-proventos')}
      className="@min-[1040px]:col-span-6"
    >
      <div className="flex flex-col gap-3">
        <Measure
          size="md"
          label={t('impostos-retido-fonte')}
          value={<Money cents={e.investments_withheld} currency={e.currency} />}
          origin={{ source: t('impostos-retido-fonte-origem') }}
        />
        <p className="text-[12.5px] leading-relaxed text-dim">
          {t('impostos-retido-texto')}{' '}
          <Link
            to="/carteira?aba=proventos"
            className="text-accent underline-offset-2 hover:underline"
          >
            {t('impostos-ver-proventos')}
          </Link>
        </p>
      </div>
    </InstrumentCard>
  );
}
