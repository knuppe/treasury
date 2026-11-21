/**
 * Paletas categóricas dos gráficos, validadas com a skill dataviz
 * (`validate_palette.js`, faixa de luminosidade, croma, separação para daltonismo,
 * separação normal e contraste) sobre a superfície de cada tema:
 * - Escuro, sobre `#0c1117` e `#121921`: todas as checagens passam (pior CVD adjacente ΔE 8,4).
 * - Claro, sobre `#ffffff`: passa, com aviso de contraste < 3:1 em aqua, amarelo e
 *   magenta — por isso todo gráfico com ≥ 2 séries tem legenda e a visão em tabela.
 * - Meia-noite, sobre `#0c1428` e `#111b33`: passa (CVD ΔE 10,0; normal 16,2).
 * - Papel, sobre `#fbf8f2` e `#f3eee5`: passa, todas ≥ 3:1 (CVD ΔE 9,9; normal 16,9).
 * - Vidro, sobre o vidro composto em `#11162e`, `#1a213e` e sobre o vidro em cima de
 *   cada cor da malha (`#273278`, `#143854`, `#3b276a`): passa (CVD ΔE 10,6; normal 18,0).
 * - Scanner, sobre `#0c0c0e` e `#141417`: passa (CVD ΔE 8,3; normal 18,1). O vermelho
 *   do SCANNER abre a lista, longe do âmbar (vermelho e âmbar vizinhos não passam no CVD).
 * - Órbita, sobre `#0a0e15` e `#10151e`: passa (CVD ΔE 9,6; normal 16,1).
 * - Neon, sobre `#0e0a17` e `#151021`: passa (CVD ΔE 10,1; normal 24,7).
 * - Lo-fi, sobre `#fbf8f2` e `#f1ebe0`: passa, todas ≥ 3:1 (CVD ΔE 8,9). Verde e vermelho
 *   nunca vizinhos (não passam no CVD).
 *
 * Regras: cores atribuídas em ordem fixa e presas à entidade (categoria, conta,
 * titular), nunca à posição no ranking; a 9ª série em diante vira "Outras".
 * O Mono tem um só matiz: a identidade vem de passos de luminosidade (até 4 séries) +
 * legenda e rótulos diretos; acima disso, "Outras".
 */
import type { ThemeId } from '@/design/themes';

const DARK = [
  '#3987e5',
  '#d95926',
  '#199e70',
  '#c98500',
  '#d55181',
  '#008300',
  '#9085e9',
  '#e66767',
];
const LIGHT = [
  '#2a78d6',
  '#eb6834',
  '#1baf7a',
  '#eda100',
  '#e87ba4',
  '#008300',
  '#4a3aa7',
  '#e34948',
];
/** Azul, âmbar, verde-água, coral, orquídea, verde, lavanda, rosa. */
const MIDNIGHT = [
  '#4b8df0',
  '#c0821f',
  '#1ea38a',
  '#e0664f',
  '#c561b8',
  '#4c9a2a',
  '#7f7ce0',
  '#dc5c7a',
];
/** Tons de tinta e terra, todos com 3:1 sobre o papel. */
const PAPER = [
  '#2f62b0',
  '#c4572a',
  '#1d8a72',
  '#a87800',
  '#a8467a',
  '#4f7f1d',
  '#5b4bb0',
  '#c43d3d',
];
/** Mais luminosas e frias, para ler sobre o vidro. */
const GLASS = [
  '#4f86f7',
  '#e0682e',
  '#1fae86',
  '#b98700',
  '#dc5aa0',
  '#2f9a3a',
  '#9a7cf0',
  '#e05e5e',
];

/** Vermelho do scanner primeiro, depois as luzes do painel (azul, âmbar, verde-água…). */
const SCANNER = [
  '#e8432f',
  '#3a8fd9',
  '#b98700',
  '#1fa98a',
  '#c95fc0',
  '#5aa02c',
  '#8a7ff0',
  '#d55a7e',
];

/** O dourado do disco do buraco negro primeiro; o oliva do milharal da fazenda fecha. */
const ORBIT = [
  '#b98700',
  '#4f8fe0',
  '#1fa891',
  '#d4602f',
  '#8a7ff0',
  '#5d9e3a',
  '#d35a8f',
  '#8f8f2a',
];
/** Neon rosa primeiro, depois o ciano dos letreiros e o âmbar das torres. */
const NEON = [
  '#e8459a',
  '#2a96c8',
  '#b98700',
  '#8f6df0',
  '#16a88c',
  '#d65f2a',
  '#5c86e8',
  '#4f9a3a',
];

/** Índigo ai-iro, vermelho shu, azul-petróleo, rosa sakura, ocre, glicínia, matcha e ameixa. */
const LOFI = [
  '#3a6ea5',
  '#c4572e',
  '#0b8fa0',
  '#c2486a',
  '#a8801a',
  '#7a5ab0',
  '#4f8a4a',
  '#b04ab0',
];

/**
 * Verde fósforo, verde escuro, verde-claro, musgo, o amarelo do alerta e o vermelho; a
 * sétima e a oitava são o verde do texto e o do texto secundário.
 */
const PHOSPHOR = [
  '#00ff41',
  '#00b32d',
  '#7dffa0',
  '#2e7d32',
  '#e5ff00',
  '#ff3b3b',
  '#b8ffc8',
  '#4e9a5e',
];

type FullTheme =
  | 'dark'
  | 'light'
  | 'midnight'
  | 'paper'
  | 'glass'
  | 'kitt'
  | 'interstellar'
  | 'bladerunner'
  | 'lofi'
  | 'matrix';
type SingleHueTheme = Exclude<ThemeId, FullTheme>;

const FULL: Record<FullTheme, string[]> = {
  dark: DARK,
  light: LIGHT,
  midnight: MIDNIGHT,
  paper: PAPER,
  glass: GLASS,
  kitt: SCANNER,
  interstellar: ORBIT,
  bladerunner: NEON,
  lofi: LOFI,
  matrix: PHOSPHOR,
};

const SINGLE_HUE: Record<SingleHueTheme, string[]> = {
  mono: ['#e6e6e6', '#9a9a9a', '#c4c4c4', '#6e6e6e'],
};

function isFull(theme: ThemeId): theme is FullTheme {
  return theme in FULL;
}

export const MAX_SERIES = 8;

export function categoricalPalette(theme: ThemeId): string[] {
  return isFull(theme) ? FULL[theme] : SINGLE_HUE[theme];
}

/** Quantas séries distintas o tema aguenta antes de juntar o resto em "Outras". */
export function seriesCap(theme: ThemeId): number {
  return isFull(theme) ? MAX_SERIES : 4;
}

/**
 * Cor estável por entidade: a mesma chave recebe sempre o mesmo slot dentro de uma
 * lista ordenada de chaves (ex.: ids de categoria na ordem da árvore), mesmo que um
 * filtro esconda outras. Use `assignColors(todasAsChaves)` uma vez e consulte o mapa.
 */
export function assignColors(keys: readonly string[], theme: ThemeId): Map<string, string> {
  const pal = categoricalPalette(theme);
  const cap = seriesCap(theme);
  const map = new Map<string, string>();
  keys.forEach((k, i) => {
    if (i < cap) map.set(k, pal[i % pal.length]!);
  });
  return map;
}

/**
 * Rampa sequencial de um matiz, do fraco ao forte, para heatmaps. O primeiro passo é
 * o "sem valor" (`transparent` vira a superfície elevada do tema); os seis seguintes
 * passam no validador ordinal da skill dataviz (luminosidade monótona, ΔL ≥ 0,06 entre
 * vizinhos, um só matiz e o passo mais fraco com 2:1 sobre a superfície do tema).
 */
const SEQUENTIAL: Record<FullTheme, string[]> = {
  dark: ['transparent', '#17559b', '#2b6ebd', '#4288dd', '#63a3f4', '#84bfff', '#a4dbff'],
  light: ['transparent', '#7ebaff', '#5d9def', '#3a81d7', '#2566b3', '#0f4c90', '#00336e'],
  midnight: ['transparent', '#2356a0', '#356ec1', '#4c88df', '#6ba3f6', '#8abeff', '#a9daff'],
  paper: ['transparent', '#88b3f2', '#6997da', '#4d7cc1', '#38629f', '#25497f', '#113260'],
  glass: ['transparent', '#2b56a9', '#3e6ecb', '#5688e8', '#73a4fe', '#91bfff', '#afdbff'],
  kitt: ['transparent', '#921f15', '#b0291c', '#ce3524', '#e8472f', '#fc705a', '#ff9b87'],
  interstellar: ['transparent', '#8a5c12', '#a56f17', '#c2851c', '#dc9d2e', '#f0b85a', '#ffd58f'],
  bladerunner: ['transparent', '#8a2160', '#a82a75', '#c6368b', '#e049a2', '#f56fbb', '#ff9bd2'],
  lofi: ['transparent', '#97aed0', '#7593bd', '#5677a6', '#3f5f8c', '#2c4870', '#1c3354'],
  matrix: ['transparent', '#0b5a1f', '#0f7a2a', '#139b36', '#1abd43', '#3fe066', '#8dffa8'],
};

export function sequentialRamp(theme: ThemeId): string[] {
  if (isFull(theme)) return SEQUENTIAL[theme];
  const base = SINGLE_HUE[theme];
  return ['transparent', base[3]!, base[1]!, base[0]!, base[2]!];
}
