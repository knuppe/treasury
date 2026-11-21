/**
 * Alocação por setor (docs/agent-enrichment.md, fase 2): o valor de mercado das posições
 * agrupado pelo setor do cadastro de cada ativo (o que o usuário digitou ou uma sugestão
 * aprovada em Saúde dos ativos › Dados dos ativos). Ativos sem setor ficam numa linha à
 * parte, com o atalho para completar.
 */
import { Link } from 'react-router';

import type { AssetClass } from '@/api/generated/AssetClass';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import { useAssets } from '@/api/hooks/assets';
import { InstrumentCard, Money } from '@/design/components';
import { t } from '@/i18n';

import { Pct } from './components/Pct';

interface Slice {
  key: string;
  label: string;
  value: number;
  count: number;
  /** Classe sem setor por natureza (renda fixa, Tesouro, fundos, cripto…). */
  byClass: boolean;
}

/** Classes que têm setor (as mesmas do catálogo do núcleo, `assets::enrich::FIELDS`). */
const WITH_SECTOR: ReadonlySet<AssetClass> = new Set([
  'stock_br',
  'bdr',
  'stock_us',
  'reit',
  'stock_intl',
]);

export function sectorSlices(
  summary: PortfolioSummary,
  sectorOf: (assetId: string) => string | null,
): Slice[] {
  const map = new Map<string, Slice>();
  for (const p of summary.positions) {
    const v = p.market_value_base ?? 0;
    if (v <= 0) continue;
    const sector = sectorOf(p.asset.id)?.trim() || null;
    const byClass = !sector && !WITH_SECTOR.has(p.asset.class);
    const key = sector ? sector.toLocaleLowerCase() : byClass ? `class:${p.asset.class}` : '';
    const cur = map.get(key) ?? {
      key,
      label: sector ?? (byClass ? p.asset.class_label : t('carteira-setor-sem')),
      value: 0,
      count: 0,
      byClass,
    };
    cur.value += v;
    cur.count += 1;
    map.set(key, cur);
  }
  // Setores primeiro, depois as classes sem setor, e por último os que faltam completar.
  const rank = (x: Slice) => (x.key === '' ? 2 : x.byClass ? 1 : 0);
  return [...map.values()].sort((a, b) => rank(a) - rank(b) || b.value - a.value);
}

export function SectorAllocation({ summary, index }: { summary: PortfolioSummary; index: string }) {
  const assets = useAssets({ include_archived: true });
  const sectors = new Map((assets.data ?? []).map((a) => [a.id, a.sector]));
  const slices = sectorSlices(summary, (id) => sectors.get(id) ?? null);
  const total = slices.reduce((s, x) => s + x.value, 0);
  const missing = slices.find((s) => s.key === '');
  return (
    <InstrumentCard
      index={index}
      title={t('carteira-setor-titulo')}
      footer={<span className="text-[12px] text-dim">{t('carteira-setor-rodape')}</span>}
    >
      {slices.length === 0 ? (
        <p className="text-[12.5px] text-dim">{t('carteira-setor-vazio')}</p>
      ) : (
        <ul className="flex flex-col gap-2" aria-label={t('carteira-setor-titulo')}>
          {slices.map((s) => {
            const w = total > 0 ? s.value / total : 0;
            return (
              <li key={s.key || '-'} className="flex flex-col gap-1">
                <span className="flex items-baseline gap-2 text-[13px]">
                  <span className={s.key && !s.byClass ? 'text-fg' : 'text-dim italic'}>
                    {s.label}
                  </span>
                  <span className="text-[11px] text-dim">
                    {t('carteira-setor-ativos', { n: s.count })}
                  </span>
                  <span className="grow" />
                  <Money cents={s.value} currency={summary.base_currency} />
                  <span className="num w-14 text-right">
                    <Pct value={w} />
                  </span>
                </span>
                <span className="h-1.5 w-full rounded-full bg-line/50">
                  <span
                    className={
                      s.key && !s.byClass
                        ? 'block h-full rounded-full bg-accent'
                        : 'block h-full rounded-full bg-dim/60'
                    }
                    style={{ width: `${Math.max(1, w * 100)}%` }}
                  />
                </span>
              </li>
            );
          })}
        </ul>
      )}
      {missing ? (
        <Link
          to="/carteira?aba=saude"
          className="mt-3 inline-block text-[12px] text-accent underline-offset-2 hover:underline"
        >
          {t('carteira-setor-completar', { n: missing.count })}
        </Link>
      ) : null}
    </InstrumentCard>
  );
}
