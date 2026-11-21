/**
 * Importação de notas de corretagem (PDF, padrão SINACOR): conta da corretora e
 * arquivos → prévia (`notes.preview`: notas com custos e líquido, operações com
 * taxas e IRRF rateados, ativos não identificados) → gravação (`notes.commit`) com
 * desfazer pelo histórico.
 *
 * A senha dos PDFs protegidos fica só no estado deste diálogo enquanto ele está
 * aberto: não vai para o cache, para o armazenamento do navegador nem para o núcleo
 * além das chamadas de leitura e gravação.
 */
import { supportEvent } from '@/api/hooks/support';
import { useMemo, useState } from 'react';

import type { NoteAssetChoice } from '@/api/generated/NoteAssetChoice';
import type { NoteCommitResult } from '@/api/generated/NoteCommitResult';
import type { NotePreview } from '@/api/generated/NotePreview';
import type { NoteSummary } from '@/api/generated/NoteSummary';
import type { NoteTrade } from '@/api/generated/NoteTrade';
import type { Asset } from '@/api/generated/Asset';
import { useAssets } from '@/api/hooks/assets';
import { useUndo } from '@/api/hooks/changes';
import { previewNotes, useCommitNotes } from '@/api/hooks/notes';
import { useInvestmentAccounts } from '@/api/hooks/portfolio';
import {
  Button,
  Dialog,
  Field,
  Input,
  KeyValue,
  Money,
  PasswordInput,
  Private,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { formatDate } from '@/lib/format';
import { Checkbox, FileInput, InlineError, Switch } from '@/features/transactions/ui/controls';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { t as tr } from '@/i18n';

import { AccountSelect, AssetSelect } from '../components/Fields';
import { formatQuantity, formatUnitPrice } from '../decimal';
import { NOTE_STATUS, NOTE_STATUS_ORDER } from './labels';
import {
  PreviewTable,
  readFiles,
  RowNote,
  SideLabel,
  StatusChip,
  StatusFilter,
  Steps,
  TD,
  decimal,
  Warnings,
  type PickedFile,
} from './shared';

type Step = 'files' | 'preview' | 'done';

const steps = (): { id: Step; label: string }[] => [
  { id: 'files', label: tr('b3-etapa-notas') },
  { id: 'preview', label: tr('b3-etapa-previa') },
  { id: 'done', label: tr('b3-etapa-resultado') },
];

function NoteCard({ note }: { note: NoteSummary }) {
  const c = note.costs;
  // Como o núcleo: diferença de até 1 centavo é arredondamento.
  const mismatch = note.net != null && Math.abs(note.net - note.computed_net) > 1;
  return (
    <section
      className="flex flex-col gap-2 rounded-card border border-line p-3"
      aria-label={tr('b3-nota-aria', { nota: note.number ?? note.filename })}
    >
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <h4 className="label-caps text-fg">
          {tr('b3-nota-titulo', { nota: note.number ?? tr('b3-sem-numero') })}
          {note.date ? (
            <span className="num text-dim">{` ${tr('b3-pregao', { data: formatDate(note.date) })}`}</span>
          ) : null}
        </h4>
        <span className="text-[12px] text-dim">
          {tr('b3-nota-arquivo', { arquivo: note.filename, n: note.trades })}
        </span>
      </div>
      <KeyValue
        items={[
          { label: tr('b3-custo-liquidacao'), value: <Money cents={c.settlement} /> },
          { label: tr('b3-custo-registro'), value: <Money cents={c.registration} /> },
          { label: tr('b3-custo-emolumentos'), value: <Money cents={c.emoluments} /> },
          { label: tr('b3-custo-corretagem'), value: <Money cents={c.brokerage} /> },
          { label: 'ISS', value: <Money cents={c.iss} /> },
          ...(c.other ? [{ label: tr('b3-custo-outras'), value: <Money cents={c.other} /> }] : []),
          { label: tr('b3-custo-total'), value: <Money cents={c.total} /> },
          { label: 'IRRF', value: <Money cents={c.irrf} /> },
          {
            label: tr('b3-liquido-nota'),
            value: note.net != null ? <Money cents={note.net} tone="auto" signed /> : '—',
          },
          {
            label: tr('b3-liquido-calculado'),
            value: <Money cents={note.computed_net} tone="auto" signed />,
          },
        ]}
      />
      {mismatch ? (
        <p role="status" className="text-[12.5px] font-medium text-warning">
          {tr('b3-liquido-diferente')}
        </p>
      ) : null}
      <Warnings items={note.warnings} label={tr('b3-avisos-nota', { nota: note.number ?? '' })} />
    </section>
  );
}

/**
 * Correção do ativo de um título: um ativo cadastrado ou um código de negociação (o ativo
 * é cadastrado ao gravar). Vale para todas as operações do mesmo título.
 */
function AssetFix({
  spec,
  assets,
  choice,
  onChange,
  disabled,
}: {
  spec: string;
  assets: readonly Asset[];
  choice: NoteAssetChoice | undefined;
  onChange: (c: NoteAssetChoice | null) => void;
  disabled?: boolean;
}) {
  const [code, setCode] = useState(choice?.ticker ?? '');
  const valid = /^[A-Za-z]{4}\d{1,2}[A-Za-z]?$/.test(code.trim());
  return (
    <div
      className="flex flex-wrap items-end gap-2"
      role="group"
      aria-label={tr('b3-ativo-para', { titulo: spec })}
    >
      <Field label={tr('b3-ativo-cadastrado')} className="min-w-56 flex-1">
        <AssetSelect
          assets={assets}
          value={choice?.asset_id}
          disabled={disabled}
          onChange={(id) => onChange({ spec, asset_id: id })}
        />
      </Field>
      <form
        className="flex items-end gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          if (valid) onChange({ spec, ticker: code.trim().toUpperCase() });
        }}
      >
        <Field label={tr('b3-ou-codigo')} className="w-28">
          <Input
            value={code}
            placeholder="GGRC11"
            aria-label={tr('b3-codigo-para', { titulo: spec })}
            disabled={disabled}
            onChange={(e) => setCode(e.target.value)}
          />
        </Field>
        <Button type="submit" size="sm" disabled={!valid || disabled}>
          {tr('b3-usar-codigo')}
        </Button>
      </form>
      {choice ? (
        <Button size="sm" variant="ghost" disabled={disabled} onClick={() => onChange(null)}>
          {tr('b3-voltar-automatico')}
        </Button>
      ) : null}
    </div>
  );
}

export function NotesImportDialog({
  open,
  onOpenChange,
  initialAccountId,
  initialFiles,
  onImported,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  initialAccountId?: string;
  /** Notas já escolhidas (janela única de importação). */
  initialFiles?: PickedFile[];
  /** Depois de gravar com sucesso, com os nomes dos arquivos gravados. */
  onImported?: (filenames: string[]) => void;
}) {
  const accountsQ = useInvestmentAccounts();
  const accounts = useMemo(
    () => (accountsQ.data ?? []).filter((a) => a.currency === 'BRL'),
    [accountsQ.data],
  );
  const assetsQ = useAssets({}, { enabled: open });
  const commit = useCommitNotes();
  const undo = useUndo();

  const [step, setStep] = useState<Step>('files');
  const [accountId, setAccountId] = useState(initialAccountId ?? '');
  const [files, setFiles] = useState<PickedFile[]>([]);
  const [password, setPassword] = useState('');
  const [preview, setPreview] = useState<NotePreview | null>(null);
  const [skip, setSkip] = useState<Set<number>>(new Set());
  const [choices, setChoices] = useState<Map<string, NoteAssetChoice>>(new Map());
  const [group, setGroup] = useState(true);
  const [editing, setEditing] = useState<number | null>(null);
  const [cash, setCash] = useState(false);
  const [filter, setFilter] = useState<NoteTrade['status'] | 'all'>('all');
  const [result, setResult] = useState<NoteCommitResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [lastOpen, setLastOpen] = useState(open);

  if (open !== lastOpen) {
    setLastOpen(open);
    // Ao abrir e ao fechar: nada da importação anterior (nem a senha) fica no estado.
    setStep('files');
    setFiles(open && initialFiles ? initialFiles : []);
    setPassword('');
    setPreview(null);
    setSkip(new Set());
    setChoices(new Map());
    setGroup(true);
    setEditing(null);
    setCash(false);
    setFilter('all');
    setResult(null);
    setError(null);
    if (initialAccountId) setAccountId(initialAccountId);
  }
  if (!accountId && accounts[0]) setAccountId(accounts[0].id);
  const account = accounts.find((a) => a.id === accountId);

  const run = async (fn: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  };

  const pwd = password.trim() ? { password } : {};
  const fileArgs = files.map((f) => ({ filename: f.filename, content_base64: f.content_base64 }));

  const loadPreview = async (
    list: PickedFile[],
    ch: Map<string, NoteAssetChoice> = choices,
    g: boolean = group,
  ) => {
    const p = await previewNotes({
      files: list.map((f) => ({ filename: f.filename, content_base64: f.content_base64 })),
      account_id: accountId,
      ...pwd,
      assets: [...ch.values()],
      group: g,
    });
    setPreview(p);
    // Agrupar ou separar muda os índices: o que estava desmarcado deixa de valer.
    if (g !== group) setSkip(new Set());
    setStep('preview');
  };
  const read = (list: PickedFile[]) => void run(() => loadPreview(list));
  /** Corrige (ou volta ao automático) o ativo de um título e refaz a prévia. */
  const choose = (spec: string, c: NoteAssetChoice | null) => {
    const next = new Map(choices);
    if (c) next.set(spec, c);
    else next.delete(spec);
    setChoices(next);
    setEditing(null);
    void run(() => loadPreview(files, next));
  };
  const regroup = (g: boolean) => {
    setGroup(g);
    void run(() => loadPreview(files, choices, g));
  };

  const onFiles = (list: FileList | null) =>
    void run(async () => {
      const picked = await readFiles(list);
      setFiles(picked);
      if (picked.length && accountId) await loadPreview(picked);
    });

  const willWrite = (t: NoteTrade) =>
    !skip.has(t.index) && (t.status === 'new' || t.status === 'fill_costs');
  const writeCount = preview ? preview.trades.filter(willWrite).length : 0;
  const needsPassword = preview?.errors.some((e) => e.needs_password) ?? false;
  const blocked = (preview?.errors.length ?? 0) > 0;

  const doCommit = () =>
    void run(async () => {
      if (!preview) return;
      const res = await commit.mutateAsync({
        files: fileArgs,
        account_id: accountId,
        ...pwd,
        assets: [...choices.values()],
        group,
        skip: [...skip],
        cash_movement: cash,
      });
      setPassword('');
      setResult(res);
      setStep('done');
      onImported?.(files.map((f) => f.filename));
      supportEvent('import');
      toast({
        tone: 'success',
        title: tr('b3-notas-gravadas', { n: res.created + res.updated }),
        description: files.map((f) => f.filename).join(', '),
      });
    });

  const visible = useMemo(
    () => (preview ? preview.trades.filter((t) => filter === 'all' || t.status === filter) : []),
    [preview, filter],
  );
  const assets = assetsQ.data ?? [];

  const footer =
    step === 'files' ? (
      <>
        <Button variant="ghost" onClick={() => onOpenChange(false)}>
          {tr('comum-cancelar')}
        </Button>
        <Button
          variant="primary"
          loading={busy}
          disabled={!files.length || !accountId}
          onClick={() => read(files)}
        >
          {tr('b3-ler-notas')}
        </Button>
      </>
    ) : step === 'preview' ? (
      <>
        <Button variant="ghost" onClick={() => setStep('files')}>
          {tr('comum-voltar')}
        </Button>
        <Button
          variant="primary"
          loading={busy}
          disabled={writeCount === 0 || blocked}
          onClick={doCommit}
        >
          {tr('b3-gravar-n', { n: writeCount })}
        </Button>
      </>
    ) : (
      <>
        {result?.change_set_id ? (
          <Button
            variant="ghost"
            loading={undo.isPending}
            onClick={() =>
              undo.mutate(
                { id: result.change_set_id! },
                {
                  onSuccess: () => {
                    toast({ tone: 'success', title: tr('b3-notas-desfeita') });
                    onOpenChange(false);
                  },
                  onError: setError,
                },
              )
            }
          >
            {tr('b3-desfazer')}
          </Button>
        ) : null}
        <Button variant="primary" onClick={() => onOpenChange(false)}>
          {tr('b3-concluir')}
        </Button>
      </>
    );

  return (
    <Dialog
      resizable
      resizeId="notes-import"
      open={open}
      onOpenChange={onOpenChange}
      index="06"
      size="xl"
      title={tr('b3-notas-titulo')}
      footer={footer}
    >
      <Steps steps={steps()} current={step} />

      {step === 'files' ? (
        <div className="flex flex-col gap-4">
          <Field label={tr('b3-conta-corretora')} hint={tr('b3-conta-corretora-dica')}>
            <AccountSelect
              accounts={accounts}
              value={accountId || undefined}
              onChange={setAccountId}
            />
          </Field>
          <Field label={tr('b3-notas-pdf')} hint={tr('b3-notas-pdf-dica')}>
            <FileInput
              multiple
              accept=".pdf,application/pdf,.txt,text/plain"
              disabled={!accountId || busy}
              onChange={(e) => onFiles(e.target.files)}
            />
          </Field>
          {files.length ? (
            <ul
              className="flex flex-col gap-1 text-[12.5px] text-dim"
              aria-label={tr('b3-arquivos-escolhidos')}
            >
              {files.map((f) => (
                <li key={f.filename}>
                  <span className="text-fg">{f.filename}</span>
                  {` · ${formatBytes(f.size)}`}
                </li>
              ))}
            </ul>
          ) : null}
          {busy ? <p className="label-caps text-accent">{tr('b3-lendo-notas')}</p> : null}
        </div>
      ) : null}

      {step === 'preview' && preview ? (
        <div className="flex flex-col gap-4">
          <KeyValue
            items={[
              { label: tr('b3-conta'), value: account?.name ?? '' },
              { label: tr('b3-arquivos'), value: files.length },
              { label: tr('b3-notas-lidas'), value: preview.notes.length },
              { label: tr('b3-aba-operacoes'), value: preview.trades.length },
            ]}
          />

          {preview.errors.length ? (
            <div
              className="flex flex-col gap-3 rounded-card border border-negative/40 bg-[var(--negative-soft)] p-3"
              aria-label={tr('b3-arquivos-nao-lidos')}
              role="group"
            >
              <ul className="flex flex-col gap-1 text-[12.5px]">
                {preview.errors.map((e) => (
                  <li key={e.filename}>
                    <span className="text-fg">{e.filename}</span>:{' '}
                    <span className="text-negative">{e.message}</span>
                  </li>
                ))}
              </ul>
              {needsPassword ? (
                <form
                  className="flex flex-wrap items-end gap-3"
                  onSubmit={(e) => {
                    e.preventDefault();
                    read(files);
                  }}
                >
                  <Field
                    label={tr('b3-senha-pdf')}
                    className="min-w-60 flex-1"
                    hint={tr('b3-senha-pdf-dica')}
                  >
                    <PasswordInput
                      value={password}
                      autoComplete="off"
                      onChange={(e) => setPassword(e.target.value)}
                    />
                  </Field>
                  <Button
                    type="submit"
                    variant="primary"
                    loading={busy}
                    disabled={!password.trim()}
                  >
                    {tr('b3-abrir-senha')}
                  </Button>
                </form>
              ) : null}
              <p className="text-[12px] text-dim">{tr('b3-nada-gravado')}</p>
            </div>
          ) : null}

          {preview.notes.map((n) => (
            <NoteCard key={`${n.filename}-${n.number}`} note={n} />
          ))}

          {preview.unresolved_specs.length ? (
            <section className="flex flex-col gap-2" aria-label={tr('b3-nao-identificados')}>
              <h4 className="label-caps text-fg">{tr('b3-nao-identificados')}</h4>
              <p className="text-[12.5px] text-dim">{tr('b3-nao-identificados-texto')}</p>
              <div className="flex flex-col gap-3">
                {preview.unresolved_specs.map((spec) => (
                  <div key={spec} className="flex flex-col gap-1">
                    <span className="text-[12.5px] font-medium text-fg">{spec}</span>
                    <AssetFix
                      spec={spec}
                      assets={assets}
                      choice={choices.get(spec)}
                      disabled={busy}
                      onChange={(c) => choose(spec, c)}
                    />
                  </div>
                ))}
              </div>
            </section>
          ) : null}

          {preview.trades.length ? (
            <>
              <Warnings
                label={tr('b3-inconsistencias')}
                items={preview.trades
                  .filter((t) => t.warning && willWrite(t))
                  .map((t) =>
                    tr('b3-inconsistencia', {
                      data: formatDate(t.date),
                      ativo: t.ticker ?? t.spec,
                      // O núcleo une as duas partes do aviso com "; " em qualquer idioma: o resumo
                      // mostra só a primeira.
                      aviso: t.warning!.split('; ')[0] ?? '',
                    }),
                  )}
              />
              <Switch
                checked={group}
                onChange={regroup}
                disabled={busy}
                label={tr('b3-agrupar')}
                description={tr('b3-agrupar-dica')}
              />
              <StatusFilter
                label={tr('b3-filtrar-operacoes')}
                value={filter}
                onChange={setFilter}
                total={preview.trades.length}
                options={NOTE_STATUS_ORDER.map((s) => ({
                  value: s,
                  label: NOTE_STATUS[s].label,
                  tone: NOTE_STATUS[s].tone,
                  count: preview.trades.filter((t) => t.status === s).length,
                }))}
              />
              <PreviewTable
                label={tr('b3-notas-operacoes-tabela')}
                prefsId="Operações das notas"
                rows={visible}
                getRowId={(t) => t.index}
                rowClassName={(t) => !willWrite(t) && 'opacity-60'}
                after={(t, colSpan) =>
                  editing === t.index ? (
                    <tr>
                      <td className={TD} colSpan={colSpan}>
                        <AssetFix
                          spec={t.spec}
                          assets={assets}
                          choice={choices.get(t.spec)}
                          disabled={busy}
                          onChange={(c) => choose(t.spec, c)}
                        />
                      </td>
                    </tr>
                  ) : null
                }
                columns={[
                  {
                    key: 'write',
                    header: tr('b3-gravar'),
                    hideable: false,
                    sortable: false,
                    cell: (t) => (
                      <Checkbox
                        hideLabel
                        label={tr('b3-gravar-operacao', { n: t.index + 1 })}
                        checked={willWrite(t)}
                        disabled={!(t.status === 'new' || t.status === 'fill_costs')}
                        onChange={(on) =>
                          setSkip((s) => {
                            const next = new Set(s);
                            if (on) next.delete(t.index);
                            else next.add(t.index);
                            return next;
                          })
                        }
                      />
                    ),
                  },
                  {
                    key: 'status',
                    header: tr('b3-situacao'),
                    sortValue: (t) => NOTE_STATUS_ORDER.indexOf(t.status),
                    cell: (t) => (
                      <>
                        <StatusChip
                          tone={NOTE_STATUS[t.status].tone}
                          title={NOTE_STATUS[t.status].hint}
                        >
                          {NOTE_STATUS[t.status].label}
                        </StatusChip>
                        <RowNote>{t.note ?? NOTE_STATUS[t.status].hint}</RowNote>
                        {t.warning ? (
                          <span className="mt-0.5 block text-[11.5px] font-medium text-warning first-letter:uppercase">
                            ⚠ {t.warning}
                          </span>
                        ) : null}
                      </>
                    ),
                  },
                  {
                    key: 'note',
                    header: tr('b3-nota'),
                    className: cx(TD, 'num whitespace-nowrap'),
                    sortValue: (t) => t.date,
                    cell: (t) => (
                      <>
                        {t.note_number ?? '—'}
                        <span className="block text-[11px] text-dim">
                          {t.date ? formatDate(t.date) : '—'}
                        </span>
                        {t.executions > 1 ? (
                          <span className="block text-[11px] text-accent">
                            {tr('b3-execucoes', { n: t.executions })}
                          </span>
                        ) : null}
                      </>
                    ),
                  },
                  {
                    key: 'side',
                    header: tr('b3-tipo'),
                    className: cx(TD, 'whitespace-nowrap'),
                    cell: (t) => (
                      <>
                        <SideLabel side={t.side} />
                        <span className="block text-[11px] text-dim">
                          {t.market}
                          {t.day_trade ? ' · day trade' : ''}
                        </span>
                      </>
                    ),
                  },
                  {
                    key: 'asset',
                    header: tr('b3-titulo-coluna'),
                    hideable: false,
                    sortValue: (t) =>
                      t.ticker ?? assets.find((a) => a.id === t.asset_id)?.ticker ?? t.spec,
                    cell: (t) => (
                      <>
                        <span className="font-medium text-fg">
                          {t.ticker ?? assets.find((a) => a.id === t.asset_id)?.ticker ?? '?'}
                        </span>
                        <span className="block text-[11px] text-dim">{t.spec}</span>
                        {t.create_asset ? (
                          <span className="block text-[11px] text-accent">
                            {tr('b3-cadastra', {
                              ativo: t.create_asset.ticker,
                              nome: t.create_asset.name,
                            })}
                          </span>
                        ) : null}
                        {t.chosen ? (
                          <span className="block text-[11px] text-accent">
                            {tr('b3-corrigido')}
                          </span>
                        ) : null}
                        {t.status !== 'duplicate' ? (
                          <button
                            type="button"
                            className="mt-0.5 text-[11px] text-dim underline-offset-2 hover:text-accent hover:underline"
                            aria-expanded={editing === t.index}
                            onClick={() => setEditing(editing === t.index ? null : t.index)}
                          >
                            {editing === t.index ? tr('comum-fechar') : tr('b3-corrigir-ativo')}
                          </button>
                        ) : null}
                      </>
                    ),
                  },
                  {
                    key: 'quantity',
                    header: tr('b3-qtd'),
                    numeric: true,
                    sortValue: (t) => decimal(t.quantity),
                    cell: (t) => <Private>{formatQuantity(t.quantity)}</Private>,
                  },
                  {
                    key: 'price',
                    header: tr('b3-preco'),
                    numeric: true,
                    sortValue: (t) => decimal(t.price),
                    cell: (t) => <Private>{formatUnitPrice(t.price, 'BRL')}</Private>,
                  },
                  {
                    key: 'gross',
                    header: tr('b3-valor'),
                    numeric: true,
                    cell: (t) => <Money cents={t.gross} />,
                  },
                  {
                    key: 'fees',
                    header: tr('b3-taxas'),
                    numeric: true,
                    cell: (t) => <Money cents={t.fees} />,
                  },
                  {
                    key: 'taxes',
                    header: 'IRRF',
                    numeric: true,
                    cell: (t) => <Money cents={t.taxes} />,
                  },
                ]}
              />
              <Switch
                checked={cash}
                onChange={setCash}
                label={tr('b3-lancar-caixa')}
                description={tr('b3-notas-lancar-caixa-dica')}
              />
            </>
          ) : null}
        </div>
      ) : null}

      {step === 'done' && result ? (
        <div className="flex flex-col gap-3" role="status">
          <p className="text-[15px] text-fg">
            {tr('b3-notas-resultado', {
              novas: result.created,
              completadas: result.updated,
              conta: account?.name ?? '',
              ignoradas: result.skipped,
            })}
          </p>
          <KeyValue
            items={[
              { label: tr('b3-ativos-cadastrados'), value: result.assets_created },
              { label: tr('b3-desfazer-rotulo'), value: tr('b3-desfazer-como') },
            ]}
          />
        </div>
      ) : null}

      <InlineError error={error} />
    </Dialog>
  );
}
