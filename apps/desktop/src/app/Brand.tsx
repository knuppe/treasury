import { useId } from 'react';

import { cx } from '@/design/cx';

/**
 * Movimento da marca: `reveal` traça o sinal ao montar (abrir o cofre) e depois só
 * respira; `loop` traça em ciclo (telas de espera); `static` não anima. Passar o
 * mouse sobre a marca traça o sinal de novo. Com movimento reduzido, nada se move.
 */
export type BrandMotion = 'reveal' | 'loop' | 'static';

/** Marca do Treasury: moldura de instrumento com um sinal medido. */
export function BrandMark({
  size = 20,
  motion = 'reveal',
  className,
}: {
  size?: number;
  motion?: BrandMotion;
  className?: string;
}) {
  const id = useId().replace(/:/g, '');
  const clip = `brand-clip-${id}`;
  const beam = `brand-beam-${id}`;
  return (
    <svg
      aria-hidden
      width={size}
      height={size}
      viewBox="0 0 20 20"
      fill="none"
      data-motion={motion}
      className={cx('brand-mark shrink-0 overflow-visible', className)}
    >
      <defs>
        <clipPath id={clip}>
          <rect x="2" y="2" width="16" height="16" rx="2" />
        </clipPath>
        <linearGradient id={beam} x1="0" x2="1" y1="0" y2="0">
          <stop offset="0" stopColor="var(--accent)" stopOpacity="0" />
          <stop offset="0.7" stopColor="var(--accent)" stopOpacity="0.28" />
          <stop offset="1" stopColor="var(--accent)" stopOpacity="0" />
        </linearGradient>
      </defs>
      <rect
        className="brand-frame"
        x="1.5"
        y="1.5"
        width="17"
        height="17"
        rx="2.5"
        stroke="var(--line-strong)"
      />
      {motion === 'static' ? null : (
        <g clipPath={`url(#${clip})`}>
          <rect className="brand-scan" x="0" y="2" width="5" height="16" fill={`url(#${beam})`} />
        </g>
      )}
      <path
        className="brand-signal"
        pathLength={1}
        d="M4.5 13h2.2l1.7-5.2 2.4 7 1.7-4.3H15.5"
        stroke="var(--accent)"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      {motion === 'static' ? null : (
        <circle className="brand-dot" cx="15.5" cy="10.5" r="1.25" fill="var(--accent)" />
      )}
    </svg>
  );
}

export const APP_VERSION = '1.1.0';

/** Nome do app no estilo técnico: `TREASURY  v0.2.1`. */
export function Brand({ className, motion }: { className?: string; motion?: BrandMotion }) {
  return (
    <span className={cx('brand flex items-center gap-2.5', className)}>
      <BrandMark motion={motion} />
      {/* A marca não se traduz. */}
      <span className="font-mono text-[12px] font-semibold tracking-[0.22em] text-fg uppercase">
        {'Treasury'}
      </span>
      <span className="num text-[10px] text-dim">{`v${APP_VERSION}`}</span>
    </span>
  );
}
