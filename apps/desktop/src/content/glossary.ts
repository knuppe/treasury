/**
 * Explicações curtas dos termos que aparecem no app (docs/profile.md). Usadas pelo `Explain`
 * ("O que é isso?") no painel simples e pelos ícones `?` (`HelpTip`) que aparecem ao lado
 * dos rótulos para quem respondeu no questionário que está começando.
 *
 * Os textos ficam nos catálogos (`locales/<idioma>/glossario.ftl`, docs/i18n-design.md):
 * - `-termo`: o nome do termo.
 * - `-simples`: linguagem do dia a dia (tom "bem simples" do questionário).
 * - `-equilibrado`: um pouco mais preciso, para o tom "equilibrado" e o "técnico" (sem ele,
 *   vale o `simples`). Só os termos marcados com `equilibrado: true` abaixo têm essa versão.
 * - `-busca`: expressão regular testada contra o rótulo (minúsculo e sem acento), com as
 *   alternativas separadas por `|`, escrita para os rótulos de cada idioma. O primeiro termo
 *   que casar vence, na ordem de `TERMS`, então os mais específicos vêm antes.
 *
 * Aqui fica só o que não muda com o idioma: a ordem da busca, `upTo`, até que nível o `?`
 * aparece, e `br`, os termos que só existem no Brasil (fora dele, sem `?`;
 * docs/international.md). Termos do dia a dia (`iniciante`) só para quem está começando; termos técnicos
 * (`intermediario`) também para o nível do meio. No avançado, nenhum `?` (a não ser que a
 * pessoa ligue em Ajustes › Perfil).
 */
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import type { ProfileTone } from '@/api/generated/ProfileTone';
import { t, useI18n } from '@/i18n';

type HelpLevel = Exclude<ProfileLevel, 'avancado'>;

const TERMS = {
  reserva: { upTo: 'intermediario', equilibrado: true },
  sobra: { upTo: 'iniciante' },
  essenciais: { upTo: 'iniciante' },
  taxa_poupanca: { upTo: 'intermediario', equilibrado: true },
  orcamento: { upTo: 'iniciante' },
  patrimonio_liquido: { upTo: 'intermediario', equilibrado: true },
  competencia: { upTo: 'intermediario', equilibrado: true },
  juros_compostos: { upTo: 'intermediario' },
  cdi: { upTo: 'intermediario', equilibrado: true, br: true },
  selic: { upTo: 'intermediario', equilibrado: true, br: true },
  ipca: { upTo: 'intermediario', equilibrado: true, br: true },
  xirr: { upTo: 'intermediario', equilibrado: true },
  real: { upTo: 'intermediario', equilibrado: true },
  nominal: { upTo: 'intermediario' },
  rentabilidade: { upTo: 'intermediario', equilibrado: true },
  valor_mercado: { upTo: 'intermediario', equilibrado: true },
  nao_realizado: { upTo: 'intermediario' },
  realizado: { upTo: 'intermediario' },
  preco_medio: { upTo: 'intermediario', equilibrado: true },
  custo: { upTo: 'intermediario' },
  provento: { upTo: 'intermediario', equilibrado: true },
  dividendo: { upTo: 'intermediario' },
  jcp: { upTo: 'intermediario', br: true },
  yield: { upTo: 'intermediario' },
  come_cotas: { upTo: 'intermediario', equilibrado: true, br: true },
  alocacao: { upTo: 'intermediario' },
  rebalanceamento: { upTo: 'intermediario' },
  desvio: { upTo: 'intermediario' },
  liquidez: { upTo: 'intermediario' },
  renda_fixa: { upTo: 'intermediario' },
  renda_variavel: { upTo: 'intermediario' },
  fii: { upTo: 'intermediario', br: true },
  etf: { upTo: 'intermediario' },
  acoes: { upTo: 'iniciante' },
  tesouro: { upTo: 'intermediario', equilibrado: true, br: true },
  cdb: { upTo: 'intermediario', br: true },
  rdb: { upTo: 'intermediario', br: true },
  lci_lca: { upTo: 'intermediario', br: true },
  previdencia: { upTo: 'intermediario' },
  aporte: { upTo: 'iniciante' },
  resgate: { upTo: 'iniciante' },
  indexador: { upTo: 'intermediario' },
  fatura: { upTo: 'iniciante' },
  rotativo: { upTo: 'intermediario' },
  cet: { upTo: 'intermediario' },
  amortizacao: { upTo: 'intermediario' },
  price_sac: { upTo: 'intermediario' },
  divida_cara: { upTo: 'intermediario' },
  saldo_devedor: { upTo: 'iniciante' },
  conciliado: { upTo: 'iniciante' },
  previsto: { upTo: 'iniciante' },
  recorrencia: { upTo: 'iniciante' },
  transferencia: { upTo: 'iniciante' },
  titular: { upTo: 'iniciante' },
  vencimento: { upTo: 'iniciante' },
  cambio: { upTo: 'intermediario' },
  deflator: { upTo: 'intermediario' },
  m2: { upTo: 'intermediario' },
  score_saude: { upTo: 'intermediario' },
  volatilidade: { upTo: 'intermediario' },
  drawdown: { upTo: 'intermediario' },
  aliquota: { upTo: 'intermediario' },
  darf: { upTo: 'intermediario', br: true },
  ir: { upTo: 'intermediario', br: true },
} as const satisfies Record<string, { upTo: HelpLevel; equilibrado?: true; br?: true }>;

export type GlossaryId = keyof typeof TERMS;

/** Termos com versão equilibrada no catálogo (`glossario-<id>-equilibrado`). */
type BalancedId = {
  [K in GlossaryId]: (typeof TERMS)[K] extends { equilibrado: true } ? K : never;
}[GlossaryId];

export interface GlossaryEntry {
  readonly term: string;
  readonly simples: string;
  readonly equilibrado?: string;
  readonly upTo: HelpLevel;
  /** Busca do idioma atual (vazia se o catálogo não tiver). */
  readonly match?: readonly RegExp[];
}

const IDS = Object.keys(TERMS) as GlossaryId[];

const isBalanced = (id: GlossaryId): id is BalancedId =>
  (TERMS[id] as { equilibrado?: true }).equilibrado === true;

/**
 * Buscas compiladas do idioma atual, na ordem de `TERMS`. Refeitas quando o idioma (os
 * catálogos carregados) muda. A expressão vem crua do catálogo, sem passar pelo `t()`: o
 * pseudo-idioma alongaria e acentuaria o texto.
 */
let compiled: { bundles: unknown; byId: Map<GlossaryId, RegExp> } | null = null;

function searches(): Map<GlossaryId, RegExp> {
  const { bundles } = useI18n.getState();
  if (compiled?.bundles === bundles) return compiled.byId;
  const byId = new Map<GlossaryId, RegExp>();
  for (const id of IDS) {
    const key = `glossario-${id}-busca`;
    for (const b of bundles) {
      const m = b.getMessage(key);
      if (!m?.value) continue;
      const source = b.formatPattern(m.value).trim();
      try {
        if (source) byId.set(id, new RegExp(source));
      } catch {
        // Expressão inválida no catálogo: o termo fica sem `?` nesse idioma.
      }
      break;
    }
  }
  compiled = { bundles, byId };
  return byId;
}

function entry(id: GlossaryId): GlossaryEntry {
  return {
    get term() {
      return t(`glossario-${id}-termo`);
    },
    get simples() {
      return t(`glossario-${id}-simples`);
    },
    get equilibrado() {
      return isBalanced(id) ? t(`glossario-${id}-equilibrado`) : undefined;
    },
    upTo: TERMS[id].upTo,
    get match() {
      const re = searches().get(id);
      return re ? [re] : undefined;
    },
  };
}

/** Os termos, com os textos lidos do catálogo na hora (nada é lido ao importar). */
export const GLOSSARY = Object.fromEntries(IDS.map((id) => [id, entry(id)])) as Readonly<
  Record<GlossaryId, GlossaryEntry>
>;

/** Minúsculo e sem acento, para casar o rótulo com os termos. */
export function normalizeLabel(text: string): string {
  return text.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase().trim();
}

/** Termo só do Brasil (CDI, Selic, Tesouro, DARF…). */
export const brazilOnly = (id: GlossaryId) => (TERMS[id] as { br?: true }).br === true;

/**
 * Termo do glossário que explica o rótulo, ou `null`. Rótulos longos (frases) não casam. Fora
 * do Brasil (`brazil: false`), os termos só de lá ficam de fora.
 */
export function findTerm(label: string, brazil = true): GlossaryId | null {
  const text = normalizeLabel(label);
  if (!text || text.length > 48) return null;
  for (const [id, re] of searches()) {
    if (!brazil && brazilOnly(id)) continue;
    if (re.test(text)) return id;
  }
  return null;
}

/** Texto do termo no tom do perfil. */
export function explanation(id: GlossaryId, tone: ProfileTone): string {
  const e = GLOSSARY[id];
  return tone === 'simples' ? e.simples : (e.equilibrado ?? e.simples);
}

/** O `?` deste termo aparece neste nível? */
export function showsAt(id: GlossaryId, level: ProfileLevel): boolean {
  const upTo: HelpLevel = TERMS[id].upTo;
  if (level === 'avancado') return false;
  return level === 'iniciante' || upTo === 'intermediario';
}

/** O termo dentro de uma frase ("O que é …?"): minúscula no começo, menos em siglas (CDI, FII). */
export function termInSentence(term: string): string {
  const second = term.charAt(1);
  return second && second === second.toUpperCase() && second !== second.toLowerCase()
    ? term
    : term.charAt(0).toLowerCase() + term.slice(1);
}
