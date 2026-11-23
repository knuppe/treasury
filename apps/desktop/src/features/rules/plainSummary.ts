/**
 * Resumo das condições de uma regra em palavras, para a lista de regras: uma expressão regular
 * de fábrica vira "Descrição contém ifood, rappi, uber eats e mais 4" e "Valor menor ou igual a
 * -0,01" vira "é uma saída". O texto exato (`conditionsSummary`) continua no `title` da linha e
 * no diálogo de edição.
 */
import type { RuleInput } from '@/api/generated/RuleInput';
import { RULE_FIELD } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';

import { conditionsSummary } from './RuleDialog';

/** Quantos termos aparecem antes do "e mais N". */
export const TERMS_SHOWN = 4;

/** Divide no `|` de primeiro nível (fora de parênteses e colchetes). */
function topLevelAlternatives(p: string): string[] {
  const out: string[] = [];
  let depth = 0;
  let cur = '';
  for (let i = 0; i < p.length; i += 1) {
    const ch = p[i]!;
    if (ch === '\\' && i + 1 < p.length) {
      cur += ch + p[i + 1]!;
      i += 1;
      continue;
    }
    if (ch === '(' || ch === '[') depth += 1;
    if (ch === ')' || ch === ']') depth -= 1;
    if (ch === '|' && depth === 0) {
      out.push(cur);
      cur = '';
      continue;
    }
    cur += ch;
  }
  out.push(cur);
  return out;
}

/** O parêntese de abertura em 0 fecha no último caractere? */
function wrapsAll(p: string): boolean {
  if (!p.startsWith('(') || !p.endsWith(')')) return false;
  let depth = 0;
  for (let i = 0; i < p.length; i += 1) {
    const ch = p[i]!;
    if (ch === '\\') {
      i += 1;
      continue;
    }
    if (ch === '(') depth += 1;
    if (ch === ')') depth -= 1;
    if (depth === 0 && i < p.length - 1) return false;
  }
  return true;
}

/** Um termo legível de uma alternativa, ou `null` quando ela tem sintaxe que não vira palavra. */
function readable(alt: string): string | null {
  let out = '';
  for (let i = 0; i < alt.length; i += 1) {
    const ch = alt[i]!;
    const next = alt[i + 1];
    if (ch === '\\') {
      if (next === undefined) return null;
      i += 1;
      if (next === 's') {
        out += ' ';
        if (alt[i + 1] && '*+?'.includes(alt[i + 1]!)) i += 1;
        continue;
      }
      if (next === 'b' || next === 'B') continue;
      if (/[a-zA-Z0-9]/.test(next)) return null;
      // Literal escapado; seguido de "?" ou "*" é opcional e sai do termo.
      if (alt[i + 1] === '?' || alt[i + 1] === '*') {
        i += 1;
        continue;
      }
      out += next;
      continue;
    }
    if (ch === '^' && i === 0) continue;
    if (ch === '$' && i === alt.length - 1) continue;
    if ('()[]{}|.^$*+?'.includes(ch)) return null;
    out += ch;
  }
  const term = out.replace(/\s+/g, ' ').trim();
  return term.length >= 2 ? term : null;
}

/** Os termos de uma lista de alternativas (`\b(ifood|rappi|uber\s*eats)`), sem repetição. */
export function regexTerms(pattern: string): string[] {
  let p = pattern.trim().replace(/^\(\?i\)/, '');
  while (wrapsAll(p)) p = p.slice(1, -1).replace(/^\?:/, '');
  // `\b(…)` e `(…)\b`: o limite de palavra em volta do grupo.
  const bare = p.replace(/^\\b/, '').replace(/\\b$/, '');
  if (wrapsAll(bare)) p = bare.slice(1, -1).replace(/^\?:/, '');
  const seen = new Set<string>();
  const terms: string[] = [];
  for (const alt of topLevelAlternatives(p)) {
    const term = readable(alt);
    if (!term) continue;
    const key = term.toLocaleLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    terms.push(term);
  }
  return terms;
}

type Condition = RuleInput['conditions'][number];

function plainCondition(c: Condition, field: string): string | null {
  if (c.op === 'regex' && typeof c.value === 'string') {
    const terms = regexTerms(c.value);
    if (!terms.length) return t('regras-condicao-padrao', { campo: field });
    return t('regras-condicao-termos', {
      campo: field,
      termos: terms.slice(0, TERMS_SHOWN).join(', '),
      mais: Math.max(0, terms.length - TERMS_SHOWN),
    });
  }
  // "Valor ≤ -0,01" e "Valor ≥ 0,01" são o jeito de a regra dizer "saída" e "entrada".
  if (c.field === 'amount' && typeof c.value === 'number') {
    if (c.op === 'lte' && c.value < 0 && c.value >= -1) return t('regras-condicao-saida');
    if (c.op === 'gte' && c.value > 0 && c.value <= 1) return t('regras-condicao-entrada');
  }
  return null;
}

/** As condições em palavras; o que não tem forma simples sai igual ao resumo exato. */
export function plainSummary(rule: Pick<RuleInput, 'conditions' | 'match_all'>): string {
  const joiner = rule.match_all === false ? ` ${t('regras-ou')} ` : ` ${t('regras-e')} `;
  return rule.conditions
    .map((c) => {
      return (
        plainCondition(c, RULE_FIELD[c.field]) ??
        conditionsSummary({ conditions: [c], match_all: true })
      );
    })
    .join(joiner);
}
