/**
 * Linha da vida (docs/life-expectancy.md): as contas da interface sobre a perspectiva que o
 * núcleo calcula — fases em segmentos, a posição de cada idade no eixo, os marcos agrupados e
 * os textos (no tom do perfil e a alternativa em texto da barra, com os mesmos números).
 */
import type { LifeMilestone } from '@/api/generated/LifeMilestone';
import type { LifeStage } from '@/api/generated/LifeStage';
import type { MemberOutlook } from '@/api/generated/MemberOutlook';
import type { ProfileTone } from '@/api/generated/ProfileTone';
import type { StageInfo } from '@/api/generated/StageInfo';
import { t } from '@/i18n';
import { formatLocale } from '@/lib/format';

/** As fases do núcleo e a subdivisão visual da longevidade avançada (80+). */
export type StageKey = LifeStage | 'advanced';

export const ADVANCED_FROM = 80;

/** A tabela do núcleo (seção 5), usada enquanto `longevity.stages` carrega. */
export const DEFAULT_STAGES: readonly StageInfo[] = [
  { stage: 'childhood', from: 0, to: 12 },
  { stage: 'adolescence', from: 12, to: 18 },
  { stage: 'youth', from: 18, to: 30 },
  { stage: 'adulthood', from: 30, to: 60 },
  { stage: 'later_life', from: 60, to: null },
];

export function stageLabel(s: StageKey): string {
  switch (s) {
    case 'childhood':
      return t('longevidade-fase-infancia');
    case 'adolescence':
      return t('longevidade-fase-adolescencia');
    case 'youth':
      return t('longevidade-fase-juventude');
    case 'adulthood':
      return t('longevidade-fase-vida-adulta');
    case 'later_life':
      return t('longevidade-fase-longevidade');
    case 'advanced':
      return t('longevidade-fase-longevidade-avancada');
  }
}

export type MilestoneKind = LifeMilestone['kind'];

export function milestoneKindLabel(kind: MilestoneKind): string {
  switch (kind) {
    case 'retirement':
      return t('longevidade-marco-aposentadoria');
    case 'inss':
      return t('longevidade-marco-inss');
    case 'plan_end':
      return t('longevidade-marco-plano');
    case 'debt_end':
      return t('longevidade-marco-divida');
    case 'goal':
      return t('longevidade-marco-meta');
    case 'life_event':
      return t('longevidade-marco-evento');
  }
}

/** Fase de uma idade (a subdivisão 80+ só com o ajuste). */
export function stageAt(stages: readonly StageInfo[], age: number, advanced = false): StageKey {
  if (advanced && age >= ADVANCED_FROM) return 'advanced';
  return stages.find((s) => age >= s.from && (s.to == null || age < s.to))?.stage ?? 'later_life';
}

export interface Segment {
  key: StageKey;
  from: number;
  to: number;
  /** Posição na ordem das fases (para a cor). */
  index: number;
}

/** As fases como segmentos de idade até `endAge` (a última vai até o fim do eixo). */
export function segments(
  stages: readonly StageInfo[],
  endAge: number,
  advanced = false,
): Segment[] {
  const out: Segment[] = [];
  for (const s of stages) {
    const to = Math.min(s.to ?? endAge, endAge);
    if (to <= s.from) continue;
    if (advanced && s.stage === 'later_life' && to > ADVANCED_FROM) {
      out.push({ key: s.stage, from: s.from, to: ADVANCED_FROM, index: out.length });
      out.push({ key: 'advanced', from: ADVANCED_FROM, to, index: out.length });
    } else {
      out.push({ key: s.stage, from: s.from, to, index: out.length });
    }
  }
  return out;
}

/** Eixo da barra: idade (0 a 100, por padrão) ou ano civil (a vista "Linha do tempo"). */
export interface Axis {
  mode: 'age' | 'year';
  from: number;
  to: number;
}

/** Um eixo de idade para todos (de 0 até passar do "1 em 10" de quem mostra o final). */
export function ageAxis(outlooks: readonly MemberOutlook[]): Axis {
  const ends = outlooks.filter((o) => o.show_end).map((o) => Math.ceil(o.p10_age) + 3);
  return { mode: 'age', from: 0, to: Math.max(100, ...ends) };
}

/** Eixo de anos: de hoje até passar do "1 em 10" do membro mais novo que mostra o final. */
export function yearAxis(outlooks: readonly MemberOutlook[], thisYear: number): Axis {
  const ends = outlooks.map(
    (o) => o.birth_year + (o.show_end ? Math.ceil(o.p10_age) + 2 : Math.max(80, o.age + 40)),
  );
  return { mode: 'year', from: thisYear, to: Math.max(thisYear + 40, ...ends) };
}

/** Idade no fim do eixo, para um membro. */
export function axisEndAge(axis: Axis, o: Pick<MemberOutlook, 'birth_year'>): number {
  return axis.mode === 'age' ? axis.to : axis.to - o.birth_year;
}

/** Posição (0 a 100) de uma idade do membro no eixo. */
export function position(axis: Axis, o: Pick<MemberOutlook, 'birth_year'>, age: number): number {
  const v = axis.mode === 'age' ? age : o.birth_year + age;
  return ((v - axis.from) / (axis.to - axis.from)) * 100;
}

export const clampPct = (n: number) => Math.max(0, Math.min(100, n));

/** "Passa dos N": a idade inteira que a fração já passou (piso). */
export const passAge = (n: number) => Math.floor(n);

const round1 = (n: number) => Math.round(n * 10) / 10;

/** Marcos próximos (menos de `gap` pontos do eixo) viram um pino só. */
export function clusterMilestones<T extends { x: number }>(items: readonly T[], gap = 2): T[][] {
  const sorted = [...items].sort((a, b) => a.x - b.x);
  const out: T[][] = [];
  for (const m of sorted) {
    const last = out[out.length - 1];
    if (last && m.x - last[0]!.x < gap) last.push(m);
    else out.push([m]);
  }
  return out;
}

/** Nomes numa lista ("A, B e C"), no idioma do app. */
export function listText(items: readonly string[]): string {
  const locale = formatLocale();
  return new Intl.ListFormat(locale === 'xx' ? 'pt-BR' : locale, { type: 'conjunction' }).format(
    items,
  );
}

export interface Place {
  code: string;
  name: string;
}

const sexArg = (o: MemberOutlook) => (o.sex === 'male' || o.sex === 'female' ? o.sex : 'other');

/** O texto ao lado da barra, no tom do perfil (simples, equilibrado ou técnico). */
export function outlookSentence(
  o: MemberOutlook,
  tone: ProfileTone,
  place: Place,
  sourceName: string,
  advanced = false,
  stages: readonly StageInfo[] = DEFAULT_STAGES,
): string {
  if (!o.show_end) {
    return t('longevidade-texto-crianca', { fase: stageLabel(stageAt(stages, o.age, advanced)) });
  }
  if (tone === 'tecnico') {
    const lines = [
      t('longevidade-texto-tecnico', {
        mediana: round1(o.median_age),
        p25: Math.round(o.p25_age),
        p10: Math.round(o.p10_age),
        idade: o.age,
        restante: round1(o.remaining_mean),
        fonte: sourceName,
        pais: place.code,
        sexo: sexArg(o),
      }),
    ];
    if (o.cohort_e0 != null)
      lines.push(
        t('longevidade-texto-coorte', { ano: String(o.birth_year), e0: round1(o.cohort_e0) }),
      );
    if (o.period_ref)
      lines.push(
        t('longevidade-texto-periodo', {
          fonte: o.period_ref.source,
          e0: round1(o.period_ref.e0),
          idade: o.age,
          restante: round1(o.period_ref.remaining_at_age),
        }),
      );
    return lines.join(' ');
  }
  const args = {
    onde: t('longevidade-onde', { pais: place.code, nome: place.name }),
    sexo: sexArg(o),
    ano: String(o.birth_year),
    idade: o.age,
    mediana: passAge(o.median_age),
    p25: passAge(o.p25_age),
    p10: passAge(o.p10_age),
  };
  return tone === 'simples'
    ? t('longevidade-texto-simples', args)
    : t('longevidade-texto-equilibrado', args);
}

/** Alternativa em texto da barra: a fase, o final (quando aparece) e os marcos. */
export function altText(
  o: MemberOutlook,
  milestones: readonly LifeMilestone[],
  advanced = false,
  stages: readonly StageInfo[] = DEFAULT_STAGES,
): string {
  const parts = [
    t('longevidade-alt', {
      nome: o.name,
      idade: o.age,
      fase: stageLabel(stageAt(stages, o.age, advanced)),
    }),
  ];
  if (o.show_end)
    parts.push(
      t('longevidade-alt-final', {
        mediana: passAge(o.median_age),
        p25: passAge(o.p25_age),
        p10: passAge(o.p10_age),
      }),
    );
  if (milestones.length)
    parts.push(
      t('longevidade-alt-marcos', {
        marcos: listText(
          milestones.map((m) =>
            t('longevidade-alt-marco', {
              rotulo: m.label,
              idade: Math.round(m.age),
              ano: String(m.year),
            }),
          ),
        ),
      }),
    );
  return parts.join(' ');
}
