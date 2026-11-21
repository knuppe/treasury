/**
 * Rótulos do feedback (catálogo `desenvolvedor.ftl`) ao desenvolvedor e o Markdown para virar issue no GitHub (um
 * feedback) ou um arquivo com todos.
 */
import type { DevFeedback } from '@/api/generated/DevFeedback';
import type { DevFeedbackArea } from '@/api/generated/DevFeedbackArea';
import type { DevFeedbackKind } from '@/api/generated/DevFeedbackKind';
import type { DevFeedbackStatus } from '@/api/generated/DevFeedbackStatus';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

export const AREA_LABEL: Record<DevFeedbackArea, string> = {
  get agente() {
    return t('desenvolvedor-area-agente');
  },
  get mcp() {
    return t('desenvolvedor-area-mcp');
  },
  get importacao() {
    return t('desenvolvedor-area-importacao');
  },
  get lancamentos() {
    return t('desenvolvedor-area-lancamentos');
  },
  get carteira() {
    return t('desenvolvedor-area-carteira');
  },
  get ui() {
    return t('desenvolvedor-area-ui');
  },
  get dados() {
    return t('desenvolvedor-area-dados');
  },
  get outro() {
    return t('desenvolvedor-area-outro');
  },
};

export const KIND_LABEL: Record<DevFeedbackKind, string> = {
  get bug() {
    return t('desenvolvedor-tipo-bug');
  },
  get melhoria() {
    return t('desenvolvedor-tipo-melhoria');
  },
  get ferramenta_faltando() {
    return t('desenvolvedor-tipo-ferramenta-faltando');
  },
  get dados() {
    return t('desenvolvedor-tipo-dados');
  },
  get documentacao() {
    return t('desenvolvedor-tipo-documentacao');
  },
};

export const STATUS_LABEL: Record<DevFeedbackStatus, string> = {
  get aberto() {
    return t('desenvolvedor-status-aberto');
  },
  get em_andamento() {
    return t('desenvolvedor-status-em-andamento');
  },
  get resolvido() {
    return t('desenvolvedor-status-resolvido');
  },
  get descartado() {
    return t('desenvolvedor-status-descartado');
  },
};

export const AREAS = Object.keys(AREA_LABEL) as DevFeedbackArea[];
export const KINDS = Object.keys(KIND_LABEL) as DevFeedbackKind[];
export const STATUSES = Object.keys(STATUS_LABEL) as DevFeedbackStatus[];

/** Quem registrou: `user` vira "você". */
export function authorLabel(who: string): string {
  return who === 'user' ? t('desenvolvedor-voce') : who;
}

/** Seções do painel: o que o agente informou nos campos ou, nos antigos, escreveu no texto. */
export interface FeedbackSections {
  problem: string;
  proposal: string | null;
  acceptance: string | null;
  /** Instruções deixadas nos reforços ("Instrução: …"). */
  instructions: string[];
  pointers: string[];
}

/** Marcadores de seção no começo de uma linha do texto ("Proposta:", "Aceite:"…). */
const MARKERS: [RegExp, 'problem' | 'proposal' | 'acceptance'][] = [
  [/^(problema|problem)\b[^:\n]{0,60}:\s*/i, 'problem'],
  [
    /^(proposta|propuesta|proposal|sugest[aã]o|sugerencia|suggestion)\b[^:\n]{0,40}:\s*/i,
    'proposal',
  ],
  [
    /^(crit[eé]rios? de aceite|aceite|criterio de aceptaci[oó]n|aceptaci[oó]n|acceptance criteria|acceptance)\b[^:\n]{0,40}:\s*/i,
    'acceptance',
  ],
];

/** Separa o texto livre em problema, proposta e aceite pelos marcadores no começo das linhas. */
export function splitBody(body: string): Record<'problem' | 'proposal' | 'acceptance', string> {
  const out = { problem: [] as string[], proposal: [] as string[], acceptance: [] as string[] };
  let at: keyof typeof out = 'problem';
  for (const raw of body.split('\n')) {
    let line = raw;
    for (const [re, key] of MARKERS) {
      const m = re.exec(line.trim());
      if (m) {
        at = key;
        line = line.trim().slice(m[0].length);
        break;
      }
    }
    out[at].push(line);
  }
  const join = (l: string[]) => l.join('\n').trim();
  return {
    problem: join(out.problem),
    proposal: join(out.proposal),
    acceptance: join(out.acceptance),
  };
}

/** Caminhos de código citados no texto (`crates/x/src/y.rs:120`, `mcp/tools.rs`). */
const POINTER = /(?:[\w.-]+\/)+[\w.-]+\.(?:rs|tsx?|sql|ftl|md|json|toml|swift|mjs)(?::\d+)?/g;

export function findPointers(text: string): string[] {
  return [...new Set(text.match(POINTER) ?? [])];
}

const INSTRUCTION = /^(instru[cç][aã]o|instrucci[oó]n|instruction)s?\s*:\s*/i;
const ACCEPT_INLINE =
  /(crit[eé]rios? de aceite|aceite|criterio de aceptaci[oó]n|acceptance criteria)\s*:\s*/i;

export function sections(f: DevFeedback): FeedbackSections {
  const parsed = splitBody(f.body);
  const instructions: string[] = [];
  let acceptance = f.acceptance_criteria ?? (parsed.acceptance || null);
  for (const n of f.notes) {
    const text = n.note?.trim();
    if (!text || n.kind !== 'reforco' || !INSTRUCTION.test(text)) continue;
    let rest = text.replace(INSTRUCTION, '');
    // "Instrução: … Aceite: …": o aceite do reforço vale quando o item não tem um.
    const m = ACCEPT_INLINE.exec(rest);
    if (m) {
      const acc = rest.slice(m.index + m[0].length).trim();
      rest = rest.slice(0, m.index).trim();
      acceptance ??= acc || null;
    }
    instructions.push(rest);
  }
  const pointers =
    f.code_pointers.length > 0
      ? f.code_pointers
      : findPointers([f.body, ...f.notes.map((n) => n.note ?? '')].join('\n'));
  return {
    problem: parsed.problem || f.body.trim(),
    proposal: f.proposal ?? (parsed.proposal || null),
    acceptance,
    instructions,
    pointers,
  };
}

/** "Prioridade 3": a importância, que começa em 1 e sobe com cada reforço. */
export function priorityLabel(f: Pick<DevFeedback, 'importance'>): string {
  return t('desenvolvedor-prioridade-n', { n: f.importance });
}

/**
 * Para colar num agente de código: o contexto, o problema, o que fazer, onde mexer e como saber que
 * terminou, mais como fechar o item pelo MCP.
 */
export function taskMarkdown(f: DevFeedback): string {
  const s = sections(f);
  const lines = [
    `# ${f.title}`,
    '',
    t('desenvolvedor-cc-contexto', {
      id: f.id,
      area: AREA_LABEL[f.area],
      tipo: KIND_LABEL[f.kind],
      prioridade: f.importance,
      status: STATUS_LABEL[f.status],
    }),
    '',
    `## ${t('desenvolvedor-secao-problema')}`,
    '',
    s.problem,
  ];
  if (f.expected || f.actual) {
    lines.push('');
    if (f.expected) lines.push(`- ${t('desenvolvedor-cc-esperado')}: ${f.expected}`);
    if (f.actual) lines.push(`- ${t('desenvolvedor-cc-obtido')}: ${f.actual}`);
  }
  if (f.repro_steps) lines.push('', `### ${t('desenvolvedor-secao-passos')}`, '', f.repro_steps);
  if (s.proposal || s.instructions.length > 0) {
    lines.push('', `## ${t('desenvolvedor-secao-proposta')}`, '');
    if (s.proposal) lines.push(s.proposal);
    for (const i of s.instructions) lines.push(`- ${i}`);
  }
  lines.push(
    '',
    `## ${t('desenvolvedor-secao-aceite')}`,
    '',
    s.acceptance ?? t('desenvolvedor-cc-sem-aceite'),
  );
  if (s.pointers.length > 0) {
    lines.push(
      '',
      `## ${t('desenvolvedor-secao-onde')}`,
      '',
      ...s.pointers.map((p) => `- \`${p}\``),
    );
  }
  if (f.links.length > 0) {
    lines.push('', `## ${t('desenvolvedor-secao-relacionados')}`, '');
    for (const l of f.links)
      lines.push(`- ${linkLabel(l)}: ${l.related_title} (\`${l.related_id}\`)`);
  }
  lines.push('', '---', '', t('desenvolvedor-cc-fechar', { id: f.id }));
  return `${lines.join('\n')}\n`;
}

/** Como o vínculo se lê deste lado ("Duplicado de", "Tem duplicado", "Depende de"…). */
export function linkLabel(l: DevFeedback['links'][number]): string {
  return t('desenvolvedor-vinculo', { kind: l.kind, entrada: l.incoming ? 'sim' : 'nao' });
}

/** Corpo de uma issue do GitHub: título, metadados, texto e a linha do tempo dos reforços. */
export function feedbackMarkdown(f: DevFeedback, heading = '##'): string {
  const lines = [
    `${heading} ${f.title}`,
    '',
    `- ${t('desenvolvedor-md-metadados', {
      area: AREA_LABEL[f.area],
      tipo: KIND_LABEL[f.kind],
      status: STATUS_LABEL[f.status],
    })}`,
    `- ${t('desenvolvedor-md-importancia', { importancia: f.importance, n: f.bumps })}`,
    `- ${t('desenvolvedor-md-registrado', {
      autor: authorLabel(f.created_by),
      data: formatDateTime(f.created_at),
    })}`,
    '',
    f.body.trim(),
  ];
  if (f.proposal)
    lines.push('', `${heading}# ${t('desenvolvedor-secao-proposta')}`, '', f.proposal);
  if (f.acceptance_criteria)
    lines.push('', `${heading}# ${t('desenvolvedor-secao-aceite')}`, '', f.acceptance_criteria);
  if (f.code_pointers.length > 0)
    lines.push(
      '',
      `${heading}# ${t('desenvolvedor-secao-onde')}`,
      '',
      ...f.code_pointers.map((p) => `- \`${p}\``),
    );
  if (f.notes.length > 0) {
    lines.push('', `${heading}# ${t('desenvolvedor-md-reforcos')}`, '');
    for (const n of f.notes) {
      const who = authorLabel(n.client);
      lines.push(`- ${formatDateTime(n.created_at)} · ${who}${n.note ? `: ${n.note}` : ''}`);
    }
  }
  return `${lines.join('\n')}\n`;
}

/** Todos num arquivo, na ordem da lista (mais importante primeiro). */
export function allMarkdown(items: readonly DevFeedback[], now = new Date()): string {
  const head = [
    `# ${t('desenvolvedor-md-titulo')}`,
    '',
    t('desenvolvedor-md-exportado', { data: formatDateTime(now), n: items.length }),
    '',
  ].join('\n');
  return `${head}\n${items.map((f) => feedbackMarkdown(f, '##')).join('\n---\n\n')}`;
}
