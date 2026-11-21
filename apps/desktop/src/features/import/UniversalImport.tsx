/**
 * Janela única de importação: o usuário solta ou escolhe arquivos, o núcleo diz o que cada
 * um é (`import.sniff`) e a janela abre o importador certo com o arquivo já carregado
 * (extrato OFX/CSV, notas de corretagem SINACOR, relatórios da Área do Investidor B3, cupom
 * fiscal em PDF ou XML).
 *
 * Imagens são lidas no computador antes de qualquer IA: primeiro o QR (o link da consulta da
 * NFC-e abre a nota oficial), depois o texto (`import.analyze_image`, com o OCR embutido):
 * cupom abre a compra preenchida em Consumo; boleto e Pix viram conta a pagar prevista, e o
 * comprovante se liga ao lançamento do extrato (`PaymentDialog`, docs/ocr-other-documents.md); o
 * resto vai para o agente de IA, como último recurso, com o tipo já dito.
 *
 * Arquivos vindos da pasta Downloads vão para a Lixeira depois de importados com sucesso,
 * se o ajuste "Apagar de Downloads depois de importar" estiver ligado; os que vieram da fila do
 * e-mail de ingestão saem dela.
 */
import { useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useState, type DragEvent } from 'react';
import { useNavigate } from 'react-router';

import { call } from '@/api/client';
import type { FileKind } from '@/api/generated/FileKind';
import type { ImageAnalysis } from '@/api/generated/ImageAnalysis';
import type { ReceiptArgs } from '@/api/generated/ReceiptArgs';
import type { ReceiptPreview } from '@/api/generated/ReceiptPreview';
import type { SniffResult } from '@/api/generated/SniffResult';
import { useEngines } from '@/api/hooks/ai';
import { usePreviewReceipt } from '@/api/hooks/receipts';
import { photoArgs } from '@/features/receipts/photo';
import { usePhotoReceipt } from '@/features/receipts/PhotoReceipt';
import { readQr } from '@/features/receipts/qr';
import { PurchaseDialog } from '@/features/food/PurchaseDialog';
import { useAppSettings } from '@/api/hooks/settings';
import { Button, Dialog, Icon, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { B3ImportDialog } from '@/features/portfolio/import/B3ImportDialog';
import { NotesImportDialog } from '@/features/portfolio/import/NotesImportDialog';
import type { PickedFile } from '@/features/portfolio/import/shared';
import { fileToBase64 } from '@/features/transactions/ui/hooks';
import { InlineError } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import type { MessageKey } from '@/i18n/keys';
import { useCompanion } from '@/state/companion';

import { trashOfferedFile } from './downloadsWatch';
import { ImportWizard } from './ImportWizard';
import { useImportHub } from './importHub';
import { PaymentDialog } from './PaymentDialog';

/** O que a leitura local da imagem achou (QR do cupom ou o texto). */
type Analysis =
  { state: 'reading' } | { state: 'qr'; url: string } | { state: 'read'; image: ImageAnalysis };

interface Item {
  file: File;
  picked: PickedFile;
  sniff: SniffResult;
  analysis?: Analysis;
}

/** Grupo na janela: o tipo do arquivo, refinado pela leitura local das imagens. */
type Group = FileKind | 'payment';

const KIND_TITLE = {
  statement: 'importacao-tipo-extratos',
  brokerage_note: 'importacao-tipo-notas',
  b3_report: 'importacao-tipo-b3',
  pension_statement: 'importacao-tipo-previdencia',
  receipt: 'importacao-tipo-cupons',
  payment: 'importacao-tipo-comprovantes',
  image: 'importacao-tipo-imagens',
  unknown: 'importacao-tipo-desconhecidos',
} as const satisfies Record<Group, MessageKey>;

const ORDER: Group[] = [
  'statement',
  'brokerage_note',
  'b3_report',
  'receipt',
  'pension_statement',
  'payment',
  'image',
  'unknown',
];

/** Link da consulta pública da NFC-e (o QR do cupom). */
const NFCE_LINK = /^https:\/\/([^/?#]+\.)?gov\.br([/?#]|$)/i;

export function groupOf(i: Pick<Item, 'sniff' | 'analysis'>): Group {
  const a = i.analysis;
  if (a?.state === 'qr') return 'receipt';
  if (a?.state === 'read') {
    if (a.image.kind === 'receipt') return 'receipt';
    // Foto ou print do extrato por período da Caixa: importação de extratos, sem IA.
    if (a.image.kind === 'statement') return 'statement';
    if (a.image.kind === 'payment_proof' || a.image.kind === 'bill') return 'payment';
  }
  return i.sniff.kind;
}

/** Rótulo e dica de um arquivo (a leitura local, quando houver, vale mais que o tipo). */
function describe(i: Item): string {
  const a = i.analysis;
  if (a?.state === 'reading') return t('importacao-lendo-imagem');
  if (a?.state === 'qr') return t('importacao-cupom-qr');
  if (a?.state === 'read') return [a.image.label, a.image.note].filter(Boolean).join(' · ');
  return [i.sniff.label, i.sniff.note].filter(Boolean).join(' · ');
}

export function UniversalImport() {
  const open = useImportHub((s) => s.open);
  const setOpen = useImportHub((s) => s.setOpen);
  const engines = useEngines();
  const hasEngine = (engines.data ?? []).some((e) => e.enabled);
  const [items, setItems] = useState<Item[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [over, setOver] = useState(false);
  const [wizardFile, setWizardFile] = useState<PickedFile | null>(null);
  const [notes, setNotes] = useState<PickedFile[] | null>(null);
  const [b3, setB3] = useState<PickedFile[] | null>(null);
  const [payment, setPayment] = useState<Item | null>(null);
  const { data: settings } = useAppSettings();
  const preview = usePreviewReceipt();
  const photo = usePhotoReceipt();
  const [pending, setPending] = useState<{ args: ReceiptArgs; preview: ReceiptPreview } | null>(
    null,
  );
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  /**
   * Depois de importar: os arquivos que vieram da pasta Downloads vão para a Lixeira; as peças
   * do e-mail de ingestão saem da fila de revisão.
   */
  const afterImport = (names: string[]) => {
    const hub = useImportHub.getState();
    const origins = names.flatMap((n) => (hub.origins[n] ? [[n, hub.origins[n]] as const] : []));
    hub.forget(names);
    for (const [, o] of origins)
      if (o.startsWith('mail:'))
        void call('mail.mark_part', { id: o.slice(5), status: 'imported' }).then(() =>
          queryClient.invalidateQueries({ queryKey: ['mail'] }),
        );
      // Caixa de entrada: o arquivo importado sai da lista (o conteúdo continua no cofre).
      else if (o.startsWith('inbox:'))
        void call('import.inbox_set_status', { id: o.slice(6), status: 'imported' }).then(() =>
          queryClient.invalidateQueries({ queryKey: ['import', 'inbox'] }),
        );
    const paths = origins.filter(([, o]) => !o.startsWith('mail:') && !o.startsWith('inbox:'));
    if (!settings?.downloads_delete_after_import || !paths.length) return;
    void Promise.allSettled(paths.map(([, p]) => trashOfferedFile(p))).then((rs) => {
      const failed = rs.filter((r) => r.status === 'rejected');
      if (failed.length < rs.length)
        toast({
          tone: 'success',
          title: t('importacao-n-lixeira', { n: rs.length - failed.length }),
          description: paths.map(([n]) => n).join(', '),
        });
      if (failed.length)
        toast({
          tone: 'error',
          title: t('importacao-erro-lixeira'),
          description: errorText((failed[0] as PromiseRejectedResult).reason),
        });
    });
  };

  const close = (o: boolean) => {
    setOpen(o);
    if (!o) {
      setItems([]);
      setError(null);
    }
  };

  const add = useCallback(async (list: File[]) => {
    setBusy(true);
    setError(null);
    try {
      const out: Item[] = [];
      for (const file of list) {
        const picked = {
          filename: file.name,
          size: file.size,
          content_base64: await fileToBase64(file),
        };
        const sniff = await call<SniffResult>('import.sniff', {
          filename: picked.filename,
          content_base64: picked.content_base64,
        });
        out.push({ file, picked, sniff });
      }
      setItems((cur) => [
        ...cur.filter((c) => !out.some((o) => o.file.name === c.file.name)),
        ...out.map((o) =>
          o.sniff.kind === 'image' ? { ...o, analysis: { state: 'reading' } as const } : o,
        ),
      ]);
      // Imagens: QR e texto lidos no computador, uma de cada vez (a IA fica para o fim).
      for (const o of out.filter((x) => x.sniff.kind === 'image')) {
        const analysis = await analyzeImage(o.file);
        setItems((cur) => cur.map((c) => (c.file === o.file ? { ...c, analysis } : c)));
      }
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }, []);

  // Arquivos entregues por outra tela (ex.: o aviso da pasta Downloads).
  useEffect(
    () =>
      useImportHub.subscribe((s) => {
        if (s.open && s.pending.length) void add(s.takePending());
      }),
    [add],
  );

  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    setOver(false);
    const files = Array.from(e.dataTransfer.files ?? []);
    if (files.length) void add(files);
  };

  const of = (k: Group) => items.filter((i) => groupOf(i) === k);
  const openPreview = (args: ReceiptArgs, onFail?: () => void) =>
    preview.mutate(args, {
      onSuccess: (p) => setPending({ args, preview: p }),
      onError: () => onFail?.(),
    });
  /** Cupom: a nota oficial (QR, PDF, XML) ou a compra preenchida pelo texto da foto. */
  const openReceipt = (i: Item) => {
    const a = i.analysis;
    if (a?.state === 'qr')
      // Consulta fora do ar ou com captcha: a foto vale pelo texto lido.
      openPreview({ url: a.url }, () => void photo.start(i.file));
    else if (a?.state === 'read') void photo.start(i.file);
    else
      openPreview({
        files: [{ filename: i.picked.filename, content_base64: i.picked.content_base64 }],
      });
  };
  /** Comprovantes e boletos: o agente registra, já sabendo o que é. */
  const registerPayments = (list: Item[]) => {
    useCompanion.getState().attach(
      list.map((i) => i.file),
      t('importacao-agente-comprovantes', { n: list.length }),
    );
    close(false);
  };
  // Imagens: só anexadas no painel do agente, com o campo em foco para o usuário dizer o
  // que quer (sem texto pronto).
  const registerImages = (list: Item[]) => {
    const files = list.map((i) => i.file);
    close(false);
    useCompanion.getState().attach(files);
  };
  const analyze = (list: Item[]) => {
    const names = list.map((i) => i.file.name).join(', ');
    useCompanion.getState().attach(
      list.map((i) => i.file),
      t('importacao-agente-analisar', { n: list.length, nomes: names }),
    );
    close(false);
  };

  return (
    <>
      <Dialog
        resizable
        resizeId="universal-import"
        open={open}
        onOpenChange={close}
        size="lg"
        title={t('importacao-arquivos-titulo')}
        description={t('importacao-arquivos-descricao')}
      >
        <div className="flex flex-col gap-4">
          <label
            onDragOver={(e) => {
              e.preventDefault();
              setOver(true);
            }}
            onDragLeave={() => setOver(false)}
            onDrop={onDrop}
            className={cx(
              'flex cursor-pointer flex-col items-center justify-center gap-1.5 rounded-card border-2 border-dashed px-4 py-8 text-center transition-colors',
              over ? 'border-accent bg-accent-soft' : 'border-line hover:border-line-strong',
            )}
          >
            <Icon name="clip" size={18} className="text-accent" />
            <span className="text-[13.5px] text-fg">
              {busy ? t('importacao-lendo-arquivos') : t('importacao-soltar')}
            </span>
            <span className="text-[12px] text-dim">{t('importacao-formatos')}</span>
            <input
              type="file"
              multiple
              hidden
              data-testid="universal-import-input"
              onChange={(e) => {
                const files = Array.from(e.target.files ?? []);
                e.target.value = '';
                if (files.length) void add(files);
              }}
            />
          </label>
          <InlineError error={error ?? preview.error ?? photo.error} />
          <p className="text-[12px] text-dim">
            {tRich('importacao-email-dica', {
              link: (
                <button
                  type="button"
                  className="text-accent underline-offset-2 hover:underline"
                  onClick={() => {
                    close(false);
                    void navigate('/lancamentos?aba=email');
                  }}
                >
                  {t('importacao-email-link')}
                </button>
              ),
            })}
          </p>
          {ORDER.filter((k) => of(k).length).map((k) => {
            const list = of(k);
            const title = t(KIND_TITLE[k]);
            return (
              <section key={k} aria-label={title} className="flex flex-col gap-2">
                <div className="flex flex-wrap items-center gap-2">
                  <h3 className="label-caps text-fg">{title}</h3>
                  <span className="flex-1" />
                  {k === 'brokerage_note' ? (
                    <Button
                      size="sm"
                      variant="primary"
                      onClick={() => setNotes(list.map((i) => i.picked))}
                    >
                      {t('importacao-importar-notas', { n: list.length })}
                    </Button>
                  ) : k === 'b3_report' ? (
                    <Button
                      size="sm"
                      variant="primary"
                      onClick={() => setB3(list.map((i) => i.picked))}
                    >
                      {t('importacao-importar-b3')}
                    </Button>
                  ) : k === 'pension_statement' ? (
                    hasEngine ? (
                      <Button
                        size="sm"
                        variant="primary"
                        icon={<Icon name="agent" size={12} />}
                        onClick={() => analyze(list)}
                      >
                        {t('importacao-importar-agente')}
                      </Button>
                    ) : (
                      <span className="text-[12px] text-dim">
                        {t('importacao-previdencia-dica')}
                      </span>
                    )
                  ) : k === 'payment' ? (
                    hasEngine ? (
                      <Button
                        size="sm"
                        variant="ghost"
                        icon={<Icon name="agent" size={12} />}
                        onClick={() => registerPayments(list)}
                      >
                        {t('importacao-registrar-agente')}
                      </Button>
                    ) : null
                  ) : k === 'image' ? (
                    hasEngine ? (
                      <Button
                        size="sm"
                        variant="secondary"
                        icon={<Icon name="agent" size={12} />}
                        disabled={list.some((i) => i.analysis?.state === 'reading')}
                        onClick={() => registerImages(list)}
                      >
                        {t('importacao-anexar-agente')}
                      </Button>
                    ) : (
                      <span className="text-[12px] text-dim">{t('importacao-motor-imagens')}</span>
                    )
                  ) : k === 'unknown' ? (
                    hasEngine ? (
                      <Button
                        size="sm"
                        variant="secondary"
                        icon={<Icon name="agent" size={12} />}
                        onClick={() => analyze(list)}
                      >
                        {t('importacao-analisar-ia')}
                      </Button>
                    ) : (
                      <span className="text-[12px] text-dim">{t('importacao-motor-analisar')}</span>
                    )
                  ) : null}
                </div>
                <ul className="divide-y divide-line rounded-control border border-line">
                  {list.map((i) => (
                    <li key={i.file.name} className="flex flex-wrap items-center gap-2 px-3 py-2">
                      <span className="min-w-0 flex-1">
                        <span className="block truncate text-[13px] text-fg">{i.file.name}</span>
                        <span className="block text-[11.5px] text-dim">{describe(i)}</span>
                      </span>
                      {k === 'receipt' ? (
                        <Button
                          size="sm"
                          variant="primary"
                          loading={preview.isPending || photo.pending}
                          onClick={() => openReceipt(i)}
                        >
                          {t('importacao-revisar-compra')}
                        </Button>
                      ) : k === 'payment' ? (
                        <Button size="sm" variant="primary" onClick={() => setPayment(i)}>
                          {t('importacao-pagamento-revisar')}
                        </Button>
                      ) : k === 'statement' ? (
                        <Button size="sm" variant="primary" onClick={() => setWizardFile(i.picked)}>
                          {t('importacao-importar')}
                        </Button>
                      ) : k === 'brokerage_note' && i.sniff.needs_password ? (
                        <Button size="sm" variant="ghost" onClick={() => setWizardFile(i.picked)}>
                          {t('importacao-fatura-cartao')}
                        </Button>
                      ) : null}
                      <button
                        type="button"
                        aria-label={t('importacao-tirar', { nome: i.file.name })}
                        className="grid size-6 place-items-center rounded-control text-dim hover:text-fg"
                        onClick={() => setItems((cur) => cur.filter((c) => c !== i))}
                      >
                        <Icon name="x" size={11} />
                      </button>
                    </li>
                  ))}
                </ul>
              </section>
            );
          })}
        </div>
      </Dialog>
      {photo.dialog}
      {payment ? (
        <PaymentDialog
          file={payment.file}
          onClose={() => setPayment(null)}
          onDone={() => {
            const done = payment;
            setPayment(null);
            setItems((cur) => cur.filter((c) => c !== done));
            afterImport([done.file.name]);
          }}
          {...(hasEngine
            ? {
                onAgent: () => {
                  const it = payment;
                  setPayment(null);
                  registerPayments([it]);
                },
              }
            : {})}
        />
      ) : null}
      {pending ? (
        <PurchaseDialog purchase={null} fiscal={pending} onClose={() => setPending(null)} />
      ) : null}
      <ImportWizard
        open={wizardFile != null}
        onOpenChange={(o) => !o && setWizardFile(null)}
        onImported={() => {
          if (!wizardFile) return;
          afterImport([wizardFile.filename]);
          // O arquivo importado sai da lista; sem mais nada para importar, a janela fecha.
          const left = items.filter((c) => c.file.name !== wizardFile.filename);
          setItems(left);
          if (left.length === 0) close(false);
        }}
        initialFile={
          wizardFile
            ? {
                name: wizardFile.filename,
                size: wizardFile.size,
                content_base64: wizardFile.content_base64,
              }
            : null
        }
      />
      <NotesImportDialog
        open={notes != null}
        onOpenChange={(o) => !o && setNotes(null)}
        onImported={afterImport}
        initialFiles={notes ?? undefined}
      />
      <B3ImportDialog
        open={b3 != null}
        onOpenChange={(o) => !o && setB3(null)}
        onImported={afterImport}
        initialFiles={b3 ?? undefined}
      />
    </>
  );
}

/** Lê a imagem no computador: o QR do cupom primeiro, depois o texto (OCR embutido). */
async function analyzeImage(file: File): Promise<Analysis> {
  const link = await readQr(file).catch(() => null);
  if (link && NFCE_LINK.test(link)) return { state: 'qr', url: link };
  // QR do Pix ("copia e cola"): conta a pagar, lida em `PaymentDialog`.
  if (link?.startsWith('000201'))
    return {
      state: 'read',
      image: {
        kind: 'bill',
        label: t('importacao-pagamento-pix-qr'),
        note: null,
        reading: null,
        lines: 0,
      },
    };
  try {
    const image = await call<ImageAnalysis>('import.analyze_image', await photoArgs(file));
    return { state: 'read', image };
  } catch {
    return {
      state: 'read',
      image: {
        kind: 'unreadable',
        label: t('importacao-imagem-nao-lida'),
        note: null,
        reading: null,
        lines: 0,
      },
    };
  }
}
