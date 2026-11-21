/**
 * Mensagens do catálogo com elementos no meio (link, valor, código) e trechos em **negrito**:
 * cada argumento que não é texto nem número entra no lugar do `{ $nome }` dele, na ordem que o
 * idioma pede, sem quebrar a frase em pedaços traduzidos separados.
 */
import { Fragment, type ReactNode } from 'react';

import { t } from '@/i18n';
import type { MessageKey, Messages } from '@/i18n/keys';

type RichArgs<K extends MessageKey> = { [P in keyof Messages[K]]: ReactNode };

/** Marca do lugar de cada elemento no texto traduzido (caracteres de uso privado). */
const SLOT = /\uE000(\d+)\uE001/;

const translate = t as unknown as (key: string, args: Record<string, string | number>) => string;

export function tRich<K extends MessageKey>(
  key: K,
  args: RichArgs<K>,
  bold: (content: ReactNode) => ReactNode = (content) => <strong>{content}</strong>,
): ReactNode {
  const nodes: ReactNode[] = [];
  const plain: Record<string, string | number> = {};
  for (const [name, value] of Object.entries(args as Record<string, ReactNode>)) {
    if (typeof value === 'string' || typeof value === 'number') plain[name] = value;
    else {
      plain[name] = `\uE000${nodes.length}\uE001`;
      nodes.push(value);
    }
  }
  return translate(key, plain)
    .split('**')
    .map((chunk, b) => {
      const parts = chunk
        .split(SLOT)
        .map((p, i) => (i % 2 ? <Fragment key={i}>{nodes[Number(p)]}</Fragment> : p));
      return <Fragment key={b}>{b % 2 ? bold(parts) : parts}</Fragment>;
    });
}
