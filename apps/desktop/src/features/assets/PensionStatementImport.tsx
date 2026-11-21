/**
 * Importar o extrato do plano de previdência (docs/brazil/pension-statement.md): o PDF do
 * "Extrato Financeiro do Participante" vira uma prévia (plano, contribuições, seguros de risco,
 * saldo e conferência das somas) e, ao confirmar, as cotas, as cotações e o plano no ativo.
 */
import { useState } from 'react';

import type { Asset } from '@/api/generated/Asset';
import type { PensionRowKind } from '@/api/generated/PensionRowKind';
import type { PensionStatementPreview } from '@/api/generated/PensionStatementPreview';
import type { PensionStatementRow } from '@/api/generated/PensionStatementRow';
import { useAccounts } from '@/api/hooks/accounts';
import { previewPensionStatement, useImportPensionStatement } from '@/api/hooks/pension';
import {
  Button,
  Dialog,
  Field,
  Icon,
  KeyValue,
  Money,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { fileToBase64 } from '@/features/transactions/ui/hooks';
import { FileInput, InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { formatDecimal } from '../portfolio/decimal';

const KIND = {
  participant: 'ativos-prev-linha-participante',
  employer: 'ativos-prev-linha-empregador',
  risk_disability: 'ativos-prev-linha-invalidez',
  risk_death: 'ativos-prev-linha-morte',
} as const satisfies Record<PensionRowKind, string>;

const STATUS = {
  new: 'ativos-prev-linha-nova',
  duplicate: 'ativos-prev-linha-duplicada',
  info: 'ativos-prev-linha-informativa',
} as const;

const NONE = '__none__';

const COLUMNS: TechColumn<PensionStatementRow>[] = [
  {
    key: 'paid_date',
    get header() {
      return t('ativos-pagamento');
    },
    width: '100px',
    cell: (r) => formatDate(r.paid_date),
  },
  {
    key: 'competence',
    get header() {
      return t('ativos-prev-competencia');
    },
    width: '96px',
  },
  {
    key: 'kind',
    get header() {
      return t('ativos-termos-tipo');
    },
    cell: (r) => t(KIND[r.kind]),
  },
  {
    key: 'paid',
    get header() {
      return t('ativos-valor');
    },
    numeric: true,
    cell: (r) => <Money cents={r.retirement || r.risk} />,
  },
  {
    key: 'quotas',
    get header() {
      return t('ativos-prev-cotas');
    },
    numeric: true,
    cell: (r) => (r.retirement ? formatDecimal(r.quotas) : '—'),
  },
  {
    key: 'current_value',
    get header() {
      return t('ativos-prev-valor-hoje');
    },
    numeric: true,
    cell: (r) => (r.retirement ? <Money cents={r.current_value} /> : '—'),
  },
  {
    key: 'status',
    get header() {
      return t('ativos-watch-situacao');
    },
    cell: (r) => <span className="text-dim">{t(STATUS[r.status])}</span>,
  },
];

export function PensionStatementImport({ asset }: { asset: Asset }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button
        variant="secondary"
        icon={<Icon name="arrowDown" size={13} />}
        onClick={() => setOpen(true)}
      >
        {t('ativos-prev-importar')}
      </Button>
      <Dialog
        open={open}
        onOpenChange={setOpen}
        size="xl"
        index="07"
        title={t('ativos-prev-importar-titulo')}
        description={t('ativos-prev-importar-descricao')}
      >
        {open ? <ImportBody asset={asset} onDone={() => setOpen(false)} /> : null}
      </Dialog>
    </>
  );
}

function ImportBody({ asset, onDone }: { asset: Asset; onDone: () => void }) {
  const accounts = useAccounts().data ?? [];
  const save = useImportPensionStatement();
  const [file, setFile] = useState<{ filename: string; content_base64: string } | null>(null);
  const [preview, setPreview] = useState<PensionStatementPreview | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const brokers = accounts.filter((a) => !a.archived && a.kind === 'brokerage');
  const [account, setAccount] = useState<string | null>(null);
  const [riskAccount, setRiskAccount] = useState<string>(NONE);
  const accountId = account ?? brokers[0]?.id ?? null;

  const read = async (f: File | undefined, risk: string = riskAccount) => {
    if (!f) return;
    setBusy(true);
    setError(null);
    try {
      const content = { filename: f.name, content_base64: await fileToBase64(f) };
      setFile(content);
      setPreview(
        await previewPensionStatement({
          files: [content],
          asset_id: asset.id,
          ...(risk !== NONE ? { risk_account_id: risk } : {}),
        }),
      );
    } catch (e) {
      setError(e);
      setPreview(null);
    } finally {
      setBusy(false);
    }
  };

  const recheck = async (risk: string) => {
    setRiskAccount(risk);
    if (!file) return;
    setPreview(
      await previewPensionStatement({
        files: [file],
        asset_id: asset.id,
        ...(risk !== NONE ? { risk_account_id: risk } : {}),
      }),
    );
  };

  const toSave = preview?.rows.filter((r) => r.status === 'new').length ?? 0;
  const tot = preview?.totals;
  const h = preview?.header;
  return (
    <div className="flex flex-col gap-4">
      <Field label={t('ativos-prev-extrato-pdf')}>
        <FileInput
          accept=".pdf,application/pdf"
          disabled={busy}
          onChange={(e) => void read(e.target.files?.[0])}
        />
      </Field>
      {busy ? <p className="label-caps text-accent">{t('ativos-prev-lendo')}</p> : null}
      <InlineError error={error} />
      {preview && tot && h ? (
        <>
          <KeyValue
            items={[
              { label: t('ativos-prev-plano'), value: h.plan_name ?? '—' },
              {
                label: t('ativos-prev-layout'),
                value: `${preview.layout.issuer} · ${preview.layout.version}`,
              },
              { label: t('ativos-prev-cnpj-plano'), value: h.plan_cnpj ?? '—' },
              { label: t('ativos-prev-adesao'), value: h.adhesion ? formatDate(h.adhesion) : '—' },
              {
                label: t('ativos-prev-taxas'),
                value: t('ativos-prev-taxas-valor', {
                  adm: formatDecimal(h.admin_fee_pct),
                  carregamento: formatDecimal(h.loading_fee_pct),
                }),
              },
              {
                label: t('ativos-prev-regime'),
                value:
                  h.tax_regime === 'progressive'
                    ? t('ativos-plano-progressivo')
                    : h.tax_regime === 'regressive'
                      ? t('ativos-plano-regressivo')
                      : '—',
              },
            ]}
          />
          <KeyValue
            items={[
              {
                label: t('ativos-prev-suas-contribuicoes'),
                value: <Money cents={tot.participant} />,
              },
              { label: t('ativos-prev-extras-empregador'), value: <Money cents={tot.employer} /> },
              { label: t('ativos-prev-seguros-risco'), value: <Money cents={tot.risk} /> },
              {
                label: t('ativos-prev-saldo-extrato'),
                value: tot.balance != null ? <Money cents={tot.balance} /> : '—',
              },
              {
                label: t('ativos-prev-cota'),
                value: tot.quota_value
                  ? tot.quota_date
                    ? t('ativos-cotacao-informada-em', {
                        preco: formatDecimal(tot.quota_value),
                        data: formatDate(tot.quota_date),
                      })
                    : formatDecimal(tot.quota_value)
                  : '—',
              },
            ]}
          />
          {preview.warnings.length ? (
            <ul className="flex flex-col gap-1 rounded-card border border-warning/50 bg-warning/5 px-3 py-2 text-[12.5px] text-warning">
              {preview.warnings.map((w) => (
                <li key={w} className="first-letter:uppercase">
                  {w}.
                </li>
              ))}
            </ul>
          ) : (
            <p className="flex items-center gap-2 text-[12.5px] text-positive">
              <Icon name="check" size={13} /> {t('ativos-prev-cotas-conferem')}
            </p>
          )}
          <TechTable
            label={t('ativos-prev-linhas')}
            prefsId="Linhas do extrato"
            columns={COLUMNS}
            rows={preview.rows}
            getRowId={(r) => String(r.index)}
            maxHeight="280px"
            minWidth="760px"
            dense
          />
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label={t('ativos-prev-conta-cotas')} hint={t('ativos-prev-conta-cotas-dica')}>
              <Select
                value={accountId ?? ''}
                onValueChange={setAccount}
                options={brokers.map((a) => ({ value: a.id, label: a.name }))}
                placeholder={t('ativos-prev-crie-corretora')}
              />
            </Field>
            <Field label={t('ativos-prev-seguros-risco')} hint={t('ativos-prev-seguros-dica')}>
              <Select
                value={riskAccount}
                onValueChange={(v) => void recheck(v)}
                options={[
                  { value: NONE, label: t('ativos-prev-nao-lancar') },
                  ...accounts
                    .filter((a) => !a.archived && a.kind !== 'brokerage')
                    .map((a) => ({
                      value: a.id,
                      label: t('ativos-prev-despesa-em', { conta: a.name }),
                    })),
                ]}
              />
            </Field>
          </div>
          <InlineError error={save.error} />
          <div className="flex justify-end gap-2 border-t border-line pt-3">
            <Button variant="ghost" onClick={onDone}>
              {t('comum-cancelar')}
            </Button>
            <Button
              variant="primary"
              disabled={!accountId || toSave === 0}
              loading={save.isPending}
              onClick={() =>
                file &&
                accountId &&
                save.mutate(
                  {
                    files: [file],
                    asset_id: asset.id,
                    account_id: accountId,
                    ...(riskAccount !== NONE ? { risk_account_id: riskAccount } : {}),
                  },
                  {
                    onSuccess: (r) => {
                      toast({
                        tone: 'success',
                        title: t('ativos-prev-importado'),
                        description: t('ativos-prev-importado-resumo', {
                          contribuicoes: r.contributions,
                          seguros: r.risk_expenses,
                          puladas: r.skipped,
                        }),
                      });
                      onDone();
                    },
                  },
                )
              }
            >
              {toSave
                ? t('ativos-prev-importar-linhas', { n: toSave })
                : t('ativos-prev-nada-novo')}
            </Button>
          </div>
        </>
      ) : null}
    </div>
  );
}
