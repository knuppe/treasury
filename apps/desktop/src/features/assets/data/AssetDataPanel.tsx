/**
 * Ativo › Cadastro › Dados do cadastro (docs/agent-enrichment.md §10.1): cada campo com o
 * valor, o selo da origem (você, já preenchido, fonte · data · confiança), a sugestão das
 * fontes do cofre quando houver, "de onde veio" (as leituras) e a trava ("não sugerir
 * mudanças neste campo"). "Completar dados" grava só as sugestões de confiança alta dos
 * campos vazios; as demais vão uma a uma pelo "Usar".
 */
import { useState } from 'react';

import type { FieldSources } from '@/api/generated/FieldSources';
import type { FieldStatus } from '@/api/generated/FieldStatus';
import {
  useDataGaps,
  useEnrichApply,
  useEnrichLookup,
  useFieldLock,
  useFieldSources,
} from '@/api/hooks/assetData';
import { Button, Icon, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { ErrorBlock, LoadingBlock } from '../../portfolio/components/States';
import {
  ConfidenceChip,
  fieldLabel,
  itemFor,
  OriginChip,
  SourceChip,
  sourceLabel,
  stateLabel,
  valueLabel,
} from './parts';

export function AssetDataPanel({ assetId }: { assetId: string }) {
  const q = useFieldSources(assetId);
  const apply = useEnrichApply();
  const lookup = useEnrichLookup();
  if (q.isPending) return <LoadingBlock label={t('dados-cadastro-titulo')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const fields = q.data.fields;
  const auto = fields
    .map((f) => f.status)
    .filter((f) => f.state === 'fill' && f.candidate?.band === 'high');

  const run = (items: Parameters<typeof apply.mutate>[0]['items']) =>
    apply.mutate(
      { items },
      {
        onSuccess: (r) =>
          toast({
            tone: 'success',
            title: t('dados-aplicado', { n: r.fields }),
            description: t('dados-aplicado-desfazer'),
          }),
        onError: (e: Error) =>
          toast({ tone: 'error', title: t('dados-nao-aplicou'), description: e.message }),
      },
    );

  return (
    <section className="flex flex-col gap-3" aria-label={t('dados-cadastro-titulo')}>
      <div className="flex flex-wrap items-center gap-2">
        <span className="label-caps">{t('dados-cadastro-titulo')}</span>
        <span className="grow" />
        <Button
          size="sm"
          variant="ghost"
          loading={lookup.isPending}
          title={t('dados-consultar-dica')}
          onClick={() =>
            lookup.mutate(
              { asset_id: assetId },
              {
                onSuccess: (r) =>
                  toast({
                    tone: 'success',
                    title:
                      r.observations > 0
                        ? t('dados-consultado', { n: r.observations })
                        : t('dados-consultado-nada'),
                  }),
                onError: (e: Error) =>
                  toast({ tone: 'error', title: t('dados-nao-consultou'), description: e.message }),
              },
            )
          }
        >
          {t('dados-consultar')}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={auto.length === 0}
          loading={apply.isPending}
          title={auto.length === 0 ? t('dados-completar-nada') : t('dados-completar-dica')}
          onClick={() =>
            run(auto.flatMap((f) => (f.candidate ? [itemFor(assetId, f, f.candidate)] : [])))
          }
        >
          {t('dados-completar')}
        </Button>
      </div>
      <dl className="flex flex-col divide-y divide-line/60 text-[13px]">
        {fields.map((f) => (
          <FieldRow
            key={f.status.field}
            assetId={assetId}
            f={f}
            pending={apply.isPending}
            onUse={(c) => run([itemFor(assetId, f.status, c)])}
          />
        ))}
      </dl>
      <p className="text-[12px] leading-snug text-dim">{t('dados-rodape')}</p>
    </section>
  );
}

function FieldRow({
  assetId,
  f,
  pending,
  onUse,
}: {
  assetId: string;
  f: FieldSources;
  pending: boolean;
  onUse: (c: NonNullable<FieldStatus['candidate']>) => void;
}) {
  const [open, setOpen] = useState(false);
  const lock = useFieldLock();
  const s = f.status;
  const suggestions = s.state === 'conflict' ? s.options : s.candidate ? [s.candidate] : [];
  return (
    <div className="flex flex-col gap-1 py-2">
      <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <dt className="w-40 shrink-0 text-dim">{fieldLabel(s.field)}</dt>
        <dd className="flex min-w-0 grow flex-wrap items-baseline gap-2">
          <span className={cx('break-words', s.current ? 'text-fg' : 'text-dim italic')}>
            {s.current != null ? valueLabel(s.field, s.current) : t('dados-vazio')}
          </span>
          <OriginChip f={s} />
          {s.locked ? (
            <span className="text-[11px] text-dim">
              <Icon name="lock" size={11} /> {t('dados-travado')}
            </span>
          ) : null}
          {s.state !== 'ok' && s.state !== 'missing' && s.state !== 'fill' ? (
            <span className="text-[11px] text-warning">{stateLabel(s.state)}</span>
          ) : null}
        </dd>
        <span className="flex items-center gap-1">
          <Button size="sm" variant="ghost" onClick={() => setOpen((o) => !o)} aria-expanded={open}>
            {t('dados-de-onde-veio')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            title={t('dados-travar-dica')}
            loading={lock.isPending}
            onClick={() => lock.mutate({ asset_id: assetId, field: s.field, locked: !s.locked })}
          >
            {s.locked ? t('dados-destravar') : t('dados-travar')}
          </Button>
        </span>
      </div>
      {suggestions.map((c) => (
        <div key={c.value} className="flex flex-wrap items-baseline gap-2 pl-0 sm:pl-43">
          <span className="text-fg">→ {valueLabel(s.field, c.value)}</span>
          <SourceChip c={c} />
          <ConfidenceChip c={c} />
          {s.state === 'diverge' ? (
            <span className="text-[11px] text-dim">
              {t('dados-diverge-dica', { valor: c.value })}
            </span>
          ) : null}
          {s.effect ? <span className="text-[11px] text-dim">{s.effect}</span> : null}
          <Button size="sm" variant="ghost" loading={pending} onClick={() => onUse(c)}>
            {s.state === 'diverge' ? t('dados-trocar-seu') : t('dados-usar')}
          </Button>
        </div>
      ))}
      {open ? (
        <div className="flex flex-col gap-1 rounded-sm border border-line/60 p-2 text-[12px] sm:ml-43">
          <span className="label-caps">{t('dados-leituras')}</span>
          {f.observations.length === 0 ? (
            <span className="text-dim">{t('dados-sem-leituras')}</span>
          ) : (
            f.observations.map((o, i) => (
              <span key={`${o.source}-${i}`} className="flex flex-wrap gap-2">
                <span className="text-fg">{o.value}</span>
                <span className="text-dim">
                  {sourceLabel(o.source)} · {t(`dados-nivel-${o.tier}` as 'dados-nivel-1')}
                  {o.as_of ? ` · ${o.as_of}` : ''}
                </span>
                {o.source_ref ? (
                  <a href={o.source_ref} target="_blank" rel="noreferrer" className="text-accent">
                    ↗
                  </a>
                ) : null}
              </span>
            ))
          )}
          {f.history.length > 0 ? (
            <>
              <span className="label-caps mt-1">{t('dados-historico')}</span>
              {f.history.map((h, i) => (
                <span key={`${h.source}-${i}`} className="text-dim">
                  {h.value} · {sourceLabel(h.source)} ·{' '}
                  {t('dados-observado-em', { data: formatDateTime(h.observed_at) })}
                </span>
              ))}
            </>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}

/** Atalho no cabeçalho do ativo quando há dados para completar (abre a aba Cadastro). */
export function AssetDataChip({ assetId, onOpen }: { assetId: string; onOpen: () => void }) {
  const q = useDataGaps({ asset_ids: [assetId] });
  const n = q.data?.assets[0]?.fields.length ?? 0;
  if (n === 0) return null;
  return (
    <button
      type="button"
      onClick={onOpen}
      className="w-fit text-[12px] text-accent underline-offset-2 hover:underline"
    >
      {t('dados-chip-ativo', { n })}
    </button>
  );
}
