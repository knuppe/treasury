import { useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';

import type { ExportResult } from '@/api/generated/ExportResult';
import type { BrapiUsageView } from '@/api/generated/BrapiUsageView';
import {
  useBrapiTest,
  useDataSources,
  useMarketSync,
  useSetBrapiLimit,
  useSetBrapiToken,
  useSetFredKey,
} from '@/api/hooks/dataSources';
import { useExportCsv, useExportJson } from '@/api/hooks/export';
import { useMcpAudit, useMcpConfig } from '@/api/hooks/mcp';
import { invalidatePerformance } from '@/api/hooks/portfolio';
import { useAppInfo, useAppSettings, useSaveAppSettings } from '@/api/hooks/settings';
import { useUndoToast } from '@/api/hooks/undoToast';
import {
  Button,
  Field,
  Icon,
  InstrumentCard,
  Input,
  KeyValue,
  PasswordInput,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { formatDate, formatDateTime, formatMoney } from '@/lib/format';
import { copySecret, copyText } from '@/lib/osGuard';
import { useUi } from '@/state/ui';
import type { AuditEntry } from '@/api/generated/AuditEntry';
import type { DataSyncReport } from '@/api/generated/DataSyncReport';
import { InlineError, QueryState, Segmented, Switch } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { ResearchDataPanel } from '@/features/research/DataPanel';
import { useLevel } from '@/app/profile';
import { useDeveloperMode } from '@/app/useVisibleScreens';
import { setLocale, t, useLocale, type AppLocale } from '@/i18n';
import { LanguagePicker } from '@/i18n/LanguagePicker';

import { offerCategoryTranslation } from './categoryTranslation';
import { ProvidersCard, ReferenceCountryCard } from './CountrySources';
import { ExternalSourcesCard } from './ExternalSources';
import { tRich } from './richText';
import { SourceKeyCards } from './SourceKeys';
import { ThemePicker } from './ThemePicker';
import { McpAccess } from './McpAccess';

/** Nome do idioma da interface, escrito na própria língua. */
function languageName(code: AppLocale): string {
  switch (code) {
    case 'pt-BR':
      return t('comum-idioma-pt-BR');
    case 'en':
      return t('comum-idioma-en');
    case 'es':
      return t('comum-idioma-es');
    case 'pt-PT':
      return t('comum-idioma-pt-PT');
    default:
      return t('casca-idioma-pseudo');
  }
}

// ── Preferências ────────────────────────────────────────────────

/** 11.9 · Preferências: competência × caixa, tema, privacidade, idioma. */
export function PreferencesSection() {
  const { data: settings } = useAppSettings();
  const save = useSaveAppSettings();
  const theme = useUi((s) => s.theme);
  const setTheme = useUi((s) => s.setTheme);
  const privacy = useUi((s) => s.privacy);
  const setPrivacy = useUi((s) => s.setPrivacy);
  const privacyOnUnlock = useUi((s) => s.privacyOnUnlock);
  const setPrivacyOnUnlock = useUi((s) => s.setPrivacyOnUnlock);
  const locale = useLocale();
  const developer = useDeveloperMode();
  const advanced = useLevel() === 'avancado';
  const client = useQueryClient();
  const undoToast = useUndoToast();
  return (
    <div className="grid gap-5 @min-[1040px]:grid-cols-2">
      <InstrumentCard
        index="12.9"
        title={t('ajustes-pref-exibicao')}
        className="@min-[1040px]:col-span-2"
      >
        <div className="flex flex-col gap-4">
          <fieldset className="max-w-[420px]">
            <legend className="label-caps mb-3">{t('casca-idioma')}</legend>
            <LanguagePicker
              label={t('casca-idioma')}
              value={locale}
              developer={developer}
              onChange={(code) =>
                void setLocale(code).then(
                  // Com o idioma novo: traduzir também as categorias padrão?
                  () => offerCategoryTranslation(client, undoToast),
                  (e: unknown) =>
                    toast({ tone: 'error', title: t('casca-idioma'), description: errorText(e) }),
                )
              }
            />
            <p className="mt-3 text-[12.5px] text-dim">{t('casca-idioma-dica')}</p>
          </fieldset>
          <fieldset>
            <legend className="label-caps mb-3">{t('ajustes-pref-tema')}</legend>
            <ThemePicker value={theme} onChange={setTheme} />
            <p className="mt-3 text-[12.5px] text-dim">{t('ajustes-pref-tema-dica')}</p>
          </fieldset>
          <Switch
            checked={privacy}
            onChange={setPrivacy}
            label={t('ajustes-pref-privacidade')}
            description={t('ajustes-pref-privacidade-dica')}
          />
          <Switch
            checked={privacyOnUnlock}
            onChange={setPrivacyOnUnlock}
            label={t('ajustes-pref-privacidade-abrir')}
            description={t('ajustes-pref-privacidade-abrir-dica')}
          />
        </div>
      </InstrumentCard>
      <InstrumentCard index="12.10" title={t('ajustes-pref-orcamento')}>
        {settings ? (
          <div className="flex flex-col gap-4">
            <Field label={t('ajustes-pref-base')} hint={t('ajustes-pref-base-dica')}>
              <Segmented
                label={t('ajustes-pref-base')}
                value={settings.budget_basis}
                onChange={(budget_basis) =>
                  save.mutate(
                    { budget_basis },
                    {
                      onSuccess: () =>
                        toast({
                          tone: 'success',
                          title:
                            budget_basis === 'cash'
                              ? t('ajustes-pref-base-caixa')
                              : t('ajustes-pref-base-competencia'),
                        }),
                    },
                  )
                }
                options={[
                  { value: 'accrual', label: t('ajustes-pref-competencia') },
                  { value: 'cash', label: t('ajustes-pref-caixa') },
                ]}
              />
            </Field>
            <Field label={t('ajustes-pref-base-fluxo')} hint={t('ajustes-pref-base-fluxo-dica')}>
              <Segmented
                label={t('ajustes-pref-base-fluxo')}
                value={settings.flow_basis}
                onChange={(flow_basis) =>
                  save.mutate(
                    { flow_basis },
                    {
                      onSuccess: () =>
                        toast({
                          tone: 'success',
                          title:
                            flow_basis === 'cash'
                              ? t('ajustes-pref-base-fluxo-caixa')
                              : t('ajustes-pref-base-fluxo-competencia'),
                        }),
                    },
                  )
                }
                options={[
                  { value: 'accrual', label: t('ajustes-pref-competencia') },
                  { value: 'cash', label: t('ajustes-pref-caixa') },
                ]}
              />
            </Field>
            <Switch
              checked={settings.close_to_tray}
              onChange={(close_to_tray) => save.mutate({ close_to_tray })}
              label={t('ajustes-pref-bandeja')}
              description={t('ajustes-pref-bandeja-dica')}
            />
            <KeyValue
              items={[
                { label: t('casca-idioma'), value: languageName(locale) },
                {
                  label: t('ajustes-pref-formatos'),
                  value: `${formatMoney(123456)} · ${formatDate('2026-10-14')}`,
                },
              ]}
            />
            <InlineError error={save.error} />
          </div>
        ) : null}
      </InstrumentCard>
      <InstrumentCard index="12.22" title={t('ajustes-pref-downloads')}>
        {settings ? (
          <div className="flex flex-col gap-3">
            <Switch
              checked={settings.watch_downloads}
              onChange={(watch_downloads) =>
                save.mutate(
                  { watch_downloads },
                  {
                    onSuccess: () =>
                      toast({
                        tone: 'success',
                        title: watch_downloads
                          ? t('ajustes-pref-downloads-ligado')
                          : t('ajustes-pref-downloads-desligado'),
                        description: watch_downloads
                          ? t('ajustes-pref-downloads-ligado-dica')
                          : undefined,
                      }),
                  },
                )
              }
              label={t('ajustes-pref-downloads-acompanhar')}
              description={t('ajustes-pref-downloads-acompanhar-dica')}
            />
            <Switch
              checked={settings.downloads_delete_after_import}
              disabled={!settings.watch_downloads}
              onChange={(downloads_delete_after_import) =>
                save.mutate({ downloads_delete_after_import })
              }
              label={t('ajustes-pref-downloads-apagar')}
              description={t('ajustes-pref-downloads-apagar-dica')}
            />
            <p className="text-[12px] text-dim">{t('ajustes-pref-downloads-privacidade')}</p>
            <Switch
              checked={settings.watch_inbox}
              onChange={(watch_inbox) => save.mutate({ watch_inbox })}
              label={t('ajustes-pref-caixa-entrada')}
              description={t('ajustes-pref-caixa-entrada-dica', {
                pasta: settings.inbox_folder.trim() || '~/Treasury/Inbox',
              })}
            />
          </div>
        ) : null}
      </InstrumentCard>
      {/* Opção avançada: só aparece no nível Avançado (o mensal segue como padrão). */}
      {advanced && settings ? (
        <InstrumentCard index="12.23" title={t('ajustes-pref-carteira')}>
          <Switch
            checked={settings.twr_daily}
            onChange={(twr_daily) =>
              save.mutate(
                { twr_daily },
                {
                  onSuccess: () => {
                    void invalidatePerformance(client);
                    toast({
                      tone: 'success',
                      title: twr_daily
                        ? t('ajustes-pref-twr-diario-ligado')
                        : t('ajustes-pref-twr-diario-desligado'),
                    });
                  },
                },
              )
            }
            label={t('ajustes-pref-twr-diario')}
            description={t('ajustes-pref-twr-diario-dica')}
          />
        </InstrumentCard>
      ) : null}
      <InstrumentCard
        index="12.D"
        title={t('ajustes-pref-dev')}
        className="@min-[1040px]:col-span-2"
      >
        {settings ? (
          <div className="flex flex-col gap-3">
            <Switch
              checked={settings.developer_tools}
              onChange={(developer_tools) =>
                save.mutate(
                  { developer_tools },
                  {
                    onSuccess: () =>
                      toast({
                        tone: 'success',
                        title: developer_tools
                          ? t('ajustes-pref-dev-ligado')
                          : t('ajustes-pref-dev-desligado'),
                        description: developer_tools
                          ? t('ajustes-pref-dev-ligado-dica')
                          : undefined,
                      }),
                  },
                )
              }
              label={t('ajustes-pref-dev-opcoes')}
              description={t('ajustes-pref-dev-opcoes-dica')}
            />
            <p className="text-[12px] text-dim">{t('ajustes-pref-dev-privacidade')}</p>
          </div>
        ) : null}
      </InstrumentCard>
    </div>
  );
}

// ── Fontes de dados ─────────────────────────────────────────────

/** Data ISO (`2026-10-14`) no formato do idioma; outro texto passa como veio. */
function shortDate(iso: string): string {
  return /^\d{4}-\d{2}-\d{2}$/.test(iso) ? formatDate(iso) : iso;
}

function SyncReport({ report }: { report: DataSyncReport }) {
  const rows = [
    ...report.indices.series.map((s) => ({
      id: s.series_id,
      kind: t('ajustes-sync-indice'),
      written: s.written,
      last: s.last_date,
      error: s.error,
    })),
    ...report.indices.fx.map((s) => ({
      id: s.series_id,
      kind: t('ajustes-sync-cambio'),
      written: s.written,
      last: s.last_date,
      error: s.error,
    })),
    ...report.prices.assets.map((a) => ({
      id: a.ticker ?? a.asset_id,
      kind: t('ajustes-sync-cotacao'),
      written: a.prices,
      last: a.last_date,
      error: a.error,
    })),
  ];
  const unpriced = report.prices.without_source;
  const failures = rows.filter((r) => r.error);
  return (
    <section aria-label={t('ajustes-sync-resultado')} className="flex flex-col gap-2" role="status">
      <p className={cx('text-[13px]', report.ok ? 'text-positive' : 'text-warning')}>
        {report.ok
          ? t('ajustes-sync-ok')
          : t('ajustes-sync-parcial', {
              ok: rows.length - failures.length,
              total: rows.length,
              falhas: failures.length,
            })}
        {report.indices.ipca_estimated_months > 0
          ? ` ${t('ajustes-sync-ipca', { n: report.indices.ipca_estimated_months })}`
          : ''}
      </p>
      <ul className="max-h-60 overflow-auto rounded-card border border-line text-[12.5px]">
        {rows.map((r) => (
          <li
            key={`${r.kind}${r.id}`}
            className="grid grid-cols-[80px_minmax(0,1fr)_80px_96px] items-center gap-3 border-b border-line/60 px-3 py-1.5 last:border-0"
          >
            <span className="label-caps">{r.kind}</span>
            <span
              className={cx('truncate', r.error && 'text-warning')}
              title={r.error ?? undefined}
            >
              <span className="num text-fg">{r.id}</span>
              {r.error ? <span className="ml-2">{errorText(r.error)}</span> : null}
            </span>
            <span className="num text-right text-dim">
              {t('ajustes-sync-novos', { n: r.written })}
            </span>
            <span className="num text-right text-dim">{r.last ? shortDate(r.last) : '—'}</span>
          </li>
        ))}
      </ul>
      {unpriced.length > 0 ? (
        <div className="flex flex-col gap-1 text-[12.5px]">
          <p className="label-caps">{t('ajustes-sync-sem-fonte', { n: unpriced.length })}</p>
          <ul className="flex flex-col gap-1 text-dim">
            {unpriced.map((a) => (
              <li key={a.asset_id}>
                <span className="num text-fg">{a.ticker ?? a.name}</span>
                {' — '}
                {a.reason}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}

/**
 * 11.13 · brapi.dev (opcional): token, plano e consumo da cota; reserva da cotação da B3,
 * tickers renomeados e, nos planos pagos, a segunda fonte de indicadores e proventos.
 */
function BrapiCard() {
  const { data } = useDataSources();
  const setToken = useSetBrapiToken();
  const [token, setTokenText] = useState('');
  const valid = /^[A-Za-z0-9._-]{10,200}$/.test(token.trim());
  return (
    <InstrumentCard index="12.13" title={t('ajustes-brapi-titulo')}>
      <div className="flex flex-col gap-3">
        <p className="max-w-[64ch] text-[13px] leading-relaxed text-dim">
          {tRich('ajustes-brapi-texto', {
            link: (
              <a
                className="text-accent underline"
                href="https://brapi.dev/dashboard"
                target="_blank"
                rel="noreferrer"
              >
                {'brapi.dev'}
              </a>
            ),
          })}
        </p>
        <KeyValue
          items={[
            {
              label: t('ajustes-brapi-token'),
              value: data?.brapi_token_set ? (
                <span className="text-positive">{t('ajustes-brapi-cadastrado')}</span>
              ) : (
                t('ajustes-brapi-nao-cadastrado')
              ),
            },
          ]}
        />
        <div className="flex flex-wrap items-end gap-2">
          <Field
            label={data?.brapi_token_set ? t('ajustes-brapi-trocar') : t('ajustes-brapi-campo')}
            className="w-80"
          >
            <PasswordInput
              value={token}
              onChange={(e) => setTokenText(e.target.value.trim())}
              autoComplete="off"
            />
          </Field>
          <Button
            variant="secondary"
            disabled={!valid}
            loading={setToken.isPending}
            onClick={() =>
              setToken.mutate(
                { key: token },
                {
                  onSuccess: () => (
                    toast({ tone: 'success', title: t('ajustes-brapi-salvo') }),
                    setTokenText('')
                  ),
                },
              )
            }
          >
            {t('ajustes-brapi-salvar')}
          </Button>
          {data?.brapi_token_set ? (
            <Button
              variant="ghost"
              onClick={() =>
                setToken.mutate(
                  { key: '' },
                  {
                    onSuccess: () => toast({ tone: 'success', title: t('ajustes-brapi-removido') }),
                  },
                )
              }
            >
              {t('ajustes-remover')}
            </Button>
          ) : null}
        </div>
        <InlineError error={setToken.error} />
        {data?.brapi_token_set ? <BrapiUsage view={data.brapi ?? null} /> : null}
      </div>
    </InstrumentCard>
  );
}

/** Plano, consumo do mês e o limite que o app se impõe (docs/brazil/brapi.md). */
function BrapiUsage({ view }: { view: BrapiUsageView | null }) {
  const test = useBrapiTest();
  const setLimit = useSetBrapiLimit();
  const v = test.data ?? view;
  const planLabel = (p: string | null | undefined) =>
    p === 'free'
      ? t('ajustes-brapi-plano-gratuito')
      : p === 'startup'
        ? 'Startup'
        : p === 'pro'
          ? 'Pro'
          : t('ajustes-brapi-plano-desconhecido');
  const limit = v ? Math.floor((v.quota * v.limit_pct) / 100) : 0;
  const share = v && limit > 0 ? Math.min(1, v.used / limit) : 0;
  return (
    <section
      className="flex flex-col gap-2 border-t border-line pt-3"
      aria-label={t('ajustes-brapi-uso')}
    >
      <KeyValue
        items={[
          { label: t('ajustes-brapi-plano'), value: planLabel(v?.plan) },
          ...(v
            ? [
                {
                  label: t('ajustes-brapi-uso'),
                  value: t('ajustes-brapi-uso-valor', {
                    usados: v.used,
                    limite: limit,
                    cota: v.quota,
                  }),
                },
              ]
            : []),
        ]}
      />
      {v ? (
        <span className="h-1.5 w-full max-w-md rounded-full bg-line/50" aria-hidden>
          <span
            className={
              share >= 0.9
                ? 'block h-full rounded-full bg-warning'
                : 'block h-full rounded-full bg-accent'
            }
            style={{ width: `${Math.max(1, share * 100)}%` }}
          />
        </span>
      ) : null}
      <p className="max-w-[64ch] text-[12px] leading-snug text-dim">
        {v?.paid ? t('ajustes-brapi-pago-texto') : t('ajustes-brapi-gratuito-texto')}
      </p>
      <div className="flex flex-wrap items-end gap-2">
        <Field label={t('ajustes-brapi-limite')} className="w-48">
          <Select
            value={String(v?.limit_pct ?? 80)}
            onValueChange={(p) => setLimit.mutate({ pct: Number(p) })}
            aria-label={t('ajustes-brapi-limite')}
            options={[50, 80, 100].map((p) => ({
              value: String(p),
              label: t('ajustes-brapi-limite-opcao', { pct: p }),
            }))}
          />
        </Field>
        <Button
          variant="secondary"
          loading={test.isPending}
          onClick={() =>
            test.mutate(
              {},
              {
                onSuccess: (r) =>
                  toast({
                    tone: 'success',
                    title: t('ajustes-brapi-testado', { plano: planLabel(r.plan) }),
                  }),
              },
            )
          }
        >
          {t('ajustes-brapi-testar')}
        </Button>
      </div>
      <InlineError error={test.error ?? setLimit.error} />
    </section>
  );
}

/**
 * 11.11 · Fontes de dados: chave da FRED, atualização de índices, câmbio e cotações, e a
 * coleta e os ajustes da análise de ativos (User-Agent da SEC, premissas do Bazin).
 */
export function DataSourcesSection() {
  const { data, isLoading, error } = useDataSources();
  const setKey = useSetFredKey();
  const sync = useMarketSync();
  const [key, setKeyText] = useState('');
  const [report, setReport] = useState<DataSyncReport | null>(null);
  const valid = /^[A-Za-z0-9]{32}$/.test(key.trim());
  return (
    <div className="grid gap-5 @min-[1040px]:grid-cols-2">
      <div className="@min-[1040px]:col-span-2">
        <ExternalSourcesCard />
      </div>
      <InstrumentCard index="12.11" title={t('ajustes-fontes-indices')}>
        <div className="flex flex-col gap-4">
          <p className="max-w-[64ch] text-[13px] leading-relaxed text-dim">
            {t('ajustes-fontes-indices-texto')}
          </p>
          <QueryState loading={isLoading} error={error}>
            <KeyValue
              items={[
                {
                  label: t('ajustes-fontes-ultima'),
                  value: data?.last_sync_at
                    ? formatDateTime(data.last_sync_at)
                    : t('ajustes-nunca'),
                },
              ]}
            />
          </QueryState>
          <Button
            variant="primary"
            className="self-start"
            loading={sync.isPending}
            loadingText={t('ajustes-fontes-atualizando')}
            icon={<Icon name="refresh" size={13} />}
            onClick={() => sync.mutate({}, { onSuccess: setReport })}
          >
            {t('ajustes-fontes-atualizar')}
          </Button>
          <InlineError error={sync.error} />
          {report ? <SyncReport report={report} /> : null}
        </div>
      </InstrumentCard>
      <InstrumentCard index="12.12" title={t('ajustes-fred-titulo')}>
        <div className="flex flex-col gap-3">
          <p className="max-w-[64ch] text-[13px] leading-relaxed text-dim">
            {tRich('ajustes-fred-texto', {
              link: (
                <a
                  className="text-accent underline"
                  href="https://fred.stlouisfed.org/docs/api/api_key.html"
                  target="_blank"
                  rel="noreferrer"
                >
                  {'fred.stlouisfed.org'}
                </a>
              ),
            })}
          </p>
          <KeyValue
            items={[
              {
                label: t('ajustes-fred-chave'),
                value: data?.fred_key_set ? (
                  <span className="text-positive">{t('ajustes-fred-cadastrada')}</span>
                ) : (
                  t('ajustes-fred-nao-cadastrada')
                ),
              },
            ]}
          />
          <div className="flex flex-wrap items-end gap-2">
            <Field
              label={data?.fred_key_set ? t('ajustes-fred-trocar') : t('ajustes-fred-campo')}
              className="w-80"
            >
              <PasswordInput
                value={key}
                onChange={(e) => setKeyText(e.target.value.trim())}
                autoComplete="off"
                placeholder={t('ajustes-fred-placeholder')}
              />
            </Field>
            <Button
              variant="secondary"
              disabled={!valid}
              loading={setKey.isPending}
              onClick={() =>
                setKey.mutate(
                  { key },
                  {
                    onSuccess: () => (
                      toast({ tone: 'success', title: t('ajustes-fred-salva') }),
                      setKeyText('')
                    ),
                  },
                )
              }
            >
              {t('ajustes-fred-salvar')}
            </Button>
            {data?.fred_key_set ? (
              <Button
                variant="ghost"
                onClick={() =>
                  setKey.mutate(
                    { key: '' },
                    {
                      onSuccess: () =>
                        toast({ tone: 'success', title: t('ajustes-fred-removida') }),
                    },
                  )
                }
              >
                {t('ajustes-remover')}
              </Button>
            ) : null}
          </div>
          {key && !valid ? (
            <p className="text-[12px] text-warning">
              {t('ajustes-fred-tamanho', { n: key.length })}
            </p>
          ) : null}
          <InlineError error={setKey.error} />
        </div>
      </InstrumentCard>
      <BrapiCard />
      <SourceKeyCards />
      <ReferenceCountryCard />
      <ProvidersCard />
      <div className="flex flex-col gap-2 @min-[1040px]:col-span-2">
        <h3 className="label-caps text-fg">{t('ajustes-fontes-analise')}</h3>
        <ResearchDataPanel />
      </div>
    </div>
  );
}

// ── Dados (exportação) ──────────────────────────────────────────

/**
 * 11.13 · Dados: exporta tudo (JSON) ou os lançamentos (CSV). O caminho é
 * digitado (o diálogo nativo de salvar exige o plugin de diálogo do Tauri, ainda
 * não registrado no app).
 */
export function DataSection() {
  const info = useAppInfo();
  const json = useExportJson();
  const csv = useExportCsv();
  const home = info.data?.data_root.replace(/\/\.treasury$/, '') ?? '~';
  const stamp = new Date().toISOString().slice(0, 10);
  const [jsonPath, setJsonPath] = useState('');
  const [csvPath, setCsvPath] = useState('');
  const [result, setResult] = useState<ExportResult | null>(null);
  const done = (r: ExportResult) => {
    setResult(r);
    toast({ tone: 'warning', title: t('ajustes-exportar-feito'), description: r.path });
  };
  return (
    <InstrumentCard index="12.13" title={t('ajustes-secao-dados')}>
      <div className="flex flex-col gap-4">
        <p
          role="note"
          className="flex items-start gap-2 rounded-card border border-warning/60 bg-warning/5 px-3 py-2 text-[13px] text-warning"
        >
          <Icon name="alert" size={14} className="mt-0.5 shrink-0" />
          <span>{tRich('ajustes-exportar-aviso', {})}</span>
        </p>
        <div className="grid gap-4 @min-[790px]:grid-cols-2">
          <div className="flex flex-col gap-2">
            <Field label={t('ajustes-exportar-json')} hint={t('ajustes-exportar-json-dica')}>
              <Input
                value={jsonPath}
                placeholder={`${home}/Documents/treasury-${stamp}.json`}
                onChange={(e) => setJsonPath(e.target.value)}
              />
            </Field>
            <Button
              className="self-start"
              disabled={!jsonPath.trim()}
              loading={json.isPending}
              onClick={() => json.mutate({ path: jsonPath.trim() }, { onSuccess: done })}
            >
              {t('ajustes-exportar-json-botao')}
            </Button>
          </div>
          <div className="flex flex-col gap-2">
            <Field label={t('ajustes-exportar-csv')} hint={t('ajustes-exportar-csv-dica')}>
              <Input
                value={csvPath}
                placeholder={`${home}/Documents/${t('ajustes-exportar-csv-nome', { data: stamp })}`}
                onChange={(e) => setCsvPath(e.target.value)}
              />
            </Field>
            <Button
              className="self-start"
              disabled={!csvPath.trim()}
              loading={csv.isPending}
              onClick={() => csv.mutate({ path: csvPath.trim() }, { onSuccess: done })}
            >
              {t('ajustes-exportar-csv-botao')}
            </Button>
          </div>
        </div>
        <InlineError error={json.error ?? csv.error} />
        {result ? (
          <KeyValue
            items={[
              { label: t('ajustes-arquivo'), value: result.path },
              { label: t('ajustes-tamanho'), value: formatBytes(result.bytes) },
              {
                label: t('ajustes-linhas'),
                value: Object.values(result.counts).reduce((a, b) => a + b, 0),
              },
            ]}
          />
        ) : null}
        {info.data ? (
          <KeyValue
            className="border-t border-line pt-3"
            items={[
              { label: t('ajustes-cofre'), value: info.data.db_path },
              { label: t('ajustes-secao-backups'), value: info.data.backups_dir },
              {
                label: t('ajustes-exportar-versao'),
                value: t('ajustes-exportar-versao-valor', {
                  versao: info.data.version,
                  esquema: info.data.schema_version,
                }),
              },
            ]}
          />
        ) : null}
      </div>
    </InstrumentCard>
  );
}

// ── Integrações MCP ─────────────────────────────────────────────

/** `secret`: cópia oculta, só neste Mac e apagada em 60 s (chave de API). */
export function CopyButton({
  text,
  label,
  secret = false,
}: {
  text: string;
  label: string;
  secret?: boolean;
}) {
  const [copied, setCopied] = useState(false);
  return (
    <Button
      size="sm"
      variant="secondary"
      onClick={() =>
        void (secret ? copySecret : copyText)(text)
          .then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
          })
          .catch(() =>
            toast({
              tone: 'error',
              title: t('ajustes-nao-copiou'),
              description: t('ajustes-copiar-a-mao'),
            }),
          )
      }
    >
      {copied ? t('ajustes-copiado') : label}
    </Button>
  );
}

/** 11.14 · Integrações MCP: configuração para Claude Desktop, Claude Code e OpenCode, e auditoria. */
export function McpSection() {
  const config = useMcpConfig();
  const audit = useMcpAudit({ limit: 20 });
  const json = config.data ? JSON.stringify(config.data, null, 2) : '';
  const server = (
    config.data as
      { mcpServers?: { treasury?: { command?: string; env?: Record<string, string> } } } | undefined
  )?.mcpServers?.treasury;
  const cli = server?.command
    ? `claude mcp add treasury${Object.entries(server.env ?? {})
        .map(([k, v]) => ` -e ${k}=${v}`)
        .join('')} -- ${server.command.includes(' ') ? `"${server.command}"` : server.command} mcp`
    : '';
  // OpenCode: formato `mcp` com `type: local` (aceito pela 1.x e pela 2.x).
  const opencode = server?.command
    ? JSON.stringify(
        {
          $schema: 'https://opencode.ai/config.json',
          mcp: {
            treasury: {
              type: 'local',
              command: [server.command, 'mcp'],
              ...(server.env && Object.keys(server.env).length > 0
                ? { environment: server.env }
                : {}),
              enabled: true,
            },
          },
        },
        null,
        2,
      )
    : '';
  const columns: TechColumn<AuditEntry>[] = [
    {
      key: 'when',
      header: t('ajustes-quando'),
      numeric: true,
      align: 'left',
      width: '132px',
      cell: (a) => formatDateTime(a.created_at),
    },
    {
      key: 'client',
      header: t('ajustes-mcp-col-cliente'),
      width: '24%',
      cell: (a) => <span className="block truncate text-dim">{a.client}</span>,
    },
    {
      key: 'tool',
      header: t('ajustes-mcp-col-ferramenta'),
      cell: (a) => (
        <span className="flex min-w-0 flex-col">
          <span className="num truncate">{a.tool}</span>
          {a.error ? <span className="truncate text-[11.5px] text-negative">{a.error}</span> : null}
        </span>
      ),
    },
    {
      key: 'ok',
      header: t('ajustes-mcp-col-resultado'),
      width: '84px',
      cell: (a) =>
        a.ok ? (
          <span className="text-positive">{t('ajustes-mcp-ok')}</span>
        ) : (
          <span className="text-negative">{t('ajustes-mcp-erro')}</span>
        ),
    },
    {
      key: 'ms',
      header: t('ajustes-mcp-col-tempo'),
      numeric: true,
      width: '72px',
      cell: (a) => `${a.duration_ms} ms`,
    },
  ];
  return (
    <div className="flex flex-col gap-5">
      <McpAccess />
      <InstrumentCard index="12.14" title={t('ajustes-mcp-usar')}>
        <QueryState loading={config.isLoading} error={config.error}>
          <div className="grid gap-5 @min-[1040px]:grid-cols-2">
            <div className="flex min-w-0 flex-col gap-2">
              <h3 className="label-caps text-fg">{'Claude Desktop'}</h3>
              <p className="text-[13px] text-dim">
                {tRich('ajustes-mcp-desktop-texto', {
                  arquivo: <span className="num">{'claude_desktop_config.json'}</span>,
                })}
              </p>
              <pre
                className="num max-h-56 overflow-auto rounded-card border border-line bg-bg p-3 text-[12px] text-fg"
                aria-label={t('ajustes-mcp-config')}
              >
                {json}
              </pre>
              <CopyButton text={json} label={t('ajustes-mcp-copiar-config')} />
            </div>
            <div className="flex min-w-0 flex-col gap-2">
              <h3 className="label-caps text-fg">{'Claude Code'}</h3>
              <p className="text-[13px] text-dim">{t('ajustes-mcp-terminal')}</p>
              <pre
                className="num overflow-auto rounded-card border border-line bg-bg p-3 text-[12px] whitespace-pre-wrap text-fg"
                aria-label={t('ajustes-mcp-comando')}
              >
                {cli}
              </pre>
              <CopyButton text={cli} label={t('ajustes-mcp-copiar-comando')} />
              <p className="text-[12.5px] leading-relaxed text-dim">
                {t('ajustes-mcp-code-texto')}
              </p>
            </div>
            <div className="flex min-w-0 flex-col gap-2 @min-[1040px]:col-span-2">
              <h3 className="label-caps text-fg">{'OpenCode'}</h3>
              <p className="text-[13px] text-dim">
                {tRich('ajustes-mcp-opencode-texto', {
                  arquivo: <span className="num">{'opencode.json'}</span>,
                })}
              </p>
              <pre
                className="num max-h-56 overflow-auto rounded-card border border-line bg-bg p-3 text-[12px] text-fg"
                aria-label={t('ajustes-mcp-opencode-config')}
              >
                {opencode}
              </pre>
              <CopyButton text={opencode} label={t('ajustes-mcp-copiar-opencode')} />
            </div>
          </div>
        </QueryState>
      </InstrumentCard>
      <InstrumentCard index="12.15" title={t('ajustes-mcp-ultimas')} padded={false}>
        <QueryState loading={audit.isLoading} error={audit.error}>
          <TechTable
            label={t('ajustes-mcp-auditoria')}
            prefsId="Auditoria MCP"
            dense
            columns={columns}
            rows={audit.data ?? []}
            getRowId={(a) => a.id}
            className="[&_table]:table-fixed"
            empty={t('ajustes-mcp-vazio')}
          />
        </QueryState>
      </InstrumentCard>
    </div>
  );
}
