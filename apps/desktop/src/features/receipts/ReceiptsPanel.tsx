/**
 * O que a família (e cada membro) compra, por segmento de consumo, com o imposto de cada um
 * (docs/receipts.md), em 06 · Consumo › Compras. Só onde há imposto por item (Brasil). A
 * revisão do cupom é a Compra (`features/food/PurchaseDialog`).
 */
import { useMemo } from 'react';

import { useReceiptSummary } from '@/api/hooks/receipts';
import { InstrumentCard, Money, Select, TechTable } from '@/design/components';
import { QueryState } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatLocale, todayIso } from '@/lib/format';

const FAMILY = '__familia__';

function pct(v: number): string {
  const code = formatLocale() === 'xx' ? 'pt-BR' : formatLocale();
  return `${v.toLocaleString(code, { maximumFractionDigits: 1 })}%`;
}

// ── O que a família compra ──────────────────────────────────────

const periods = () => [
  { value: 'mes', label: t('cupons-periodo-mes') },
  { value: '3m', label: t('cupons-periodo-3m') },
  { value: 'ano', label: t('cupons-periodo-ano') },
];

function range(period: string): { from: string; to: string } {
  const to = todayIso();
  const [y, m] = to.split('-').map(Number) as [number, number];
  if (period === 'mes') return { from: `${to.slice(0, 7)}-01`, to };
  if (period === '3m') {
    const d = new Date(y, m - 3, 1);
    return {
      from: `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-01`,
      to,
    };
  }
  return { from: `${y}-01-01`, to };
}

export function PurchasesCard({
  period,
  onPeriod,
}: {
  period: string;
  onPeriod: (p: string) => void;
}) {
  const r = useMemo(() => range(period), [period]);
  const q = useReceiptSummary(r);
  const s = q.data;
  return (
    <InstrumentCard
      index="06.C2"
      title={t('cupons-familia-compra')}
      actions={
        <Select
          aria-label={t('cupons-periodo-compras')}
          value={period}
          onValueChange={onPeriod}
          options={periods()}
        />
      }
    >
      <QueryState loading={q.isLoading} error={q.error}>
        {s && s.receipts > 0 ? (
          <div className="flex flex-col gap-4">
            <p className="text-[13px] text-dim">
              {withSlots(
                t(s.tax_printed > 0 ? 'cupons-resumo-impresso' : 'cupons-resumo', {
                  n: s.receipts,
                  gasto: slot(0),
                  imposto: slot(1),
                  pct: s.spent > 0 ? pct((s.tax / s.spent) * 100) : '0%',
                }),
                [<Money key="g" cents={s.spent} />, <Money key="i" cents={s.tax} />],
              )}
            </p>
            <TechTable
              label={t('cupons-compras-grupo')}
              prefsId="Compras por grupo"
              dense
              rows={s.groups}
              getRowId={(g) => g.segment_id ?? FAMILY}
              columns={[
                { key: 'label', header: t('cupons-grupo'), cell: (g) => g.label },
                {
                  key: 'spent',
                  header: t('cupons-gasto'),
                  numeric: true,
                  cell: (g) => <Money cents={g.spent} />,
                },
                {
                  key: 'share',
                  header: t('cupons-pct-compras'),
                  numeric: true,
                  cell: (g) => pct(s.spent > 0 ? (g.spent / s.spent) * 100 : 0),
                },
                {
                  key: 'tax',
                  header: t('cupons-imposto'),
                  numeric: true,
                  cell: (g) => <Money cents={g.tax} />,
                },
              ]}
            />
            {s.members.length > 1 ? (
              <div className="grid gap-3 @min-[720px]:grid-cols-2">
                {s.members.map((m) => (
                  <section
                    key={m.member_id ?? FAMILY}
                    aria-label={t('cupons-compras-de', { membro: m.member_name })}
                    className="rounded-card border border-line px-3 py-2.5"
                  >
                    <h3 className="text-[13.5px] font-medium text-fg">
                      {m.member_name}
                      {' · '}
                      <Money cents={m.spent} />
                    </h3>
                    <ul className="mt-1 flex flex-col gap-0.5 text-[12.5px] text-dim">
                      {m.groups.slice(0, 4).map((g) => (
                        <li key={g.segment_id ?? FAMILY}>
                          {g.label}
                          {': '}
                          <Money cents={g.spent} />
                        </li>
                      ))}
                    </ul>
                  </section>
                ))}
              </div>
            ) : null}
            {s.top_items.length ? (
              <div className="flex flex-col gap-1">
                <h3 className="label-caps">{t('cupons-itens-pesam')}</h3>
                <ol className="grid gap-x-6 gap-y-0.5 text-[12.5px] @min-[720px]:grid-cols-2">
                  {s.top_items.slice(0, 10).map((it) => (
                    <li key={it.description} className="flex justify-between gap-2">
                      <span className="truncate text-fg">
                        {it.description}
                        {it.times > 1 ? (
                          <span className="text-dim">{` · ${it.times}×`}</span>
                        ) : null}
                      </span>
                      <Money cents={it.spent} />
                    </li>
                  ))}
                </ol>
              </div>
            ) : null}
          </div>
        ) : (
          <p className="text-[13px] text-dim">{t('cupons-vazio-familia')}</p>
        )}
      </QueryState>
    </InstrumentCard>
  );
}
