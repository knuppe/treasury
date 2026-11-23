/**
 * "Para resolver" (`attention.list`, docs/to-resolve.md): a lista única do que precisa da
 * pessoa, no topo do Painel em todos os níveis. Junta vencimentos, contas sem extrato, tarefas,
 * alertas e as percepções de atenção, do mais grave para o menos grave: os três primeiros e
 * "Ver os N". O sino da barra superior abre o Painel com `?resolver=1`, que rola até aqui e
 * mostra a lista inteira.
 */
import { useEffect, useRef, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';

import type { AttentionItem } from '@/api/generated/AttentionItem';
import { useAttention, useDismissAttention } from '@/api/hooks/attention';
import { Button, Icon, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { alertHref } from '@/features/agent/AlertsPanel';
import { QuietRecurrenceButton } from '@/features/attention/QuietRecurrenceButton';
import { t } from '@/i18n';

const DOT: Record<AttentionItem['severity'], string> = {
  critical: 'bg-negative',
  warning: 'bg-warning',
  info: 'bg-accent',
};

/** Onde resolver o item: o destino do alerta ou a rota da tarefa. */
export function attentionHref(i: AttentionItem): string | null {
  if (i.target) return alertHref(i.target);
  return i.route ?? null;
}

/**
 * O corpo em texto corrido e curto: os relatórios dos alertas vêm em Markdown e podem ser
 * longos; o texto completo fica em Agente › Alertas.
 */
export function attentionExcerpt(body: string, max = 260): string {
  const plain = body
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/[*_`|]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
  if (plain.length <= max) return plain;
  const cut = plain.slice(0, max);
  return `${cut.slice(0, Math.max(cut.lastIndexOf(' '), max - 40))}…`;
}

function Item({ i }: { i: AttentionItem }) {
  const navigate = useNavigate();
  const dismiss = useDismissAttention();
  const href = attentionHref(i);
  const body = attentionExcerpt(i.body);
  return (
    <li className="flex gap-3 border-b border-line/60 py-2.5 last:border-b-0">
      <span aria-hidden className={cx('mt-1.5 size-2 shrink-0 rounded-full', DOT[i.severity])} />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <span className="text-[13.5px] font-medium text-fg">
          <span className="sr-only">{t(`atencao-gravidade-${i.severity}`)}: </span>
          {i.title}
        </span>
        {body ? <span className="text-[12.5px] text-dim">{body}</span> : null}
        {i.action ? <span className="text-[12.5px] text-fg">{i.action}</span> : null}
        <div className="flex flex-wrap gap-2 pt-0.5">
          {href ? (
            <Button size="sm" variant="ghost" onClick={() => void navigate(href)}>
              {t('painel-abrir')}
            </Button>
          ) : null}
          {i.alert_id ? (
            <Button
              size="sm"
              variant="ghost"
              disabled={dismiss.isPending}
              onClick={() => dismiss.mutate(i.id)}
            >
              {t('atencao-resolver')}
            </Button>
          ) : (
            <Tooltip content={t('atencao-dispensar-dica')}>
              <Button
                size="sm"
                variant="ghost"
                disabled={dismiss.isPending}
                onClick={() => dismiss.mutate(i.id)}
              >
                {t('painel-dispensar')}
              </Button>
            </Tooltip>
          )}
          {i.recurrence_id ? <QuietRecurrenceButton recurrenceId={i.recurrence_id} /> : null}
        </div>
      </div>
    </li>
  );
}

export function AttentionCard({ limit = 3 }: { limit?: number }) {
  const q = useAttention();
  const [params] = useSearchParams();
  const focus = params.get('resolver') === '1';
  // Vindo do sino: a lista inteira, à vista (até a pessoa recolher).
  const [expanded, setExpanded] = useState<boolean | null>(null);
  const all = expanded ?? focus;
  const ref = useRef<HTMLElement>(null);
  const loaded = q.data !== undefined;
  useEffect(() => {
    if (focus && loaded) ref.current?.scrollIntoView?.({ block: 'start' });
  }, [focus, loaded]);
  if (!q.data) return null;
  const items = q.data.items;
  const shown = all ? items : items.slice(0, limit);
  const hasAlerts = items.some((i) => i.alert_id);
  return (
    <section
      ref={ref}
      aria-label={t('atencao-titulo')}
      className="flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3.5"
    >
      <div className="flex items-center gap-2">
        <Icon name="bell" size={14} className="text-accent" />
        <h2 className="text-[13px] text-dim">{t('atencao-titulo')}</h2>
        {items.length ? (
          <span className="num rounded-full bg-accent-soft px-1.5 text-[10.5px] text-accent">
            {items.length}
          </span>
        ) : null}
        <span className="ml-auto text-[11.5px] text-dim">{t('atencao-origem')}</span>
      </div>
      {items.length === 0 ? (
        <p className="flex items-center gap-2 py-1.5 text-[13px] text-fg">
          <Icon name="check" size={14} className="text-positive" />
          {t('atencao-vazio')}
        </p>
      ) : (
        <ul className="flex flex-col">
          {shown.map((i) => (
            <Item key={i.id} i={i} />
          ))}
        </ul>
      )}
      {items.length > limit || hasAlerts ? (
        <div className="flex flex-wrap items-center gap-3">
          {items.length > limit ? (
            <Button size="sm" variant="ghost" onClick={() => setExpanded(!all)}>
              {all ? t('painel-mostrar-menos') : t('atencao-ver-todos', { n: items.length })}
            </Button>
          ) : null}
          {hasAlerts ? (
            <Link
              to="/agente?aba=alertas"
              className="text-[12px] text-dim underline-offset-2 hover:text-accent hover:underline"
            >
              {t('atencao-ver-alertas')}
            </Link>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}
