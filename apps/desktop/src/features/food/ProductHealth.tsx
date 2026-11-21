/**
 * No produto (aba Produtos de 06 · Consumo): a saúde pelo Open Food Facts (grupo NOVA, "alto em",
 * nutrientes por 100 g ou 100 ml, alérgenos e traços, Nutri-Score só em texto) e os outros
 * tamanhos da mesma linha, com o preço por kg, L ou unidade, para ver a reduflação
 * (docs/gtin-inflation.md §4 e §7).
 */
import type { FoodLineSibling } from '@/api/generated/FoodLineSibling';
import type { ProductHealth } from '@/api/generated/ProductHealth';
import type { ProductOpenData } from '@/api/generated/ProductOpenData';
import { SectionTitle } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatNumber } from '@/lib/format';

import { UnitPrice } from './units';
import { allergenLabel, HighInBadges, novaLabel, OffCredit } from './healthLabels';

type Nutrient = {
  key: keyof ProductOpenData;
  label: () => string;
  unit: string;
  decimals: number;
};

const NUTRIENTS: Nutrient[] = [
  {
    key: 'energy_kcal',
    label: () => t('alimentacao-nutriente-energia'),
    unit: 'kcal',
    decimals: 0,
  },
  { key: 'sugars', label: () => t('alimentacao-nutriente-acucares'), unit: 'g', decimals: 1 },
  {
    key: 'added_sugars',
    label: () => t('alimentacao-nutriente-acucar-adicionado'),
    unit: 'g',
    decimals: 1,
  },
  {
    key: 'saturated_fat',
    label: () => t('alimentacao-nutriente-gordura-saturada'),
    unit: 'g',
    decimals: 1,
  },
  { key: 'sodium_mg', label: () => t('alimentacao-nutriente-sodio'), unit: 'mg', decimals: 0 },
  { key: 'protein', label: () => t('alimentacao-nutriente-proteina'), unit: 'g', decimals: 1 },
  { key: 'fiber', label: () => t('alimentacao-nutriente-fibra'), unit: 'g', decimals: 1 },
];

const per = (p: string | null) => (p === '100ml' ? '100 ml' : '100 g');

export function ProductHealthSection({ health }: { health: ProductHealth }) {
  const d = health.data;
  const rows = NUTRIENTS.flatMap((n) => {
    const v = d[n.key];
    return typeof v === 'number' ? [{ ...n, value: v }] : [];
  });
  return (
    <section aria-label={t('alimentacao-produto-saude')} className="flex flex-col gap-3">
      <SectionTitle>{t('alimentacao-produto-saude')}</SectionTitle>
      {!d.found ? (
        <p className="text-[12.5px] text-dim">{t('alimentacao-produto-sem-dados')}</p>
      ) : (
        <div className="grid gap-4 text-[13px] @min-[700px]:grid-cols-2">
          <div className="flex flex-col gap-3">
            {d.nova != null ? (
              <div className="flex flex-col gap-1">
                <span className="label-caps text-dim">{t('alimentacao-produto-nova')}</span>
                <span className="text-fg">{novaLabel(d.nova)}</span>
                <span className="text-[12px] text-dim">{t('alimentacao-nova-explica')}</span>
              </div>
            ) : null}
            {health.high_in.length ? <HighInBadges items={health.high_in} /> : null}
            {d.allergens.length ? (
              <p>
                <span className="text-dim">{t('alimentacao-produto-alergenos')}: </span>
                {d.allergens.map(allergenLabel).join(', ')}
              </p>
            ) : null}
            {d.traces.length ? (
              <p>
                <span className="text-dim">{t('alimentacao-produto-tracos')}: </span>
                {d.traces.map(allergenLabel).join(', ')}
              </p>
            ) : null}
            {d.nutriscore ? (
              <p className="text-[12.5px] text-dim">
                {t('alimentacao-produto-nutriscore', { nota: d.nutriscore.toUpperCase() })}
              </p>
            ) : null}
          </div>
          {rows.length ? (
            <div className="flex flex-col gap-1">
              <span className="label-caps text-dim">
                {t('alimentacao-produto-nutrientes', { base: per(d.per) })}
              </span>
              <dl className="flex flex-col gap-0.5">
                {rows.map((n) => (
                  <div key={n.key} className="flex justify-between gap-3">
                    <dt className="text-dim">{n.label()}</dt>
                    <dd className="num">
                      {formatNumber(n.value, n.decimals)} {n.unit}
                    </dd>
                  </div>
                ))}
              </dl>
            </div>
          ) : null}
        </div>
      )}
      <OffCredit productUrl={d.page_url} />
    </section>
  );
}

const size = (s: FoodLineSibling) =>
  s.pack_qty ? `${s.pack_qty}${s.pack_unit ? ` ${s.pack_unit}` : ''}` : '—';

/** Os outros tamanhos da mesma linha (marca + nome). */
export function SameLineList({ rows }: { rows: readonly FoodLineSibling[] }) {
  return (
    <section aria-label={t('alimentacao-outros-tamanhos')} className="flex flex-col gap-2">
      <SectionTitle>{t('alimentacao-outros-tamanhos')}</SectionTitle>
      <ul className="flex flex-col gap-1 text-[13px]">
        {rows.map((s) => (
          <li key={s.id} className="flex flex-wrap items-baseline justify-between gap-x-3">
            <span className="min-w-0 truncate" title={s.name}>
              {s.name} <span className="text-dim">· {size(s)}</span>
            </span>
            <span className="shrink-0">
              {s.last_price != null ? (
                <>
                  <UnitPrice cents={s.last_price} unit={s.base_unit} />
                </>
              ) : (
                '—'
              )}
              {s.last_date ? <span className="text-dim"> · {formatDate(s.last_date)}</span> : null}
            </span>
          </li>
        ))}
      </ul>
      <p className="text-[12px] text-dim">{t('alimentacao-outros-tamanhos-dica')}</p>
    </section>
  );
}
