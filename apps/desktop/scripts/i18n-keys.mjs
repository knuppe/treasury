#!/usr/bin/env node
// Gera src/i18n/keys.ts a partir dos catálogos em pt-BR (locales/pt-BR/*.ftl): a lista das chaves
// e os argumentos de cada uma, para o `t()` da interface ser tipado (chave errada ou argumento
// faltando vira erro do `tsc -b`). Rode `npm run i18n:keys` depois de mexer num `.ftl` do pt-BR.
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const dir = join(here, '../../../locales/pt-BR');
const out = join(here, '../src/i18n/keys.ts');

/** Mensagens e variáveis (`{ $nome }`) de um `.ftl`. */
export function parse(src) {
  const map = new Map();
  let current = null;
  for (const line of src.split('\n')) {
    if (/^[a-zA-Z]/.test(line)) {
      const k = line.split('=')[0].trim();
      current = k;
      if (!map.has(k)) map.set(k, new Set());
    }
    if (!current || line.startsWith('#')) continue;
    for (const m of line.matchAll(/\$([a-zA-Z0-9_-]+)/g)) map.get(current).add(m[1]);
  }
  return map;
}

const all = new Map();
for (const f of readdirSync(dir)
  .filter((f) => f.endsWith('.ftl'))
  .sort()) {
  for (const [k, v] of parse(readFileSync(join(dir, f), 'utf8'))) all.set(k, v);
}
const keys = [...all.keys()].sort();
const lines = [
  '// Gerado por scripts/i18n-keys.mjs a partir de locales/pt-BR/*.ftl. Não edite à mão.',
  '',
  '/** Argumentos de cada mensagem (`never`: sem argumentos). */',
  'export interface Messages {',
  ...keys.map((k) => {
    const vars = [...all.get(k)].sort();
    return vars.length
      ? `  '${k}': { ${vars.map((v) => `${/^[a-z_][a-z0-9_]*$/i.test(v) ? v : `'${v}'`}: string | number`).join('; ')} };`
      : `  '${k}': never;`;
  }),
  '}',
  '',
  'export type MessageKey = keyof Messages;',
  '',
  '/** As chaves, na ordem (o teste confere com os catálogos). */',
  'export const KEYS: readonly MessageKey[] = [',
  ...keys.map((k) => `  '${k}',`),
  '];',
  '',
];
writeFileSync(out, lines.join('\n'));
console.log(`${keys.length} chaves em src/i18n/keys.ts`);
