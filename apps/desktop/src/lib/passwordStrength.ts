/**
 * Medidor heurístico de força de senha, sem dependência externa.
 *
 * Estima a entropia (em bits) percorrendo a senha em pedaços e cobrando pouco
 * pelo que um atacante adivinha cedo: palavras e senhas comuns (inclusive com
 * trocas como 4→a), nomes da família, anos, repetições, sequências (abc, 123)
 * e linhas do teclado (qwerty). Palavras pronunciáveis valem como palavras de
 * um dicionário (~16 bits), não como letras aleatórias, para não superestimar
 * frases. É uma estimativa conservadora para orientar, não uma garantia.
 */

import { t } from '@/i18n';

export type StrengthScore = 0 | 1 | 2 | 3 | 4;

export interface PasswordStrength {
  score: StrengthScore;
  /** Entropia estimada, em bits. */
  bits: number;
  label: string;
  /** Sugestões no idioma do app, da mais para a menos importante. */
  suggestions: string[];
}

/** Nome da nota no idioma do app (0 = muito fraca, 4 = forte). */
export function strengthLabel(score: StrengthScore): string {
  const labels = [
    t('cofre-forca-muito-fraca'),
    t('cofre-forca-fraca'),
    t('cofre-forca-razoavel'),
    t('cofre-forca-boa'),
    t('cofre-forca-forte'),
  ] as const;
  return labels[score];
}

/** Limites de bits para cada nota (abaixo de 28 = muito fraca). */
const THRESHOLDS = [28, 40, 56, 72] as const;

/** Força mínima exigida para a senha mestra. */
export const MIN_MASTER_SCORE: StrengthScore = 2;
export const MIN_MASTER_LENGTH = 10;

const COMMON = [
  // senhas e padrões mais usados
  'password',
  'senha',
  'minhasenha',
  'qwerty',
  'abc',
  'admin',
  'iloveyou',
  'letmein',
  'welcome',
  'master',
  'dragon',
  'monkey',
  'segredo',
  'mudar',
  'teste',
  'acesso',
  'entrar',
  'login',
  // tema do app
  'treasury',
  'tesouro',
  'cofre',
  'banco',
  'dinheiro',
  'familia',
  'financas',
  // palavras e nomes muito comuns em senhas brasileiras
  'brasil',
  'amor',
  'deus',
  'jesus',
  'maria',
  'joao',
  'jose',
  'minha',
  'meu',
  'casa',
  'vida',
  'feliz',
  'princesa',
  'futebol',
  'flamengo',
  'corinthians',
  'palmeiras',
  'saopaulo',
  'vasco',
  'gremio',
  'cruzeiro',
  'santos',
  'botafogo',
  'fluminense',
  'internacional',
  'janeiro',
  'fevereiro',
  'marco',
  'abril',
  'maio',
  'junho',
  'julho',
  'agosto',
  'setembro',
  'outubro',
  'novembro',
  'dezembro',
];

const KEYBOARD_ROWS = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm', '1qaz2wsx3edc', 'qazwsxedc'];

const LEET: Record<string, string> = {
  '4': 'a',
  '@': 'a',
  '3': 'e',
  '1': 'i',
  '!': 'i',
  '0': 'o',
  '5': 's',
  $: 's',
  '7': 't',
  '+': 't',
};

function stripAccents(s: string): string {
  return s.normalize('NFD').replace(/[\u0300-\u036f]/g, '');
}

function poolSize(password: string): number {
  let pool = 0;
  if (/[a-z]/.test(password)) pool += 26;
  if (/[A-Z]/.test(password)) pool += 26;
  if (/\d/.test(password)) pool += 10;
  if (/[ -/:-@[-`{-~]/.test(password)) pool += 33;
  if (/\P{ASCII}/u.test(password)) pool += 64;
  return Math.max(pool, 10);
}

function isLetter(ch: string | undefined): boolean {
  return ch !== undefined && /\p{L}/u.test(ch);
}

/** Pedaço de letras "pronunciável" (parece palavra), e não letras ao acaso. */
function isWordLike(run: string): boolean {
  const s = stripAccents(run.toLowerCase());
  const vowels = (s.match(/[aeiouy]/g) ?? []).length;
  if (vowels / s.length < 0.25) return false;
  return !/[^aeiouy]{4,}/.test(s);
}

function wordBits(length: number): number {
  return Math.min(length * 4.7, 16) + Math.max(0, length - 12) * 2;
}

function longestMatch(haystack: string, at: number, words: readonly string[]): number {
  let best = 0;
  for (const word of words) {
    if (word.length > best && word.length >= 3 && haystack.startsWith(word, at)) best = word.length;
  }
  return best;
}

function runLength(s: string, at: number, step: (a: number, b: number) => boolean): number {
  let len = 1;
  while (at + len < s.length && step(s.charCodeAt(at + len - 1), s.charCodeAt(at + len))) len += 1;
  return len;
}

function keyboardRun(lower: string, at: number): number {
  let best = 0;
  for (const row of KEYBOARD_ROWS) {
    for (const r of [row, [...row].reverse().join('')]) {
      let len = 0;
      const start = r.indexOf(lower[at] ?? '');
      if (start < 0) continue;
      while (at + len < lower.length && r[start + len] === lower[at + len]) len += 1;
      best = Math.max(best, len);
    }
  }
  return best;
}

/**
 * Estima a força de uma senha. `context` são palavras que um atacante saberia
 * (nome da família, titulares) e que valem quase nada dentro da senha.
 */
export function estimatePasswordStrength(
  password: string,
  context: readonly string[] = [],
): PasswordStrength {
  if (password.length === 0) {
    return { score: 0, bits: 0, label: strengthLabel(0), suggestions: [] };
  }

  const lower = stripAccents(password.toLowerCase());
  const leet = [...lower].map((ch) => LEET[ch] ?? ch).join('');
  const contextWords = context
    .flatMap((c) => stripAccents(c.toLowerCase()).split(/[^a-z0-9]+/))
    .filter((w) => w.length >= 3);
  const charBits = Math.log2(poolSize(password));

  const flags = {
    common: false,
    context: false,
    year: false,
    repeat: false,
    sequence: false,
  };
  let bits = 0;
  let i = 0;

  while (i < password.length) {
    const ctx = Math.max(longestMatch(lower, i, contextWords), longestMatch(leet, i, contextWords));
    if (ctx > 0) {
      bits += 3;
      flags.context = true;
      i += ctx;
      continue;
    }

    const common = Math.max(longestMatch(lower, i, COMMON), longestMatch(leet, i, COMMON));
    if (common > 0) {
      const piece = password.slice(i, i + common);
      bits += Math.log2(COMMON.length * 4) + (/[A-Z]/.test(piece) ? 1 : 0);
      flags.common = true;
      i += common;
      continue;
    }

    if (/^(19|20)\d\d/.test(password.slice(i, i + 4))) {
      bits += 6;
      flags.year = true;
      i += 4;
      continue;
    }

    const repeat = runLength(password, i, (a, b) => a === b);
    if (repeat >= 3) {
      bits += charBits + (repeat - 1) * 0.5;
      flags.repeat = true;
      i += repeat;
      continue;
    }

    const up = runLength(lower, i, (a, b) => b - a === 1);
    const down = runLength(lower, i, (a, b) => a - b === 1);
    const sequence = Math.max(up, down);
    if (sequence >= 3) {
      bits += charBits + (sequence - 1);
      flags.sequence = true;
      i += sequence;
      continue;
    }

    const keyboard = keyboardRun(lower, i);
    if (keyboard >= 4) {
      bits += charBits + (keyboard - 1) * 1.5;
      flags.sequence = true;
      i += keyboard;
      continue;
    }

    if (isLetter(password[i])) {
      let end = i;
      while (end < password.length && isLetter(password[end])) end += 1;
      const run = password.slice(i, end);
      if (run.length >= 3 && isWordLike(run)) {
        // Maiúscula só na primeira letra é previsível; no meio vale um pouco mais.
        const inner = /\p{Lu}/u.test(run.slice(1)) ? 2 : /\p{Lu}/u.test(run[0] ?? '') ? 1 : 0;
        bits += wordBits(run.length) + inner;
        i = end;
        continue;
      }
    }

    bits += charBits;
    i += 1;
  }

  // Separadores entre palavras (espaço, hífen) quase não somam: são previsíveis.
  const separators = (password.match(/[\s\-_.]/g) ?? []).length;
  bits -= separators * Math.max(0, charBits - 1.5);
  bits = Math.max(0, Math.round(bits));

  let score: StrengthScore = 0;
  for (const limit of THRESHOLDS) if (bits >= limit) score = (score + 1) as StrengthScore;
  if (password.length < 8) score = 0;
  else if (password.length < MIN_MASTER_LENGTH) score = Math.min(score, 1) as StrengthScore;

  const words = password.split(/[\s\-_.]+/).filter((w) => w.length >= 3).length;
  const suggestions: string[] = [];
  if (password.length < 12) {
    suggestions.push(t('cofre-sugestao-longa'));
  }
  if (flags.context) suggestions.push(t('cofre-sugestao-nomes'));
  if (flags.common) suggestions.push(t('cofre-sugestao-comuns'));
  if (flags.year) suggestions.push(t('cofre-sugestao-anos'));
  if (flags.sequence) suggestions.push(t('cofre-sugestao-sequencias'));
  if (flags.repeat) suggestions.push(t('cofre-sugestao-repetir'));
  if (score < 3 && words < 4) suggestions.push(t('cofre-sugestao-frase'));

  return { score, bits, label: strengthLabel(score), suggestions };
}
