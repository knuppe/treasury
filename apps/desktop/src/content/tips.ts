/**
 * Dicas do dia do painel simples (docs/profile.md): curtas, práticas e sem julgamento. Cada
 * uma vale para alguns níveis e, às vezes, só para um objetivo ou tipo de renda do perfil.
 * Os textos ficam nos catálogos (`locales/<idioma>/dicas.ftl`); aqui, só os filtros.
 */
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import type { UserProfile } from '@/api/generated/UserProfile';
import { t } from '@/i18n';

export interface Tip {
  /** Texto no idioma atual (lido do catálogo na hora). */
  readonly text: string;
  levels: readonly ProfileLevel[];
  /** Só para quem marcou este objetivo no questionário. */
  goal?: string;
  /** Só para este tipo de renda. */
  income?: string;
}

const BASIC: readonly ProfileLevel[] = ['iniciante', 'intermediario'];

const RULES = [
  { key: 'dicas-anotar-logo', levels: ['iniciante'] },
  { key: 'dicas-reserva-com-pouco', levels: BASIC, goal: 'reserva' },
  { key: 'dicas-guarde-primeiro', levels: BASIC },
  { key: 'dicas-gastos-pequenos', levels: BASIC, goal: 'onde_vai' },
  { key: 'dicas-juros-mais-altos', levels: BASIC, goal: 'dividas' },
  { key: 'dicas-renda-variavel', levels: BASIC, income: 'variavel' },
  { key: 'dicas-beneficios', levels: BASIC, income: 'beneficios' },
  { key: 'dicas-assinaturas', levels: ['iniciante', 'intermediario', 'avancado'] },
  { key: 'dicas-meta', levels: BASIC, goal: 'objetivo' },
  { key: 'dicas-comparar-mes', levels: ['intermediario', 'avancado'] },
] as const satisfies readonly {
  key: string;
  levels: readonly ProfileLevel[];
  goal?: string;
  income?: string;
}[];

export const TIPS: readonly Tip[] = RULES.map(({ key, ...rule }) => ({
  ...rule,
  get text() {
    return t(key);
  },
}));

/** A dica do dia: muda todo dia, sempre a mesma no mesmo dia, entre as que servem ao perfil. */
export function tipOfTheDay(
  profile: Pick<UserProfile, 'level' | 'goals' | 'income'>,
  isoDate: string,
): string {
  const fits = TIPS.filter(
    (tip) =>
      tip.levels.includes(profile.level) &&
      (!tip.goal || profile.goals.includes(tip.goal)) &&
      (!tip.income || profile.income === tip.income),
  );
  const pool = fits.length ? fits : TIPS;
  const day = Math.floor(Date.parse(`${isoDate}T12:00:00Z`) / 86_400_000);
  return pool[((day % pool.length) + pool.length) % pool.length]!.text;
}
