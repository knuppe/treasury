/**
 * Importação da Área do Investidor B3, no mesmo passo a passo da importação de
 * extratos: arquivos (`b3.detect` de cada um: relatórios, instituições e período) →
 * contas (cada instituição numa corretora; a sugerida pelo nome vem marcada) →
 * prévia (`b3.preview`: ativos a cadastrar, operações, proventos, eventos, linhas
 * ignoradas e conferência com a posição da B3) → gravação (`b3.commit`) com desfazer
 * pelo histórico.
 */
import { supportEvent } from '@/api/hooks/support';
import { useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router';

import type { AssetClass } from '@/api/generated/AssetClass';
import type { B3CommitResult } from '@/api/generated/B3CommitResult';
import type { B3DetectResult } from '@/api/generated/B3DetectResult';
import type { B3Preview } from '@/api/generated/B3Preview';
import type { B3RowStatus } from '@/api/generated/B3RowStatus';
import type { B3Source } from '@/api/generated/B3Source';
import { useAssets } from '@/api/hooks/assets';
import { useUndo } from '@/api/hooks/changes';
import { detectB3, previewB3, useCommitB3 } from '@/api/hooks/b3';
import { useCustodyAccounts } from '@/api/hooks/portfolio';
import {
  Button,
  Dialog,
  Field,
  IconButton,
  Icon,
  KeyValue,
  Money,
  Private,
  Select,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { formatDate } from '@/lib/format';
import { Checkbox, FileInput, InlineError, Switch } from '@/features/transactions/ui/controls';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { t, t as tr } from '@/i18n';

import { DecimalInput } from '../components/DecimalInput';
import { Notice } from '../components/States';
import { accountLabel, B3AccountsStep, type B3Choice } from './B3AccountsStep';
import { formatDecimal, formatQuantity, formatUnitPrice } from '../decimal';
import { ACTION_KIND_LABEL, CLASS_LABEL, CLASS_ORDER, INCOME_KIND_LABEL } from '../labels';
import {
  B3_KIND_LABEL,
  B3_ORIGIN_LABEL,
  B3_POSITION_STATUS,
  B3_RULE_LABEL,
  B3_STATUS,
  B3_STATUS_ORDER,
} from './labels';
import {
  decimal,
  PreviewTable,
  readFiles,
  RowNote,
  SideLabel,
  StatusChip,
  StatusFilter,
  Steps,
  TD,
  Warnings,
  type PickedFile,
} from './shared';

type Step = 'files' | 'accounts' | 'preview' | 'done';
type TabId = 'assets' | 'trades' | 'incomes' | 'actions' | 'ignored' | 'position' | 'openings';

const steps = (): { id: Step; label: string }[] => [
  { id: 'files', label: t('b3-etapa-arquivos') },
  { id: 'accounts', label: t('b3-etapa-contas') },
  { id: 'preview', label: t('b3-etapa-previa') },
  { id: 'done', label: t('b3-etapa-resultado') },
];

/** An opening position ticked on the openings tab, with its cost. */
interface OpeningPick {
  on: boolean;
  cost: string | null;
}

const openingKey = (o: { asset_key: string; account_id: string }) =>
  `${o.asset_key}|${o.account_id}`;

interface Approval {
  approved: boolean;
  factor: string | null;
  unitCost: string | null;
}

function period(from: string | null, to: string | null): string {
  if (!from && !to) return '—';
  if (from === to || !to) return formatDate(from!);
  return t('b3-periodo', { de: from ? formatDate(from) : '…', ate: formatDate(to) });
}

const statusRank = (s: B3RowStatus) => B3_STATUS_ORDER.indexOf(s);

function sourceLabel(s: B3Source): string {
  return t('b3-origem-linha', { aba: s.sheet, linha: String(s.line) });
}

function StatusCell({
  status,
  note,
  error,
  warning,
}: {
  status: B3RowStatus;
  note?: string | null;
  error?: string | null;
  /** What a row is imported with (a sale larger than the position): it does not block. */
  warning?: string | null;
}) {
  const s = B3_STATUS[status];
  return (
    <>
      <StatusChip tone={s.tone} title={s.hint}>
        {s.label}
      </StatusChip>
      {error ? (
        <RowNote tone="negative">{error}</RowNote>
      ) : warning ? (
        <RowNote tone="warning">{warning}</RowNote>
      ) : (
        <RowNote>{note ?? s.hint}</RowNote>
      )}
    </>
  );
}

/** Valor do select que mantém o cadastro de um ativo novo. */
const NEW_ASSET = '__novo__';
export function B3ImportDialog({
  open,
  onOpenChange,
  initialFiles,
  onImported,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Arquivos já escolhidos (janela única de importação). */
  initialFiles?: PickedFile[];
  /** Depois de gravar com sucesso, com os nomes dos arquivos gravados. */
  onImported?: (filenames: string[]) => void;
}) {
  const accountsQ = useCustodyAccounts();
  const accounts = useMemo(() => accountsQ.data ?? [], [accountsQ.data]);
  const commit = useCommitB3();
  const undo = useUndo();

  const [step, setStep] = useState<Step>('files');
  const [files, setFiles] = useState<{ file: PickedFile; detect: B3DetectResult }[]>([]);
  const [choices, setChoices] = useState<Map<string, B3Choice>>(new Map());
  const [openings, setOpenings] = useState<Map<string, OpeningPick>>(new Map());
  const [preview, setPreview] = useState<B3Preview | null>(null);
  const [tab, setTab] = useState<TabId>('trades');
  const [filter, setFilter] = useState<B3RowStatus | 'all'>('all');
  const [skip, setSkip] = useState<Set<number>>(new Set());
  const [approvals, setApprovals] = useState<Map<number, Approval>>(new Map());
  const [classes, setClasses] = useState<Map<string, AssetClass>>(new Map());
  // Ativo proposto ligado a um já cadastrado (ex.: ticker antigo), em vez de criar.
  const [links, setLinks] = useState<Map<string, string>>(new Map());
  const existing = useAssets({ include_archived: true });
  const [includePossible, setIncludePossible] = useState(false);
  const [cash, setCash] = useState(false);
  const [result, setResult] = useState<B3CommitResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [lastOpen, setLastOpen] = useState(open);

  if (open !== lastOpen) {
    setLastOpen(open);
    setStep('files');
    setFiles([]);
    setChoices(new Map());
    setOpenings(new Map());
    setPreview(null);
    setTab('trades');
    setFilter('all');
    setSkip(new Set());
    setApprovals(new Map());
    setClasses(new Map());
    setLinks(new Map());
    setIncludePossible(false);
    setCash(false);
    setResult(null);
    setError(null);
  }

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

  const onFiles = (list: FileList | null) => addPicked(readFiles(list));
  const addPicked = (pickedP: Promise<PickedFile[]>) =>
    void run(async () => {
      const picked = await pickedP;
      const detected: { file: PickedFile; detect: B3DetectResult }[] = [];
      for (const f of picked) {
        const d = await detectB3({ filename: f.filename, content_base64: f.content_base64 });
        detected.push({ file: f, detect: d });
      }
      setFiles((cur) => [
        ...cur.filter((c) => !detected.some((d) => d.file.filename === c.file.filename)),
        ...detected,
      ]);
    });
  // Arquivos vindos da janela única de importação: lidos ao abrir.
  useEffect(() => {
    if (open && initialFiles?.length) addPicked(Promise.resolve(initialFiles));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, initialFiles]);

  const fileArgs = () =>
    files.map(({ file }) => ({ filename: file.filename, content_base64: file.content_base64 }));

  /** Primeira prévia sem escolhas: o núcleo devolve as instituições com a conta sugerida. */
  const toAccounts = () =>
    void run(async () => {
      const p = await previewB3({ files: fileArgs() });
      setChoices(
        new Map(p.institutions.map((i) => [i.name, { accountId: i.account_id, newAccount: null }])),
      );
      setOpenings(new Map());
      setPreview(p);
      setStep('accounts');
    });

  const makePreview = (keepTab = false) =>
    void run(async () => {
      const picked = (preview?.openings ?? []).filter((o) => openings.get(openingKey(o))?.on);
      const p = await previewB3({
        files: fileArgs(),
        accounts: [...choices].map(([institution, c]) => ({
          institution,
          account_id: c.newAccount ? null : c.accountId,
          ...(c.newAccount ? { new_account: c.newAccount } : {}),
        })),
        ...(picked.length
          ? {
              openings: picked.map((o) => {
                const cost = openings.get(openingKey(o))?.cost;
                return {
                  asset_key: o.asset_key,
                  account_id: o.account_id,
                  quantity: o.quantity,
                  ...(cost ? { unit_cost: cost } : {}),
                };
              }),
            }
          : {}),
      });
      setPreview(p);
      setSkip(new Set());
      setApprovals(new Map());
      setClasses(new Map());
      setLinks(new Map());
      setIncludePossible(false);
      setFilter('all');
      if (keepTab) {
        setStep('preview');
        return;
      }
      setTab(
        p.trades.length
          ? 'trades'
          : p.incomes.length
            ? 'incomes'
            : p.actions.length
              ? 'actions'
              : p.position_check
                ? 'position'
                : 'assets',
      );
      setStep('preview');
    });

  const isTrade = (index: number) => Boolean(preview?.trades.some((t) => t.index === index));
  // A trade in review lacks the cost; an event in review, the factor.
  const approved = (index: number) => {
    const a = approvals.get(index);
    return Boolean(a?.approved && (isTrade(index) ? a.unitCost : a.factor));
  };
  const willWrite = (index: number, status: B3RowStatus) =>
    !skip.has(index) &&
    (status === 'new' ||
      // A sale without its purchase is an official record: imported unless left out.
      status === 'warning' ||
      (status === 'possible_duplicate' && includePossible) ||
      (status === 'review' && approved(index)));
  const toggleSkip = (index: number, on: boolean) =>
    setSkip((s) => {
      const next = new Set(s);
      if (on) next.delete(index);
      else next.add(index);
      return next;
    });
  const setApproval = (index: number, patch: Partial<Approval>) =>
    setApprovals((m) => {
      const cur = m.get(index) ?? { approved: false, factor: null, unitCost: null };
      return new Map(m).set(index, { ...cur, ...patch });
    });

  const writeCount = preview
    ? [...preview.trades, ...preview.incomes, ...preview.actions].filter((r) =>
        willWrite(r.index, r.status),
      ).length
    : 0;

  const doCommit = () =>
    void run(async () => {
      if (!preview) return;
      const res = await commit.mutateAsync({
        token: preview.token,
        skip: [...skip],
        include_possible_duplicates: includePossible,
        approve_actions: [...approvals]
          .filter(([index, a]) =>
            isTrade(index) ? a.approved && a.unitCost : (a.approved && a.factor) || a.unitCost,
          )
          .map(([index, a]) => ({
            index,
            ...(a.factor && !isTrade(index) ? { factor: a.factor } : {}),
            ...(a.unitCost ? { unit_cost: a.unitCost } : {}),
          })),
        assets: [...new Set([...classes.keys(), ...links.keys()])].map((key) => {
          const assetId = links.get(key);
          return assetId ? { key, asset_id: assetId } : { key, class: classes.get(key)! };
        }),
        cash_movement: cash,
      });
      setResult(res);
      setStep('done');
      onImported?.(files.map((f) => f.file.filename));
      supportEvent('import');
      toast({
        tone: 'success',
        title: t('b3-importados-toast', { n: res.trades + res.incomes + res.actions }),
        description: files.map((f) => f.file.filename).join(', '),
      });
    });

  const accountName = (id: string | null) =>
    accountLabel(id, accounts, preview?.institutions ?? []);

  const rowsOf = <R extends { status: B3RowStatus }>(rows: readonly R[]) =>
    rows.filter((r) => filter === 'all' || r.status === filter);
  const filterFor = (rows: readonly { status: B3RowStatus }[], label: string) => (
    <StatusFilter
      label={label}
      value={filter}
      onChange={setFilter}
      total={rows.length}
      options={B3_STATUS_ORDER.map((s) => ({
        value: s,
        label: B3_STATUS[s].label,
        tone: B3_STATUS[s].tone,
        count: rows.filter((r) => r.status === s).length,
      }))}
    />
  );
  const writeBox = (index: number, status: B3RowStatus, label: string) => {
    const possible = status === 'possible_duplicate';
    const enabled =
      status === 'new' ||
      status === 'warning' ||
      (possible && includePossible) ||
      (status === 'review' && approved(index));
    return (
      <Checkbox
        hideLabel
        label={label}
        checked={willWrite(index, status)}
        disabled={!enabled}
        onChange={(on) => toggleSkip(index, on)}
      />
    );
  };

  const footer =
    step === 'files' ? (
      <>
        <Button variant="ghost" onClick={() => onOpenChange(false)}>
          {t('comum-cancelar')}
        </Button>
        <Button
          variant="primary"
          loading={busy}
          disabled={!files.some((f) => f.detect.kinds.length)}
          onClick={toAccounts}
        >
          {t('comum-continuar')}
        </Button>
      </>
    ) : step === 'accounts' ? (
      <>
        <Button variant="ghost" onClick={() => setStep('files')}>
          {t('comum-voltar')}
        </Button>
        <Button variant="primary" loading={busy} onClick={() => makePreview()}>
          {t('b3-gerar-previa')}
        </Button>
      </>
    ) : step === 'preview' ? (
      <>
        <Button variant="ghost" onClick={() => setStep('accounts')}>
          {t('comum-voltar')}
        </Button>
        <Button variant="primary" loading={busy} disabled={writeCount === 0} onClick={doCommit}>
          {t('b3-importar-n', { n: writeCount })}
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
                    toast({ tone: 'success', title: t('b3-desfeita') });
                    onOpenChange(false);
                  },
                  onError: setError,
                },
              )
            }
          >
            {t('b3-desfazer')}
          </Button>
        ) : null}
        <Button variant="primary" onClick={() => onOpenChange(false)}>
          {t('b3-concluir')}
        </Button>
      </>
    );

  const p = preview;

  return (
    <Dialog
      resizable
      resizeId="b3-import"
      open={open}
      onOpenChange={onOpenChange}
      index="06"
      size="xl"
      title={t('b3-titulo')}
      footer={footer}
    >
      <Steps steps={steps()} current={step} />

      {step === 'files' ? (
        <div className="flex flex-col gap-4">
          <Field label={t('b3-relatorios-area')} hint={t('b3-relatorios-dica')}>
            <FileInput
              multiple
              accept=".xlsx,.csv,text/csv,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
              disabled={busy}
              onChange={(e) => {
                onFiles(e.target.files);
                e.target.value = '';
              }}
            />
          </Field>
          {busy ? <p className="label-caps text-accent">{t('b3-lendo')}</p> : null}
          {files.length ? (
            <ul className="flex flex-col gap-2" aria-label={t('b3-relatorios-escolhidos')}>
              {files.map(({ file, detect }) => (
                <li
                  key={file.filename}
                  className="flex flex-col gap-2 rounded-card border border-line p-3"
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="min-w-0">
                      <p className="truncate text-[13.5px] text-fg">{file.filename}</p>
                      <p className="text-[12px] text-dim">
                        {detect.format.toUpperCase()} · {formatBytes(file.size)}
                        {detect.encoding ? ` · ${detect.encoding}` : ''}
                      </p>
                    </div>
                    <IconButton
                      size="sm"
                      label={t('b3-tirar', { arquivo: file.filename })}
                      icon={<Icon name="x" />}
                      onClick={() =>
                        setFiles((cur) => cur.filter((c) => c.file.filename !== file.filename))
                      }
                    />
                  </div>
                  <KeyValue
                    items={[
                      {
                        label: t('b3-relatorios'),
                        value: detect.kinds.length
                          ? detect.kinds.map((k) => B3_KIND_LABEL[k]).join(', ')
                          : t('b3-nenhum-reconhecido'),
                      },
                      {
                        label: t('b3-periodo-rotulo'),
                        value: period(detect.period_from, detect.period_to),
                      },
                      {
                        label: t('b3-instituicoes'),
                        value: detect.institutions.length ? detect.institutions.join(' · ') : '—',
                      },
                      {
                        label: t('b3-abas'),
                        value: detect.sheets
                          .map((s) =>
                            t('b3-aba-resumo', {
                              aba: s.name,
                              tipo: s.kind ? B3_KIND_LABEL[s.kind] : t('b3-aba-ignorada'),
                              n: s.row_count,
                            }),
                          )
                          .join(' · '),
                      },
                    ]}
                  />
                  <Warnings
                    items={detect.warnings}
                    label={t('b3-avisos-de', { arquivo: file.filename })}
                  />
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      ) : null}

      {step === 'accounts' && p ? (
        <B3AccountsStep
          institutions={p.institutions}
          accounts={accounts}
          choices={choices}
          onChange={(name, c) => setChoices((m) => new Map(m).set(name, c))}
        />
      ) : null}

      {step === 'preview' && p ? (
        <div className="flex flex-col gap-3">
          <KeyValue
            items={[
              { label: t('b3-periodo-rotulo'), value: period(p.period_from, p.period_to) },
              {
                label: t('b3-relatorios'),
                value: [...new Set(p.files.flatMap((f) => f.kinds))]
                  .map((k) => B3_KIND_LABEL[k])
                  .join(', '),
              },
              {
                label: t('b3-contas'),
                value: p.institutions
                  .map(
                    (i) =>
                      `${i.name.split(' ')[0]} → ${i.account_name ?? accountName(i.account_id)}`,
                  )
                  .join(' · '),
              },
            ]}
          />
          <Warnings items={p.warnings} />
          <div className="flex flex-col gap-3 rounded-card border border-line p-3">
            {p.counts.possible_duplicates > 0 ? (
              <Switch
                checked={includePossible}
                onChange={setIncludePossible}
                label={t('b3-incluir-possiveis', { n: p.counts.possible_duplicates })}
                description={t('b3-incluir-possiveis-dica')}
              />
            ) : null}
            <Switch
              checked={cash}
              onChange={setCash}
              label={t('b3-lancar-caixa')}
              description={t('b3-lancar-caixa-dica')}
            />
          </div>

          <Tabs
            value={tab}
            onValueChange={(v) => {
              setTab(v as TabId);
              setFilter('all');
            }}
          >
            <TabsList aria-label={t('b3-secoes')} className="overflow-x-auto overflow-y-hidden">
              <TabsTrigger value="assets">
                {t('b3-aba-ativos')} <span className="num">{p.assets.length}</span>
              </TabsTrigger>
              <TabsTrigger value="trades">
                {t('b3-aba-operacoes')} <span className="num">{p.trades.length}</span>
              </TabsTrigger>
              <TabsTrigger value="incomes">
                {t('b3-aba-proventos')} <span className="num">{p.incomes.length}</span>
              </TabsTrigger>
              <TabsTrigger value="actions">
                {t('b3-aba-eventos')} <span className="num">{p.actions.length}</span>
              </TabsTrigger>
              <TabsTrigger value="ignored">
                {t('b3-aba-ignoradas')} <span className="num">{p.ignored.length}</span>
              </TabsTrigger>
              {p.openings.length ? (
                <TabsTrigger value="openings">
                  {t('b3-aba-abertura')} <span className="num">{p.openings.length}</span>
                </TabsTrigger>
              ) : null}
              {p.position_check ? (
                <TabsTrigger value="position">
                  {t('b3-aba-posicao')} <span className="num">{p.position_check.divergent}</span>
                </TabsTrigger>
              ) : null}
            </TabsList>

            <TabsContent value="assets" className="flex flex-col gap-2">
              <p className="text-[12.5px] text-dim">
                {t('b3-ativos-texto', { n: p.counts.new_assets })}
              </p>
              <PreviewTable
                label={t('b3-ativos-tabela')}
                prefsId="Ativos da importação"
                rows={p.assets}
                getRowId={(a) => a.key}
                columns={[
                  {
                    key: 'asset',
                    header: t('b3-ativo'),
                    hideable: false,
                    sortValue: (a) => a.ticker ?? a.name,
                    cell: (a) => (
                      <>
                        <span className="font-medium text-fg">{a.ticker ?? a.name}</span>
                        {a.ticker ? (
                          <span className="block text-[11px] text-dim">{a.name}</span>
                        ) : null}
                      </>
                    ),
                  },
                  {
                    key: 'status',
                    header: t('b3-situacao'),
                    sortValue: (a) => (a.create ? 0 : 1) + (a.needs_review ? 0 : 2),
                    cell: (a) => (
                      <>
                        {a.create ? (
                          <StatusChip tone="positive">{t('b3-cadastrar')}</StatusChip>
                        ) : (
                          <StatusChip tone="dim">{t('b3-ja-cadastrado')}</StatusChip>
                        )}
                        {a.needs_review ? (
                          <span className="ml-1">
                            <StatusChip tone="warning">{t('b3-conferir')}</StatusChip>
                          </span>
                        ) : null}
                        <RowNote>{a.note}</RowNote>
                      </>
                    ),
                  },
                  {
                    key: 'class',
                    header: t('b3-classe'),
                    className: cx(TD, 'min-w-44'),
                    sortValue: (a) => CLASS_LABEL[classes.get(a.key) ?? a.class],
                    cell: (a) =>
                      a.create && !links.get(a.key) ? (
                        <Select
                          aria-label={t('b3-classe-de', { ativo: a.key })}
                          value={classes.get(a.key) ?? a.class}
                          onValueChange={(v) =>
                            v && setClasses((m) => new Map(m).set(a.key, v as AssetClass))
                          }
                          options={CLASS_ORDER.map((c) => ({ value: c, label: CLASS_LABEL[c] }))}
                        />
                      ) : (
                        CLASS_LABEL[a.class]
                      ),
                  },
                  {
                    key: 'link',
                    header: t('b3-usar-cadastrado'),
                    className: cx(TD, 'min-w-48'),
                    sortable: false,
                    cell: (a) =>
                      a.create ? (
                        <Select
                          aria-label={t('b3-ativo-cadastrado-para', { ativo: a.key })}
                          value={links.get(a.key) ?? NEW_ASSET}
                          onValueChange={(v) =>
                            setLinks((m) => {
                              const next = new Map(m);
                              if (!v || v === NEW_ASSET) next.delete(a.key);
                              else next.set(a.key, v);
                              return next;
                            })
                          }
                          options={[
                            { value: NEW_ASSET, label: t('b3-cadastrar-novo') },
                            ...(existing.data ?? []).map((x) => ({
                              value: x.id,
                              label: x.ticker ? `${x.ticker} · ${x.name}` : x.name,
                            })),
                          ]}
                        />
                      ) : (
                        <span className="text-dim">—</span>
                      ),
                  },
                  {
                    key: 'class_reason',
                    header: t('b3-por-que'),
                    className: cx(TD, 'text-dim first-letter:uppercase'),
                    cell: (a) => a.class_reason,
                  },
                ]}
              />
            </TabsContent>

            <TabsContent value="trades" className="flex flex-col gap-2">
              {filterFor(p.trades, t('b3-filtrar-operacoes'))}
              <PreviewTable
                label={t('b3-operacoes-tabela')}
                prefsId="Operações da importação"
                rows={rowsOf(p.trades)}
                getRowId={(t) => t.index}
                rowClassName={(t) => !willWrite(t.index, t.status) && 'opacity-60'}
                columns={[
                  {
                    key: 'write',
                    header: t('b3-importar'),
                    hideable: false,
                    sortable: false,
                    cell: (t) =>
                      t.status === 'review' ? (
                        <Checkbox
                          label={tr('b3-aprovar')}
                          checked={Boolean(approvals.get(t.index)?.approved)}
                          onChange={(on) => {
                            setApproval(t.index, { approved: on });
                            if (on) toggleSkip(t.index, true);
                          }}
                        />
                      ) : (
                        writeBox(t.index, t.status, tr('b3-importar-operacao', { n: t.index + 1 }))
                      ),
                  },
                  {
                    key: 'status',
                    header: t('b3-situacao'),
                    sortValue: (t) => statusRank(t.status),
                    cell: (t) => (
                      <StatusCell
                        status={t.status}
                        note={t.note}
                        error={t.error}
                        warning={t.warning}
                      />
                    ),
                  },
                  {
                    key: 'date',
                    header: t('b3-data'),
                    className: cx(TD, 'num whitespace-nowrap'),
                    cell: (t) => (
                      <>
                        {formatDate(t.date)}
                        <span className="block text-[11px] text-dim">
                          {B3_ORIGIN_LABEL[t.origin]}
                        </span>
                      </>
                    ),
                  },
                  {
                    key: 'asset',
                    header: t('b3-ativo'),
                    sortValue: (t) => t.ticker ?? t.asset_key,
                    cell: (t) => (
                      <>
                        <span className="font-medium text-fg">{t.ticker ?? t.asset_key}</span>
                        <span className="block text-[11px] text-dim">{sourceLabel(t.source)}</span>
                      </>
                    ),
                  },
                  {
                    key: 'side',
                    header: t('b3-tipo'),
                    className: cx(TD, 'whitespace-nowrap'),
                    cell: (t) => <SideLabel side={t.side} />,
                  },
                  {
                    key: 'quantity',
                    header: t('b3-qtd'),
                    numeric: true,
                    sortValue: (t) => decimal(t.quantity),
                    cell: (t) => <Private>{formatQuantity(t.quantity)}</Private>,
                  },
                  {
                    key: 'price',
                    header: t('b3-preco'),
                    numeric: true,
                    sortValue: (t) => decimal(t.price),
                    cell: (t) =>
                      t.status === 'review' ? (
                        <DecimalInput
                          aria-label={tr('b3-custo-operacao', { n: t.index + 1 })}
                          placeholder={tr('b3-custo')}
                          value={approvals.get(t.index)?.unitCost ?? null}
                          onValueChange={(unitCost) => setApproval(t.index, { unitCost })}
                          invalid={Boolean(
                            approvals.get(t.index)?.approved && !approvals.get(t.index)?.unitCost,
                          )}
                        />
                      ) : (
                        <Private>{formatUnitPrice(t.price, 'BRL')}</Private>
                      ),
                  },
                  {
                    key: 'gross',
                    header: t('b3-valor'),
                    numeric: true,
                    cell: (t) => <Money cents={t.gross} />,
                  },
                  {
                    key: 'account',
                    header: t('b3-conta'),
                    className: cx(TD, 'text-dim'),
                    sortValue: (t) => accountName(t.account_id),
                    cell: (t) => (
                      <>
                        {accountName(t.account_id)}
                        <span className="block text-[11px]">{t.institution.split(' ')[0]}</span>
                      </>
                    ),
                  },
                ]}
              />
            </TabsContent>

            <TabsContent value="incomes" className="flex flex-col gap-2">
              {filterFor(p.incomes, t('b3-filtrar-proventos'))}
              <PreviewTable
                label={t('b3-proventos-tabela')}
                prefsId="Proventos da importação"
                rows={rowsOf(p.incomes)}
                getRowId={(i) => i.index}
                rowClassName={(i) => !willWrite(i.index, i.status) && 'opacity-60'}
                columns={[
                  {
                    key: 'write',
                    header: t('b3-importar'),
                    hideable: false,
                    sortable: false,
                    cell: (i) =>
                      writeBox(i.index, i.status, tr('b3-importar-provento', { n: i.index + 1 })),
                  },
                  {
                    key: 'status',
                    header: t('b3-situacao'),
                    sortValue: (i) => statusRank(i.status),
                    cell: (i) => <StatusCell status={i.status} note={i.note} error={i.error} />,
                  },
                  {
                    key: 'pay_date',
                    header: t('b3-pagamento'),
                    className: cx(TD, 'num whitespace-nowrap'),
                    cell: (i) => formatDate(i.pay_date),
                  },
                  {
                    key: 'asset',
                    header: t('b3-ativo'),
                    sortValue: (i) => i.ticker ?? i.asset_key,
                    cell: (i) => (
                      <>
                        <span className="font-medium text-fg">{i.ticker ?? i.asset_key}</span>
                        <span className="block text-[11px] text-dim">{sourceLabel(i.source)}</span>
                      </>
                    ),
                  },
                  {
                    key: 'kind',
                    header: t('b3-tipo'),
                    sortValue: (i) => INCOME_KIND_LABEL[i.kind],
                    cell: (i) => (
                      <>
                        {INCOME_KIND_LABEL[i.kind]}
                        <span className="block text-[11px] text-dim">{i.movement}</span>
                      </>
                    ),
                  },
                  {
                    key: 'quantity',
                    header: t('b3-qtd'),
                    numeric: true,
                    sortValue: (i) => decimal(i.quantity),
                    cell: (i) => <Private>{i.quantity ? formatQuantity(i.quantity) : '—'}</Private>,
                  },
                  {
                    key: 'gross',
                    header: t('b3-bruto'),
                    numeric: true,
                    cell: (i) => <Money cents={i.gross} />,
                  },
                  {
                    key: 'tax_withheld',
                    header: 'IR',
                    numeric: true,
                    cell: (i) => <Money cents={i.tax_withheld} />,
                  },
                  {
                    key: 'net',
                    header: t('b3-liquido'),
                    numeric: true,
                    cell: (i) => <Money cents={i.net} />,
                  },
                  {
                    key: 'account',
                    header: t('b3-conta'),
                    className: cx(TD, 'text-dim'),
                    sortValue: (i) => accountName(i.account_id),
                    cell: (i) => accountName(i.account_id),
                  },
                ]}
              />
            </TabsContent>

            <TabsContent value="actions" className="flex flex-col gap-2">
              {filterFor(p.actions, t('b3-filtrar-eventos'))}
              <PreviewTable
                label={t('b3-eventos-tabela')}
                prefsId="Eventos corporativos da importação"
                rows={rowsOf(p.actions)}
                getRowId={(a) => a.index}
                rowClassName={(a) =>
                  !willWrite(a.index, a.status) && a.status !== 'review' && 'opacity-60'
                }
                columns={[
                  {
                    key: 'write',
                    header: t('b3-importar'),
                    hideable: false,
                    sortable: false,
                    cell: (a) => {
                      const ap = approvals.get(a.index);
                      return a.status === 'review' ? (
                        <Checkbox
                          label={t('b3-aprovar')}
                          checked={Boolean(ap?.approved)}
                          onChange={(on) => {
                            setApproval(a.index, {
                              approved: on,
                              ...(on && !ap ? { factor: a.factor, unitCost: a.unit_cost } : {}),
                            });
                            if (on) toggleSkip(a.index, true);
                          }}
                        />
                      ) : (
                        writeBox(a.index, a.status, tr('b3-importar-evento', { n: a.index + 1 }))
                      );
                    },
                  },
                  {
                    key: 'status',
                    header: t('b3-situacao'),
                    sortValue: (a) => statusRank(a.status),
                    cell: (a) => <StatusCell status={a.status} note={a.note} error={a.error} />,
                  },
                  {
                    key: 'date',
                    header: t('b3-data'),
                    className: cx(TD, 'num whitespace-nowrap'),
                    cell: (a) => formatDate(a.date),
                  },
                  {
                    key: 'asset',
                    header: t('b3-ativo'),
                    sortValue: (a) => a.ticker ?? a.asset_key,
                    cell: (a) => (
                      <>
                        <span className="font-medium text-fg">{a.ticker ?? a.asset_key}</span>
                        <span className="block text-[11px] text-dim">
                          {a.sources.map(sourceLabel).join(' · ')}
                        </span>
                      </>
                    ),
                  },
                  {
                    key: 'kind',
                    header: t('b3-evento'),
                    sortValue: (a) => ACTION_KIND_LABEL[a.kind],
                    cell: (a) => (
                      <>
                        {ACTION_KIND_LABEL[a.kind]}
                        <span className="block text-[11px] text-dim">{a.movement}</span>
                      </>
                    ),
                  },
                  {
                    key: 'file_quantity',
                    header: t('b3-no-arquivo'),
                    numeric: true,
                    sortValue: (a) => decimal(a.file_quantity),
                    cell: (a) => (
                      <Private>{a.file_quantity ? formatQuantity(a.file_quantity) : '—'}</Private>
                    ),
                  },
                  {
                    key: 'held_quantity',
                    header: t('b3-posicao-antes'),
                    numeric: true,
                    sortValue: (a) => decimal(a.held_quantity),
                    cell: (a) => (
                      <Private>{a.held_quantity ? formatQuantity(a.held_quantity) : '—'}</Private>
                    ),
                  },
                  {
                    key: 'factor',
                    header: t('b3-fator'),
                    className: cx(TD, 'min-w-28'),
                    sortValue: (a) => decimal(approvals.get(a.index)?.factor ?? a.factor),
                    cell: (a) => {
                      const ap = approvals.get(a.index);
                      return a.status === 'review' ? (
                        <DecimalInput
                          aria-label={t('b3-fator-do-evento', { n: a.index + 1 })}
                          placeholder={t('b3-fator-exemplo')}
                          value={ap?.factor ?? a.factor}
                          onValueChange={(factor) => setApproval(a.index, { factor })}
                          invalid={Boolean(ap?.approved && !(ap.factor ?? a.factor))}
                        />
                      ) : a.factor ? (
                        <span className="num">{formatDecimal(a.factor)}</span>
                      ) : (
                        '—'
                      );
                    },
                  },
                  {
                    key: 'unit_cost',
                    header: t('b3-custo-acao'),
                    className: cx(TD, 'min-w-28'),
                    sortValue: (a) => decimal(approvals.get(a.index)?.unitCost ?? a.unit_cost),
                    cell: (a) => {
                      const ap = approvals.get(a.index);
                      return a.kind === 'bonus' &&
                        (a.status === 'review' || (a.status === 'new' && !a.unit_cost)) ? (
                        <DecimalInput
                          aria-label={t('b3-custo-acao-evento', { n: a.index + 1 })}
                          value={ap?.unitCost ?? a.unit_cost}
                          onValueChange={(unitCost) => setApproval(a.index, { unitCost })}
                        />
                      ) : a.unit_cost ? (
                        <Private>{formatUnitPrice(a.unit_cost, 'BRL')}</Private>
                      ) : (
                        '—'
                      );
                    },
                  },
                ]}
              />
              {p.actions.some((a) => a.status === 'review') ? (
                <p className="text-[12px] text-dim">{t('b3-eventos-revisao')}</p>
              ) : null}
            </TabsContent>

            <TabsContent value="ignored" className="flex flex-col gap-2">
              <p className="text-[12.5px] text-dim">{t('b3-ignoradas-texto')}</p>
              <PreviewTable
                label={t('b3-ignoradas-tabela')}
                prefsId="Linhas ignoradas"
                rows={p.ignored}
                getRowId={(r) => `${r.source.filename}-${r.source.sheet}-${r.source.line}`}
                columns={[
                  {
                    key: 'origin',
                    header: t('b3-origem'),
                    className: cx(TD, 'whitespace-nowrap text-dim'),
                    sortValue: (r) => `${B3_KIND_LABEL[r.report]} ${sourceLabel(r.source)}`,
                    cell: (r) => (
                      <>
                        {B3_KIND_LABEL[r.report]}
                        <span className="block text-[11px]">{sourceLabel(r.source)}</span>
                      </>
                    ),
                  },
                  {
                    key: 'rule',
                    header: t('b3-regra'),
                    sortValue: (r) => B3_RULE_LABEL[r.rule],
                    cell: (r) => <StatusChip tone="dim">{B3_RULE_LABEL[r.rule]}</StatusChip>,
                  },
                  { key: 'description', header: t('b3-linha'), cell: (r) => r.description },
                  {
                    key: 'reason',
                    header: t('b3-motivo'),
                    className: cx(TD, 'text-dim first-letter:uppercase'),
                    cell: (r) => r.reason,
                  },
                ]}
              />
            </TabsContent>

            {p.openings.length ? (
              <TabsContent value="openings" className="flex flex-col gap-2">
                <p className="text-[12.5px] text-dim">{t('b3-abertura-texto')}</p>
                <PreviewTable
                  label={t('b3-abertura-tabela')}
                  prefsId="Posições de abertura"
                  rows={p.openings}
                  getRowId={openingKey}
                  columns={[
                    {
                      key: 'include',
                      header: t('b3-importar'),
                      hideable: false,
                      sortable: false,
                      cell: (o) => {
                        const pick = openings.get(openingKey(o));
                        return (
                          <Checkbox
                            hideLabel
                            label={t('b3-abertura-incluir', { ativo: o.ticker ?? o.name })}
                            checked={pick?.on ?? o.chosen}
                            onChange={(on) =>
                              setOpenings((m) =>
                                new Map(m).set(openingKey(o), { on, cost: pick?.cost ?? null }),
                              )
                            }
                          />
                        );
                      },
                    },
                    {
                      key: 'asset',
                      header: t('b3-ativo'),
                      sortValue: (o) => o.ticker ?? o.name,
                      cell: (o) => (
                        <>
                          <span className="font-medium text-fg">{o.ticker ?? o.name}</span>
                          {o.ticker ? (
                            <span className="block text-[11px] text-dim">{o.name}</span>
                          ) : null}
                        </>
                      ),
                    },
                    {
                      key: 'date',
                      header: t('b3-data'),
                      className: cx(TD, 'num whitespace-nowrap'),
                      cell: (o) => formatDate(o.date),
                    },
                    {
                      key: 'quantity',
                      header: t('b3-qtd'),
                      numeric: true,
                      sortValue: (o) => decimal(o.quantity),
                      cell: (o) => <Private>{formatQuantity(o.quantity)}</Private>,
                    },
                    {
                      key: 'cost',
                      header: t('b3-abertura-custo'),
                      className: cx(TD, 'min-w-28'),
                      sortable: false,
                      cell: (o) => {
                        const pick = openings.get(openingKey(o));
                        return (
                          <DecimalInput
                            aria-label={t('b3-abertura-custo-de', { ativo: o.ticker ?? o.name })}
                            value={pick?.cost ?? null}
                            onValueChange={(cost) =>
                              setOpenings((m) =>
                                new Map(m).set(openingKey(o), { on: pick?.on ?? true, cost }),
                              )
                            }
                          />
                        );
                      },
                    },
                    {
                      key: 'account',
                      header: t('b3-conta'),
                      className: cx(TD, 'text-dim'),
                      cell: (o) => accountName(o.account_id),
                    },
                    {
                      key: 'evidence',
                      header: t('b3-abertura-evidencia'),
                      className: cx(TD, 'text-dim first-letter:uppercase'),
                      cell: (o) => o.evidence,
                    },
                  ]}
                />
                <div>
                  <Button
                    variant="secondary"
                    loading={busy}
                    disabled={![...openings.values()].some((o) => o.on)}
                    onClick={() => makePreview(true)}
                  >
                    {t('b3-abertura-aplicar')}
                  </Button>
                </div>
              </TabsContent>
            ) : null}

            {p.position_check ? (
              <TabsContent value="position" className="flex flex-col gap-2">
                <p className="text-[12.5px] text-dim">
                  {t('b3-posicao-texto', {
                    data: formatDate(p.position_check.date),
                    conferem: p.position_check.matched,
                    divergem: p.position_check.divergent,
                  })}
                </p>
                <PreviewTable
                  label={t('b3-posicao-tabela')}
                  prefsId="Conferência com a posição da B3"
                  rows={p.position_check.lines}
                  getRowId={(l) => `${l.asset_key}-${l.institution ?? ''}-${l.account_id ?? ''}`}
                  columns={[
                    {
                      key: 'asset',
                      header: t('b3-ativo'),
                      hideable: false,
                      sortValue: (l) => l.ticker ?? l.name,
                      cell: (l) => (
                        <>
                          <span className="font-medium text-fg">{l.ticker ?? l.name}</span>
                          {l.ticker ? (
                            <span className="block text-[11px] text-dim">{l.name}</span>
                          ) : null}
                        </>
                      ),
                    },
                    {
                      key: 'account',
                      header: t('b3-conta'),
                      className: cx(TD, 'text-dim'),
                      sortValue: (l) => l.account_name,
                      cell: (l) => (
                        <>
                          {l.account_name ?? t('b3-sem-conta')}
                          {l.institution ? (
                            <span className="block text-[11px]">{l.institution.split(' ')[0]}</span>
                          ) : null}
                        </>
                      ),
                    },
                    {
                      key: 'b3_quantity',
                      header: 'B3',
                      numeric: true,
                      sortValue: (l) => decimal(l.b3_quantity),
                      cell: (l) => (
                        <Private>{l.b3_quantity ? formatQuantity(l.b3_quantity) : '—'}</Private>
                      ),
                    },
                    {
                      key: 'computed_quantity',
                      header: t('b3-carteira'),
                      numeric: true,
                      sortValue: (l) => decimal(l.computed_quantity),
                      cell: (l) => (
                        <Private>
                          {l.computed_quantity ? formatQuantity(l.computed_quantity) : '—'}
                        </Private>
                      ),
                    },
                    {
                      key: 'difference',
                      header: t('b3-diferenca'),
                      numeric: true,
                      sortValue: (l) => decimal(l.difference),
                      cell: (l) => (
                        <Private>{l.difference ? formatQuantity(l.difference) : '—'}</Private>
                      ),
                    },
                    {
                      key: 'status',
                      header: t('b3-situacao'),
                      sortValue: (l) => B3_POSITION_STATUS[l.status].label,
                      cell: (l) => (
                        <>
                          <StatusChip tone={B3_POSITION_STATUS[l.status].tone}>
                            {B3_POSITION_STATUS[l.status].label}
                          </StatusChip>
                          <RowNote>{l.note}</RowNote>
                        </>
                      ),
                    },
                  ]}
                />
              </TabsContent>
            ) : null}
          </Tabs>
        </div>
      ) : null}

      {step === 'done' && result ? (
        <div className="flex flex-col gap-3" role="status">
          <p className="text-[15px] text-fg">
            {t('b3-resultado', {
              operacoes: result.trades,
              proventos: result.incomes,
              eventos: result.actions,
              ativos: result.assets_created,
              ignoradas: result.skipped,
            })}
          </p>
          {result.pending_sales > 0 ? (
            <Notice tone="warning" title={t('b3-pendencias-titulo', { n: result.pending_sales })}>
              {t('b3-pendencias-texto')}{' '}
              <Link
                className="text-accent underline-offset-2 hover:underline"
                to="/carteira?aba=vendas"
                onClick={() => onOpenChange(false)}
              >
                {t('b3-pendencias-link')}
              </Link>
            </Notice>
          ) : null}
          {result.errors.length ? (
            <ul
              className="flex flex-col gap-1 text-[12.5px] text-negative"
              aria-label={t('b3-linhas-erro')}
            >
              {result.errors.map((e) => (
                <li key={e.index}>{t('b3-linha-erro', { n: e.index + 1, mensagem: e.message })}</li>
              ))}
            </ul>
          ) : null}
          <KeyValue
            items={[
              {
                label: t('b3-backup-antes'),
                value: result.backup_id ? t('b3-backup-feito') : t('b3-backup-nao-precisou'),
              },
              { label: t('b3-desfazer-rotulo'), value: t('b3-desfazer-como') },
            ]}
          />
        </div>
      ) : null}

      <InlineError error={error} />
    </Dialog>
  );
}
