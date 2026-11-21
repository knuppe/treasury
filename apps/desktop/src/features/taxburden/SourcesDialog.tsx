/**
 * "Ver fontes e metodologia": a versão e a data dos dados do país, como os números foram
 * escolhidos, os cenários, as regras de renda e cada fonte (quem publica, título, link e
 * datas). Também a lista curta de fontes de um grupo de consumo ("Ver base").
 */
import type { ReactElement, ReactNode } from 'react';

import type { DatasetInfo } from '@/api/generated/DatasetInfo';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import type { TaxSource } from '@/api/generated/TaxSource';
import { Button, Dialog, DialogClose, Icon, KeyValue } from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { formatRate, isDraft, SCENARIOS } from './model';

/** Datas das fontes: `2025-05-07` → `07/05/2025`, `2026-10` → `10/2026`; texto livre fica. */
export function sourceDate(text: string): string {
  if (/^\d{4}-\d{2}-\d{2}$/.test(text)) {
    try {
      return formatDate(text);
    } catch {
      return text;
    }
  }
  const m = /^(\d{4})-(\d{2})$/.exec(text);
  if (m) return `${m[2]}/${m[1]}`;
  return text.replace(/\b(\d{4})-(\d{2})-(\d{2})\b/g, '$3/$2/$1');
}

export function SourceList({ sources }: { sources: readonly TaxSource[] }) {
  return (
    <ol className="flex flex-col gap-2.5" aria-label={t('impostos-fontes')}>
      {sources.map((s) => (
        <li key={s.id} className="flex flex-col gap-0.5 border-l-2 border-line pl-3">
          <span className="text-[12px] text-dim">{s.publisher}</span>
          <a
            href={s.url}
            target="_blank"
            rel="noreferrer"
            className="text-[13px] leading-snug text-accent underline-offset-2 hover:underline"
          >
            {s.title}
          </a>
          <span className="text-[11.5px] text-dim">
            {s.published
              ? t('impostos-fonte-publicada', {
                  publicacao: sourceDate(s.published),
                  consulta: sourceDate(s.accessed),
                })
              : t('impostos-fonte-consultada', { consulta: sourceDate(s.accessed) })}
          </span>
        </li>
      ))}
    </ol>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section aria-label={title} className="flex flex-col gap-2">
      <h3 className="label-caps text-fg">{title}</h3>
      {children}
    </section>
  );
}

export function SourcesDialog({
  dataset: ds,
  info,
  trigger,
}: {
  dataset: TaxDataset;
  info: DatasetInfo;
  trigger: ReactElement;
}) {
  return (
    <Dialog
      trigger={trigger}
      title={t('impostos-fontes-metodologia')}
      description={t('impostos-fontes-descricao', { dados: ds.name })}
      size="lg"
      resizable
      resizeId="taxburden.sources"
      footer={
        <DialogClose asChild>
          <Button>{t('comum-fechar')}</Button>
        </DialogClose>
      }
    >
      <KeyValue
        items={[
          { label: t('impostos-dados'), value: ds.name },
          { label: t('impostos-versao'), value: ds.version },
          { label: t('impostos-atualizado-em'), value: sourceDate(ds.updated) },
          {
            label: t('impostos-cobertura'),
            value: isDraft(ds.coverage) ? t('impostos-esboco') : ds.coverage,
          },
          { label: t('impostos-fontes'), value: String(ds.sources.length) },
        ]}
      />
      {info.stale ? (
        <Notice tone="warning" title={t('impostos-desatualizados')}>
          {t('impostos-fontes-desatualizados', { data: sourceDate(ds.updated) })}
        </Notice>
      ) : null}
      {isDraft(ds.coverage) ? (
        <Notice tone="warning" title={t('impostos-esboco-maiusc')}>
          {t('impostos-fontes-esboco', { dados: ds.name })}
        </Notice>
      ) : null}
      <Section title={t('impostos-secao-aviso')}>
        <p className="text-[13px] leading-relaxed text-fg">{ds.disclaimer}</p>
      </Section>
      <Section title={t('impostos-metodologia')}>
        <ol className="flex list-decimal flex-col gap-1.5 pl-5 text-[13px] leading-relaxed text-fg">
          {ds.methodology.map((m) => (
            <li key={m}>{m}</li>
          ))}
        </ol>
      </Section>
      <Section title={t('impostos-cenarios')}>
        <ul className="flex flex-col gap-1 text-[13px] leading-relaxed text-fg">
          {SCENARIOS.map((s) => (
            <li key={s.value}>
              <span className="font-medium">{s.label}:</span> {s.hint}
            </li>
          ))}
          <li className="text-dim">{t('impostos-cenarios-nota')}</li>
        </ul>
      </Section>
      {ds.reference_icms != null ? (
        <Section title={t('impostos-ajuste-estado')}>
          <p className="text-[13px] leading-relaxed text-fg">
            {t('impostos-ajuste-estado-texto', { icms: formatRate(ds.reference_icms) })}
          </p>
        </Section>
      ) : null}
      {ds.payroll.length > 0 ? (
        <Section title={t('impostos-renda-trabalho')}>
          <ul className="flex flex-col gap-1 text-[13px] leading-relaxed text-fg">
            {ds.payroll.map((p) => (
              <li key={p.from}>
                <span className="num text-dim">
                  {t('impostos-a-partir-de', { data: sourceDate(p.from) })}
                </span>
                {' · '}
                {p.legal_basis}
              </li>
            ))}
          </ul>
        </Section>
      ) : null}
      {ds.context ? (
        <Section title={t('impostos-contexto')}>
          <p className="text-[13px] leading-relaxed text-fg">
            {t('impostos-contexto-texto', {
              rotulo: ds.context.label,
              valor: formatRate(ds.context.value, 1),
              ano: String(ds.context.year),
            })}
          </p>
        </Section>
      ) : null}
      <Section title={t('impostos-fontes-n', { n: ds.sources.length })}>
        <SourceList sources={ds.sources} />
      </Section>
    </Dialog>
  );
}

/** Botão "Ver fontes e metodologia" com o diálogo. */
export function SourcesButton({
  dataset,
  info,
  size = 'sm',
}: {
  dataset: TaxDataset;
  info: DatasetInfo;
  size?: 'sm' | 'md';
}) {
  return (
    <SourcesDialog
      dataset={dataset}
      info={info}
      trigger={
        <Button size={size} variant="secondary" icon={<Icon name="info" size={13} />}>
          {t('impostos-ver-fontes')}
        </Button>
      }
    />
  );
}
