/**
 * Dados da análise de ativos: estado da coleta (`research.status`, por fonte e tamanho),
 * ajustes (`research.get_settings`/`set_settings`: User-Agent da SEC com e-mail e premissas
 * padrão do Bazin) e "Coletar agora" (`research.sync`), com confirmação: a primeira coleta
 * baixa ~230 MB e leva 1–2 minutos.
 */
import { useEffect, useState, type FormEvent } from 'react';

import type { ResearchSyncReport } from '@/api/generated/ResearchSyncReport';
import type { SyncSource } from '@/api/generated/SyncSource';
import {
  useResearchSettings,
  useResearchStatus,
  useResearchSync,
  useSetResearchSettings,
} from '@/api/hooks/research';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Icon,
  Input,
  InstrumentCard,
  KeyValue,
  LiveIndicator,
  Select,
  TechTable,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDateTime, formatNumber } from '@/lib/format';
import { FormError } from '@/vault/FormError';

import { Check } from '../portfolio/components/Fields';
import { ErrorBlock, LoadingBlock, Notice } from '../portfolio/components/States';
import { safeDate } from '../portfolio/labels';
import { formatBytes } from './format';
import { SYNC_SOURCE_LABEL, SYNC_SOURCE_ORDER, SYNC_STATUS_LABEL } from './labels';
import { numberToInput, parseNumberInput } from './query';

export interface SourceSummary {
  source: SyncSource;
  files: number;
  updated: number;
  unchanged: number;
  failed: number;
  /** Pulados de propósito (plano gratuito, limite de uso, nada a atualizar): não é problema. */
  skipped: number;
  bytes: number;
  rows: number;
  errors: string[];
  /** Por que foi pulado (o rótulo do item explica). */
  notes: string[];
}

const problem = (i: ResearchSyncReport['items'][number]) =>
  i.status === 'error' || (i.status === 'skipped' && Boolean(i.error));

/** Itens da coleta agregados por fonte (na ordem fixa das fontes). */
export function summarizeReport(r: ResearchSyncReport): SourceSummary[] {
  return SYNC_SOURCE_ORDER.flatMap((source) => {
    const items = r.items.filter((i) => i.source === source);
    if (!items.length) return [];
    return [
      {
        source,
        files: items.length,
        updated: items.filter((i) => i.status === 'updated').length,
        unchanged: items.filter((i) => i.status === 'unchanged' || i.status === 'not_found').length,
        // Pulado com aviso (ex.: SEC sem contato) pede ação; sem aviso é de propósito
        // (plano gratuito da brapi, limite de uso).
        failed: items.filter((i) => problem(i)).length,
        skipped: items.filter((i) => i.status === 'skipped' && !i.error).length,
        bytes: items.reduce((t, i) => t + i.bytes, 0),
        rows: items.reduce((t, i) => t + i.rows, 0),
        errors: [...new Set(items.flatMap((i) => (problem(i) && i.error ? [i.error] : [])))],
        notes: [
          ...new Set(items.flatMap((i) => (i.status === 'skipped' && !i.error ? [i.label] : []))),
        ],
      },
    ];
  });
}

function seconds(a: string, b: string): number {
  return Math.max(0, Math.round((new Date(b).getTime() - new Date(a).getTime()) / 1000));
}

function ReportTable({ report }: { report: ResearchSyncReport }) {
  const rows = summarizeReport(report);
  return (
    <div className="flex flex-col gap-2">
      <TechTable
        label={t('pesquisa-dados-coleta-fonte')}
        prefsId="Coleta por fonte"
        dense
        rows={rows}
        getRowId={(r) => r.source}
        columns={[
          {
            key: 'source',
            header: t('pesquisa-dados-fonte'),
            cell: (r) => <span className="font-sans">{SYNC_SOURCE_LABEL[r.source]}</span>,
          },
          {
            key: 'files',
            header: t('pesquisa-dados-arquivos'),
            numeric: true,
            cell: (r) => r.files,
          },
          {
            key: 'status',
            header: t('pesquisa-dados-situacao'),
            cell: (r) => (
              <span className="font-sans text-[12.5px]">
                {r.failed ? (
                  <span className="text-warning" title={r.errors.join(' · ')}>
                    {t('pesquisa-dados-com-problema', { n: r.failed })}
                  </span>
                ) : r.updated ? (
                  <span className="text-positive">
                    {t('pesquisa-dados-n-atualizados', { n: r.updated })}
                  </span>
                ) : r.skipped ? (
                  <span className="text-dim" title={r.notes.join(' · ')}>
                    {t('pesquisa-dados-n-pulados', { n: r.skipped })}
                  </span>
                ) : (
                  <span className="text-dim">{SYNC_STATUS_LABEL.unchanged}</span>
                )}
                {r.failed && r.updated ? (
                  <span className="text-dim">
                    {' · '}
                    {t('pesquisa-dados-n-ok', { n: r.updated })}
                  </span>
                ) : null}
                {r.skipped && (r.failed || r.updated) ? (
                  <span className="text-dim" title={r.notes.join(' · ')}>
                    {' · '}
                    {t('pesquisa-dados-n-pulados', { n: r.skipped })}
                  </span>
                ) : null}
              </span>
            ),
          },
          {
            key: 'bytes',
            header: t('pesquisa-dados-baixado'),
            numeric: true,
            cell: (r) => formatBytes(r.bytes),
          },
          {
            key: 'rows',
            header: t('pesquisa-dados-linhas'),
            numeric: true,
            cell: (r) => formatNumber(r.rows),
          },
        ]}
        footer={{
          source: t('pesquisa-dados-total'),
          bytes: formatBytes(report.bytes),
        }}
      />
      {report.warnings.map((w) => (
        <Notice key={w} tone="warning" title={t('pesquisa-dados-aviso')}>
          {w}
        </Notice>
      ))}
    </div>
  );
}

function Elapsed({ since }: { since: number }) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, []);
  const s = Math.max(0, Math.floor((now - since) / 1000));
  return (
    <span className="num">
      {Math.floor(s / 60)}:{String(s % 60).padStart(2, '0')}
    </span>
  );
}

function SyncCard() {
  const status = useResearchStatus();
  const sync = useResearchSync();
  const [confirming, setConfirming] = useState(false);
  const [force, setForce] = useState(false);
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [report, setReport] = useState<ResearchSyncReport | null>(null);
  const s = status.data;
  const first = s != null && !s.last_sync_at;
  const shown = report ?? s?.last_report ?? null;
  const start = () => {
    setConfirming(false);
    setStartedAt(Date.now());
    sync.mutate(force ? { force: true } : {}, {
      onSuccess: (r) => {
        setReport(r);
        toast({
          tone: r.ok ? 'success' : 'warning',
          title: r.ok ? t('pesquisa-dados-concluida') : t('pesquisa-dados-concluida-avisos'),
          description: t('pesquisa-dados-concluida-texto', {
            tamanho: formatBytes(r.bytes),
            n: r.universe,
          }),
        });
      },
      onSettled: () => setStartedAt(null),
    });
  };
  return (
    <InstrumentCard
      index="01"
      title={t('pesquisa-dados-coleta-titulo')}
      live={sync.isPending ? t('pesquisa-dados-coletando-live') : undefined}
      footer={[
        { label: t('pesquisa-dados-fontes'), value: 'CVM Dados Abertos · B3 COTAHIST · SEC EDGAR' },
        { label: t('pesquisa-dados-primeira-coleta'), value: '~230 MB, 1–2 min' },
      ]}
    >
      {status.isPending ? (
        <LoadingBlock />
      ) : status.isError ? (
        <ErrorBlock error={status.error} onRetry={() => void status.refetch()} />
      ) : (
        <div className="flex flex-col gap-4">
          <KeyValue
            items={[
              {
                label: t('pesquisa-dados-ultima-coleta'),
                value: s?.last_sync_at ? formatDateTime(s.last_sync_at) : t('pesquisa-dados-nunca'),
              },
              {
                label: t('pesquisa-dados-universo'),
                value: s?.universe_size
                  ? t('pesquisa-dados-universo-valor', {
                      n: s.universe_size,
                      data: safeDate(s.universe_as_of),
                    })
                  : t('pesquisa-dados-vazio'),
              },
              { label: t('pesquisa-dados-emissores'), value: formatNumber(s?.issuers ?? 0) },
              { label: t('pesquisa-dados-papeis'), value: formatNumber(s?.securities ?? 0) },
              { label: t('pesquisa-dados-fatos'), value: formatNumber(s?.facts ?? 0) },
              { label: t('pesquisa-dados-documentos'), value: formatNumber(s?.documents ?? 0) },
              { label: t('pesquisa-dados-cotacoes-ate'), value: safeDate(s?.quotes_last_date) },
            ]}
          />
          <div className="flex flex-wrap items-center gap-3">
            <Button
              variant="primary"
              icon={<Icon name="refresh" size={13} />}
              loading={sync.isPending}
              loadingText={t('pesquisa-dados-coletando')}
              onClick={() => {
                sync.reset();
                setConfirming(true);
              }}
            >
              {t('pesquisa-dados-coletar-agora')}
            </Button>
            {sync.isPending && startedAt ? (
              <span className="flex items-center gap-2 text-[12.5px] text-dim" role="status">
                <LiveIndicator label={t('pesquisa-dados-baixando')} tone="accent" />{' '}
                <Elapsed since={startedAt} />
                <span>
                  {'· '}
                  {first ? t('pesquisa-dados-primeira-leva') : t('pesquisa-dados-so-mudou')}
                </span>
              </span>
            ) : null}
          </div>
          {sync.error ? <ErrorBlock error={sync.error} title={t('pesquisa-dados-falhou')} /> : null}
          {shown ? (
            <section
              aria-label={t('pesquisa-dados-ultima-por-fonte')}
              className="flex flex-col gap-2"
            >
              <p className="text-[12.5px] text-dim">
                {t('pesquisa-dados-resumo', {
                  qual: report ? 'esta' : 'ultima',
                  tamanho: formatBytes(shown.bytes),
                  segundos: seconds(shown.started_at, shown.finished_at),
                  universo: shown.universe,
                  radar: shown.radar,
                })}
              </p>
              <ReportTable report={shown} />
            </section>
          ) : null}
        </div>
      )}
      <Dialog
        open={confirming}
        onOpenChange={setConfirming}
        size="sm"
        index="07"
        title={t('pesquisa-dados-coletar-titulo')}
        description={
          first ? t('pesquisa-dados-coletar-primeira') : t('pesquisa-dados-coletar-incremental')
        }
        footer={
          <>
            <DialogClose asChild>
              <Button variant="ghost">{t('comum-cancelar')}</Button>
            </DialogClose>
            <Button variant="primary" onClick={start}>
              {first || force ? t('pesquisa-dados-baixar-230') : t('pesquisa-dados-coletar')}
            </Button>
          </>
        }
      >
        {!first ? (
          <Check
            checked={force}
            onChange={setForce}
            label={t('pesquisa-dados-baixar-tudo')}
            hint={t('pesquisa-dados-baixar-tudo-dica')}
          />
        ) : null}
      </Dialog>
    </InstrumentCard>
  );
}

function SettingsCard() {
  const q = useResearchSettings();
  const save = useSetResearchSettings();
  const [ua, setUa] = useState<string | null>(null);
  const [rate, setRate] = useState<string | null>(null);
  const [years, setYears] = useState<number | null>(null);
  const cur = q.data?.settings;
  const uaValue = ua ?? cur?.sec_user_agent ?? '';
  const rateValue = rate ?? (cur ? numberToInput(cur.bazin_rate, true) : '');
  const yearsValue = years ?? cur?.bazin_years ?? 5;
  const rateNum = parseNumberInput(rateValue, true);
  const rateOk = rateNum != null && rateNum > 0 && rateNum <= 1;
  const uaOk = uaValue.trim().includes('@');
  const dirty = ua != null || rate != null || years != null;
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (!rateOk || !uaOk) return;
    save.mutate(
      { sec_user_agent: uaValue.trim(), bazin_rate: rateNum, bazin_years: yearsValue },
      {
        onSuccess: () => {
          toast({ tone: 'success', title: t('pesquisa-dados-ajustes-salvos') });
          setUa(null);
          setRate(null);
          setYears(null);
        },
      },
    );
  };
  return (
    <InstrumentCard index="02" title={t('pesquisa-dados-ajustes')}>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} />
      ) : (
        <form
          onSubmit={submit}
          className="flex flex-col gap-4"
          aria-label={t('pesquisa-dados-ajustes')}
          noValidate
        >
          {q.data.sec_hint ? (
            <Notice tone="warning" title={t('pesquisa-dados-sec-email')}>
              {q.data.sec_hint}
            </Notice>
          ) : null}
          <Field
            label={t('pesquisa-dados-sec-ua')}
            hint={t('pesquisa-dados-sec-ua-dica')}
            error={!uaOk ? t('pesquisa-dados-sec-ua-erro') : undefined}
          >
            <Input
              value={uaValue}
              onChange={(e) => setUa(e.target.value)}
              placeholder={t('pesquisa-dados-sec-ua-exemplo')}
            />
          </Field>
          <div className="flex flex-wrap items-start gap-3">
            <Field
              label={t('pesquisa-taxa-minima-bazin')}
              className="w-52"
              error={!rateOk ? t('pesquisa-dados-taxa-invalida') : undefined}
            >
              <Input
                numeric
                inputMode="decimal"
                value={rateValue}
                suffix="%"
                onChange={(e) => setRate(e.target.value)}
              />
            </Field>
            <Field label={t('pesquisa-media-dividendos')} className="w-52">
              <Select
                value={String(yearsValue)}
                onValueChange={(v) => setYears(Number(v))}
                options={Array.from({ length: 10 }, (_, i) => ({
                  value: String(i + 1),
                  label: t('pesquisa-n-anos', { n: i + 1 }),
                }))}
              />
            </Field>
          </div>
          <p className="text-[12px] text-dim">{t('pesquisa-dados-bazin-dica')}</p>
          <FormError error={save.error} />
          <div className="flex justify-end gap-2 border-t border-line pt-3">
            {dirty ? (
              <Button
                variant="ghost"
                onClick={() => {
                  setUa(null);
                  setRate(null);
                  setYears(null);
                }}
              >
                {t('pesquisa-dados-descartar')}
              </Button>
            ) : null}
            <Button type="submit" variant="primary" disabled={!dirty} loading={save.isPending}>
              {t('pesquisa-dados-salvar-ajustes')}
            </Button>
          </div>
        </form>
      )}
    </InstrumentCard>
  );
}

/** Estado, coleta e ajustes da análise de ativos (aba de Ativos e Ajustes › Fontes de dados). */
export function ResearchDataPanel() {
  return (
    <div className="grid gap-5 @min-[1040px]:grid-cols-2">
      <SyncCard />
      <SettingsCard />
    </div>
  );
}
