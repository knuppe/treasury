import { isValidElement, useId, type HTMLAttributes, type ReactNode, type Ref } from 'react';

import { cx } from '../cx';
import { AutoHelp } from '../help';
import { KeyValue, type KeyValueItem } from './KeyValue';
import { LiveIndicator } from './LiveIndicator';
import { WindowDots } from './WindowDots';

export interface InstrumentCardProps extends Omit<HTMLAttributes<HTMLElement>, 'title'> {
  /** Índice do painel (`02`). */
  index?: string;
  /** Título (caixa alta via CSS). */
  title: ReactNode;
  /** Ações no canto direito do cabeçalho. */
  actions?: ReactNode;
  /** Mostra `● AO VIVO` (ou o texto dado) no cabeçalho. */
  live?: boolean | string;
  /** Rodapé: pares chave/valor (`FONTE  BCB/SGS 433`) ou conteúdo livre. */
  footer?: readonly KeyValueItem[] | ReactNode;
  /** Nível do título no documento (padrão 2). */
  headingLevel?: 2 | 3 | 4;
  bodyClassName?: string;
  /** `false` tira o espaçamento interno do corpo (para tabelas e gráficos de borda a borda). */
  padded?: boolean;
  /** O `<section>` do card (ex.: o flash ao terminar uma tarefa, state/activity.ts). */
  ref?: Ref<HTMLElement>;
}

/**
 * Card de instrumento: cabeçalho com `● ● ●`, índice e título; corpo com a
 * medida; rodapé com a origem dos dados em pares chave/valor.
 */
export function InstrumentCard({
  index,
  title,
  actions,
  live,
  footer,
  headingLevel = 2,
  bodyClassName,
  padded = true,
  className,
  children,
  ...rest
}: InstrumentCardProps) {
  const titleId = useId();
  const Heading = `h${headingLevel}` as const;
  const footerIsPairs = Array.isArray(footer) && !isValidElement(footer);
  return (
    <section
      aria-labelledby={titleId}
      className={cx(
        'panel card-arrive flex min-w-0 flex-col rounded-card border border-panel',
        className,
      )}
      {...rest}
    >
      {/* O título quebra em até duas linhas em vez de virar "Retorno anual (XIR…" num card
          estreito; com uma linha, a altura é a mesma de sempre. */}
      <header className="flex min-h-[var(--card-head)] shrink-0 items-center gap-3 border-b border-line/80 px-4 py-1">
        <WindowDots />
        {index ? <span className="num text-[11px] font-medium text-accent">{index}</span> : null}
        <Heading id={titleId} className="label-caps line-clamp-2 min-w-0 break-words text-fg">
          {title}
        </Heading>
        <AutoHelp label={title} className="-ml-1.5" />
        <span className="flex-1" aria-hidden />
        {live ? <LiveIndicator label={typeof live === 'string' ? live : undefined} /> : null}
        {actions ? <div className="-mr-1.5 flex items-center gap-1">{actions}</div> : null}
      </header>
      <div className={cx('min-w-0 flex-1', padded && 'p-[var(--card-pad)]', bodyClassName)}>
        {children}
      </div>
      {footer ? (
        <footer className="rounded-b-[inherit] border-t border-line/80 bg-[color-mix(in_srgb,var(--surface-2)_35%,transparent)] px-4 py-2.5">
          {footerIsPairs ? (
            <KeyValue items={footer as readonly KeyValueItem[]} />
          ) : (
            (footer as ReactNode)
          )}
        </footer>
      ) : null}
    </section>
  );
}
