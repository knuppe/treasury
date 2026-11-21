/**
 * Painel simples (níveis Iniciante e Intermediário, docs/profile.md): quanto entrou, quanto
 * saiu e o que sobrou no mês, em frases do dia a dia; a meta de reserva com barra de
 * progresso; para onde foi o dinheiro; e a dica do dia. Nada de patrimônio, carteira ou
 * percentuais avançados: isso fica em "Ver painel completo".
 */
import { Link, useNavigate } from 'react-router';

import type { Dashboard } from '@/api/generated/Dashboard';
import { useEmergencyReserve } from '@/api/hooks/goals';
import { useProfile } from '@/api/hooks/profile';
import { tipOfTheDay } from '@/content/tips';
import { Button, Icon, InstrumentCard, Money, Private } from '@/design/components';
import { cx } from '@/design/cx';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { formatMoney, formatMonthLong, minorUnits, todayIso } from '@/lib/format';

import { Explain } from './Explain';

const RESERVE_MONTHS = 3;

/** Valor redondo sem os centavos zerados (`R$ 1.000`, `$1,000`, `1000 €`). */
function wholeMoney(cents: number, currency: string): string {
  const text = formatMoney(cents, currency);
  const decimals = minorUnits(currency);
  return decimals > 0 ? text.replace(new RegExp(`[.,]0{${decimals}}(?=\\D*$)`), '') : text;
}

function Figure({
  label,
  cents,
  currency,
  tone,
  hint,
}: {
  label: string;
  cents: number;
  currency: string;
  tone: 'positive' | 'negative' | 'auto';
  hint?: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3.5">
      <span className="text-[13px] text-dim">{label}</span>
      <Money cents={cents} currency={currency} tone={tone} className="text-[26px] leading-tight" />
      {hint ? <div className="text-[12px] text-dim">{hint}</div> : null}
    </div>
  );
}

/** Uma frase sobre o mês, sem julgamento. */
export function monthSentence(d: Pick<Dashboard, 'flow' | 'currency'>): string {
  const { income, expense } = d.flow;
  const net = income - expense;
  const fmt = (c: number) => formatMoney(c, d.currency);
  if (income === 0 && expense === 0) return t('perfil-mes-vazio');
  if (income === 0) return t('perfil-mes-sem-entrada', { saida: fmt(expense) });
  if (net >= 0)
    return t('perfil-mes-sobrou', { sobra: fmt(net), pct: Math.round((net / income) * 100) });
  return t('perfil-mes-faltou', { falta: fmt(-net) });
}

/** As três figuras do mês com a frase (também usadas no topo do painel intermediário). */
export function MonthSummary({ dashboard }: { dashboard: Dashboard }) {
  const d = dashboard;
  return (
    <section aria-label={t('perfil-resumo-mes')} className="flex flex-col gap-3">
      <div className="grid grid-cols-1 gap-3 @min-[720px]:grid-cols-3">
        <Figure
          label={t('perfil-entrou')}
          cents={d.flow.income}
          currency={d.currency}
          tone="positive"
        />
        <Figure
          label={t('perfil-saiu')}
          cents={d.flow.expense}
          currency={d.currency}
          tone="negative"
        />
        <Figure
          label={t('perfil-sobrou')}
          cents={d.flow.income - d.flow.expense}
          currency={d.currency}
          tone="auto"
          hint={<Explain id="sobra" />}
        />
      </div>
      <p className="text-[14px] text-fg">
        <Private>{monthSentence(d)}</Private>
      </p>
    </section>
  );
}

function ReserveCard() {
  const reserve = useEmergencyReserve({ target_months: RESERVE_MONTHS, months_back: 6 });
  const r = reserve.data;
  const progress = r && r.target_amount > 0 ? Math.min(1, r.current_amount / r.target_amount) : 0;
  return (
    <InstrumentCard index="★" title={t('perfil-reserva')}>
      <div className="flex flex-col gap-3 text-[13px]">
        {!r ? (
          <p className="text-dim">{t('perfil-calculando')}</p>
        ) : r.avg_essential_monthly === 0 ? (
          <p>
            {tRich(
              'perfil-reserva-comecar',
              { valor: wholeMoney(100_000, r.currency) },
              (content) => (
                <strong className="font-medium">{content}</strong>
              ),
            )}
          </p>
        ) : (
          <>
            <p>
              {tRich('perfil-reserva-meta', {
                meta: <Money cents={r.target_amount} currency={r.currency} />,
                meses: RESERVE_MONTHS,
              })}{' '}
              {r.current_amount > 0
                ? tRich('perfil-reserva-ja-tem', {
                    valor: <Money cents={r.current_amount} currency={r.currency} />,
                  })
                : t('perfil-reserva-nada')}
            </p>
            <div
              role="progressbar"
              aria-label={t('perfil-reserva-progresso')}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={Math.round(progress * 100)}
              className="h-3 w-full overflow-hidden rounded-full bg-surface-2"
            >
              <div
                className={cx('h-full rounded-full', progress >= 1 ? 'bg-positive' : 'bg-accent')}
                style={{ width: `${Math.max(progress * 100, 2)}%` }}
              />
            </div>
            <p className="text-dim">
              {progress >= 1
                ? t('perfil-reserva-completa')
                : t('perfil-reserva-caminho', { pct: Math.round(progress * 100) })}
            </p>
          </>
        )}
        {r && !r.goal_id ? (
          <p className="text-dim">
            {tRich('perfil-reserva-criar-frase', {
              link: (
                <Link to="/orcamento" className="text-accent underline">
                  {t('perfil-reserva-criar')}
                </Link>
              ),
            })}
          </p>
        ) : null}
        <Explain id="reserva" />
      </div>
    </InstrumentCard>
  );
}

function WhereCard({ dashboard }: { dashboard: Dashboard }) {
  const d = dashboard;
  const top = d.top_categories.filter((c) => c.amount > 0).slice(0, 3);
  return (
    <InstrumentCard index="→" title={t('perfil-onde-foi')}>
      {top.length ? (
        <ul className="flex flex-col gap-2 text-[13px]">
          {top.map((c) => (
            <li key={c.key ?? c.label} className="flex items-baseline justify-between gap-3">
              <span>{c.label}</span>
              <span className="text-dim">
                {tRich('perfil-onde-item', {
                  valor: <Money cents={c.amount} currency={d.currency} />,
                  pct: Math.round(c.share * 100),
                })}
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-[13px] text-dim">{t('perfil-onde-vazio')}</p>
      )}
      {d.uncategorized_count > 0 ? (
        <p className="mt-3 text-[12.5px] text-dim">
          {t('perfil-sem-categoria', { n: d.uncategorized_count })}{' '}
          <Link to="/lancamentos" className="text-accent underline">
            {t('perfil-escolher-categoria')}
          </Link>
        </p>
      ) : null}
    </InstrumentCard>
  );
}

/** Painel do nível Iniciante. */
export function SimpleDashboard({
  dashboard,
  onShowFull,
}: {
  dashboard: Dashboard;
  onShowFull: () => void;
}) {
  const navigate = useNavigate();
  const profile = useProfile().data;
  const tip = profile ? tipOfTheDay(profile, todayIso()) : null;
  return (
    <div className="flex flex-col gap-5">
      <p className="text-[15px] text-fg">
        {t('perfil-seu-mes', { mes: formatMonthLong(dashboard.month) })}
      </p>
      <MonthSummary dashboard={dashboard} />
      <div className="flex flex-wrap gap-2">
        <Button
          variant="primary"
          icon={<Icon name="minus" size={14} />}
          onClick={() => void navigate('/lancamentos?novo=despesa')}
        >
          {t('perfil-anotar-gasto')}
        </Button>
        <Button
          variant="secondary"
          icon={<Icon name="plus" size={14} />}
          onClick={() => void navigate('/lancamentos?novo=receita')}
        >
          {t('perfil-anotar-entrada')}
        </Button>
      </div>
      <div className="grid grid-cols-1 gap-4 @min-[1040px]:grid-cols-2">
        <ReserveCard />
        <WhereCard dashboard={dashboard} />
      </div>
      {tip ? (
        <aside
          aria-label={t('perfil-dica-do-dia')}
          className="flex gap-3 rounded-card border border-accent/40 bg-accent-soft/40 px-4 py-3 text-[13.5px]"
        >
          <Icon name="info" className="mt-0.5 shrink-0 text-accent" />
          <p>
            <span className="label-caps mr-2">{t('perfil-dica-do-dia')}</span>
            {tip}
          </p>
        </aside>
      ) : null}
      <div>
        <Button variant="ghost" size="sm" onClick={onShowFull}>
          {t('perfil-ver-completo')}
        </Button>
      </div>
    </div>
  );
}
