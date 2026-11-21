/**
 * Análise fundamentalista de um ativo (cadastrado por `asset_id` ou do universo por
 * `ticker`): indicadores, preço justo, histórico e documentos. Usada na página do ativo e
 * na aba "Indicadores" (busca por ticker).
 */
import { useState, type FormEvent } from 'react';
import { Link } from 'react-router';

import type { SubjectArgs } from '@/api/generated/SubjectArgs';
import { useIsObserved, useObserve, useUnobserve } from '@/api/hooks/research';
import {
  Button,
  Icon,
  Input,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  toast,
} from '@/design/components';
import { t } from '@/i18n';

import { errorMessage, LoadingBlock, Notice } from '../portfolio/components/States';
import { NoUniverse, subjectHref, useUniverseList } from './common';
import { DocumentsPanel } from './DocumentsPanel';
import { FairPricePanel } from './FairPricePanel';
import { HistoryPanel } from './HistoryPanel';
import { IndicatorsPanel } from './IndicatorsPanel';

export function ResearchSection({ subject, title }: { subject: SubjectArgs; title?: string }) {
  const [tab, setTab] = useState('indicadores');
  return (
    <section
      aria-label={title ?? t('pesquisa-analise-fundamentalista')}
      className="flex flex-col gap-3"
    >
      {title ? <h3 className="label-caps text-fg">{title}</h3> : null}
      <Tabs value={tab} onValueChange={setTab}>
        <TabsList aria-label={t('pesquisa-analise-do-ativo')}>
          <TabsTrigger value="indicadores">{t('pesquisa-aba-indicadores')}</TabsTrigger>
          <TabsTrigger value="preco-justo">{t('pesquisa-aba-preco-justo')}</TabsTrigger>
          <TabsTrigger value="historico">{t('pesquisa-aba-historico')}</TabsTrigger>
          <TabsTrigger value="documentos">{t('pesquisa-aba-documentos')}</TabsTrigger>
        </TabsList>
        <TabsContent value="indicadores">
          <IndicatorsPanel subject={subject} />
        </TabsContent>
        <TabsContent value="preco-justo">
          <FairPricePanel subject={subject} />
        </TabsContent>
        <TabsContent value="historico">
          <HistoryPanel subject={subject} />
        </TabsContent>
        <TabsContent value="documentos">
          <DocumentsPanel subject={subject} />
        </TabsContent>
      </Tabs>
    </section>
  );
}

/**
 * Papel fora do cadastro: "Você não tem este ativo" com Observar (cria o ativo e põe na
 * watchlist, sem operação: o score com histórico, os alertas e o Estudo com IA passam a
 * valer). Ativo criado assim mostra "Deixar de observar", que o tira do cadastro se ele
 * nunca tiver tido operação.
 */
export function ObserveBar({
  ticker,
  assetId,
  onPage,
}: {
  ticker: string;
  assetId: string | null;
  /** Já na página do ativo: sem o link para ela. */
  onPage?: boolean;
}) {
  const observe = useObserve();
  const unobserve = useUnobserve();
  const observed = useIsObserved(assetId);
  const fail = (e: unknown) =>
    toast({ tone: 'error', title: t('pesquisa-observar-erro'), description: errorMessage(e) });
  if (assetId && !observed.data) return null;
  return (
    <div
      role="note"
      className="flex flex-wrap items-center gap-3 rounded-card border border-line bg-surface px-3 py-2 text-[12.5px]"
    >
      <Icon name="eye" size={13} className="text-accent" />
      <span className="flex-1 text-fg">
        {assetId ? t('pesquisa-observado-pelo-mercado') : t('pesquisa-nao-tem-ativo')}
      </span>
      {assetId ? (
        <>
          {onPage ? null : (
            <Link
              className="text-accent underline-offset-2 hover:underline"
              to={subjectHref(assetId, ticker)}
            >
              {t('pesquisa-abrir-pagina-ativo')}
            </Link>
          )}
          <Button
            size="sm"
            variant="ghost"
            loading={unobserve.isPending}
            onClick={() =>
              unobserve.mutate(
                { asset_id: assetId },
                {
                  onSuccess: () => toast({ tone: 'info', title: t('pesquisa-deixou-observar') }),
                  onError: fail,
                },
              )
            }
          >
            {t('pesquisa-deixar-observar')}
          </Button>
        </>
      ) : (
        <Button
          size="sm"
          variant="primary"
          icon={<Icon name="eye" size={12} />}
          loading={observe.isPending}
          onClick={() =>
            observe.mutate(
              { ticker },
              {
                onSuccess: () =>
                  toast({
                    tone: 'success',
                    title: t('pesquisa-observando', { ticker }),
                    description: t('pesquisa-observando-texto'),
                  }),
                onError: fail,
              },
            )
          }
        >
          {t('pesquisa-observar')}
        </Button>
      )}
    </div>
  );
}

/** Aba "Indicadores": analisa qualquer papel do universo pelo ticker (`?ticker=`). */
export function TickerAnalysis({
  ticker,
  onTicker,
}: {
  ticker: string | null;
  onTicker: (ticker: string | null) => void;
}) {
  const universe = useUniverseList();
  const [text, setText] = useState(ticker ?? '');
  const items = universe.data?.items ?? [];
  const hit = ticker ? items.find((i) => i.ticker === ticker.toUpperCase()) : undefined;
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const next = text.trim().toUpperCase();
    onTicker(next || null);
  };
  if (universe.isPending) return <LoadingBlock />;
  if (universe.data && universe.data.total === 0 && !ticker) return <NoUniverse what="ticker" />;
  return (
    <div className="flex flex-col gap-4">
      <form
        onSubmit={submit}
        className="flex flex-wrap items-end gap-2"
        aria-label={t('pesquisa-analisar-papel')}
      >
        <div className="w-64">
          <Input
            aria-label={t('pesquisa-ticker-para-analisar')}
            placeholder={t('pesquisa-ticker-exemplo-analise')}
            value={text}
            list="research-universe"
            onChange={(e) => setText(e.target.value)}
            prefix={<Icon name="search" size={13} />}
          />
          <datalist id="research-universe">
            {items.map((i) => (
              <option key={i.ticker} value={i.ticker}>
                {i.name}
              </option>
            ))}
          </datalist>
        </div>
        <Button type="submit" variant="primary">
          {t('pesquisa-analisar')}
        </Button>
        <span className="ml-2 text-[12px] text-dim">
          {universe.data ? t('pesquisa-n-papeis-universo', { n: universe.data.total }) : ''}
        </span>
      </form>
      {ticker ? (
        <>
          <div className="flex flex-wrap items-baseline gap-3 border-b border-line pb-3">
            <h2 className="num text-[20px] font-semibold tracking-[0.04em] text-fg">
              {ticker.toUpperCase()}
            </h2>
            {hit ? <span className="text-[14px] text-dim">{hit.name}</span> : null}
            {hit?.asset_id ? (
              <Link
                className="ml-auto text-[12.5px] text-accent underline"
                to={subjectHref(hit.asset_id, hit.ticker)}
              >
                {t('pesquisa-abrir-pagina-ativo')}
              </Link>
            ) : null}
          </div>
          <ObserveBar ticker={ticker.toUpperCase()} assetId={hit?.asset_id ?? null} />
          <ResearchSection key={ticker} subject={{ ticker: ticker.toUpperCase() }} />
        </>
      ) : (
        <Notice title={t('pesquisa-escolha-papel')}>{t('pesquisa-escolha-papel-texto')}</Notice>
      )}
    </div>
  );
}
