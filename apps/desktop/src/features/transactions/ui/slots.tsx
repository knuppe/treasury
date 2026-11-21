/**
 * Frases do catálogo com componentes no meio (um `<Money>`, um `<Link>`): o argumento vai para o
 * `t()` como uma marca (`slot(0)`) e `withSlots` troca a marca pelo componente. A ordem das
 * palavras fica com a tradução, sem montar a frase com pedaços.
 *
 * ```tsx
 * withSlots(t('x-soma', { soma: slot(0), total: slot(1) }), [<Money … />, <Money … />])
 * ```
 */
import { Fragment, type ReactNode } from 'react';

// Separador invisível (U+2063) em volta do número: o pseudo-idioma não mexe em dígitos.
const MARK = '⁣';
const SPLIT = new RegExp(`${MARK}(\\d+)${MARK}`);

/** Marca do componente `i` dentro do texto traduzido. */
export function slot(i: number): string {
  return `${MARK}${i}${MARK}`;
}

/** Texto com as marcas de `slot()` trocadas pelos componentes. */
export function withSlots(text: string, nodes: readonly ReactNode[]): ReactNode {
  return text
    .split(SPLIT)
    .map((part, i) => <Fragment key={i}>{i % 2 === 1 ? nodes[Number(part)] : part}</Fragment>);
}
