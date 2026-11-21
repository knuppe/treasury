/**
 * Mock de `profile.*`: as mesmas perguntas, pontuação e regras do núcleo (`profile`):
 * sem perfil salvo, avançado para quem tem contas ou lançamentos e questionário pendente
 * para um cofre vazio.
 */
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import type { ProfileQuestion } from '@/api/generated/ProfileQuestion';
import type { ProfileTone } from '@/api/generated/ProfileTone';
import type { UserProfile } from '@/api/generated/UserProfile';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { nowUtc } from './ledger/dates';
import { invalid } from './ledger/errors';
import { rows } from './ledger/store';
import { enableLongevity } from './longevity';

type Spec = [
  id: string,
  title: string,
  hint: string | null,
  multiple: boolean,
  options: [string, string, number | null][],
];

const QUESTIONS: Spec[] = [
  [
    'confianca',
    'Como você se sente em relação a cuidar do seu dinheiro?',
    null,
    false,
    [
      ['nada', 'Nada seguro: quero aprender o básico', 0],
      ['mais_ou_menos', 'Mais ou menos: sei algumas coisas, mas preciso de ajuda', 0.7],
      ['bastante', 'Bastante seguro: já controlo bem', 1.4],
      ['muito', 'Muito seguro: entendo investimentos e estratégias avançadas', 2],
    ],
  ],
  [
    'controle',
    'Você já anota ou controla seus gastos hoje?',
    null,
    false,
    [
      ['nunca', 'Não, nunca controlei', 0],
      ['as_vezes', 'Às vezes, de cabeça ou no papel', 0.5],
      ['planilha', 'Sim, com planilha ou um app simples', 1.3],
      ['detalhado', 'Sim, de forma detalhada e regular', 2],
    ],
  ],
  [
    'objetivo',
    'O que você mais quer agora com o seu dinheiro?',
    'Pode escolher mais de um.',
    true,
    [
      ['onde_vai', 'Saber para onde o dinheiro está indo', 0],
      ['reserva', 'Juntar uma reserva para imprevistos', 0.4],
      ['dividas', 'Sair das dívidas ou diminuir', 0.8],
      ['objetivo', 'Juntar para um objetivo (viagem, reforma…)', 1],
      ['organizar', 'Organizar melhor o que já faço', 1.5],
      ['investir', 'Investir e fazer o dinheiro render', 2],
    ],
  ],
  [
    'reserva',
    'E a reserva de emergência (dinheiro guardado para imprevistos)?',
    null,
    false,
    [
      ['nao_sei', 'Não sei o que é', 0],
      ['nao_tenho', 'Sei o que é, mas ainda não tenho', 0.5],
      ['um_pouco', 'Tenho um pouco, mas não o suficiente', 1],
      ['suficiente', 'Tenho o suficiente (de 3 a 6 meses de gastos)', 2],
    ],
  ],
  [
    'renda',
    'Como é a sua renda principal?',
    'Serve para ajustar as dicas; não muda o nível.',
    false,
    [
      ['fixa', 'Fixa e previsível (salário, CLT ou parecido)', null],
      ['variavel', 'Variável (freelancer, comissão, autônomo, bicos)', null],
      ['beneficios', 'Benefícios do governo (Bolsa Família, BPC, aposentadoria ou outros)', null],
      ['mista', 'Mista (salário com benefícios ou renda variável)', null],
      ['nao_informar', 'Prefiro não informar agora', null],
    ],
  ],
  [
    'termos',
    'Quando você vê termos como “juros compostos”, “CDI” ou “alocação de ativos”…',
    null,
    false,
    [
      ['perdido', 'Não entendo e fico perdido', 0],
      ['mais_ou_menos', 'Entendo mais ou menos', 1],
      ['entendo', 'Entendo bem e uso no dia a dia', 2],
    ],
  ],
  [
    'tom',
    'Como você prefere que o app fale com você?',
    null,
    false,
    [
      ['simples', 'Bem simples, com explicações e exemplos do dia a dia', 0],
      ['equilibrado', 'Com equilíbrio: claro, mas sem infantilizar', 1],
      ['tecnico', 'Detalhado e técnico, com análises a fundo', 2],
    ],
  ],
  // Linha da vida (docs/life-expectancy.md, seção 8): sem pontos, só no questionário da
  // família; "Sim" liga a função.
  [
    'longevity',
    'Quer ver a linha da vida da família?',
    'Uma barra com as fases da vida de cada pessoa e os planos de dinheiro ao longo dela, com base em médias da população do país. Algumas pessoas preferem não ver esse tipo de informação.',
    false,
    [
      ['yes', 'Sim, quero ver', null],
      ['no', 'Não, obrigado', null],
      ['later', 'Decidir depois', null],
    ],
  ],
];

const TONE_OF: Record<ProfileLevel, ProfileTone> = {
  iniciante: 'simples',
  intermediario: 'equilibrado',
  avancado: 'tecnico',
};

let saved: UserProfile | null = null;
/** Perfil de cada membro (sem questionário próprio: o da família). */
const byMember = new Map<string, UserProfile>();
onMockReset(() => {
  saved = null;
  byMember.clear();
});

const memberOf = (args: Record<string, unknown>) =>
  typeof args.member_id === 'string' && args.member_id ? args.member_id : null;

export function getFor(member: string | null): UserProfile {
  if (!member) return get();
  return (
    byMember.get(member) ?? {
      ...get(),
      source: 'familia',
      answers: {},
      goals: [],
      income: null,
      onboarding_pending: false,
      updated_at: null,
    }
  );
}

function saveFor(member: string | null, p: UserProfile): UserProfile {
  if (member) byMember.set(member, p);
  else saved = p;
  return p;
}

function get(): UserProfile {
  if (saved) return saved;
  return {
    level: 'avancado',
    tone: 'tecnico',
    source: 'padrao',
    goals: [],
    income: null,
    answers: {},
    onboarding_pending: rows('account').length === 0 && rows('txn').length === 0,
    updated_at: null,
  };
}

function score(answers: Record<string, string[]>): [ProfileLevel, ProfileTone] {
  const values: number[] = [];
  for (const [id, , , , opts] of QUESTIONS) {
    const pts = (answers[id] ?? [])
      .map((p) => opts.find((o) => o[0] === p)?.[2])
      .filter((v): v is number => typeof v === 'number');
    if (pts.length) values.push(pts.reduce((a, b) => a + b, 0) / pts.length);
  }
  const avg = values.length ? values.reduce((a, b) => a + b, 0) / values.length : 1;
  const level: ProfileLevel = avg < 0.8 ? 'iniciante' : avg < 1.5 ? 'intermediario' : 'avancado';
  const t = answers.tom?.[0];
  const tone: ProfileTone =
    t === 'simples' || t === 'equilibrado' || t === 'tecnico' ? t : TONE_OF[level];
  return [level, tone];
}

function validate(answers: Record<string, string[]>) {
  for (const [q, picked] of Object.entries(answers)) {
    const spec = QUESTIONS.find((s) => s[0] === q);
    if (!spec) invalid(`pergunta desconhecida: ${q}`);
    if (!spec[3] && picked.length > 1) invalid(`a pergunta ${q} aceita uma resposta só`);
    for (const p of picked)
      if (!spec[4].some((o) => o[0] === p)) invalid(`opção desconhecida em ${q}: ${p}`);
  }
}

registerMockHandlers(
  guarded({
    'profile.get': (args) => getFor(memberOf(args)),
    'profile.questions': (): ProfileQuestion[] =>
      QUESTIONS.map(([id, title, hint, multiple, opts]) => ({
        id,
        title,
        hint,
        multiple,
        options: opts.map(([oid, label]) => ({ id: oid, label })),
      })),
    'profile.answer': (args) => {
      const answers = (args.answers ?? {}) as Record<string, string[]>;
      validate(answers);
      const [level, tone] = score(answers);
      const income = answers.renda?.[0];
      // A chave é da família: o questionário de um membro não liga a função.
      if (!memberOf(args) && answers.longevity?.[0] === 'yes') enableLongevity();
      return saveFor(memberOf(args), {
        level,
        tone,
        source: 'questionario',
        goals: answers.objetivo ?? [],
        income: income && income !== 'nao_informar' ? income : null,
        answers,
        onboarding_pending: false,
        updated_at: nowUtc(),
      });
    },
    'profile.skip': (args) => {
      return saveFor(memberOf(args), {
        level: 'avancado',
        tone: 'tecnico',
        source: 'pulou',
        goals: [],
        income: null,
        answers: {},
        onboarding_pending: false,
        updated_at: nowUtc(),
      });
    },
    'profile.set': (args) => {
      const cur = getFor(memberOf(args));
      const level = (args.level as ProfileLevel | undefined) ?? cur.level;
      return saveFor(memberOf(args), {
        ...cur,
        level,
        tone: (args.tone as ProfileTone | undefined) ?? cur.tone,
        source: level !== cur.level ? 'escolha' : cur.source,
        onboarding_pending: false,
        updated_at: nowUtc(),
      });
    },
  }),
);
