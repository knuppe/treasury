/**
 * Ícones de traço fino, desenhados à mão no grid de 16px (sem biblioteca
 * externa). Herdam a cor do texto (`currentColor`) e são decorativos por
 * padrão: o rótulo acessível fica no botão que os contém.
 */
import type { SVGProps } from 'react';

const PATHS = {
  lock: (
    <>
      <rect x="3" y="7" width="10" height="7" rx="1.5" />
      <path d="M5.25 7V5a2.75 2.75 0 0 1 5.5 0v2" />
      <path d="M8 10v1.5" />
    </>
  ),
  unlock: (
    <>
      <rect x="3" y="7" width="10" height="7" rx="1.5" />
      <path d="M5.25 7V5a2.75 2.75 0 0 1 5.3-1" />
    </>
  ),
  eye: (
    <>
      <path d="M1.5 8S3.9 3.5 8 3.5 14.5 8 14.5 8 12.1 12.5 8 12.5 1.5 8 1.5 8Z" />
      <circle cx="8" cy="8" r="2" />
    </>
  ),
  eyeOff: (
    <>
      <path d="M6.2 3.8A6.6 6.6 0 0 1 8 3.5C12.1 3.5 14.5 8 14.5 8a11.6 11.6 0 0 1-1.7 2.3M9.8 12.2a6.6 6.6 0 0 1-1.8.3C3.9 12.5 1.5 8 1.5 8a11.6 11.6 0 0 1 2.3-2.9" />
      <path d="M6.6 6.6a2 2 0 0 0 2.8 2.8" />
      <path d="m2.5 2.5 11 11" />
    </>
  ),
  sun: (
    <>
      <circle cx="8" cy="8" r="2.75" />
      <path d="M8 1.5v1.5M8 13v1.5M1.5 8H3M13 8h1.5M3.4 3.4l1.06 1.06M11.54 11.54l1.06 1.06M3.4 12.6l1.06-1.06M11.54 4.46l1.06-1.06" />
    </>
  ),
  moon: <path d="M13.2 9.6A5.5 5.5 0 1 1 6.4 2.8a4.4 4.4 0 0 0 6.8 6.8Z" />,
  contrast: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M8 2a6 6 0 0 1 0 12Z" fill="currentColor" stroke="none" />
    </>
  ),
  search: (
    <>
      <circle cx="7" cy="7" r="4.5" />
      <path d="m10.4 10.4 3.6 3.6" />
    </>
  ),
  plus: <path d="M8 3v10M3 8h10" />,
  pin: <path d="M6 2.5h4M7 2.5v4L4.5 9h7L9 6.5v-4M8 9v4.5" />,
  copy: (
    <>
      <rect x="5.5" y="5.5" width="8" height="8" rx="1.25" />
      <path d="M10.5 5.5V3.75c0-.7-.55-1.25-1.25-1.25h-5.5c-.7 0-1.25.55-1.25 1.25v5.5c0 .7.55 1.25 1.25 1.25H5.5" />
    </>
  ),
  clip: (
    <path d="M13 7.2 8.1 12.1a3 3 0 0 1-4.2-4.2l5-5a2 2 0 0 1 2.8 2.8l-4.9 4.9a1 1 0 0 1-1.4-1.4L9.9 4.7" />
  ),
  minus: <path d="M3 8h10" />,
  x: <path d="m4 4 8 8M12 4l-8 8" />,
  check: <path d="m3 8.5 3 3 7-7" />,
  chevronDown: <path d="m4 6 4 4 4-4" />,
  chevronUp: <path d="m4 10 4-4 4 4" />,
  chevronRight: <path d="m6 4 4 4-4 4" />,
  chevronLeft: <path d="m10 4-4 4 4 4" />,
  arrowUp: <path d="M8 13V3M4.5 6.5 8 3l3.5 3.5" />,
  arrowDown: <path d="M8 3v10M4.5 9.5 8 13l3.5-3.5" />,
  sort: <path d="m5 6 3-3 3 3M5 10l3 3 3-3" />,
  bell: (
    <>
      <path d="M4 11V7.5a4 4 0 0 1 8 0V11l1.2 1.5H2.8Z" />
      <path d="M6.6 14a1.5 1.5 0 0 0 2.8 0" />
    </>
  ),
  alert: (
    <>
      <path d="M8 2.2 14.3 13.5H1.7Z" />
      <path d="M8 6.5v3" />
      <path d="M8 11.6v.1" strokeWidth="2" />
    </>
  ),
  info: (
    <>
      <circle cx="8" cy="8" r="6.25" />
      <path d="M8 7.25v4" />
      <path d="M8 4.9v.1" strokeWidth="2" />
    </>
  ),
  key: (
    <>
      <circle cx="5.5" cy="10.5" r="2.75" />
      <path d="m7.5 8.5 5.5-5.5M11 5l1.75 1.75" />
    </>
  ),
  reticle: (
    <>
      <circle cx="8" cy="8" r="5" />
      <circle cx="8" cy="8" r="1.1" fill="currentColor" stroke="none" />
      <path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2" />
    </>
  ),
  reticleDashed: (
    <>
      <circle cx="8" cy="8" r="5" strokeDasharray="2 1.6" />
      <circle cx="8" cy="8" r="1.1" fill="currentColor" stroke="none" />
      <path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2" />
    </>
  ),
  refresh: (
    <>
      <path d="M13.5 8A5.5 5.5 0 1 1 11.9 4.1" />
      <path d="M12.5 1.8v2.7H9.8" />
    </>
  ),
  shield: <path d="M8 1.75 13 3.6v4.1c0 3.1-2.1 5.6-5 6.55C5.1 13.3 3 10.8 3 7.7V3.6Z" />,
  keyboard: (
    <>
      <rect x="1.5" y="4" width="13" height="8" rx="1.5" />
      <path d="M4 6.75h.01M6.5 6.75h.01M9 6.75h.01M11.5 6.75h.01M5 9.5h6" />
    </>
  ),
  history: (
    <>
      <path d="M2.6 8a5.4 5.4 0 1 0 1.6-3.8" />
      <path d="M2.5 2.2v2.6h2.6M8 5.2V8l1.9 1.4" />
    </>
  ),
  sliders: (
    <>
      <path d="M2.5 4.5h6M11.5 4.5h2M2.5 11.5h2M7.5 11.5h6" />
      <circle cx="10" cy="4.5" r="1.5" />
      <circle cx="6" cy="11.5" r="1.5" />
    </>
  ),
  maximize: <path d="M9.5 2.5h4v4M6.5 13.5h-4v-4M13.5 2.5 9 7M2.5 13.5 7 9" />,
  restore: <path d="M13.5 6.5h-4v-4M2.5 9.5h4v4M9.5 6.5 14 2M6.5 9.5 2 14" />,
  arrowRight: <path d="M3 8h10M9.5 4.5 13 8l-3.5 3.5" />,
  arrowLeft: <path d="M13 8H3M6.5 4.5 3 8l3.5 3.5" />,
  activity: <path d="M1.5 8.5h2.5l2-5 3.5 9 2-4h3" />,
  agent: (
    <>
      <rect x="2.5" y="4" width="11" height="8.5" rx="2" />
      <path d="M8 4V1.75M5.75 8.25h.01M10.25 8.25h.01" strokeWidth="1.75" />
    </>
  ),
} as const;

export type IconName = keyof typeof PATHS;

export interface IconProps extends Omit<SVGProps<SVGSVGElement>, 'name'> {
  name: IconName;
  size?: number;
  /** Texto para leitores de tela quando o ícone carrega significado sozinho. */
  title?: string;
}

export function Icon({ name, size = 16, title, className, ...rest }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden={title ? undefined : true}
      role={title ? 'img' : undefined}
      focusable="false"
      className={className}
      {...rest}
    >
      {title ? <title>{title}</title> : null}
      {PATHS[name]}
    </svg>
  );
}
