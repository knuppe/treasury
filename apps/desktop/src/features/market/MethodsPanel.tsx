/**
 * Aba Métodos do Mercado (docs/design/market-explorer.md, seção 5): um cartão por
 * método (ideia, quantos cumprem hoje), a receita inteira (critérios, o que foi adaptado à
 * B3, higiene, ordem e a quem não se aplica), a lista de quem cumpre e, para um ticker, por que
 * ele passa ou fica de fora. Cumprir critérios não é recomendação (CVM Res. 20/2021).
 */
import { useState, type FormEvent } from 'react';
import { useNavigate } from 'react-router';

import type { MethodCheck } from '@/api/generated/MethodCheck';
import type { MethodRow } from '@/api/generated/MethodRow';
import type { ScreenMethod } from '@/api/generated/ScreenMethod';
import { useMethodCounts, useMethodExplain, useMethodRun, useMethods } from '@/api/hooks/research';
import {
  Button,
  Icon,
  InstrumentCard,
  Input,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { formatNumber } from '@/lib/format';

import { ErrorBlock, LoadingBlock } from '../portfolio/components/States';
import { subjectHref, TickerLink } from '../research/common';
import { NotAdvice } from './MarketPanels';

function Recipe({ m }: { m: ScreenMethod }) {
  return (
    <div className="flex flex-col gap-3 text-[13px]">
      <p className="text-fg">{m.idea}</p>
      <ul aria-label={t('mercado-metodo-criterios')} className="flex flex-col gap-1">
        {m.criteria.map((c) => (
          <li key={c.id} className="flex flex-col">
            <span className="text-fg">
              {c.label}
              {c.optional ? (
                <span className="ml-1 text-[11.5px] text-dim">{t('mercado-metodo-opcional')}</span>
              ) : null}
            </span>
            {c.adapted ? (
              <span className="text-[11.5px] text-warning">
                {t('mercado-metodo-adaptado', { original: c.adapted })}
              </span>
            ) : null}
          </li>
        ))}
      </ul>
      <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1 text-[12px] text-dim">
        <dt>{t('mercado-metodo-higiene')}</dt>
        <dd className="text-fg">{m.hygiene}</dd>
        <dt>{t('mercado-metodo-ordem')}</dt>
        <dd className="text-fg">{m.order}</dd>
        {m.not_applicable ? (
          <>
            <dt>{t('mercado-metodo-nao-se-aplica')}</dt>
            <dd className="text-fg">{m.not_applicable}</dd>
          </>
        ) : null}
        <dt>{t('mercado-metodo-fonte')}</dt>
        <dd className="text-fg">
          {t('mercado-metodo-fonte-versao', { fonte: m.reference, versao: m.version })}
        </dd>
      </dl>
    </div>
  );
}

function Checks({ checks }: { checks: MethodCheck[] }) {
  return (
    <span className="flex flex-wrap gap-1">
      {checks.map((c) => (
        <span
          key={c.id}
          title={`${c.label}: ${c.shown}`}
          className={
            c.passed
              ? 'rounded-control border border-line px-1 text-[10.5px] text-positive'
              : 'rounded-control border border-negative/50 px-1 text-[10.5px] text-negative'
          }
        >
          {c.passed ? '✓' : '✗'} {c.shown}
        </span>
      ))}
    </span>
  );
}

function rowColumns(): TechColumn<MethodRow>[] {
  return [
    {
      key: 'rank',
      header: '#',
      width: '48px',
      numeric: true,
      cell: (r) => r.rank,
      sortValue: (r) => r.rank,
    },
    {
      key: 'ticker',
      header: t('pesquisa-scr-ticker'),
      width: '96px',
      cell: (r) => <TickerLink assetId={r.asset_id} ticker={r.ticker} />,
      sortValue: (r) => r.ticker,
    },
    {
      key: 'name',
      header: t('pesquisa-scr-nome-col'),
      cell: (r) => (
        <span className="flex flex-col font-sans leading-tight">
          <span className="truncate">{r.name}</span>
          <span className="truncate text-[11.5px] text-dim">{r.sector ?? '—'}</span>
        </span>
      ),
      sortValue: (r) => r.name,
    },
    {
      key: 'score',
      header: t('mercado-col-score-papel'),
      width: '72px',
      numeric: true,
      cell: (r) => (r.score == null ? '—' : formatNumber(r.score, 0)),
      sortValue: (r) => r.score ?? -1,
    },
    {
      key: 'checks',
      header: t('mercado-metodo-criterios'),
      width: '40%',
      cell: (r) => <Checks checks={r.checks} />,
    },
  ];
}

/** Por que um ticker cumpre (ou não) o método, critério a critério. */
function ExplainBox({ methodKey }: { methodKey: string }) {
  const [text, setText] = useState('');
  const [ticker, setTicker] = useState<string | null>(null);
  const q = useMethodExplain(methodKey, ticker);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setTicker(text.trim().toUpperCase() || null);
  };
  return (
    <section aria-label={t('mercado-metodo-por-que')} className="flex flex-col gap-2">
      <form onSubmit={submit} className="flex flex-wrap items-end gap-2">
        <div className="w-48">
          <Input
            aria-label={t('mercado-metodo-por-que')}
            placeholder={t('mercado-metodo-por-que-exemplo')}
            value={text}
            onChange={(e) => setText(e.target.value)}
            prefix={<Icon name="search" size={13} />}
          />
        </div>
        <Button type="submit" size="sm">
          {t('mercado-metodo-explicar')}
        </Button>
      </form>
      {q.isError ? <ErrorBlock error={q.error} /> : null}
      {q.data ? (
        <div className="flex flex-col gap-1 text-[12.5px]">
          <p className={q.data.passed ? 'text-positive' : 'text-fg'}>
            {q.data.passed
              ? t('mercado-metodo-passa', { ticker: q.data.ticker, rank: q.data.rank ?? 0 })
              : q.data.excluded
                ? t('mercado-metodo-fora', { ticker: q.data.ticker, motivo: q.data.excluded })
                : t('mercado-metodo-nao-passa', { ticker: q.data.ticker })}
          </p>
          <ul className="flex flex-col gap-0.5">
            {q.data.checks.map((c) => (
              <li key={c.id} className={c.passed ? 'text-dim' : 'text-negative'}>
                {c.passed ? '✓' : '✗'} {c.label}: {c.shown}
                {c.missing ? ` (${t('mercado-metodo-sem-dado')})` : ''}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}

function MethodView({ m, onBack }: { m: ScreenMethod; onBack: () => void }) {
  const q = useMethodRun(m.key);
  const navigate = useNavigate();
  return (
    <div className="flex flex-col gap-4">
      <div>
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="chevronLeft" size={12} />}
          onClick={onBack}
        >
          {t('mercado-metodo-voltar')}
        </Button>
      </div>
      <InstrumentCard index="10.4" title={m.name}>
        <Recipe m={m} />
      </InstrumentCard>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : (
        <section
          aria-label={t('mercado-metodo-lista', { metodo: m.name })}
          className="flex flex-col gap-2"
        >
          <p className="text-[12.5px] text-dim">
            {t('mercado-metodo-resultado', { passam: q.data.passing, avaliados: q.data.evaluated })}
          </p>
          <TechTable
            label={t('mercado-metodo-lista', { metodo: m.name })}
            prefsId={`metodo-${m.key}`}
            rows={q.data.rows}
            columns={rowColumns()}
            getRowId={(r) => r.ticker}
            onActivate={(r) => void navigate(subjectHref(r.asset_id, r.ticker))}
            empty={t('mercado-metodo-ninguem')}
            dense
          />
        </section>
      )}
      <ExplainBox methodKey={m.key} />
      <NotAdvice />
    </div>
  );
}

export function MethodsPanel() {
  const methods = useMethods();
  const counts = useMethodCounts();
  const [open, setOpen] = useState<string | null>(null);
  if (methods.isPending) return <LoadingBlock />;
  if (methods.isError) return <ErrorBlock error={methods.error} />;
  const current = methods.data.find((m) => m.key === open);
  if (current) return <MethodView m={current} onBack={() => setOpen(null)} />;
  return (
    <div className="flex flex-col gap-4">
      <NotAdvice />
      <div className="grid gap-4 @min-[760px]:grid-cols-2">
        {methods.data.map((m, i) => (
          <InstrumentCard key={m.key} index={`10.${i + 5}`} title={m.name}>
            <div className="flex flex-col gap-3 text-[13px]">
              <p className="text-fg">{m.idea}</p>
              <p className="text-dim">
                {counts.data
                  ? t('mercado-metodo-cumprem', { n: counts.data[m.key] ?? 0 })
                  : t('mercado-metodo-contando')}
              </p>
              <p className="text-[12px] text-dim">{m.reference}</p>
              <div>
                <Button size="sm" variant="secondary" onClick={() => setOpen(m.key)}>
                  {t('mercado-metodo-ver-lista')}
                </Button>
              </div>
            </div>
          </InstrumentCard>
        ))}
      </div>
    </div>
  );
}
