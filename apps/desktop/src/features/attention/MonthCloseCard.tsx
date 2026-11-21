/**
 * "Fechar o mês" (`attention.month_close`, docs/to-resolve.md): cinco passos para o mês que
 * terminou. Três são medidos pelo app (extratos, categorias, pendências) e dois a pessoa marca
 * (saldos conferidos, resumo lido). Some quando o mês é marcado como fechado ou quando não há
 * lançamentos nele.
 */
import { useNavigate } from 'react-router';

import type { CloseStep } from '@/api/generated/CloseStep';
import { useMonthClose, useSetMonthClose } from '@/api/hooks/attention';
import { Button, Icon } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatMonthLong } from '@/lib/format';

function Step({ step, n, month }: { step: CloseStep; n: number; month: string }) {
  const navigate = useNavigate();
  const set = useSetMonthClose();
  const route = step.route;
  return (
    <li className="flex items-start gap-2.5 border-b border-line/60 py-2 last:border-b-0">
      <span
        className={cx(
          'mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border text-[10px]',
          step.done ? 'border-positive bg-positive/15 text-positive' : 'border-line text-dim',
        )}
      >
        {step.done ? <Icon name="check" size={10} /> : <span aria-hidden>{n}</span>}
        <span className="sr-only">
          {step.done ? t('fechamento-passo-feito') : t('fechamento-passo-pendente')}
        </span>
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className={cx('text-[13px]', step.done ? 'text-dim' : 'font-medium text-fg')}>
          {step.title}
        </span>
        {step.detail ? <span className="text-[12px] text-dim">{step.detail}</span> : null}
      </div>
      <div className="flex shrink-0 flex-wrap justify-end gap-1.5">
        {!step.done && route ? (
          <Button size="sm" variant="ghost" onClick={() => void navigate(route)}>
            {t('painel-abrir')}
          </Button>
        ) : null}
        {step.manual ? (
          <Button
            size="sm"
            variant="ghost"
            disabled={set.isPending}
            onClick={() => set.mutate({ month, step: step.key, done: !step.done })}
          >
            {step.done ? t('fechamento-desmarcar') : t('fechamento-marcar')}
          </Button>
        ) : null}
      </div>
    </li>
  );
}

export function MonthCloseCard({ className }: { className?: string }) {
  const { data: c } = useMonthClose();
  const set = useSetMonthClose();
  if (!c || !c.applicable || c.closed_at) return null;
  const total = c.steps.length;
  const ready = c.done === total;
  const title = t('fechamento-cartao', { mes: formatMonthLong(c.month) });
  return (
    <section
      aria-label={title}
      className={cx(
        'flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3.5',
        className,
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <Icon name="check" size={14} className="text-accent" />
        <h2 className="text-[13px] text-dim">{title}</h2>
        <span className="num rounded-full bg-accent-soft px-1.5 text-[10.5px] text-accent">
          {t('fechamento-progresso', { n: c.done, total })}
        </span>
        <span className="ml-auto text-[11.5px] text-dim">{t('fechamento-origem')}</span>
      </div>
      <ol className="flex flex-col">
        {c.steps.map((s, i) => (
          <Step key={s.key} step={s} n={i + 1} month={c.month} />
        ))}
      </ol>
      <div className="flex flex-wrap items-center gap-3 pt-1">
        <Button
          size="sm"
          variant={ready ? 'primary' : 'ghost'}
          disabled={!ready || set.isPending}
          onClick={() => set.mutate({ month: c.month, step: 'closed', done: true })}
        >
          {t('fechamento-fechar')}
        </Button>
        {ready ? null : <span className="text-[12px] text-dim">{t('fechamento-falta')}</span>}
      </div>
    </section>
  );
}
