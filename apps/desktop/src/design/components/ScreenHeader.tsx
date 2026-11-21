import { useEffect, type ReactNode } from 'react';

import { cx } from '../cx';

export interface ScreenHeaderProps {
  /** Índice da tela (`03`). */
  index: string;
  /** Título (`Lançamentos`), mostrado em caixa alta. */
  title: string;
  description?: ReactNode;
  /** Ações à direita (botões). */
  actions?: ReactNode;
  /** Linha técnica acima do título (ex.: período ou data de atualização). */
  meta?: ReactNode;
  /** Atualiza o título da janela (padrão: sim). */
  setDocumentTitle?: boolean;
  className?: string;
}

/** Cabeçalho de tela: `04 · LANÇAMENTOS`, com descrição e ações. */
export function ScreenHeader({
  index,
  title,
  description,
  actions,
  meta,
  setDocumentTitle = true,
  className,
}: ScreenHeaderProps) {
  useEffect(() => {
    if (setDocumentTitle) document.title = `${title} · Treasury`;
  }, [title, setDocumentTitle]);

  return (
    <header
      className={cx(
        'flex flex-wrap items-end justify-between gap-x-6 gap-y-4 border-b border-line pb-6',
        className,
      )}
    >
      <div className="flex min-w-0 flex-col gap-1.5">
        {meta ? <div className="label-caps">{meta}</div> : null}
        <h1 className="flex items-baseline gap-3 text-[26px] leading-tight font-semibold tracking-[-0.018em] text-fg">
          <span className="num text-[20px] font-medium tracking-normal text-accent">{index}</span>
          <span aria-hidden className="text-[18px] font-normal text-line-strong">
            ·
          </span>
          <span className="min-w-0 truncate">{title}</span>
        </h1>
        {description ? (
          <p className="max-w-[72ch] text-[13.5px] leading-relaxed text-dim">{description}</p>
        ) : null}
      </div>
      {actions ? <div className="flex flex-wrap items-center gap-2">{actions}</div> : null}
    </header>
  );
}
