/**
 * Consumo por categoria: gasto, grupo de imposto (e de onde veio), alíquota do cenário com
 * os três cenários e o ajuste do estado na dica, e o imposto estimado. Cada linha troca o
 * grupo, fixa uma alíquota própria, volta ao padrão ou mostra a base do grupo com as fontes.
 */
import type { ConsumptionLine } from '@/api/generated/ConsumptionLine';
import type { Scenario } from '@/api/generated/Scenario';
import type { TaxEstimate } from '@/api/generated/TaxEstimate';
import {
  Button,
  Icon,
  IconButton,
  InstrumentCard,
  Money,
  Private,
  TechTable,
  Tooltip,
  type ContextMenuEntry,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import type { CategoryEditor } from './CategoryDialogs';
import { formatPoints, formatRate, ORIGIN_LABEL, pick, scenarioDef, SCENARIOS } from './model';

const rowId = (c: ConsumptionLine) => c.category_id ?? c.path;

export function consumptionMenu(
  c: { category_id: string | null; path: string; group_key?: string; origin?: string },
  editor: CategoryEditor,
  opts: { custom: boolean; basis: boolean },
): ContextMenuEntry[] {
  const id = c.category_id;
  if (!id) return [];
  return [
    {
      label: t('impostos-menu-trocar-grupo'),
      icon: 'sliders',
      onSelect: () => editor.open('group', id, c.path),
    },
    { label: t('impostos-menu-aliquota'), onSelect: () => editor.open('rate', id, c.path) },
    opts.basis
      ? {
          label: t('impostos-menu-ver-base'),
          icon: 'info',
          onSelect: () => editor.open('basis', id, c.path, c.group_key),
        }
      : null,
    { separator: true },
    {
      label: t('impostos-voltar-padrao'),
      icon: 'refresh',
      disabled: !opts.custom,
      disabledReason: t('impostos-ja-padrao'),
      onSelect: () => editor.reset(id, c.path),
    },
  ];
}

function RateCell({ line: c, scenario }: { line: ConsumptionLine; scenario: Scenario }) {
  return (
    <Tooltip
      side="left"
      content={
        <div className="flex min-w-52 flex-col gap-1.5 py-0.5">
          <span className="label-caps text-fg">{t('impostos-aliquota-pct')}</span>
          {c.own_rate != null ? (
            <span className="text-[12px] text-dim">{t('impostos-aliquota-propria-nota')}</span>
          ) : null}
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-0.5 text-[12px]">
            {SCENARIOS.map((s) => (
              <div key={s.value} className="contents">
                <dt className={cx(s.value === scenario ? 'text-fg' : 'text-dim')}>{s.label}</dt>
                <dd className="num text-right text-fg">{formatRate(c.rates[s.value])}</dd>
              </div>
            ))}
          </dl>
          {c.region_adjustment !== 0 ? (
            <span className="text-[12px] text-dim">
              {t('impostos-ajuste-icms', { pontos: formatPoints(c.region_adjustment) })}
            </span>
          ) : null}
        </div>
      }
    >
      <button
        type="button"
        className="num inline-flex items-center gap-1 rounded-sm text-fg underline decoration-line-strong decoration-dotted underline-offset-4 hover:text-accent focus-visible:outline focus-visible:outline-accent"
        aria-label={t('impostos-aliquota-de', {
          categoria: c.path,
          valores: SCENARIOS.map((s) => `${s.label} ${formatRate(c.rates[s.value])}`).join(', '),
        })}
      >
        {formatRate(pick(c.rates, scenario))}
        {c.own_rate != null ? (
          <span className="text-[10px] text-accent">{t('impostos-propria')}</span>
        ) : null}
      </button>
    </Tooltip>
  );
}

export function ConsumptionCard({
  estimate: e,
  scenario,
  editor,
  onOpenGroups,
}: {
  estimate: TaxEstimate;
  scenario: Scenario;
  editor: CategoryEditor;
  /** Abre o editor "Grupos das categorias". */
  onOpenGroups: () => void;
}) {
  const def = scenarioDef(scenario);
  const rows = e.consumption;
  const total = rows.reduce((s, c) => s + pick(c.taxes, scenario), 0);
  const custom = (c: ConsumptionLine) => c.origin === 'user' || c.own_rate != null;
  return (
    <InstrumentCard
      index="10.5"
      title={t('impostos-consumo-categoria')}
      padded={false}
      className="@min-[1040px]:col-span-12"
      actions={
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="sliders" size={12} />}
          onClick={onOpenGroups}
        >
          {t('impostos-grupos-categorias')}
        </Button>
      }
      footer={<p className="text-[12px] leading-snug text-dim">{t('impostos-consumo-nota')}</p>}
    >
      <TechTable<ConsumptionLine>
        label={t('impostos-consumo-categoria')}
        prefsId="taxburden.consumption"
        rows={rows}
        getRowId={rowId}
        minWidth="760px"
        maxHeight="520px"
        defaultSort={{ key: 'tax', direction: 'desc' }}
        empty={<p className="px-4 py-6 text-[13px] text-dim">{t('impostos-consumo-vazio')}</p>}
        rowMenu={(c) => consumptionMenu(c, editor, { custom: custom(c), basis: true })}
        rowMenuLabel={(c) => t('impostos-acoes-de', { categoria: c.path })}
        onActivate={(c) =>
          c.category_id && editor.open('basis', c.category_id, c.path, c.group_key)
        }
        footer={{
          path: <span className="label-caps">{t('impostos-total')}</span>,
          spent: <Money cents={e.consumption_spent} currency={e.currency} />,
          tax: <Money cents={total} currency={e.currency} estimated />,
        }}
        columns={[
          {
            key: 'path',
            header: t('impostos-categoria'),
            hideable: false,
            sortValue: (c) => c.path,
            cell: (c) => <span className="text-fg">{c.path}</span>,
          },
          {
            key: 'spent',
            header: t('impostos-gasto'),
            numeric: true,
            width: '130px',
            sortValue: (c) => c.spent,
            cell: (c) => <Money cents={c.spent} currency={e.currency} />,
          },
          {
            key: 'group',
            header: t('impostos-grupo'),
            sortValue: (c) => c.group_label,
            cell: (c) => (
              <span>
                {c.group_label}{' '}
                <span
                  className={cx(
                    'label-caps ml-1 text-[10px] whitespace-nowrap',
                    c.origin === 'user' ? 'text-accent' : 'text-dim',
                  )}
                >
                  {ORIGIN_LABEL[c.origin] ?? c.origin}
                </span>
              </span>
            ),
          },
          {
            key: 'rate',
            header: t('impostos-aliquota'),
            title: t('impostos-aliquota-cenario', { cenario: def.label }),
            numeric: true,
            width: '150px',
            sortValue: (c) => pick(c.rates, scenario),
            cell: (c) => <RateCell line={c} scenario={scenario} />,
          },
          {
            key: 'tax',
            header: t('impostos-imposto-estimado'),
            numeric: true,
            width: '150px',
            sortValue: (c) => pick(c.taxes, scenario),
            cell: (c) => <Money cents={pick(c.taxes, scenario)} currency={e.currency} estimated />,
          },
          {
            key: 'share',
            header: t('impostos-do-consumo'),
            numeric: true,
            width: '96px',
            defaultHidden: true,
            sortValue: (c) => pick(c.taxes, scenario),
            cell: (c) => (
              <Private>
                {total > 0 ? formatRate((pick(c.taxes, scenario) / total) * 100, 1) : '—'}
              </Private>
            ),
          },
          {
            key: 'actions',
            header: '',
            title: t('impostos-acoes'),
            sortable: false,
            width: '84px',
            align: 'right',
            cell: (c) =>
              c.category_id ? (
                <span className="inline-flex items-center gap-0.5">
                  <IconButton
                    size="sm"
                    label={t('impostos-ver-base-de', { categoria: c.path })}
                    icon={<Icon name="info" size={13} />}
                    onClick={() => editor.open('basis', c.category_id!, c.path, c.group_key)}
                  />
                  <IconButton
                    size="sm"
                    label={t('impostos-trocar-grupo-de', { categoria: c.path })}
                    icon={<Icon name="sliders" size={13} />}
                    onClick={() => editor.open('group', c.category_id!, c.path)}
                  />
                </span>
              ) : null,
          },
        ]}
      />
    </InstrumentCard>
  );
}
