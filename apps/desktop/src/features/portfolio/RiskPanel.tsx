/**
 * Risco da carteira (`portfolio.risk`): volatilidade, pior queda, Sharpe contra o CDI e
 * correlação, para a carteira de hoje no passado (pesos de hoje) e para o histórico real,
 * por classe e por ativo, com a parte de cada um no risco total.
 */
import { useCountryInfo } from '@/api/hooks/country';

import type { CorrelationMatrix } from '@/api/generated/CorrelationMatrix';
import type { RiskMetrics } from '@/api/generated/RiskMetrics';
import { useRisk } from '@/api/hooks/portfolio';
import { InstrumentCard } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatMonthShort, formatNumber } from '@/lib/format';

import { PeriodSelect, periodRange, usePeriod } from './components/PeriodSelect';
import { ErrorBlock, LoadingBlock, Notice } from './components/States';
import { classLabel } from './labels';

const pct = (v: number | null | undefined, digits = 1) =>
  v == null ? '—' : `${formatNumber(v * 100, digits)}%`;

const num = (v: number | null | undefined) => (v == null ? '—' : formatNumber(v, 2));

const month = (m: string | null | undefined) => {
  if (!m) return '—';
  try {
    return formatMonthShort(m);
  } catch {
    return m;
  }
};

function drawdown(m: RiskMetrics) {
  if (!m.drawdown_trough) return t('carteira-risco-sem-queda');
  const queda =
    m.drawdown_peak === 'início'
      ? t('carteira-risco-queda-inicio', {
          pct: pct(m.max_drawdown),
          fundo: month(m.drawdown_trough),
        })
      : t('carteira-risco-queda', {
          pct: pct(m.max_drawdown),
          pico: month(m.drawdown_peak),
          fundo: month(m.drawdown_trough),
        });
  return m.drawdown_recovered
    ? t('carteira-risco-recuperou', { queda, mes: month(m.drawdown_recovered) })
    : t('carteira-risco-nao-recuperou', { queda });
}

function Summary({ title, m, rate }: { title: string; m: RiskMetrics | null; rate: string }) {
  return (
    <div className="flex flex-col gap-1.5 rounded-card border border-line p-3">
      <span className="label-caps text-dim">{title}</span>
      {m ? (
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-[13px]">
          <dt className="text-dim">{t('carteira-risco-volatilidade')}</dt>
          <dd className="num">{t('carteira-pct-aa', { pct: pct(m.volatility) })}</dd>
          <dt className="text-dim">{t('carteira-risco-pior-queda')}</dt>
          <dd className={cx('num', m.max_drawdown < -0.1 && 'text-negative')}>{drawdown(m)}</dd>
          <dt className="text-dim">{t('carteira-risco-retorno')}</dt>
          <dd className="num">
            {t('carteira-pct-aa', { pct: pct(m.annual_return) })}
            {m.cdi_annual != null ? ` · ${rate} ${pct(m.cdi_annual)}` : ''}
          </dd>
          <dt className="text-dim">{t('carteira-risco-sharpe-de', { indice: rate })}</dt>
          <dd className={cx('num', m.sharpe != null && m.sharpe < 0 && 'text-negative')}>
            {num(m.sharpe)}
          </dd>
          <dt className="text-dim">{t('carteira-risco-melhor-pior')}</dt>
          <dd className="num">
            {pct(m.best_month)} · {pct(m.worst_month)}
          </dd>
        </dl>
      ) : (
        <p className="text-[13px] text-dim">{t('carteira-risco-insuficiente')}</p>
      )}
    </div>
  );
}

interface Row {
  key: string;
  label: string;
  hint?: string;
  weight: number;
  metrics: RiskMetrics | null;
  risk_share: number | null;
}

function RiskTable({ rows, label, first }: { rows: Row[]; label: string; first: string }) {
  return (
    <table className="w-full border-collapse text-[12.5px]" aria-label={label}>
      <thead>
        <tr className="label-caps text-left text-dim">
          <th className="pb-1 font-normal">{first}</th>
          <th className="pb-1 text-right font-normal">{t('carteira-col-peso')}</th>
          <th className="pb-1 text-right font-normal">{t('carteira-risco-retorno-aa')}</th>
          <th className="pb-1 text-right font-normal">{t('carteira-risco-volatilidade')}</th>
          <th className="pb-1 text-right font-normal">{t('carteira-risco-pior-queda')}</th>
          <th className="pb-1 text-right font-normal">{t('carteira-risco-sharpe')}</th>
          <th className="pb-1 text-right font-normal">{t('carteira-risco-parte')}</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((r) => (
          <tr key={r.key} className="border-t border-line">
            <td className="py-1" title={r.hint}>
              {r.label}
            </td>
            <td className="num py-1 text-right">{pct(r.weight)}</td>
            <td className="num py-1 text-right">{pct(r.metrics?.annual_return)}</td>
            <td className="num py-1 text-right">{pct(r.metrics?.volatility)}</td>
            <td className="num py-1 text-right">{pct(r.metrics?.max_drawdown)}</td>
            <td className="num py-1 text-right">{num(r.metrics?.sharpe)}</td>
            <td
              className={cx(
                'num py-1 text-right',
                r.risk_share != null && r.risk_share > r.weight * 1.5 && 'text-warning',
              )}
            >
              {pct(r.risk_share)}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** Correlação: vermelho = andam juntos (concentra o risco), verde = um compensa o outro. */
function Heatmap({ m, label }: { m: CorrelationMatrix; label: string }) {
  if (m.labels.length < 2) return null;
  // Rótulos das classes no idioma do app (os ids são as classes); os ativos ficam como vêm.
  const labels = m.labels.map((l, i) => classLabel(m.ids[i] ?? '', l));
  const bg = (v: number | null) => {
    if (v == null) return undefined;
    const tone = v >= 0 ? 'var(--negative)' : 'var(--positive)';
    return `color-mix(in srgb, ${tone} ${Math.round(Math.abs(v) * 45)}%, transparent)`;
  };
  return (
    <div className="overflow-x-auto">
      <table className="border-collapse text-[11.5px]" aria-label={label}>
        <thead>
          <tr>
            <th />
            {labels.map((l) => (
              <th key={l} className="px-1 pb-1 text-center font-normal text-dim">
                {l}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {labels.map((row, i) => (
            <tr key={row}>
              <th className="pr-2 text-left font-normal text-dim">{row}</th>
              {m.values[i]!.map((v, j) => (
                <td
                  key={m.ids[j]}
                  className="num min-w-12 border border-line/40 px-1 py-0.5 text-center"
                  style={{ background: i === j ? undefined : bg(v) }}
                  aria-label={`${row} × ${labels[j]}: ${v == null ? t('carteira-risco-sem-dados') : num(v)}`}
                >
                  {i === j ? '—' : v == null ? '·' : num(v)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function RiskPanel() {
  const [period, setPeriod] = usePeriod('risco', '36m');
  const q = useRisk({ months: periodRange(period).months ?? 36 });
  const r = q.data;
  // No Brasil, o CDI; fora, o juro de referência do país (docs/countries.md).
  const info = useCountryInfo();
  const rate = info && info.code !== 'BR' ? info.rate_label : 'CDI';
  return (
    <InstrumentCard index="07.10" title={t('carteira-risco-titulo')}>
      <div className="flex flex-col gap-4">
        <div className="flex flex-wrap items-end gap-3">
          <p className="max-w-[80ch] flex-1 text-[13px] text-dim">
            {t('carteira-risco-texto', { indice: rate })}
          </p>
          {/* O risco olha para trás a partir de hoje, de 12 a 120 meses. */}
          <PeriodSelect
            scope="risco"
            value={period}
            onChange={setPeriod}
            only={['12m', '24m', '36m', '60m']}
          />
        </div>
        {q.isPending ? (
          <LoadingBlock label={t('carteira-risco-calculando')} />
        ) : q.isError ? (
          <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
        ) : !r || r.assets.length === 0 ? (
          <Notice tone="info" title={t('carteira-risco-sem-ativos')}>
            {t('carteira-risco-sem-ativos-texto')}
          </Notice>
        ) : (
          <>
            <div className="grid gap-3 md:grid-cols-2">
              <Summary rate={rate} title={t('carteira-risco-hoje')} m={r.portfolio} />
              <Summary rate={rate} title={t('carteira-risco-historico')} m={r.realized} />
            </div>
            <RiskTable
              label={t('carteira-risco-por-classe')}
              first={t('carteira-col-classe')}
              rows={r.by_class.map((c) => ({
                key: c.class,
                label: classLabel(c.class, c.label),
                weight: c.weight,
                metrics: c.metrics,
                risk_share: c.risk_share,
              }))}
            />
            <Heatmap m={r.class_correlation} label={t('carteira-risco-correlacao-classes')} />
            <RiskTable
              label={t('carteira-risco-por-ativo')}
              first={t('carteira-col-ativo')}
              rows={r.assets.map((a) => ({
                key: a.asset_id,
                label: a.ticker ?? a.name,
                hint: a.name,
                weight: a.weight,
                metrics: a.metrics,
                risk_share: a.risk_share,
              }))}
            />
            <Heatmap m={r.asset_correlation} label={t('carteira-risco-correlacao-ativos')} />
            <ul className="flex flex-col gap-0.5 text-[12px] text-dim">
              <li>{t('carteira-risco-parte-texto')}</li>
              {r.notes.map((n) => (
                <li key={n}>{n}</li>
              ))}
            </ul>
          </>
        )}
      </div>
    </InstrumentCard>
  );
}
