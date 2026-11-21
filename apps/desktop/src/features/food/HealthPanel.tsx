/**
 * Aba Saúde de 06 · Consumo (docs/gtin-inflation.md §7): com o Open Food Facts ligado (desligado
 * por padrão; só o código de barras sai, uma vez por produto), a parte do gasto em comida com
 * ultraprocessados agora e há um ano, os produtos "alto em" pelos limites da Anvisa, a proteína
 * por real e os produtos que batem com as alergias e restrições de cada membro (marcadas no cadastro do membro,
 * em 02 · Família). Mostra fatos, sem
 * julgar a comida; o rodapé diz que é indicativo e dá o crédito da fonte (ODbL).
 */
import type { HealthAllergenAlert } from '@/api/generated/HealthAllergenAlert';
import type { HealthProduct } from '@/api/generated/HealthProduct';
import type { HealthSummary } from '@/api/generated/HealthSummary';
import { useCountryInfo } from '@/api/hooks/country';
import { useSetSource } from '@/api/hooks/dataSources';
import { useMemberDiets } from '@/api/hooks/diet';
import { useHealthSummary, useOpenDataSync } from '@/api/hooks/food';
import { Button, InstrumentCard, Measure, Money, toast } from '@/design/components';
import { errorText } from '@/features/transactions/ui/errors';
import { QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatMoney, formatNumber, formatPercent } from '@/lib/format';

import { HighInBadges, OffCredit, restrictionLabel } from './healthLabels';

/** Cobertura mínima para o núcleo mostrar as partes do gasto (`MIN_COVERAGE`). */
const MIN_COVERAGE = 0.3;

const share = (v: number) => formatPercent(v, { decimals: 0 });

/** Busca os dados dos produtos ainda sem consulta e conta o resultado. */
function SyncButton({ pending }: { pending: number }) {
  const sync = useOpenDataSync();
  return (
    <div className="flex flex-col gap-1.5">
      <div>
        <Button
          size="sm"
          variant="secondary"
          loading={sync.isPending}
          onClick={() =>
            sync.mutate(
              {},
              {
                onSuccess: (r) => {
                  const more = [
                    r.failed > 0 ? t('alimentacao-saude-buscou-falhas', { falhas: r.failed }) : '',
                    r.remaining > 0
                      ? t('alimentacao-saude-buscou-faltam', { faltam: r.remaining })
                      : '',
                  ].filter(Boolean);
                  toast({
                    tone: r.failed > 0 ? 'warning' : 'success',
                    title: t('alimentacao-saude-buscou', {
                      encontrados: r.found,
                      consultados: r.fetched,
                    }),
                    ...(more.length ? { description: more.join(' ') } : {}),
                  });
                },
                onError: (e) => toast({ tone: 'error', title: errorText(e) }),
              },
            )
          }
        >
          {pending > 0
            ? t('alimentacao-saude-buscar-n', { n: pending })
            : t('alimentacao-saude-buscar')}
        </Button>
      </div>
      <p className="text-[12px] text-dim">{t('alimentacao-saude-buscando')}</p>
    </div>
  );
}

/** A fonte desligada: o que ela faz, o que sai do computador e o botão de ligar. */
function SourceOff({ onEnabled }: { onEnabled: () => void }) {
  const set = useSetSource();
  return (
    <div className="flex flex-col gap-3 text-[13px]">
      <p className="text-fg">{t('alimentacao-saude-desligada')}</p>
      <p className="text-dim">{t('alimentacao-saude-privacidade')}</p>
      <div>
        <Button
          size="sm"
          variant="primary"
          loading={set.isPending}
          onClick={() =>
            set.mutate(
              { id: 'open_food_facts', enabled: true },
              {
                onSuccess: () => {
                  toast({ tone: 'success', title: t('alimentacao-saude-ligada') });
                  onEnabled();
                },
                onError: (e) => toast({ tone: 'error', title: errorText(e) }),
              },
            )
          }
        >
          {t('alimentacao-saude-ligar')}
        </Button>
      </div>
    </div>
  );
}

/** "31% do seu gasto em comida com dados foi com ultraprocessados (era 38% há um ano)". */
function Headline({ s }: { s: HealthSummary }) {
  if (!s.enough || s.ultra_share == null)
    return (
      <p className="text-[13px] text-dim">
        {t('alimentacao-saude-pouca', { minimo: share(MIN_COVERAGE) })}
      </p>
    );
  return (
    <p className="text-[14px] leading-relaxed text-fg">
      {s.ultra_share_year_ago != null
        ? t('alimentacao-saude-ultra-ano', {
            pct: share(s.ultra_share),
            antes: share(s.ultra_share_year_ago),
          })
        : t('alimentacao-saude-ultra', { pct: share(s.ultra_share) })}
    </p>
  );
}

function Overview({ s, onEnabled }: { s: HealthSummary; onEnabled: () => void }) {
  return (
    <InstrumentCard index="06.15" title={t('alimentacao-saude-titulo')}>
      {!s.source_enabled ? (
        <SourceOff onEnabled={onEnabled} />
      ) : (
        <div className="flex flex-col gap-4">
          <Headline s={s} />
          <div className="grid gap-5 @min-[600px]:grid-cols-3">
            <Measure
              size="md"
              label={t('alimentacao-saude-cobertura-rotulo')}
              value={share(s.coverage)}
              hint={t('alimentacao-saude-cobertura-dica')}
            />
            {s.enough && s.ultra_share != null ? (
              <>
                <Measure
                  size="md"
                  label={t('alimentacao-saude-ultra-rotulo')}
                  value={share(s.ultra_share)}
                  hint={t('alimentacao-saude-ultra-dica')}
                />
                <Measure
                  size="md"
                  label={t('alimentacao-saude-ultra-ano-rotulo')}
                  value={s.ultra_share_year_ago != null ? share(s.ultra_share_year_ago) : '—'}
                  hint={
                    s.ultra_share_year_ago == null
                      ? t('alimentacao-saude-ultra-ano-sem')
                      : undefined
                  }
                />
              </>
            ) : null}
          </div>
          <p className="text-[12.5px] text-dim">
            {t('alimentacao-saude-cobertura', { pct: share(s.coverage) })}
          </p>
          {s.pending > 0 ? <SyncButton pending={s.pending} /> : null}
        </div>
      )}
    </InstrumentCard>
  );
}

function HighIn({ rows, brazil }: { rows: readonly HealthProduct[]; brazil: boolean }) {
  return (
    <InstrumentCard index="06.16" title={t('alimentacao-saude-alto-em')}>
      {rows.length ? (
        <ul
          className="flex flex-col gap-1.5 text-[13px]"
          aria-label={t('alimentacao-saude-alto-em')}
        >
          {rows.map((p) => (
            <li key={p.product_id} className="flex items-center justify-between gap-3">
              <span className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
                <span className="min-w-0 truncate" title={p.name}>
                  {p.name}
                </span>
                <HighInBadges items={p.high_in} />
              </span>
              <Money cents={p.spent} />
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-[12.5px] text-dim">
          {brazil ? t('alimentacao-saude-alto-em-nada') : t('alimentacao-saude-alto-em-fora')}
        </p>
      )}
      <p className="mt-3 text-[12px] text-dim">{t('alimentacao-saude-alto-em-dica')}</p>
    </InstrumentCard>
  );
}

function Protein({ rows }: { rows: readonly HealthProduct[] }) {
  return (
    <InstrumentCard index="06.17" title={t('alimentacao-saude-proteina')}>
      {rows.length ? (
        <ol
          className="flex flex-col gap-1 text-[13px]"
          aria-label={t('alimentacao-saude-proteina')}
        >
          {rows.map((p) => (
            <li key={p.product_id} className="flex items-baseline justify-between gap-3">
              <span className="min-w-0 truncate" title={p.name}>
                {p.name}
              </span>
              <span className="num shrink-0">
                {t('alimentacao-saude-proteina-valor', {
                  g: formatNumber(p.protein_per_money ?? 0, 1),
                })}
              </span>
            </li>
          ))}
        </ol>
      ) : (
        <p className="text-[12.5px] text-dim">{t('alimentacao-saude-proteina-nada')}</p>
      )}
      <p className="mt-3 text-[12px] text-dim">
        {t('alimentacao-saude-proteina-dica', { moeda: formatMoney(100) })}
      </p>
    </InstrumentCard>
  );
}

function Alert({ a }: { a: HealthAllergenAlert }) {
  return (
    <li className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-0.5">
      <span className="min-w-0">
        {t('alimentacao-saude-alerta', {
          membro: a.member_name,
          produto: a.product_name,
          restricao: restrictionLabel(a.restriction),
        })}{' '}
        <span className={a.trace ? 'text-dim' : 'text-warning'}>
          · {a.trace ? t('alimentacao-saude-pode-conter') : t('alimentacao-saude-contem')}
        </span>
      </span>
      <span className="shrink-0 text-[12px] text-dim">{formatDate(a.last_date)}</span>
    </li>
  );
}

function Allergies({ rows }: { rows: readonly HealthAllergenAlert[] }) {
  const diets = useMemberDiets().data ?? [];
  const marked = diets.some((d) => d.restrictions.length > 0);
  return (
    <InstrumentCard index="06.18" title={t('alimentacao-saude-alergias')}>
      <div className="flex flex-col gap-4">
        {rows.length ? (
          <ul
            className="flex flex-col gap-1 text-[13px]"
            aria-label={t('alimentacao-saude-alergias')}
          >
            {rows.map((a) => (
              <Alert key={`${a.member_id}-${a.product_id}-${a.restriction}`} a={a} />
            ))}
          </ul>
        ) : (
          <p className="text-[12.5px] text-dim">
            {marked ? t('alimentacao-saude-alergias-nada') : t('alimentacao-saude-alergias-sem')}
          </p>
        )}
        <p className="text-[12px] text-dim">{t('alimentacao-saude-alergias-dica')}</p>
      </div>
    </InstrumentCard>
  );
}

/** Aba Saúde de 06 · Consumo. */
export function FoodHealthPanel() {
  const q = useHealthSummary();
  const country = useCountryInfo();
  const s = q.data;
  return (
    <div className="flex flex-col gap-4">
      <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
        {s ? (
          <div className="flex flex-col gap-4">
            <Overview s={s} onEnabled={() => void q.refetch()} />
            {s.source_enabled ? (
              <>
                <div className="grid gap-4 @min-[790px]:grid-cols-2">
                  <HighIn rows={s.high_in} brazil={(country?.code ?? 'BR') === 'BR'} />
                  <Protein rows={s.protein} />
                </div>
                <Allergies rows={s.allergens} />
              </>
            ) : null}
          </div>
        ) : null}
      </QueryState>
      <footer className="flex flex-col gap-1 border-t border-line pt-3">
        <p className="text-[12px] text-dim">{t('alimentacao-saude-rodape')}</p>
        <OffCredit />
      </footer>
    </div>
  );
}
