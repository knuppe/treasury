import { useEffect, useMemo, useState } from 'react';
import { supportEvent } from '@/api/hooks/support';
import { useNavigate } from 'react-router';

import type { CsvMapping } from '@/api/generated/CsvMapping';
import type { AccountView } from '@/api/generated/AccountView';
import type { DetectResult } from '@/api/generated/DetectResult';
import type { OfxMatch } from '@/api/generated/OfxMatch';
import type { ImportPreview } from '@/api/generated/ImportPreview';
import type { PreviewRow } from '@/api/generated/PreviewRow';
import type { RowOverride } from '@/api/generated/RowOverride';
import { matchOfx, useAccounts, useUpdateAccount } from '@/api/hooks/accounts';
import { useCategories } from '@/api/hooks/categories';
import { useCreateInstitution, useInstitutions } from '@/api/hooks/household';
import { detectImport, previewImport, useCommitImport, usePresets } from '@/api/hooks/import';
import {
  Button,
  Dialog,
  Field,
  Input,
  KeyValue,
  Money,
  Select,
  toast,
  useDataColumns,
  type DataColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { formatDate } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import {
  Checkbox,
  FileInput,
  InlineError,
  Segmented,
  Switch,
} from '@/features/transactions/ui/controls';
import { fileToBase64, formatBytes } from '@/features/transactions/ui/hooks';
import { ROW_STATUS, TXN_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';

type Step = 'file' | 'mapping' | 'preview';

const STEPS: readonly { id: Step; readonly label: string }[] = [
  {
    id: 'file',
    get label() {
      return t('importacao-etapa-arquivo');
    },
  },
  {
    id: 'mapping',
    get label() {
      return t('importacao-etapa-leitura');
    },
  },
  {
    id: 'preview',
    get label() {
      return t('importacao-etapa-previa');
    },
  },
];

/** Formatos de data do CSV (o valor vai para o núcleo; o rótulo, no idioma do app). */
const dateFormats = () => [
  { value: '%d/%m/%Y', label: t('importacao-data-dmy') },
  { value: '%Y-%m-%d', label: t('importacao-data-ymd') },
  { value: '%d/%m/%y', label: t('importacao-data-dmy-curto') },
  { value: '%d-%m-%Y', label: t('importacao-data-dmy-hifen') },
  { value: '%d.%m.%Y', label: t('importacao-data-dmy-ponto') },
  { value: '%d/%m/%Y|%Y-%m-%d', label: t('importacao-data-dmy-ou-ymd') },
  // Mês antes do dia: bancos dos Estados Unidos.
  { value: '%m/%d/%Y', label: t('importacao-data-mdy') },
  { value: '%m/%d/%y', label: t('importacao-data-mdy-curto') },
  { value: '%m-%d-%Y', label: t('importacao-data-mdy-hifen') },
];

/** Os formatos, com o adivinhado pelo núcleo se ele não estiver na lista. */
const dateOptions = (current: string) => {
  const list = dateFormats();
  return list.some((o) => o.value === current)
    ? list
    : [...list, { value: current, label: current }];
};

const delimiters = () => [
  { value: ';', label: t('importacao-sep-ponto-virgula') },
  { value: ',', label: t('importacao-sep-virgula') },
  { value: '\\t', label: t('importacao-sep-tabulacao') },
  { value: '|', label: t('importacao-sep-barra') },
];

const NONE = '__none__';
const CUSTOM = '__custom__';

/** Largura padrão de cada coluna da prévia (px). */
const PREVIEW_WIDTHS: Record<string, number> = {
  write: 72,
  status: 132,
  date: 104,
  description: 340,
  category: 240,
  amount: 136,
};

const STATUS_ORDER: readonly PreviewRow['status'][] = [
  'new',
  'possible_duplicate',
  'duplicate',
  'error',
  'ignored',
];

function emptyMapping(detect: DetectResult): CsvMapping {
  return {
    delimiter: detect.delimiter ?? ';',
    has_header: detect.has_header,
    skip_rows: detect.skip_rows,
    date_column: '0',
    date_format: '%d/%m/%Y',
    amount_column: '2',
    description_column: '1',
    decimal_comma: true,
    invert_sign: false,
    encoding: detect.encoding,
  };
}

/** `null` do núcleo vira ausente (o tipo diz `?: string`). */
function cleanMapping(m: CsvMapping): CsvMapping {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(m)) if (v !== null && v !== undefined && v !== '') out[k] = v;
  return out as unknown as CsvMapping;
}

const NO_ROWS: string[][] = [];

const STATUS_TONE: Record<PreviewRow['status'], string> = {
  new: 'border-positive/50 text-positive',
  duplicate: 'border-line-strong text-dim',
  possible_duplicate: 'border-warning/60 text-warning',
  error: 'border-negative/60 text-negative',
  ignored: 'border-line text-dim',
};

/**
 * Importação de extrato em etapas: conta e arquivo (OFX/CSV) → leitura
 * (`import.detect` e mapeamento com presets) → prévia (`import.preview`, com
 * duplicados, regras e ajustes por linha) → gravação (`import.commit`) com
 * desfazer pelo histórico.
 */
export function ImportWizard({
  open,
  onOpenChange,
  initialAccountId,
  initialFile,
  onImported,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  initialAccountId?: string;
  /** Arquivo já em mãos (ex.: oferecido pela pasta Downloads), em base64. */
  initialFile?: { name: string; size: number; content_base64: string } | null;
  /** Depois de gravar com sucesso (ex.: apagar o arquivo da pasta Downloads). */
  onImported?: () => void;
}) {
  const navigate = useNavigate();
  const { data: accounts = [] } = useAccounts();
  const { data: presets = [] } = usePresets();
  const { data: categories = [] } = useCategories({ include_archived: true });
  const commit = useCommitImport();
  const updateAccount = useUpdateAccount();
  const createInstitution = useCreateInstitution();
  const { data: institutions = [] } = useInstitutions();

  const [step, setStep] = useState<Step>('file');
  const [accountId, setAccountId] = useState(initialAccountId ?? '');
  const [file, setFile] = useState<{ name: string; size: number; content: string } | null>(null);
  const [detect, setDetect] = useState<DetectResult | null>(null);
  /** Conta reconhecida pelo OFX (banco, agência e número), e a que estava escolhida antes. */
  const [ofxMatch, setOfxMatch] = useState<(OfxMatch & { previous: string }) | null>(null);
  const [savedRef, setSavedRef] = useState(false);
  const [presetId, setPresetId] = useState<string>(CUSTOM);
  const [mapping, setMapping] = useState<CsvMapping | null>(null);
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [skip, setSkip] = useState<Set<number>>(new Set());
  const [overrides, setOverrides] = useState<Map<number, RowOverride>>(new Map());
  const [includePossible, setIncludePossible] = useState(false);
  const [statusFilter, setStatusFilter] = useState<'all' | PreviewRow['status']>('all');
  const [busy, setBusy] = useState(false);
  /** Senha da fatura em PDF protegida (não fica guardada). */
  const [pdfPassword, setPdfPassword] = useState('');
  const [error, setError] = useState<unknown>(null);
  const [lastOpen, setLastOpen] = useState(open);

  if (open !== lastOpen) {
    setLastOpen(open);
    if (open) {
      setStep('file');
      setFile(null);
      setDetect(null);
      setOfxMatch(null);
      setSavedRef(false);
      setPreview(null);
      setError(null);
      setSkip(new Set());
      setOverrides(new Map());
      setIncludePossible(false);
      setStatusFilter('all');
      if (initialAccountId) setAccountId(initialAccountId);
    }
  }
  // Conta padrão só na primeira etapa; depois de ler o arquivo, se a conta não foi
  // reconhecida, o usuário escolhe (o campo fica vazio até lá).
  if (step === 'file' && !accountId && accounts[0])
    setAccountId(accounts.find((a) => a.kind === 'checking')?.id ?? accounts[0].id);
  const account = accounts.find((a) => a.id === accountId);
  // Arquivo que diz com segurança o que é (OFX de fatura ou de conta, PDF de fatura ou de
  // extrato): a conta de destino só lista o tipo certo. CSV não diz: todas as contas.
  const fileKind = detect ? fileAccountKind(detect) : null;
  const destinationAccounts =
    fileKind === 'card'
      ? accounts.filter((a) => a.kind === 'credit_card')
      : fileKind === 'bank'
        ? accounts.filter((a) => a.kind !== 'credit_card')
        : accounts;

  // O OFX traz agência e conta que a conta escolhida ainda não tem: oferece gravar no cadastro.
  const ofxRef =
    detect?.format === 'ofx' && detect.ofx && !detect.ofx.credit_card && account
      ? {
          branch: detect.ofx.branch_id?.trim() || null,
          number: detect.ofx.account_id?.trim() || null,
        }
      : null;
  // De onde veio a conta do OFX: lembrada da última importação, reconhecida pelo cadastro
  // ou escolhida à mão (e lembrada ao gravar, quando o arquivo traz a conta).
  const ofxHint = ofxMatch?.account_id
    ? ofxMatch.account_id === ofxMatch.previous
      ? ofxMatch.remembered
        ? t('importacao-ofx-lembrada')
        : t('importacao-ofx-reconhecida')
      : ofxMatch.remembered
        ? t('importacao-ofx-lembrada-trocada')
        : t('importacao-ofx-reconhecida-trocada')
    : ofxMatch?.reason
      ? detect?.ofx?.account_id?.trim()
        ? t('importacao-ofx-nao-reconheci-lembra', { motivo: ofxMatch.reason })
        : t('importacao-ofx-nao-reconheci', { motivo: ofxMatch.reason })
      : null;
  const canSaveRef =
    !savedRef &&
    ofxRef?.number != null &&
    !ofxMatch?.account_id &&
    account != null &&
    account.kind !== 'credit_card';
  const saveRef = () =>
    void run(async () => {
      if (!account || !ofxRef) return;
      let institutionId: string | null = null;
      const bank = ofxMatch?.bank;
      if (!account.institution_id && bank?.code) {
        const same = institutions.find((i) => i.code === bank.code);
        institutionId =
          same?.id ??
          (
            await createInstitution.mutateAsync({
              name: bank.short_name,
              kind: 'bank',
              code: bank.code,
            })
          ).id;
      }
      await updateAccount.mutateAsync({
        id: account.id,
        branch: ofxRef.branch,
        account_number: ofxRef.number,
        ...(institutionId ? { institution_id: institutionId } : {}),
      });
      setSavedRef(true);
      toast({
        tone: 'success',
        title: t('importacao-ref-gravada'),
        description: t('importacao-ref-gravada-descricao', { conta: account.name }),
      });
    });

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

  const load = (name: string, size: number, read: () => Promise<string>) =>
    void run(async () => {
      const content = await read();
      setFile({ name, size, content });
      const d = await detectImport({ filename: name, content_base64: content });
      setDetect(d);
      setOfxMatch(null);
      setSavedRef(false);
      setPdfPassword('');
      // Extrato de conta em PDF (o extrato por período da Caixa, lido no computador): vai
      // para a conta corrente que a pessoa escolher.
      const bankPdf = d.format === 'pdf' && d.layout?.subject === 'bank_statement';
      if (bankPdf && accounts.find((a) => a.id === accountId)?.kind === 'credit_card')
        setAccountId('');
      // Fatura em PDF: só entra em cartão; sugere o primeiro cartão se a conta não for um.
      else if (
        d.format === 'pdf' &&
        !bankPdf &&
        accounts.find((a) => a.id === accountId)?.kind !== 'credit_card'
      ) {
        setAccountId(accounts.find((a) => a.kind === 'credit_card' && !a.archived)?.id ?? '');
      }
      // A conta escolhida não é do tipo do arquivo: fica vazia (OFX) para a pessoa escolher entre
      // as do tipo certo, nunca uma conta corrente para uma fatura.
      const kind = fileAccountKind(d);
      const fits = (id: string) => {
        const k = accounts.find((a) => a.id === id)?.kind;
        return kind === 'card' ? k === 'credit_card' : kind === 'bank' ? k !== 'credit_card' : true;
      };
      if (d.format === 'ofx' && d.ofx) {
        if (!fits(accountId)) setAccountId('');
        const m = await matchOfx({
          ...(d.ofx.bank_id ? { bank_id: d.ofx.bank_id } : {}),
          ...(d.ofx.branch_id ? { branch_id: d.ofx.branch_id } : {}),
          ...(d.ofx.account_id ? { account_id: d.ofx.account_id } : {}),
          credit_card: d.ofx.credit_card,
        }).catch(() => null);
        if (m) {
          // Conta lembrada de outro tipo (uma corrente para a fatura) não vale.
          const matched = m.account_id && fits(m.account_id) ? m.account_id : null;
          setOfxMatch({ ...m, account_id: matched, previous: accountId });
          if (matched) setAccountId(matched);
          // Arquivo vindo de fora (sem passar pela escolha da conta): o usuário escolhe.
          else if (initialFile && !initialAccountId) setAccountId('');
        }
      } else if ((d.format !== 'pdf' || bankPdf) && initialFile && !initialAccountId) {
        // Veio de fora (importação geral) e o CSV não diz a conta: o usuário escolhe.
        setAccountId('');
      }
      if (d.format === 'csv') {
        setPresetId(d.suggested_preset_id ?? CUSTOM);
        setMapping(d.suggested_mapping ? cleanMapping(d.suggested_mapping) : emptyMapping(d));
      } else setMapping(null);
      setStep('mapping');
    });
  const onFile = (f: File | undefined) => {
    if (f) load(f.name, f.size, () => fileToBase64(f));
  };
  // Arquivo vindo de fora (pasta Downloads): lê assim que o assistente abre.
  useEffect(() => {
    if (!open || !initialFile) return;
    void Promise.resolve().then(() =>
      load(initialFile.name, initialFile.size, () => Promise.resolve(initialFile.content_base64)),
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, initialFile]);

  const makePreview = (forAccount = accountId) =>
    void run(async () => {
      if (!file || !detect || !forAccount) return;
      const p = await previewImport({
        account_id: forAccount,
        filename: file.name,
        content_base64: file.content,
        format: detect.format,
        ...(detect.format === 'pdf' && pdfPassword ? { password: pdfPassword } : {}),
        ...(detect.format === 'csv' && mapping
          ? {
              csv_mapping: cleanMapping(mapping),
              ...(presetId !== CUSTOM ? { preset_id: presetId } : {}),
            }
          : {}),
      });
      setPreview(p);
      setSkip(new Set());
      setOverrides(new Map());
      setStep('preview');
    });

  const willImport = (r: PreviewRow) =>
    !skip.has(r.index) &&
    (r.status === 'new' || (r.status === 'possible_duplicate' && includePossible));
  const importCount = preview ? preview.rows.filter(willImport).length : 0;

  const doCommit = () =>
    void run(async () => {
      if (!preview) return;
      const res = await commit.mutateAsync({
        token: preview.token,
        skip_indexes: [...skip],
        include_possible_duplicates: includePossible,
        overrides: [...overrides.values()],
      });
      onImported?.();
      supportEvent('import');
      // Importou: a janela fecha (não volta para a prévia nem para a escolha do arquivo) e o
      // resultado fica no aviso, com o atalho para os lançamentos importados. Desfazer fica no
      // histórico de alterações.
      toast({
        tone: 'success',
        title: t('importacao-n-importados', { n: res.inserted }),
        description: [
          t('importacao-resultado', {
            inseridos: res.inserted,
            conta: account?.name ?? '',
            pulados: res.skipped,
          }),
          res.installments_paid > 0
            ? t('importacao-resultado-parcelas', { n: res.installments_paid })
            : '',
          res.transfers_linked > 0
            ? t('importacao-resultado-transferencias', { n: res.transfers_linked })
            : '',
          res.planned_settled > 0
            ? t('importacao-resultado-previstos', { n: res.planned_settled })
            : '',
          t('importacao-desfazer-historico'),
        ]
          .filter(Boolean)
          .join(' '),
        action: {
          label: t('importacao-ver-importados'),
          onClick: () => void navigate(txnLink({ import_id: res.batch_id })),
        },
      });
      onOpenChange(false);
    });

  const setOverride = (index: number, patch: Partial<RowOverride>) => {
    const next = new Map(overrides);
    const cur = { ...(next.get(index) ?? { index }), ...patch };
    for (const [k, v] of Object.entries(cur))
      if (v === undefined || v === '') delete (cur as Record<string, unknown>)[k];
    next.set(index, cur);
    setOverrides(next);
  };

  const headers = useMemo(() => detect?.headers ?? [], [detect]);
  // Com cabeçalho, a coluna vai pelo nome (como nos modelos); sem, pelo índice.
  const columnOptions = headers.map((h, i) => ({
    value: mapping?.has_header && h ? h : String(i),
    label: h || t('importacao-csv-coluna', { n: String(i + 1) }),
  }));
  const cols = (current?: string | null) =>
    current && !columnOptions.some((o) => o.value === current)
      ? [...columnOptions, { value: current, label: current.replace(/\+/g, ' + ') }]
      : columnOptions;
  const optCols = (current?: string | null) => [{ value: NONE, label: '—' }, ...cols(current)];
  const visibleRows = useMemo(
    () =>
      preview
        ? preview.rows.filter((r) => statusFilter === 'all' || r.status === statusFilter)
        : [],
    [preview, statusFilter],
  );
  // Ordena pelo que veio do arquivo (descrição e categoria sugeridas): editar uma linha não
  // a tira do lugar. Os ajustes ficam no índice da linha, então a ordem não os perde.
  const previewColumns = useMemo<DataColumn<PreviewRow>[]>(
    () => [
      { key: 'write', header: t('importacao-col-importar'), hideable: false, sortable: false },
      {
        key: 'status',
        header: t('importacao-col-situacao'),
        sortValue: (r) => STATUS_ORDER.indexOf(r.status),
      },
      { key: 'date', header: t('importacao-col-data'), sortValue: (r) => r.date },
      {
        key: 'description',
        header: t('importacao-col-descricao'),
        sortValue: (r) => r.suggested_description ?? r.description,
      },
      {
        key: 'category',
        header: t('importacao-col-categoria'),
        sortValue: (r) =>
          r.kind === 'card_payment'
            ? TXN_KIND.card_payment
            : categories.find((c) => c.id === r.suggested_category_id)?.path,
      },
      {
        key: 'amount',
        header: t('importacao-col-valor'),
        numeric: true,
        sortValue: (r) => r.amount,
      },
    ],
    [categories],
  );
  const previewTable = useDataColumns({
    label: t('importacao-previa'),
    prefsId: 'Prévia da importação',
    columns: previewColumns,
    rows: visibleRows,
    defaultWidths: PREVIEW_WIDTHS,
    fill: true,
  });
  const sampleColumns = useMemo<DataColumn<string[]>[]>(
    () =>
      headers.map((h, i) => ({
        key: String(i),
        header: h || t('importacao-csv-coluna', { n: String(i + 1) }),
        sortValue: (r) => r[i],
      })),
    [headers],
  );
  const sampleTable = useDataColumns({
    label: t('importacao-amostra'),
    prefsId: 'Amostra do arquivo',
    columns: sampleColumns,
    rows: detect?.sample_rows ?? NO_ROWS,
  });

  const setMap = (patch: Partial<CsvMapping>) => {
    setMapping((m) => (m ? { ...m, ...patch } : m));
    setPresetId(CUSTOM);
  };

  const footer =
    step === 'file' ? (
      <Button variant="ghost" onClick={() => onOpenChange(false)}>
        {t('comum-cancelar')}
      </Button>
    ) : step === 'mapping' ? (
      <>
        <Button variant="ghost" onClick={() => setStep('file')}>
          {t('comum-voltar')}
        </Button>
        <Button
          variant="primary"
          loading={busy}
          disabled={!accountId}
          title={accountId ? undefined : t('importacao-escolha-conta-destino')}
          onClick={() => makePreview()}
        >
          {t('importacao-gerar-previa')}
        </Button>
      </>
    ) : step === 'preview' ? (
      <>
        <Button variant="ghost" onClick={() => setStep('mapping')}>
          {t('comum-voltar')}
        </Button>
        <Button variant="primary" loading={busy} disabled={importCount === 0} onClick={doCommit}>
          {t('importacao-importar-n', { n: importCount })}
        </Button>
      </>
    ) : null;

  return (
    <Dialog
      resizable
      resizeId="import-wizard"
      open={open}
      onOpenChange={onOpenChange}
      index="03"
      size="xl"
      title={t('importacao-importar-extrato')}
      footer={footer}
    >
      <ol className="flex items-center gap-2" aria-label={t('importacao-etapas')}>
        {STEPS.map((s, i) => {
          const current = s.id === step;
          const past = STEPS.findIndex((x) => x.id === step) > i;
          return (
            <li
              key={s.id}
              aria-current={current ? 'step' : undefined}
              className={cx(
                'label-caps flex items-center gap-2',
                current ? 'text-accent' : past ? 'text-fg' : undefined,
              )}
            >
              <span className="num">{String(i + 1).padStart(2, '0')}</span>
              {s.label}
              {i < STEPS.length - 1 ? (
                <span aria-hidden className="text-line-strong">
                  {'—'}
                </span>
              ) : null}
            </li>
          );
        })}
      </ol>

      {step === 'file' ? (
        <div className="flex flex-col gap-4">
          <Field label={t('importacao-conta-destino')}>
            <Select
              value={accountId || undefined}
              onValueChange={setAccountId}
              options={accounts.map((a) => ({ value: a.id, label: a.name, hint: a.currency }))}
            />
          </Field>
          <Field label={t('importacao-arquivo-banco')} hint={t('importacao-arquivo-banco-dica')}>
            <FileInput
              accept=".ofx,.qfx,.csv,.txt,.n43,.aeb,.q43,.pdf,text/csv,application/x-ofx,application/pdf"
              disabled={!accountId || busy}
              onChange={(e) => onFile(e.target.files?.[0])}
            />
          </Field>
          {busy ? <p className="label-caps text-accent">{t('importacao-lendo-arquivo')}</p> : null}
        </div>
      ) : null}

      {step === 'mapping' && detect ? (
        <div className="flex flex-col gap-4">
          <AccountField
            accounts={destinationAccounts}
            value={accountId}
            onChange={(v) => {
              setAccountId(v);
              setSavedRef(false);
            }}
            hint={
              ofxHint ??
              (detect.format === 'csv'
                ? t('importacao-csv-sem-conta')
                : detect.format === 'pdf'
                  ? detect.layout?.subject === 'bank_statement'
                    ? t('importacao-pdf-conta-extrato')
                    : t('importacao-pdf-conta-cartao')
                  : undefined)
            }
            warn={!accountId || Boolean(ofxMatch && !ofxMatch.account_id)}
          />
          <KeyValue
            items={[
              {
                label: t('importacao-info-arquivo'),
                value: `${file?.name ?? ''} · ${formatBytes(file?.size ?? 0)}`,
              },
              { label: t('importacao-info-formato'), value: detect.format.toUpperCase() },
              { label: t('importacao-info-codificacao'), value: detect.encoding },
              { label: t('importacao-info-linhas'), value: detect.row_count },
            ]}
          />
          {detect.format === 'ofx' && detect.ofx ? (
            <div className="rounded-card border border-line p-3">
              <p className="label-caps mb-2">
                {detect.ofx.credit_card ? t('importacao-ofx-fatura') : t('importacao-ofx-extrato')}
              </p>
              <KeyValue
                layout="stacked"
                items={[
                  { label: t('importacao-ofx-banco'), value: detect.ofx.bank_id ?? '—' },
                  {
                    label: t('importacao-ofx-conta-arquivo'),
                    value:
                      [detect.ofx.branch_id, detect.ofx.account_id].filter(Boolean).join(' / ') ||
                      '—',
                  },
                  {
                    label: t('importacao-ofx-periodo'),
                    value:
                      detect.ofx.date_start && detect.ofx.date_end
                        ? t('importacao-ofx-periodo-valor', {
                            inicio: formatDate(detect.ofx.date_start),
                            fim: formatDate(detect.ofx.date_end),
                          })
                        : '—',
                  },
                  {
                    label: t('importacao-ofx-saldo'),
                    value:
                      detect.ofx.ledger_balance != null ? (
                        <Money
                          cents={detect.ofx.ledger_balance}
                          currency={detect.ofx.currency ?? account?.currency}
                        />
                      ) : (
                        '—'
                      ),
                  },
                ]}
              />
              <div className="mt-3 flex flex-col gap-2">
                {ofxMatch?.bank ? (
                  <p className="text-[12px] text-dim">
                    {t('importacao-banco-arquivo', {
                      banco: [ofxMatch.bank.code, ofxMatch.bank.short_name]
                        .filter(Boolean)
                        .join(' · '),
                    })}
                  </p>
                ) : null}
                {canSaveRef ? (
                  <div className="flex flex-wrap items-center gap-2">
                    <Button size="sm" variant="secondary" loading={busy} onClick={saveRef}>
                      {t('importacao-gravar-ref', { conta: account.name })}
                    </Button>
                    <span className="text-[12px] text-dim">{t('importacao-gravar-ref-dica')}</span>
                  </div>
                ) : null}
              </div>
              {account && detect.ofx.credit_card !== (account.kind === 'credit_card') ? (
                <p className="mt-2 text-[12.5px] text-warning">
                  {detect.ofx.credit_card
                    ? t('importacao-ofx-cartao-conta')
                    : t('importacao-ofx-conta-cartao')}
                </p>
              ) : null}
            </div>
          ) : null}
          {detect.format === 'pdf' && detect.layout?.subject === 'bank_statement' ? (
            <div className="flex flex-col gap-2 rounded-card border border-line p-3">
              <p className="label-caps">{t('importacao-pdf-extrato-titulo')}</p>
              <p className="text-[12.5px] text-dim">
                {t(
                  detect.encoding === 'ocr'
                    ? 'importacao-pdf-extrato-lidos-ocr'
                    : 'importacao-pdf-extrato-lidos',
                  { n: detect.row_count, leiaute: detect.layout.issuer },
                )}
              </p>
              {account?.kind === 'credit_card' ? (
                <p className="text-[12.5px] text-warning">
                  {t('importacao-pdf-extrato-nao-cartao')}
                </p>
              ) : null}
            </div>
          ) : detect.format === 'pdf' ? (
            <div className="flex flex-col gap-3 rounded-card border border-line p-3">
              <p className="label-caps">{t('importacao-pdf-titulo')}</p>
              <p className="text-[12.5px] text-dim">
                {detect.row_count
                  ? t('importacao-pdf-lidos', { n: detect.row_count })
                  : t('importacao-pdf-protegido')}
              </p>
              <Field label={t('importacao-pdf-senha')} hint={t('importacao-pdf-senha-dica')}>
                <Input
                  type="password"
                  autoComplete="off"
                  value={pdfPassword}
                  onChange={(e) => setPdfPassword(e.target.value)}
                  className="max-w-60"
                />
              </Field>
              {account && account.kind !== 'credit_card' ? (
                <p className="text-[12.5px] text-warning">{t('importacao-pdf-nao-cartao')}</p>
              ) : null}
            </div>
          ) : null}
          {detect.format === 'csv' && mapping ? (
            <div className="flex flex-col gap-3">
              <div className="grid gap-3 sm:grid-cols-3">
                <Field
                  label={t('importacao-csv-modelo')}
                  hint={
                    detect.suggested_preset_id ? t('importacao-csv-modelo-reconhecido') : undefined
                  }
                >
                  <Select
                    value={presetId}
                    onValueChange={(id) => {
                      setPresetId(id);
                      const p = presets.find((x) => x.id === id);
                      if (p)
                        setMapping(
                          cleanMapping({
                            ...p.mapping,
                            delimiter: p.mapping.delimiter ?? detect.delimiter,
                            skip_rows: detect.skip_rows,
                            encoding: detect.encoding,
                          }),
                        );
                    }}
                    options={[
                      { value: CUSTOM, label: t('importacao-csv-personalizado') },
                      ...presets.map((p) => ({ value: p.id, label: p.name })),
                    ]}
                  />
                </Field>
                <Field label={t('importacao-csv-separador')}>
                  <Select
                    value={mapping.delimiter ?? ';'}
                    onValueChange={(delimiter) => setMap({ delimiter })}
                    options={delimiters()}
                  />
                </Field>
                <Field label={t('importacao-csv-pular')}>
                  <Input
                    type="number"
                    min={0}
                    numeric
                    value={mapping.skip_rows ?? 0}
                    onChange={(e) => setMap({ skip_rows: Number(e.target.value) })}
                  />
                </Field>
                <Field label={t('importacao-csv-data')}>
                  <Select
                    value={mapping.date_column}
                    onValueChange={(date_column) => setMap({ date_column })}
                    options={cols(mapping.date_column)}
                  />
                </Field>
                <Field label={t('importacao-csv-formato-data')}>
                  <Select
                    value={mapping.date_format}
                    onValueChange={(date_format) => setMap({ date_format })}
                    options={dateOptions(mapping.date_format)}
                  />
                </Field>
                <Field label={t('importacao-csv-descricao')}>
                  <Select
                    value={mapping.description_column}
                    onValueChange={(description_column) => setMap({ description_column })}
                    options={cols(mapping.description_column)}
                  />
                </Field>
              </div>
              <Segmented
                label={t('importacao-csv-valor')}
                size="sm"
                value={mapping.amount_column ? 'single' : 'split'}
                onChange={(v) =>
                  setMap(
                    v === 'single'
                      ? {
                          amount_column: columnOptions[0]?.value,
                          debit_column: undefined,
                          credit_column: undefined,
                        }
                      : {
                          amount_column: undefined,
                          debit_column: columnOptions[0]?.value,
                          credit_column: columnOptions[1]?.value,
                        },
                  )
                }
                options={[
                  { value: 'single', label: t('importacao-csv-valor-unico') },
                  { value: 'split', label: t('importacao-csv-valor-separado') },
                ]}
              />
              <div className="grid gap-3 sm:grid-cols-3">
                {mapping.amount_column ? (
                  <Field label={t('importacao-csv-valor-sinal')}>
                    <Select
                      value={mapping.amount_column}
                      onValueChange={(amount_column) => setMap({ amount_column })}
                      options={cols(mapping.amount_column)}
                    />
                  </Field>
                ) : (
                  <>
                    <Field label={t('importacao-csv-debito')}>
                      <Select
                        value={mapping.debit_column ?? NONE}
                        onValueChange={(v) => setMap({ debit_column: v === NONE ? undefined : v })}
                        options={optCols(mapping.debit_column)}
                      />
                    </Field>
                    <Field label={t('importacao-csv-credito')}>
                      <Select
                        value={mapping.credit_column ?? NONE}
                        onValueChange={(v) => setMap({ credit_column: v === NONE ? undefined : v })}
                        options={optCols(mapping.credit_column)}
                      />
                    </Field>
                  </>
                )}
                <Field
                  label={t('importacao-csv-identificador')}
                  hint={t('importacao-csv-identificador-dica')}
                >
                  <Select
                    value={mapping.external_id_column ?? NONE}
                    onValueChange={(v) =>
                      setMap({ external_id_column: v === NONE ? undefined : v })
                    }
                    options={optCols(mapping.external_id_column)}
                  />
                </Field>
              </div>
              <div className="flex flex-wrap gap-x-6 gap-y-2">
                <Checkbox
                  label={t('importacao-csv-cabecalho')}
                  checked={mapping.has_header}
                  onChange={(has_header) => setMap({ has_header })}
                />
                <Checkbox
                  label={t('importacao-csv-virgula')}
                  checked={mapping.decimal_comma}
                  onChange={(decimal_comma) => setMap({ decimal_comma })}
                />
                <Checkbox
                  label={t('importacao-csv-inverter')}
                  checked={mapping.invert_sign}
                  onChange={(invert_sign) => setMap({ invert_sign })}
                />
              </div>
            </div>
          ) : null}
          {detect.sample_rows.length > 0 ? (
            <div className="max-h-56 overflow-auto rounded-card border border-line">
              <table
                className={cx(
                  'border-separate border-spacing-0 text-[12px]',
                  !sampleTable.fixed && 'w-full',
                )}
                style={sampleTable.tableStyle}
                aria-label={t('importacao-amostra')}
              >
                {sampleTable.colgroup}
                <thead className="sticky top-0 sticky-surface">
                  <tr>
                    {sampleTable.shown.map((c) =>
                      sampleTable.headerCell(c, {
                        className:
                          'label-caps border-b border-line px-2 py-1.5 font-medium whitespace-nowrap',
                      }),
                    )}
                    {sampleTable.pickerCell('border-b border-line px-1 py-1.5')}
                  </tr>
                </thead>
                <tbody>
                  {sampleTable.sortRows(detect.sample_rows).map((r) => (
                    <tr key={detect.sample_rows.indexOf(r)}>
                      {sampleTable.shown.map((c) => (
                        <td
                          key={c.key}
                          className={cx(
                            'num border-b border-line/60 px-2 py-1 whitespace-nowrap text-fg',
                            sampleTable.fixed && 'overflow-hidden text-ellipsis',
                          )}
                        >
                          {r[Number(c.key)]}
                        </td>
                      ))}
                      {sampleTable.fillerCell('border-b border-line/60')}
                    </tr>
                  ))}
                </tbody>
              </table>
              {sampleTable.menus}
            </div>
          ) : null}
        </div>
      ) : null}

      {step === 'preview' && preview ? (
        <div className="flex min-h-0 flex-1 flex-col gap-3">
          <AccountField
            accounts={accounts}
            value={accountId}
            onChange={(v) => {
              setAccountId(v);
              setSavedRef(false);
              makePreview(v);
            }}
            hint={t('importacao-troca-conta-dica')}
            compact
          />
          <div
            className="flex flex-wrap items-center gap-2"
            role="group"
            aria-label={t('importacao-filtrar-linhas')}
          >
            {(['all', 'new', 'possible_duplicate', 'duplicate', 'error', 'ignored'] as const).map(
              (s) => {
                const n = s === 'all' ? preview.rows.length : preview.counts[s];
                if (s !== 'all' && n === 0) return null;
                return (
                  <button
                    key={s}
                    type="button"
                    aria-pressed={statusFilter === s}
                    onClick={() => setStatusFilter(s)}
                    className={cx(
                      'label-caps inline-flex h-7 items-center gap-1.5 rounded-control border px-2',
                      statusFilter === s
                        ? 'border-accent bg-accent-soft text-accent'
                        : s === 'all'
                          ? 'border-line'
                          : STATUS_TONE[s],
                    )}
                  >
                    {s === 'all' ? t('importacao-todas') : ROW_STATUS[s]}{' '}
                    <span className="num">{n}</span>
                  </button>
                );
              },
            )}
          </div>
          {preview.card_pdf ? (
            <KeyValue
              items={[
                {
                  label: t('importacao-fatura'),
                  value: preview.card_pdf.due_date
                    ? t('importacao-fatura-vence', {
                        emissor: preview.card_pdf.issuer ?? t('importacao-cartao'),
                        data: formatDate(preview.card_pdf.due_date),
                      })
                    : (preview.card_pdf.issuer ?? t('importacao-cartao')),
                },
                {
                  label: t('importacao-total-impresso'),
                  value:
                    preview.card_pdf.total != null ? (
                      <Money cents={preview.card_pdf.total} currency={preview.currency} />
                    ) : (
                      '—'
                    ),
                },
                {
                  label: t('importacao-lido-linhas'),
                  value: (
                    <span
                      className={preview.card_pdf.total_matches === false ? 'text-warning' : ''}
                    >
                      <Money cents={preview.card_pdf.rows_total} currency={preview.currency} />
                      {preview.card_pdf.total_matches ? ` ${t('importacao-confere')}` : ''}
                    </span>
                  ),
                },
                {
                  label: t('importacao-layout'),
                  value: `${preview.card_pdf.layout.issuer} ${preview.card_pdf.layout.version}`,
                },
              ]}
            />
          ) : null}
          {preview.warnings.length ? (
            <ul
              className="flex flex-col gap-1 rounded-card border border-warning/50 bg-warning/5 px-3 py-2 text-[12.5px] text-warning"
              aria-label={t('importacao-avisos')}
            >
              {preview.warnings.map((w) => (
                <li key={w} className="first-letter:uppercase">
                  {w}
                </li>
              ))}
            </ul>
          ) : null}
          {preview.counts.possible_duplicate > 0 ? (
            <Switch
              checked={includePossible}
              onChange={setIncludePossible}
              label={t('importacao-incluir-possiveis', { n: preview.counts.possible_duplicate })}
              description={t('importacao-incluir-possiveis-dica')}
            />
          ) : null}
          <div className="min-h-[220px] flex-1 overflow-auto rounded-card border border-line">
            <table
              className="border-separate border-spacing-0 text-[12.5px]"
              style={previewTable.tableStyle}
              aria-label={t('importacao-previa')}
            >
              {previewTable.colgroup}
              <thead className="sticky top-0 z-[1] sticky-surface">
                <tr>
                  {previewTable.shown.map((c) =>
                    previewTable.headerCell(c, {
                      className: 'label-caps truncate border-b border-line px-2 py-1.5 font-medium',
                    }),
                  )}
                  {previewTable.pickerCell('border-b border-line px-1 py-1.5')}
                </tr>
              </thead>
              <tbody>
                {previewTable.sortRows(visibleRows).map((r) => {
                  const importable = r.status === 'new' || r.status === 'possible_duplicate';
                  const o = overrides.get(r.index);
                  const kind =
                    r.kind === 'income' || (r.kind === 'transfer' && (r.amount ?? 0) > 0)
                      ? 'income'
                      : 'expense';
                  const suggested = categories.find((c) => c.id === r.suggested_category_id);
                  return (
                    <tr key={r.index} className={cx(!willImport(r) && 'opacity-60')}>
                      {previewTable.shown.map((c) => {
                        switch (c.key) {
                          case 'write':
                            return (
                              <td key="write" className="border-b border-line/60 px-2 py-1">
                                <Checkbox
                                  hideLabel
                                  label={t('importacao-importar-linha', { n: String(r.index + 1) })}
                                  checked={willImport(r)}
                                  disabled={
                                    !importable ||
                                    (r.status === 'possible_duplicate' && !includePossible)
                                  }
                                  onChange={(on) => {
                                    const next = new Set(skip);
                                    if (on) next.delete(r.index);
                                    else next.add(r.index);
                                    setSkip(next);
                                  }}
                                />
                              </td>
                            );
                          case 'status':
                            return (
                              <td key="status" className="border-b border-line/60 px-2 py-1">
                                <span
                                  className={cx(
                                    'label-caps inline-flex h-5 items-center rounded-control border px-1.5 whitespace-nowrap',
                                    STATUS_TONE[r.status],
                                  )}
                                  title={r.note ?? r.error ?? undefined}
                                >
                                  {ROW_STATUS[r.status]}
                                </span>
                                {r.error || r.note ? (
                                  <span className="mt-0.5 block max-w-48 text-[11px] text-dim first-letter:uppercase">
                                    {r.error ?? r.note}
                                  </span>
                                ) : null}
                              </td>
                            );
                          case 'date':
                            return (
                              <td
                                key="date"
                                className="num border-b border-line/60 px-2 py-1 whitespace-nowrap"
                              >
                                {r.date ? formatDate(r.date) : '—'}
                              </td>
                            );
                          case 'description':
                            return (
                              <td key="description" className="border-b border-line/60 px-2 py-1">
                                {importable ? (
                                  <input
                                    aria-label={t('importacao-descricao-linha', {
                                      n: String(r.index + 1),
                                    })}
                                    defaultValue={r.suggested_description ?? r.description}
                                    onBlur={(e) => {
                                      const v = e.target.value.trim();
                                      setOverride(r.index, {
                                        description: v && v !== r.description ? v : undefined,
                                      });
                                    }}
                                    title={r.suggested_description ?? r.description}
                                    className="h-7 w-full rounded-control border border-transparent bg-transparent px-1 text-[12.5px] text-fg outline-none hover:border-line focus:border-accent"
                                  />
                                ) : (
                                  <span className="block truncate text-dim" title={r.description}>
                                    {r.description}
                                  </span>
                                )}
                                {r.rule_id ? (
                                  <span className="block px-1 text-[11px] text-accent">
                                    {t('importacao-sugestao-regra')}
                                  </span>
                                ) : null}
                              </td>
                            );
                          case 'category':
                            return (
                              <td key="category" className="border-b border-line/60 px-2 py-1">
                                {importable && r.kind !== 'card_payment' ? (
                                  <CategoryPicker
                                    compact
                                    kind={kind}
                                    label={t('importacao-categoria-linha', {
                                      n: String(r.index + 1),
                                    })}
                                    value={o?.category_id ?? r.suggested_category_id}
                                    onChange={(id) =>
                                      setOverride(r.index, { category_id: id ?? undefined })
                                    }
                                  />
                                ) : (
                                  <span className="text-dim">
                                    {r.kind === 'card_payment'
                                      ? TXN_KIND.card_payment
                                      : (suggested?.path ?? '—')}
                                  </span>
                                )}
                              </td>
                            );
                          case 'amount':
                            return (
                              <td
                                key="amount"
                                className="border-b border-line/60 px-2 py-1 text-right whitespace-nowrap"
                              >
                                {r.amount != null ? (
                                  <Money cents={r.amount} currency={preview.currency} tone="auto" />
                                ) : (
                                  '—'
                                )}
                              </td>
                            );
                          default:
                            return null;
                        }
                      })}
                      {previewTable.fillerCell('border-b border-line/60')}
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      ) : null}

      <InlineError error={error} />
    </Dialog>
  );
}

/**
 * Que tipo de conta o arquivo pede, quando ele diz com segurança: `card` para a fatura (OFX de
 * cartão, PDF de fatura) e `bank` para o extrato de conta (OFX de banco, PDF de extrato); CSV
 * e o resto, `null` (todas as contas).
 */
export function fileAccountKind(d: DetectResult): 'card' | 'bank' | null {
  if (d.format === 'ofx' && d.ofx) return d.ofx.credit_card ? 'card' : 'bank';
  if (d.format === 'pdf') return d.layout?.subject === 'bank_statement' ? 'bank' : 'card';
  return null;
}

/** Conta de destino da importação, com o motivo quando o arquivo não disse qual é. */
function AccountField({
  accounts,
  value,
  onChange,
  hint,
  warn,
  compact,
}: {
  accounts: readonly AccountView[];
  value: string;
  onChange: (id: string) => void;
  hint?: string;
  warn?: boolean;
  compact?: boolean;
}) {
  return (
    <div className={cx(compact ? 'max-w-[34rem]' : 'max-w-[40rem]')}>
      <Field
        label={t('importacao-conta-destino')}
        hint={hint}
        className={cx(
          warn && 'rounded-card border border-warning/60 bg-warning/5 p-3 [&_p]:text-warning',
        )}
      >
        <Select
          value={value || undefined}
          placeholder={t('importacao-escolha-conta')}
          invalid={!value}
          onValueChange={onChange}
          options={accounts.map((a) => ({
            value: a.id,
            label: a.name,
            hint: [a.institution_name, a.account_number].filter(Boolean).join(' · ') || a.currency,
          }))}
        />
      </Field>
    </div>
  );
}
