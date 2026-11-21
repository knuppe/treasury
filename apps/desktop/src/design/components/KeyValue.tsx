import type { ReactNode } from 'react';

import { cx } from '../cx';
import { AutoHelp } from '../help';

export interface KeyValueItem {
  label: ReactNode;
  value: ReactNode;
}

export interface KeyValueProps {
  items: readonly KeyValueItem[];
  /** `inline`: pares lado a lado (rodapé de card); `stacked`: duas colunas alinhadas. */
  layout?: 'inline' | 'stacked';
  className?: string;
}

/** Pares chave/valor de especificação técnica: `FONTE  BCB/SGS 433`. */
export function KeyValue({ items, layout = 'inline', className }: KeyValueProps) {
  if (layout === 'stacked') {
    return (
      <dl className={cx('grid grid-cols-[max-content_1fr] gap-x-6 gap-y-1.5', className)}>
        {items.map((item, i) => (
          <div key={i} className="contents">
            <dt className="label-caps inline-flex items-center gap-1.5 self-baseline">
              {item.label}
              <AutoHelp label={item.label} />
            </dt>
            <dd className="num min-w-0 self-baseline text-[12px] text-fg">{item.value}</dd>
          </div>
        ))}
      </dl>
    );
  }
  return (
    <dl className={cx('flex flex-wrap items-baseline gap-x-6 gap-y-1', className)}>
      {items.map((item, i) => (
        <div key={i} className="flex items-baseline gap-2">
          <dt className="label-caps inline-flex items-center gap-1.5">
            {item.label}
            <AutoHelp label={item.label} />
          </dt>
          <dd className="num text-[11px] tracking-[0.04em] text-fg">{item.value}</dd>
        </div>
      ))}
    </dl>
  );
}
