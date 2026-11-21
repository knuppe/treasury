/**
 * Números principais da estimativa no cenário escolhido: total, % da renda, % do consumo e
 * renda bruta estimada; a referência do país (carga tributária, com a fonte) e os avisos
 * para melhorar a estimativa.
 */
import type { Scenario } from '@/api/generated/Scenario';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import type { TaxEstimate } from '@/api/generated/TaxEstimate';
import { Icon, InstrumentCard, Measure, Money, Private } from '@/design/components';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatPercent } from '@/lib/format';

import { formatRate, intlLocale, pick, scenarioDef, sourcesOf } from './model';
import { sourceDate } from './SourcesDialog';

/** Dia do ano em que os impostos terminam de ser pagos ("30/05"), a partir de 1º de janeiro. */
function dayOfYear(year: number, days: number): string {
  const d = new Date(year, 0, 1);
  d.setDate(d.getDate() + Math.max(0, Math.round(days) - 1));
  return new Intl.DateTimeFormat(intlLocale(), { day: '2-digit', month: '2-digit' }).format(d);
}

export function SummaryCard({
  estimate: e,
  scenario,
}: {
  estimate: TaxEstimate;
  scenario: Scenario;
}) {
  const def = scenarioDef(scenario);
  const total = pick(e.totals, scenario);
  const consumptionTax = e.parts.find((p) => p.kind === 'consumption');
  return (
    <InstrumentCard
      index="10.1"
      title={t('impostos-estimativa-de', { ano: String(e.year) })}
      className="@min-[1040px]:col-span-8"
      footer={[
        {
          label: t('impostos-periodo-rotulo'),
          value: t('impostos-periodo', { de: formatDate(e.from), ate: formatDate(e.to) }),
        },
        { label: t('impostos-cenario'), value: def.label },
        {
          label: e.dataset.country === 'EU' ? t('impostos-pais') : t('impostos-regiao-estado'),
          value: e.region_name ?? t('impostos-nao-informado'),
        },
        { label: t('impostos-dados'), value: `${e.dataset.name} ${e.dataset.version}` },
      ]}
    >
      <div className="flex flex-col gap-6">
        <Measure
          size="xl"
          label={t('impostos-total-estimado-cenario', { cenario: def.label })}
          changeKey={`${scenario}:${total}`}
          value={<Money cents={total} currency={e.currency} estimated />}
          origin={{
            source: t('impostos-origem-estimativa', {
              dados: e.dataset.name,
              versao: e.dataset.version,
            }),
            date: e.to,
            status: 'estimado',
            note: t('impostos-origem-estimativa-nota'),
          }}
          hint={
            <span>
              {withSlots(t('impostos-faixa', { de: slot(0), ate: slot(1) }), [
                <Money key="de" cents={e.totals.conservative} currency={e.currency} />,
                <Money key="ate" cents={e.totals.high} currency={e.currency} />,
              ])}
            </span>
          }
        />
        <div className="grid grid-cols-1 gap-5 @min-[560px]:grid-cols-2 @min-[900px]:grid-cols-4">
          <Measure
            size="md"
            label={t('impostos-pct-renda')}
            value={<Private>{formatPercent(pick(e.share_of_income, scenario))}</Private>}
            hint={t('impostos-pct-renda-dica')}
          />
          <Measure
            size="md"
            label={t('impostos-pct-consumo')}
            value={<Private>{formatPercent(pick(e.share_of_consumption, scenario))}</Private>}
            hint={
              consumptionTax ? (
                <span>
                  {withSlots(t('impostos-pct-consumo-dica', { valor: slot(0) }), [
                    <Money key="v" cents={e.consumption_spent} currency={e.currency} />,
                  ])}
                </span>
              ) : (
                t('impostos-sem-consumo')
              )
            }
          />
          {e.work_days ? (
            <Measure
              size="md"
              label={t('impostos-dias-trabalho')}
              value={
                <Private>
                  {t('impostos-n-dias', { n: Math.round(pick(e.work_days, scenario)) })}
                </Private>
              }
              hint={
                <span>
                  {e.work_days_reference
                    ? t('impostos-dias-ate-media', {
                        data: dayOfYear(e.year, pick(e.work_days, scenario)),
                        dias: e.work_days_reference.days,
                        ano: String(e.work_days_reference.year),
                      })
                    : t('impostos-dias-ate', {
                        data: dayOfYear(e.year, pick(e.work_days, scenario)),
                      })}
                </span>
              }
            />
          ) : null}
          <Measure
            size="md"
            label={t('impostos-renda-bruta')}
            value={<Money cents={e.income_base} currency={e.currency} estimated />}
            origin={{
              source: t('impostos-renda-bruta-origem'),
              status: 'estimado',
            }}
            hint={t('impostos-renda-bruta-dica')}
          />
        </div>
      </div>
    </InstrumentCard>
  );
}

/** A carga tributária do país, só como referência (não é comparável com a da família). */
export function ContextCard({
  estimate: e,
  dataset,
}: {
  estimate: TaxEstimate;
  dataset: TaxDataset | undefined;
}) {
  const c = e.context;
  const sources = c ? sourcesOf(dataset, c.source_ids) : [];
  return (
    <InstrumentCard
      index="10.2"
      title={t('impostos-referencia-pais')}
      className="@min-[1040px]:col-span-4"
      headingLevel={2}
    >
      {c ? (
        <div className="flex flex-col gap-3">
          <Measure
            size="md"
            label={c.label}
            value={formatRate(c.value, 1)}
            hint={t('impostos-em-ano', { ano: String(c.year) })}
            origin={{
              source: sources.map((s) => s.publisher).join(' · ') || t('impostos-dados-pais'),
              status: 'oficial',
            }}
          />
          <p className="text-[12.5px] leading-relaxed text-dim">{t('impostos-referencia-texto')}</p>
          {e.work_days_reference ? (
            <p className="text-[12.5px] leading-relaxed text-dim">
              {t('impostos-referencia-dias', {
                rotulo: e.work_days_reference.label,
                dias: e.work_days_reference.days,
                ano: String(e.work_days_reference.year),
                ate: formatDate(e.work_days_reference.until),
                carga: formatRate(e.work_days_reference.share, 1),
              })}{' '}
              {sourcesOf(dataset, e.work_days_reference.source_ids).map((s) => (
                <a
                  key={s.id}
                  href={s.url}
                  target="_blank"
                  rel="noreferrer"
                  className="text-accent underline-offset-2 hover:underline"
                >
                  {s.publisher}
                </a>
              ))}
            </p>
          ) : null}
          {sources.map((s) => (
            <a
              key={s.id}
              href={s.url}
              target="_blank"
              rel="noreferrer"
              className="text-[12px] leading-snug text-accent underline-offset-2 hover:underline"
            >
              {s.publisher}: {s.title}
              {s.published ? ` (${sourceDate(s.published)})` : ''}
            </a>
          ))}
        </div>
      ) : (
        <p className="text-[13px] leading-relaxed text-dim">
          {t('impostos-sem-referencia', { dados: e.dataset.name })}
        </p>
      )}
    </InstrumentCard>
  );
}

export function NotesCard({ notes }: { notes: readonly string[] }) {
  if (notes.length === 0) return null;
  return (
    <section
      aria-label={t('impostos-melhorar')}
      className="flex flex-col gap-2 rounded-card border border-line/70 px-4 py-3 @min-[1040px]:col-span-12"
    >
      <h2 className="label-caps text-fg">{t('impostos-melhorar')}</h2>
      <ul className="flex flex-col gap-1.5 text-[12.5px] leading-snug text-dim">
        {notes.map((n) => (
          <li key={n} className="flex gap-2">
            <Icon name="info" size={12} className="mt-0.5 shrink-0" />
            <span>{n}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}
