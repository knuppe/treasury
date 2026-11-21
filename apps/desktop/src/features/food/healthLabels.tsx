/**
 * Rótulos da saúde da cesta (docs/gtin-inflation.md §7): o "alto em" da rotulagem frontal da
 * Anvisa (selo em texto, na cor do texto, sem desenhar a lupa nem logotipos de terceiros), o
 * grupo NOVA, as restrições alimentares e os alérgenos do Open Food Facts, e o crédito da fonte
 * (ODbL).
 */
import { t } from '@/i18n';

export const OFF_URL = 'https://world.openfoodfacts.org';

const HIGH_IN_KEY = {
  added_sugar: 'alimentacao-saude-alto-acucar',
  saturated_fat: 'alimentacao-saude-alto-gordura',
  sodium: 'alimentacao-saude-alto-sodio',
} as const;

export const highInLabel = (k: string) =>
  k in HIGH_IN_KEY ? t(HIGH_IN_KEY[k as keyof typeof HIGH_IN_KEY]) : k;

/** "Alto em açúcar adicionado", um selo por nutriente. */
export function HighInBadges({ items }: { items: readonly string[] }) {
  if (!items.length) return null;
  return (
    <span className="inline-flex flex-wrap gap-1">
      {items.map((k) => (
        <span
          key={k}
          className="rounded-[3px] border border-fg px-1.5 py-px text-[10.5px] leading-tight font-semibold tracking-wide text-fg uppercase"
        >
          {t('alimentacao-saude-alto-selo', { nutriente: highInLabel(k).toLowerCase() })}
        </span>
      ))}
    </span>
  );
}

const RESTRICTION_KEY = {
  gluten: 'alimentacao-restricao-gluten',
  milk: 'alimentacao-restricao-milk',
  eggs: 'alimentacao-restricao-eggs',
  peanuts: 'alimentacao-restricao-peanuts',
  nuts: 'alimentacao-restricao-nuts',
  soybeans: 'alimentacao-restricao-soybeans',
  fish: 'alimentacao-restricao-fish',
  crustaceans: 'alimentacao-restricao-crustaceans',
  sesame: 'alimentacao-restricao-sesame',
  vegan: 'alimentacao-restricao-vegan',
  vegetarian: 'alimentacao-restricao-vegetarian',
} as const;

export const restrictionLabel = (r: string) =>
  r in RESTRICTION_KEY ? t(RESTRICTION_KEY[r as keyof typeof RESTRICTION_KEY]) : r;

/** Alérgenos do Open Food Facts (etiquetas sem o prefixo de idioma). */
const ALLERGEN_KEY = {
  ...RESTRICTION_KEY,
  'sesame-seeds': 'alimentacao-restricao-sesame',
  molluscs: 'alimentacao-alergeno-molluscs',
  mustard: 'alimentacao-alergeno-mustard',
  celery: 'alimentacao-alergeno-celery',
  lupin: 'alimentacao-alergeno-lupin',
  'sulphur-dioxide-and-sulphites': 'alimentacao-alergeno-sulphites',
} as const;

export const allergenLabel = (a: string) =>
  a in ALLERGEN_KEY
    ? t(ALLERGEN_KEY[a as keyof typeof ALLERGEN_KEY])
    : a.replace(/^[a-z]{2}:/, '').replace(/-/g, ' ');

const NOVA_KEY = {
  1: 'alimentacao-nova-1',
  2: 'alimentacao-nova-2',
  3: 'alimentacao-nova-3',
  4: 'alimentacao-nova-4',
} as const;

export const novaLabel = (n: number) =>
  n in NOVA_KEY ? t(NOVA_KEY[n as keyof typeof NOVA_KEY]) : String(n);

/** "Dados: Open Food Facts (ODbL)", com o link para o site ou para o produto. */
export function OffCredit({ productUrl }: { productUrl?: string | null }) {
  const link = 'text-accent underline';
  return (
    <span className="text-[11.5px] text-dim">
      <a className={link} href={OFF_URL} target="_blank" rel="noreferrer">
        {t('alimentacao-saude-credito')}
      </a>
      {productUrl ? (
        <>
          {' · '}
          <a className={link} href={productUrl} target="_blank" rel="noreferrer">
            {t('alimentacao-saude-ver-produto')}
          </a>
        </>
      ) : null}
    </span>
  );
}
