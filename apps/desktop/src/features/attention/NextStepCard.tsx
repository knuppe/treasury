/**
 * "Seu próximo passo" (`attention.next_step`, docs/to-resolve.md): em qual dos seis degraus
 * a família está e uma coisa para fazer agora, com o porquê. A nota de saúde financeira diz
 * como a família está; este cartão diz o que fazer primeiro.
 */
import { useNavigate } from 'react-router';

import { useNextStep } from '@/api/hooks/attention';
import { Button, Icon, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

export function NextStepCard({ className }: { className?: string }) {
  const navigate = useNavigate();
  const { data: s } = useNextStep();
  if (!s) return null;
  const route = s.route;
  return (
    <section
      aria-label={t('degraus-cartao')}
      className={cx(
        'flex flex-col gap-2 rounded-card border border-line bg-surface px-4 py-3.5',
        className,
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <Icon name="reticle" size={14} className="text-accent" />
        <h2 className="text-[13px] text-dim">{t('degraus-cartao')}</h2>
        <span className="ml-auto text-[11.5px] text-dim">
          {t('degraus-posicao', { n: s.rung, total: s.rungs.length })}
        </span>
      </div>
      <ol
        className="flex gap-1"
        aria-label={t('degraus-posicao', { n: s.rung, total: s.rungs.length })}
      >
        {s.rungs.map((r, i) => {
          const state = r.done ? 'feito' : i + 1 === s.rung ? 'atual' : 'depois';
          return (
            <li key={r.key} className="min-w-0 flex-1">
              <Tooltip content={`${i + 1}. ${r.title} (${t(`degraus-${state}`)})`}>
                <span
                  tabIndex={0}
                  aria-label={`${i + 1}. ${r.title} (${t(`degraus-${state}`)})`}
                  className={cx(
                    'block h-1.5 rounded-full',
                    state === 'feito' ? 'bg-positive' : state === 'atual' ? 'bg-accent' : 'bg-line',
                  )}
                />
              </Tooltip>
            </li>
          );
        })}
      </ol>
      <p className="text-[15px] font-medium text-fg">{s.title}</p>
      <p className="text-[12.5px] text-dim">{s.why}</p>
      <p className="text-[13px] text-fg">{s.action}</p>
      {s.estimated ? <p className="text-[12px] text-warning">{t('degraus-estimado')}</p> : null}
      {route ? (
        <div>
          <Button size="sm" variant="secondary" onClick={() => void navigate(route)}>
            {t('painel-abrir')}
          </Button>
        </div>
      ) : null}
    </section>
  );
}
