/** Ajustes › Saúde financeira: o modelo da nota (I-SFB, indicadores clássicos ou desligado). */
import type { HealthModel } from '@/api/generated/HealthModel';
import { useHealthSettings, useSetHealthModel } from '@/api/hooks/health';
import { InstrumentCard, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { tRich } from '@/features/settings/richText';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

import { SOURCE_URL } from './model';

export function HealthSettingsSection() {
  const settings = useHealthSettings();
  const save = useSetHealthModel();
  const choose = (model: HealthModel, label: string) =>
    save.mutate(
      { model },
      {
        onSuccess: () =>
          toast({
            tone: 'success',
            title:
              model === 'off'
                ? t('saude-desativada-aviso')
                : t('saude-modelo-aviso', { modelo: label }),
          }),
      },
    );
  return (
    <div className="grid gap-5 @min-[1040px]:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      <InstrumentCard index="12.S" title={t('saude-modelo-titulo')}>
        <QueryState
          loading={settings.isPending}
          error={settings.error}
          label={t('controles-carregando')}
        >
          {settings.data ? (
            <div className="flex flex-col gap-3">
              <div
                role="radiogroup"
                aria-label={t('saude-modelo-titulo')}
                className="flex flex-col gap-2"
              >
                {settings.data.models.map((m) => {
                  const on = settings.data.model === m.id;
                  return (
                    <button
                      key={m.id}
                      type="button"
                      role="radio"
                      aria-checked={on}
                      disabled={save.isPending}
                      onClick={() => !on && choose(m.id, m.label)}
                      className={cx(
                        'flex w-full items-start gap-3 rounded-control border px-3.5 py-3 text-left transition-colors focus-visible:outline-2 focus-visible:outline-accent',
                        on ? 'border-accent bg-accent-soft' : 'border-line hover:bg-surface-2',
                      )}
                    >
                      <span
                        aria-hidden
                        className={cx(
                          'mt-0.5 flex size-3.5 shrink-0 items-center justify-center rounded-full border',
                          on ? 'border-accent' : 'border-line-strong',
                        )}
                      >
                        {on ? <span className="size-1.5 rounded-full bg-accent" /> : null}
                      </span>
                      <span className="flex min-w-0 flex-col gap-0.5">
                        <span className={cx('text-[13.5px]', on ? 'text-accent' : 'text-fg')}>
                          {m.label}
                        </span>
                        <span className="text-[12px] leading-snug text-dim">{m.description}</span>
                      </span>
                    </button>
                  );
                })}
              </div>
              <InlineError error={save.error} />
            </div>
          ) : null}
        </QueryState>
      </InstrumentCard>
      <InstrumentCard index="12.S2" title={t('saude-como-calculada')}>
        <div className="flex flex-col gap-3 text-[12.5px] leading-snug text-dim">
          <p>{t('saude-calculo-dados')}</p>
          <p>
            {tRich('saude-calculo-sem-dados', {}, (c) => (
              <em>{c}</em>
            ))}
          </p>
          <dl className="flex flex-col gap-2 border-t border-line/70 pt-3">
            {[
              ['I-SFB', t('saude-isf-resumo')],
              [t('saude-classicos'), t('saude-classicos-resumo')],
            ].map(([k, v]) => (
              <div key={k} className="flex flex-col gap-0.5">
                <dt className="label-caps">{k}</dt>
                <dd className="text-fg">{v}</dd>
              </div>
            ))}
          </dl>
          <a
            href={SOURCE_URL}
            target="_blank"
            rel="noreferrer"
            className="text-accent hover:underline"
          >
            {t('saude-fonte-isf')}
          </a>
        </div>
      </InstrumentCard>
    </div>
  );
}
