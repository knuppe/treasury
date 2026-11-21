/**
 * Ler cupom fiscal (docs/receipts.md, docs/ocr-receipts.md): uma área só para soltar ou escolher
 * a foto, o PDF ou o XML do cupom, ou colar o link do QR. O app descobre o que é e mostra as
 * etapas com o andamento (QR → SEFAZ ou AFIP → leitura da foto → conferência), o resultado
 * (quantos itens, se o total confere, o próximo passo) e, quando algo dá errado, o que fazer.
 *
 * O resultado abre a revisão de sempre: a prévia do cupom (NFC-e, PDF, XML) ou a compra
 * preenchida (foto sem QR, nota da Argentina).
 */
import { useEffect, useRef, useState, type DragEvent, type ReactNode } from 'react';

import { call, isApiError } from '@/api/client';
import type { ForeignReceipt } from '@/api/generated/ForeignReceipt';
import type { PhotoReading } from '@/api/generated/PhotoReading';
import type { ReceiptArgs } from '@/api/generated/ReceiptArgs';
import type { ReceiptPreview } from '@/api/generated/ReceiptPreview';
import { isForeignReceiptLink } from '@/api/hooks/receipts';
import { Button, Icon, Input, InstrumentCard, Money } from '@/design/components';
import { cx } from '@/design/cx';
import { PurchaseDialog } from '@/features/food/PurchaseDialog';
import { TextArea } from '@/features/transactions/ui/controls';
import { fileToBase64 } from '@/features/transactions/ui/hooks';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';

import { photoArgs } from './photo';
import { readQr } from './qr';
import { canOpenSefaz, captureSefaz, onSefazPage, openSefaz } from './sefaz';

type StepId = 'qr' | 'link' | 'sefaz' | 'afip' | 'ocr' | 'file' | 'text' | 'check';
type StepState = 'run' | 'ok' | 'warn' | 'skip' | 'fail';
interface Step {
  id: StepId;
  state: StepState;
  detail?: string;
}

type Outcome =
  | { kind: 'preview'; args: ReceiptArgs; preview: ReceiptPreview }
  | { kind: 'photo'; file: File; reading: PhotoReading }
  | { kind: 'foreign'; receipt: ForeignReceipt };

interface Failure {
  message: string;
  /** Link da SEFAZ que falhou (para abrir a consulta no app). */
  url?: string;
  /** Foto que tinha o QR (para ler pelo texto, sem a SEFAZ). */
  file?: File;
  conflict?: boolean;
}

const GOV_BR = /^https?:\/\/([^/?#]+\.)?gov\.br([/?#]|$)/i;
const LINK = /^https?:\/\/\S+$/i;

const STEP_KEY: Record<StepId, string> = {
  qr: 'cupons-etapa-qr',
  link: 'cupons-etapa-link',
  sefaz: 'cupons-etapa-sefaz',
  afip: 'cupons-etapa-afip',
  ocr: 'cupons-etapa-ocr',
  file: 'cupons-etapa-arquivo',
  text: 'cupons-etapa-texto',
  check: 'cupons-etapa-conferir',
};

const tk = (k: string, a?: Record<string, string | number>) =>
  (t as unknown as (k: string, a?: Record<string, string | number>) => string)(k, a);

function message(e: unknown): string {
  if (isApiError(e)) return e.message;
  return e instanceof Error ? e.message : String(e);
}

function isImage(f: File): boolean {
  return f.type.startsWith('image/') || /\.(jpe?g|png|heic|heif|webp|gif|bmp|tiff?)$/i.test(f.name);
}

/** Soma dos itens da prévia contra o valor pago. */
function previewCheck(p: ReceiptPreview): { ok: boolean; diff: number } {
  const sum = p.items.reduce((s, i) => s + i.total, 0) - p.parsed.discount;
  const diff = p.parsed.paid - sum;
  return { ok: Math.abs(diff) <= 1, diff };
}

function StepIcon({ state }: { state: StepState }) {
  if (state === 'run')
    return (
      <span
        aria-hidden
        className="inline-block size-3.5 animate-spin rounded-full border-2 border-accent border-t-transparent"
      />
    );
  const icon = { ok: 'check', warn: 'alert', skip: 'minus', fail: 'x' } as const;
  return (
    <Icon
      name={icon[state]}
      size={14}
      className={cx(
        state === 'ok' && 'text-positive',
        state === 'warn' && 'text-warning',
        state === 'skip' && 'text-dim',
        state === 'fail' && 'text-negative',
      )}
    />
  );
}

export function ReceiptReader({
  brazil = true,
  onSaved,
}: {
  brazil?: boolean;
  /** Depois de gravar uma compra por aqui (a porta de entrada fecha). */
  onSaved?: () => void;
}) {
  /** Itens colados como texto (um por linha), lidos pelo mesmo leitor da foto. */
  const [pasteText, setPasteText] = useState('');
  const [pasting, setPasting] = useState(false);
  const [pasted, setPasted] = useState<PhotoReading | null>(null);
  const [pasteError, setPasteError] = useState<string | null>(null);
  const readPasted = () => {
    setPasting(true);
    setPasteError(null);
    void call<PhotoReading>('receipts.read_text', { text: pasteText })
      .then(setPasted)
      .catch((e: unknown) => setPasteError(message(e)))
      .finally(() => setPasting(false));
  };
  const fileInput = useRef<HTMLInputElement>(null);
  const [url, setUrl] = useState('');
  const [steps, setSteps] = useState<Step[]>([]);
  const [busy, setBusy] = useState(false);
  const [outcome, setOutcome] = useState<Outcome | null>(null);
  const [failure, setFailure] = useState<Failure | null>(null);
  const [open, setOpen] = useState<'review' | 'manual' | null>(null);
  const [askingAi, setAskingAi] = useState(false);
  // Cada leitura nova da foto (com a IA, por exemplo) recria a compra preenchida.
  const [round, setRound] = useState(0);
  const [dragging, setDragging] = useState(false);
  const [showText, setShowText] = useState(false);
  const [text, setText] = useState('');
  const [sefazOpen, setSefazOpen] = useState(false);

  const put = (id: StepId, state: StepState, detail?: string) =>
    setSteps((s) => {
      const i = s.findIndex((x) => x.id === id);
      const next = { id, state, ...(detail ? { detail } : {}) };
      if (i < 0) return [...s, next];
      const copy = [...s];
      copy[i] = next;
      return copy;
    });

  const begin = () => {
    setSteps([]);
    setOutcome(null);
    setFailure(null);
    setOpen(null);
    setBusy(true);
  };

  /** Prévia do cupom (link, texto, PDF ou XML) e a conferência. */
  const preview = async (args: ReceiptArgs, step: StepId, file?: File) => {
    put(step, 'run');
    try {
      const p = await call<ReceiptPreview>('receipts.preview', args);
      put(step, 'ok', p.parsed.store_name ?? undefined);
      const c = previewCheck(p);
      put(
        'check',
        c.ok ? 'ok' : 'warn',
        c.ok
          ? tk('cupons-etapa-conferir-ok', { n: p.items.length })
          : tk('cupons-etapa-conferir-diferenca', { n: p.items.length }),
      );
      setOutcome({ kind: 'preview', args, preview: p });
    } catch (e) {
      put(step, 'fail');
      const conflict = isApiError(e) && e.code === 'conflict';
      setFailure({
        message: message(e),
        ...(args.url && GOV_BR.test(args.url) ? { url: args.url } : {}),
        ...(file ? { file } : {}),
        conflict,
      });
    }
  };

  /** Foto sem QR (ou com o QR que não abriu): o texto da foto vira a compra. */
  const ocr = async (file: File, useAi = false) => {
    put('ocr', 'run', useAi ? tk('cupons-etapa-ocr-ia') : undefined);
    try {
      const reading = await call<PhotoReading>('receipts.read_photo', await photoArgs(file, useAi));
      put('ocr', 'ok', tk('cupons-etapa-ocr-itens', { n: reading.reading.items.length }));
      const tier = reading.tier;
      put(
        'check',
        tier === 'unbalanced' ? 'warn' : 'ok',
        tier === 'unbalanced' ? t('cupons-foto-faixa-nao-fechou') : t('cupons-foto-faixa-fechou'),
      );
      setOutcome({ kind: 'photo', file, reading });
      setRound((r) => r + 1);
    } catch (e) {
      put('ocr', 'fail');
      setFailure({ message: message(e) });
    }
  };

  const link = async (raw: string, file?: File) => {
    const u = raw.trim();
    if (isForeignReceiptLink(u)) {
      put('afip', 'run');
      try {
        const r = await call<ForeignReceipt>('receipts.foreign_qr', { url: u });
        put('afip', 'ok', `${r.document} ${r.number}`);
        put('check', 'warn', t('cupons-etapa-afip-sem-itens'));
        setOutcome({ kind: 'foreign', receipt: r });
      } catch (e) {
        put('afip', 'fail');
        setFailure({ message: message(e), ...(file ? { file } : {}) });
      }
      return;
    }
    if (!brazil) {
      put('link', 'fail');
      setFailure({ message: t('cupons-erro-link-fora'), ...(file ? { file } : {}) });
      return;
    }
    await preview({ url: u }, 'sefaz', file);
  };

  const run = async (input: { file?: File; url?: string; text?: string }) => {
    begin();
    try {
      if (input.text) {
        await preview({ text: input.text, date: todayIso() }, 'text');
      } else if (input.url) {
        // O QR de Portugal é um texto, não um link.
        put(
          'link',
          'ok',
          URL.canParse(input.url.trim()) ? new URL(input.url.trim()).host : undefined,
        );
        await link(input.url);
      } else if (input.file && isImage(input.file)) {
        const f = input.file;
        put('qr', 'run');
        const found = await readQr(f).catch(() => null);
        if (found) {
          put(
            'qr',
            'ok',
            (() => {
              try {
                return new URL(found).host;
              } catch {
                return undefined;
              }
            })(),
          );
          setUrl(found);
          await link(found, f);
        } else {
          put('qr', 'skip', t('cupons-etapa-qr-nao-achado'));
          await ocr(f);
        }
      } else if (input.file) {
        if (!brazil) {
          put('file', 'fail');
          setFailure({ message: t('cupons-erro-arquivo-fora') });
        } else {
          const f = input.file;
          await preview(
            { files: [{ filename: f.name, content_base64: await fileToBase64(f) }] },
            'file',
          );
        }
      }
    } catch (e) {
      setFailure({ message: message(e) });
    } finally {
      setBusy(false);
    }
  };

  // Texto da página do cupom aberta na janela da consulta (depois do captcha).
  useEffect(
    () =>
      onSefazPage((pageText) => {
        setSefazOpen(false);
        void run({ text: pageText });
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps -- `run` muda a cada render.
    [],
  );

  const pick = (files: FileList | null | undefined) => {
    const f = files?.[0];
    if (f) void run({ file: f });
  };
  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    setDragging(false);
    if (e.dataTransfer.files.length) return pick(e.dataTransfer.files);
    const dropped = e.dataTransfer.getData('text/plain').trim();
    if (LINK.test(dropped)) {
      setUrl(dropped);
      void run({ url: dropped });
    }
  };

  const reset = () => {
    setSteps([]);
    setOutcome(null);
    setFailure(null);
    setOpen(null);
  };

  const summary = outcomeSummary(outcome);
  return (
    <InstrumentCard index="06.C1" title={brazil ? t('cupons-ler-titulo') : t('cupons-foto-titulo')}>
      <div className="flex flex-col gap-4">
        <div
          role="group"
          aria-label={t('cupons-area')}
          onDragOver={(e) => {
            e.preventDefault();
            setDragging(true);
          }}
          onDragLeave={() => setDragging(false)}
          onDrop={onDrop}
          onPaste={(e) => {
            const f = e.clipboardData.files?.[0];
            if (f) {
              e.preventDefault();
              void run({ file: f });
            }
          }}
          className={cx(
            'flex flex-col items-center gap-3 rounded-card border border-dashed px-4 py-6 text-center transition-colors',
            dragging ? 'border-accent bg-accent-soft/40' : 'border-line-strong',
          )}
        >
          <Icon name="clip" size={20} className="text-dim" />
          <p className="text-[14px] text-fg">
            {brazil ? t('cupons-area-titulo') : t('cupons-area-titulo-foto')}
          </p>
          <p className="max-w-[60ch] text-[12.5px] text-dim">
            {brazil ? t('cupons-area-dica') : t('cupons-area-dica-foto')}
          </p>
          <input
            ref={fileInput}
            type="file"
            className="sr-only"
            tabIndex={-1}
            aria-label={t('cupons-arquivo')}
            accept={
              brazil ? 'image/*,application/pdf,.pdf,application/xml,text/xml,.xml' : 'image/*'
            }
            onChange={(e) => {
              pick(e.target.files);
              e.target.value = '';
            }}
          />
          <Button
            variant="secondary"
            icon={<Icon name="plus" size={13} />}
            disabled={busy}
            onClick={() => fileInput.current?.click()}
          >
            {t('cupons-escolher-arquivo')}
          </Button>
          <form
            className="flex w-full max-w-[640px] items-center gap-2"
            onSubmit={(e) => {
              e.preventDefault();
              if (url.trim()) void run({ url: url.trim() });
            }}
          >
            <Input
              aria-label={t('cupons-link')}
              className="min-w-0 flex-1"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              onPaste={(e) => {
                const pasted = e.clipboardData.getData('text/plain').trim();
                if (LINK.test(pasted)) {
                  e.preventDefault();
                  setUrl(pasted);
                  void run({ url: pasted });
                }
              }}
              placeholder={t('cupons-link-placeholder')}
            />
            <Button type="submit" variant="primary" loading={busy} disabled={!url.trim()}>
              {t('cupons-ler')}
            </Button>
          </form>
        </div>

        {steps.length ? (
          <ol aria-label={t('cupons-etapas')} aria-live="polite" className="flex flex-col gap-1.5">
            {steps.map((s) => (
              <li key={s.id} className="flex items-center gap-2 text-[13px]">
                <StepIcon state={s.state} />
                <span className={cx(s.state === 'skip' ? 'text-dim' : 'text-fg')}>
                  {tk(STEP_KEY[s.id])}
                </span>
                {s.detail ? <span className="truncate text-dim">· {s.detail}</span> : null}
              </li>
            ))}
          </ol>
        ) : null}

        {summary && outcome ? (
          <section
            aria-label={t('cupons-resultado')}
            className={cx(
              'flex flex-wrap items-center gap-3 rounded-card border px-4 py-3',
              summary.ok ? 'border-positive/50 bg-positive/5' : 'border-warning/60 bg-warning/5',
            )}
          >
            <div className="min-w-0 flex-1 text-[13px]">
              <p className="text-fg">{summary.title}</p>
              <p className="text-dim">{summary.next}</p>
            </div>
            {outcome.kind === 'photo' &&
            outcome.reading.tier === 'unbalanced' &&
            outcome.reading.ai_available ? (
              <Button
                variant="secondary"
                loading={askingAi}
                onClick={() => {
                  setAskingAi(true);
                  void ocr(outcome.file, true).finally(() => setAskingAi(false));
                }}
              >
                {t('cupons-foto-pedir-ia')}
              </Button>
            ) : null}
            <Button variant="primary" onClick={() => setOpen('review')}>
              {summary.ok ? t('cupons-revisar-gravar') : t('cupons-revisar-corrigir')}
            </Button>
          </section>
        ) : null}

        {failure ? (
          <section
            role="alert"
            className="flex flex-col gap-3 rounded-card border border-negative/50 bg-negative/5 px-4 py-3"
          >
            <p className="flex items-start gap-2 text-[13px] text-fg">
              <Icon name="alert" size={14} className="mt-0.5 shrink-0 text-negative" />
              <span>{failure.message}</span>
            </p>
            {failure.conflict ? null : (
              <div className="flex flex-wrap gap-2">
                {failure.url && canOpenSefaz() ? (
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => void openSefaz(failure.url!).then(() => setSefazOpen(true))}
                  >
                    {t('cupons-sefaz-abrir')}
                  </Button>
                ) : null}
                {failure.file ? (
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => {
                      const f = failure.file!;
                      setFailure(null);
                      setBusy(true);
                      void ocr(f).finally(() => setBusy(false));
                    }}
                  >
                    {t('cupons-ler-pela-foto')}
                  </Button>
                ) : null}
                {brazil && failure.url ? (
                  <Button size="sm" variant="ghost" onClick={() => setShowText(true)}>
                    {t('cupons-colar-texto')}
                  </Button>
                ) : null}
                <Button size="sm" variant="ghost" onClick={() => setOpen('manual')}>
                  {t('cupons-cadastrar-mao')}
                </Button>
              </div>
            )}
          </section>
        ) : null}

        {sefazOpen ? (
          <p className="flex flex-wrap items-center gap-2 text-[12.5px] text-dim">
            {t('cupons-sefaz-aberta')}
            <Button size="sm" variant="ghost" onClick={() => void captureSefaz()}>
              {t('cupons-sefaz-ler-pagina')}
            </Button>
          </p>
        ) : null}

        {steps.length && !busy ? (
          <div>
            <Button size="sm" variant="ghost" onClick={reset}>
              {t('cupons-ler-outro')}
            </Button>
          </div>
        ) : null}

        <details>
          <summary className="cursor-pointer text-[12.5px] text-dim hover:text-fg">
            {t('alimentacao-colar-itens')}
          </summary>
          <div className="mt-2 flex flex-col gap-2">
            <TextArea
              aria-label={t('alimentacao-colar-itens')}
              rows={5}
              value={pasteText}
              onChange={(e) => setPasteText(e.target.value)}
              placeholder={t('alimentacao-colar-itens-exemplo')}
            />
            {pasteError ? <p className="text-[12.5px] text-negative">{pasteError}</p> : null}
            <div>
              <Button
                variant="secondary"
                disabled={!pasteText.trim()}
                loading={pasting}
                onClick={readPasted}
              >
                {t('alimentacao-colar-itens-ler')}
              </Button>
            </div>
          </div>
        </details>

        {/* Sem arquivo nem link: digitar os itens (feira, loja sem nota, outro país). */}
        {failure && !failure.conflict ? null : (
          <div>
            <Button
              size="sm"
              variant="secondary"
              icon={<Icon name="plus" size={12} />}
              onClick={() => setOpen('manual')}
            >
              {t('cupons-cadastrar-mao')}
            </Button>
          </div>
        )}

        {brazil ? (
          <details open={showText} onToggle={(e) => setShowText(e.currentTarget.open)}>
            <summary className="cursor-pointer text-[12.5px] text-dim hover:text-fg">
              {t('cupons-colar-texto')}
            </summary>
            <div className="mt-2 flex flex-col gap-2">
              <TextArea
                aria-label={t('cupons-texto')}
                rows={5}
                value={text}
                onChange={(e) => setText(e.target.value)}
                placeholder={t('cupons-texto-placeholder')}
              />
              <div>
                <Button
                  variant="secondary"
                  disabled={!text.trim() || busy}
                  onClick={() => void run({ text })}
                >
                  {t('cupons-ler-texto-botao')}
                </Button>
              </div>
            </div>
          </details>
        ) : null}
      </div>

      {open === 'review' && outcome?.kind === 'preview' ? (
        <PurchaseDialog
          key={round}
          purchase={null}
          fiscal={{ args: outcome.args, preview: outcome.preview }}
          onClose={() => {
            setOpen(null);
            reset();
          }}
          {...(onSaved ? { onSaved } : {})}
        />
      ) : null}
      {open === 'review' && outcome?.kind === 'photo' ? (
        <PurchaseDialog
          key={round}
          purchase={null}
          draft={{
            reading: outcome.reading,
            onAskAi: () => {
              setAskingAi(true);
              void ocr(outcome.file, true).finally(() => setAskingAi(false));
            },
            askingAi,
          }}
          onClose={() => setOpen(null)}
          {...(onSaved ? { onSaved } : {})}
        />
      ) : null}
      {open === 'review' && outcome?.kind === 'foreign' ? (
        <PurchaseDialog
          purchase={null}
          foreign={outcome.receipt}
          onClose={() => setOpen(null)}
          {...(onSaved ? { onSaved } : {})}
        />
      ) : null}
      {pasted ? (
        <PurchaseDialog
          purchase={null}
          draft={{ reading: pasted }}
          onClose={() => setPasted(null)}
          {...(onSaved ? { onSaved } : {})}
        />
      ) : null}
      {open === 'manual' ? (
        <PurchaseDialog
          purchase={null}
          onClose={() => setOpen(null)}
          {...(onSaved ? { onSaved } : {})}
        />
      ) : null}
    </InstrumentCard>
  );
}

/** O que achou e o próximo passo. */
function outcomeSummary(o: Outcome | null): { ok: boolean; title: ReactNode; next: string } | null {
  if (!o) return null;
  if (o.kind === 'preview') {
    const p = o.preview;
    const c = previewCheck(p);
    return {
      ok: c.ok,
      title: withSlots(
        t(c.ok ? 'cupons-resultado-ok' : 'cupons-resultado-diferenca', {
          n: p.items.length,
          loja: p.parsed.store_name ?? t('cupons-loja'),
          valor: slot(0),
        }),
        [<Money key="v" cents={p.parsed.paid} />],
      ),
      next: p.txn ? t('cupons-proximo-ligar') : t('cupons-proximo-sem-lancamento'),
    };
  }
  if (o.kind === 'photo') {
    const r = o.reading;
    const ok = r.tier !== 'unbalanced';
    return {
      ok,
      title: withSlots(
        t(ok ? 'cupons-resultado-foto-ok' : 'cupons-resultado-foto-diferenca', {
          n: r.reading.items.length,
          valor: slot(0),
        }),
        [<Money key="v" cents={r.reading.total ?? r.reading.items_sum} />],
      ),
      next: ok ? t('cupons-proximo-foto') : t('cupons-proximo-foto-corrigir'),
    };
  }
  return {
    ok: true,
    title: withSlots(
      t('cupons-resultado-afip', {
        pais: o.receipt.country,
        documento: `${o.receipt.document} ${o.receipt.number}`,
        valor: slot(0),
      }),
      [<Money key="v" cents={o.receipt.total} currency={o.receipt.currency} />],
    ),
    next: t('cupons-proximo-afip'),
  };
}
