/**
 * "Ver como texto": o documento em Markdown, como os agentes de IA o leem
 * (`import.read_document`, docs/pdf-markdown.md). Serve para conferir se as tabelas e o texto
 * saíram certos antes de pedir uma análise.
 */
import { useEffect, useState } from 'react';

import { call } from '@/api/client';
import type { DocumentMarkdown } from '@/api/generated/DocumentMarkdown';
import { Button, Dialog, DialogClose, toast } from '@/design/components';
import { InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

/** Lê o documento inteiro, parte a parte (até 400 mil caracteres). */
async function readAll(filename: string, content_base64: string): Promise<DocumentMarkdown> {
  let offset = 0;
  let first: DocumentMarkdown | null = null;
  let text = '';
  for (let i = 0; i < 10; i++) {
    const part = await call<DocumentMarkdown>('import.read_document', {
      filename,
      content_base64,
      offset,
      limit: 40_000,
    });
    first ??= part;
    text += part.markdown;
    if (part.next_offset == null) break;
    offset = part.next_offset;
  }
  return { ...first!, markdown: text, offset: 0, next_offset: undefined };
}

export function DocumentTextDialog({
  file,
  onClose,
}: {
  /** O arquivo (nome e conteúdo em base64); `null` fecha. */
  file: { filename: string; content_base64: string } | null;
  onClose: () => void;
}) {
  const [doc, setDoc] = useState<DocumentMarkdown | null>(null);
  const [error, setError] = useState<unknown>(null);
  useEffect(() => {
    if (!file) return;
    let alive = true;
    void readAll(file.filename, file.content_base64).then(
      (d) => alive && setDoc(d),
      (e: unknown) => alive && setError(e),
    );
    return () => {
      alive = false;
    };
  }, [file]);
  const copy = () =>
    void navigator.clipboard
      .writeText(doc?.markdown ?? '')
      .then(() => toast({ tone: 'success', title: t('documento-texto-copiado') }));
  return (
    <Dialog
      open={file != null}
      onOpenChange={(o) => !o && onClose()}
      size="lg"
      title={t('documento-texto-titulo', { nome: file?.filename ?? '' })}
      description={
        doc
          ? [
              t('documento-texto-resumo', { paginas: doc.pages, total: doc.total_chars }),
              doc.engine === 'ocr' ? t('documento-texto-ocr') : '',
              doc.plain_pages.length
                ? t('documento-texto-paginas-corridas', { paginas: doc.plain_pages.join(', ') })
                : '',
            ]
              .filter(Boolean)
              .join(' ')
          : t('documento-texto-lendo')
      }
      footer={
        <>
          <Button variant="ghost" disabled={!doc} onClick={copy}>
            {t('documento-texto-copiar')}
          </Button>
          <DialogClose asChild>
            <Button variant="primary">{t('comum-fechar')}</Button>
          </DialogClose>
        </>
      }
    >
      <InlineError error={error} />
      {doc ? (
        <pre
          aria-label={t('documento-texto-conteudo')}
          className="max-h-[60vh] overflow-auto rounded-control border border-line bg-bg p-3 font-mono text-[11.5px] whitespace-pre-wrap text-fg"
        >
          {doc.markdown}
        </pre>
      ) : null}
    </Dialog>
  );
}
