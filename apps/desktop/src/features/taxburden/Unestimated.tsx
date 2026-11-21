/**
 * Gasto que ficou fora da estimativa de consumo, agrupado pelo motivo, com a explicação de
 * cada um: sem categoria, sem grupo (escolher o grupo), fora do consumo e sem dado confiável
 * (fixar uma alíquota própria, se quiser).
 */
import { Link } from 'react-router';

import type { TaxEstimate } from '@/api/generated/TaxEstimate';
import type { UnestimatedLine } from '@/api/generated/UnestimatedLine';
import { InstrumentCard, Money } from '@/design/components';
import { uncategorizedDrill } from '@/features/reports/model/drill';
import { t } from '@/i18n';
import { txnLink } from '@/lib/txnLink';

import type { CategoryEditor } from './CategoryDialogs';
import { REASON_ORDER, REASONS } from './model';

export function UnestimatedCard({
  estimate: e,
  editor,
}: {
  estimate: TaxEstimate;
  editor: CategoryEditor;
}) {
  const groups = REASON_ORDER.map((reason) => ({
    reason,
    lines: e.unestimated.filter((u) => u.reason === reason),
  }))
    .concat(
      // Motivos que a interface ainda não conhece aparecem do mesmo jeito, no fim.
      [...new Set(e.unestimated.map((u) => u.reason))]
        .filter((r) => !REASON_ORDER.includes(r))
        .map((reason) => ({ reason, lines: e.unestimated.filter((u) => u.reason === reason) })),
    )
    .filter((g) => g.lines.length > 0);
  const total = e.unestimated.reduce((s, u) => s + u.amount, 0);
  return (
    <InstrumentCard
      index="10.6"
      title={t('impostos-fora')}
      className="@min-[1040px]:col-span-12"
      footer={[
        { label: t('impostos-gasto-fora'), value: <Money cents={total} currency={e.currency} /> },
        {
          label: t('impostos-gasto-estimado'),
          value: <Money cents={e.consumption_spent} currency={e.currency} />,
        },
      ]}
    >
      {groups.length === 0 ? (
        <p className="text-[13px] text-dim">{t('impostos-fora-vazio')}</p>
      ) : (
        <div className="grid gap-4 @min-[900px]:grid-cols-2">
          {groups.map((g) => (
            <ReasonGroup
              key={g.reason}
              reason={g.reason}
              lines={g.lines}
              currency={e.currency}
              editor={editor}
              uncategorizedLink={txnLink(uncategorizedDrill({ from: e.from, to: e.to }, 'accrual'))}
            />
          ))}
        </div>
      )}
    </InstrumentCard>
  );
}

function ReasonGroup({
  reason,
  lines,
  currency,
  editor,
  uncategorizedLink,
}: {
  reason: string;
  lines: readonly UnestimatedLine[];
  currency: string;
  editor: CategoryEditor;
  uncategorizedLink: string;
}) {
  const def = REASONS[reason] ?? { title: reason, explanation: '' };
  const sum = lines.reduce((s, u) => s + u.amount, 0);
  return (
    <section
      aria-label={def.title}
      className="flex min-w-0 flex-col gap-2 rounded-control border border-line bg-surface-2/30 p-3.5"
    >
      <header className="flex items-baseline justify-between gap-3">
        <h3 className="text-[13.5px] font-medium text-fg">{def.title}</h3>
        <Money cents={sum} currency={currency} className="text-[13px] text-fg" />
      </header>
      {def.explanation ? (
        <p className="text-[12px] leading-snug text-dim">{def.explanation}</p>
      ) : null}
      <ul className="flex flex-col divide-y divide-line/70 text-[12.5px]">
        {lines.map((u) => (
          <li key={u.category_id ?? u.path} className="flex flex-col gap-0.5 py-1.5">
            <span className="flex items-baseline justify-between gap-3">
              <span className="min-w-0 text-fg">{u.path}</span>
              <Money cents={u.amount} currency={currency} className="shrink-0" />
            </span>
            {u.category_id && (reason === 'unmapped' || reason === 'no_rate') ? (
              <span className="flex flex-wrap gap-x-3 text-[11.5px]">
                <button
                  type="button"
                  className="text-accent underline-offset-2 hover:underline"
                  aria-label={t('impostos-escolher-grupo-de', { categoria: u.path })}
                  onClick={() => editor.open('group', u.category_id!, u.path)}
                >
                  {t('impostos-escolher-grupo')}
                </button>
                {reason === 'no_rate' ? (
                  <button
                    type="button"
                    className="text-accent underline-offset-2 hover:underline"
                    aria-label={t('impostos-aliquota-propria-de', { categoria: u.path })}
                    onClick={() => editor.open('rate', u.category_id!, u.path)}
                  >
                    {t('impostos-aliquota-propria')}
                  </button>
                ) : null}
              </span>
            ) : null}
            {reason === 'uncategorized' ? (
              <Link
                to={uncategorizedLink}
                className="text-[11.5px] text-accent underline-offset-2 hover:underline"
              >
                {t('impostos-categorizar')}
              </Link>
            ) : null}
          </li>
        ))}
      </ul>
    </section>
  );
}
