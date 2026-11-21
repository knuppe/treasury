/**
 * Cor de cada entidade (categoria de primeiro nível, titular, cartão), presa à entidade e
 * nunca ao ranking: as chaves entram em ordem fixa (a da árvore de categorias, a de
 * cadastro dos titulares) e só as que têm movimento numa janela fixa disputam as cores.
 * A 9ª em diante (5ª no Mono, de um só matiz) vira "Outras", em cinza.
 */
import { assignColors } from '@/charts/palette';
import type { ThemeId } from '@/design/themes';
import { t } from '@/i18n';

export const OTHERS_KEY = 'others';
/** Nome do grupo "Outras" no idioma atual. */
export const othersLabel = () => t('relatorios-outras');

export interface EntityColors {
  /** Cor da entidade, ou `undefined` se ela cai em "Outras". */
  get(key: string | null | undefined): string | undefined;
  /** As chaves com cor, em ordem fixa. */
  keys: readonly string[];
}

/** Cores para as chaves `ordered` (ordem fixa) que estão em `active` (todas, se `null`). */
export function entityColors(
  ordered: readonly string[],
  active: ReadonlySet<string> | null,
  theme: ThemeId,
): EntityColors {
  const keys = ordered.filter((k) => !active || active.has(k));
  const map = assignColors(keys, theme);
  return {
    get: (key) => (key == null ? undefined : map.get(key)),
    keys: keys.filter((k) => map.has(k)),
  };
}

/** Cor de uma chave com cinza para "Outras" e para o que não tem cor. */
export function colorOr(
  colors: EntityColors,
  key: string | null | undefined,
  gray: string,
): string {
  return colors.get(key) ?? gray;
}

export { inkOn, luminance, mix, parseHex } from '@/charts/colorMath';
