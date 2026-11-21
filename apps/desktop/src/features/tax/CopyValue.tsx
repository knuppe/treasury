/**
 * Valor pronto para a declaração: o número formatado (escondido no modo privacidade) e um
 * botão que copia `1.234,56` (ou o texto dado) para colar no programa da Receita.
 */
import { useState } from 'react';

import { Money, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { copyText } from './transform';

export function copyToClipboard(text: string): Promise<boolean> {
  const clip = typeof navigator !== 'undefined' ? navigator.clipboard : undefined;
  if (!clip?.writeText) return Promise.resolve(false);
  return clip.writeText(text).then(
    () => true,
    () => false,
  );
}

export function CopyButton({
  text,
  label,
  className,
}: {
  text: string;
  /** O que está sendo copiado (nome acessível: "Copiar <label>", pelo catálogo). */
  label: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      aria-label={t('ir-copiar-algo', { algo: label })}
      title={t('ir-copiar-algo', { algo: text })}
      onClick={() =>
        void copyToClipboard(text).then((ok) => {
          if (ok) {
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
          } else {
            toast({
              tone: 'error',
              title: t('componentes-nao-copiou'),
              description: t('ir-copiar-a-mao'),
            });
          }
        })
      }
      className={cx(
        'label-caps rounded-control border border-line px-1.5 py-0.5 transition-colors hover:border-accent hover:text-accent focus-visible:outline focus-visible:outline-accent',
        copied && 'border-positive/60 text-positive',
        className,
      )}
    >
      {copied ? t('ir-copiado') : t('ir-copiar')}
    </button>
  );
}

export function CopyValue({
  cents,
  label,
  className,
}: {
  cents: number;
  label: string;
  className?: string;
}) {
  return (
    <span className={cx('inline-flex items-center justify-end gap-2', className)}>
      <Money cents={cents} />
      <CopyButton text={copyText(cents)} label={label} />
    </span>
  );
}
