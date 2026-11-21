import { cx } from '../cx';

/** Os três pontos `● ● ●` do cabeçalho dos cards de instrumento (decorativos). */
export function WindowDots({ className }: { className?: string }) {
  return (
    <span aria-hidden className={cx('inline-flex shrink-0 items-center gap-[5px]', className)}>
      <span className="size-[6px] rounded-full bg-line-strong/80" />
      <span className="size-[6px] rounded-full bg-line-strong/55" />
      <span className="size-[6px] rounded-full bg-line-strong/35" />
    </span>
  );
}
