/**
 * "O que o Treasury percebeu" (`reports.insights`, docs/insights.md): percepções calculadas
 * pelo app, sem IA. Aqui ficam as informativas e as notícias boas (inflação pessoal, gasto
 * caindo, divisão da renda); as de atenção vão para "Para resolver" (docs/to-resolve.md).
 * Cada uma com os números, o que fazer e a tela; dispensar esconde neste computador.
 */
import { useState } from 'react';
import { useNavigate } from 'react-router';

import type { Insight } from '@/api/generated/Insight';
import type { PersonalInflation } from '@/api/generated/PersonalInflation';
import { useInsights } from '@/api/hooks/insights';
import { Button, Icon } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';

const KEY = 'treasury.insights.dismissed';

function loadDismissed(): string[] {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? '[]') as string[];
  } catch {
    return [];
  }
}

function saveDismissed(ids: string[]) {
  try {
    localStorage.setItem(KEY, JSON.stringify(ids.slice(-200)));
  } catch {
    // Sem armazenamento: só nesta sessão.
  }
}

const pct = (v: number) => formatPercent(v);

const TONE: Record<Insight['tone'], string> = {
  warning: 'bg-warning',
  info: 'bg-accent',
  good: 'bg-positive',
};

function InflationDetail({ p }: { p: PersonalInflation }) {
  return (
    <table
      className="mt-1 w-full border-collapse text-[12px]"
      aria-label={t('painel-inflacao-grupos')}
    >
      <thead>
        <tr className="label-caps text-left text-dim">
          <th className="pb-1 font-normal">{t('painel-inflacao-grupo')}</th>
          <th className="pb-1 text-right font-normal">{t('painel-inflacao-peso')}</th>
          <th className="pb-1 text-right font-normal">{t('painel-inflacao-subiu')}</th>
        </tr>
      </thead>
      <tbody>
        {p.groups.map((g) => (
          <tr key={g.id} className="border-t border-line/60">
            <td className="py-0.5">{g.name}</td>
            <td className="num py-0.5 text-right">{pct(g.weight)}</td>
            <td className="num py-0.5 text-right">{pct(g.change)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function Item({ i, onDismiss }: { i: Insight; onDismiss: () => void }) {
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const inflation =
    i.kind === 'personal_inflation' ? (i.data as unknown as PersonalInflation) : null;
  return (
    <li className="flex gap-3 border-b border-line/60 py-2.5 last:border-b-0">
      <span aria-hidden className={cx('mt-1.5 size-2 shrink-0 rounded-full', TONE[i.tone])} />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <span className="text-[13.5px] font-medium text-fg">{i.title}</span>
        <span className="text-[12.5px] text-dim">{i.body}</span>
        {i.action ? <span className="text-[12.5px] text-fg">{i.action}</span> : null}
        {inflation && open ? <InflationDetail p={inflation} /> : null}
        <div className="flex flex-wrap gap-2 pt-0.5">
          {inflation ? (
            <Button size="sm" variant="ghost" onClick={() => setOpen((o) => !o)}>
              {open ? t('painel-esconder-grupos') : t('painel-ver-grupos')}
            </Button>
          ) : null}
          {i.screen ? (
            <Button size="sm" variant="ghost" onClick={() => void navigate(`/${i.screen}`)}>
              {t('painel-abrir')}
            </Button>
          ) : null}
          <Button size="sm" variant="ghost" onClick={onDismiss}>
            {t('painel-dispensar')}
          </Button>
        </div>
      </div>
    </li>
  );
}

export function InsightsCard({ limit = 4 }: { limit?: number }) {
  const q = useInsights();
  const [dismissed, setDismissed] = useState<string[]>(loadDismissed);
  const [all, setAll] = useState(false);
  // As percepções de atenção moram em "Para resolver" (features/attention); aqui ficam as
  // informativas, as notícias boas e a inflação pessoal, que tem a tabela dos grupos.
  const items = (q.data?.items ?? [])
    .filter((i) => i.tone !== 'warning' || i.kind === 'personal_inflation')
    .filter((i) => !dismissed.includes(i.id));
  if (!items.length) return null;
  const shown = all ? items : items.slice(0, limit);
  return (
    <section
      aria-label={t('painel-percepcoes')}
      className="flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3.5"
    >
      <div className="flex items-center gap-2">
        <Icon name="reticle" size={14} className="text-accent" />
        <h2 className="text-[13px] text-dim">{t('painel-percepcoes')}</h2>
        <span className="ml-auto text-[11.5px] text-dim">{t('painel-percepcoes-sem-ia')}</span>
      </div>
      <ul className="flex flex-col">
        {shown.map((i) => (
          <Item
            key={i.id}
            i={i}
            onDismiss={() => {
              const next = [...dismissed, i.id];
              setDismissed(next);
              saveDismissed(next);
            }}
          />
        ))}
      </ul>
      {items.length > limit ? (
        <div>
          <Button size="sm" variant="ghost" onClick={() => setAll((a) => !a)}>
            {all ? t('painel-mostrar-menos') : t('painel-ver-todas', { n: items.length })}
          </Button>
        </div>
      ) : null}
    </section>
  );
}
