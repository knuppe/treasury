/**
 * Texto do livro-caixa no mock: as mesmas normalizações do núcleo (sem acentos,
 * minúsculas, só letras e dígitos) para busca, regras e deduplicação.
 */

const ACCENTS: Record<string, string> = {
  á: 'a', à: 'a', â: 'a', ã: 'a', ä: 'a',
  Á: 'A', À: 'A', Â: 'A', Ã: 'A', Ä: 'A',
  é: 'e', è: 'e', ê: 'e', ë: 'e',
  É: 'E', È: 'E', Ê: 'E', Ë: 'E',
  í: 'i', ì: 'i', î: 'i', ï: 'i',
  Í: 'I', Ì: 'I', Î: 'I', Ï: 'I',
  ó: 'o', ò: 'o', ô: 'o', õ: 'o', ö: 'o',
  Ó: 'O', Ò: 'O', Ô: 'O', Õ: 'O', Ö: 'O',
  ú: 'u', ù: 'u', û: 'u', ü: 'u',
  Ú: 'U', Ù: 'U', Û: 'U', Ü: 'U',
  ç: 'c', Ç: 'C', ñ: 'n', Ñ: 'N',
}; // prettier-ignore

export function foldAccent(s: string): string {
  let out = '';
  for (const ch of s) out += ACCENTS[ch] ?? ch;
  return out;
}

/** Busca de lançamentos: minúsculas e sem acentos (mantém pontuação). */
export function searchFold(s: string): string {
  return foldAccent(s).toLowerCase();
}

const ALNUM = /[\p{L}\p{N}]/u;

/** Como `import::text::fold`: só letras e dígitos, separados por um espaço. */
export function fold(s: string): string {
  let out = '';
  let space = true;
  for (const ch of foldAccent(s)) {
    if (ALNUM.test(ch)) {
      out += ch.toLowerCase();
      space = false;
    } else if (!space) {
      out += ' ';
      space = true;
    }
  }
  return out.endsWith(' ') ? out.slice(0, -1) : out;
}

/** Como `transactions::normalize_description`: tira números longos (datas, códigos). */
export function normalizeDescription(s: string): string {
  return fold(s)
    .split(' ')
    .filter((w) => !(w.length >= 6 && /^\d+$/.test(w)))
    .join(' ')
    .trim();
}

/** Impressão digital de deduplicação (o núcleo usa SHA-256; aqui um hash estável basta). */
export function fingerprint(date: string, amount: number, description: string): string {
  return hashHex(`${date}|${amount}|${normalizeDescription(description)}`, 24);
}

/** Hash FNV-1a repetido até `length` dígitos hexadecimais (não criptográfico). */
export function hashHex(text: string, length = 64): string {
  let out = '';
  let seed = 0x811c9dc5;
  while (out.length < length) {
    let h = seed;
    for (let i = 0; i < text.length; i++) {
      h ^= text.charCodeAt(i);
      h = Math.imul(h, 0x01000193) >>> 0;
    }
    out += h.toString(16).padStart(8, '0');
    seed = (seed + 0x9e3779b9) >>> 0;
  }
  return out.slice(0, length);
}

/** Semelhança de descrições (Dice sobre bigramas), como `import::text::similarity`. */
export function similarity(a: string, b: string): number {
  const na = normalizeDescription(a);
  const nb = normalizeDescription(b);
  if (!na || !nb) return 0;
  if (na === nb) return 1;
  const [short, long] = na.length <= nb.length ? [na, nb] : [nb, na];
  if (short.length >= 5 && long.includes(short)) return 0.9;
  const bigrams = (s: string) => {
    const cs = [...s.replace(/ /g, '')];
    return cs.slice(1).map((c, i) => `${cs[i]}${c}`);
  };
  const ba = bigrams(na);
  const bb = bigrams(nb);
  if (ba.length === 0 || bb.length === 0) return 0;
  const pool = new Map<string, number>();
  for (const g of bb) pool.set(g, (pool.get(g) ?? 0) + 1);
  let common = 0;
  for (const g of ba) {
    const n = pool.get(g) ?? 0;
    if (n > 0) {
      common++;
      pool.set(g, n - 1);
    }
  }
  return (2 * common) / (ba.length + bb.length);
}

export function verySimilar(a: string, b: string): boolean {
  return similarity(a, b) >= 0.8;
}

/** Valor em centavos no formato do núcleo para mensagens (`R$ 1.234,56`). */
export function brl(cents: number, currency = 'BRL'): string {
  const neg = cents < 0;
  const abs = Math.abs(cents);
  const whole = Math.floor(abs / 100)
    .toString()
    .replace(/\B(?=(\d{3})+(?!\d))/g, '.');
  const frac = String(abs % 100).padStart(2, '0');
  return `${neg ? '-' : ''}${currency === 'BRL' ? 'R$' : currency} ${whole},${frac}`;
}
