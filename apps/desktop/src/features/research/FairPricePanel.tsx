/**
 * Preço justo (`research.fair_price`): Graham (√(22,5 × LPA × VPA)) e Bazin (média dos
 * dividendos por ação ÷ taxa mínima), com todas as premissas visíveis, margem até a
 * cotação e o aviso de que não é recomendação. Taxa e anos do Bazin editáveis aqui (os
 * padrões ficam em Dados da análise).
 */
import { useState } from 'react';

import type { FairPriceMethod } from '@/api/generated/FairPriceMethod';
import type { SubjectArgs } from '@/api/generated/SubjectArgs';
import { useFairPrice, useResearchSettings } from '@/api/hooks/research';
import {
  Button,
  Field,
  Input,
  InstrumentCard,
  KeyValue,
  Measure,
  Select,
  TechTable,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { LoadingBlock, Notice } from '../portfolio/components/States';
import { formatDecimal, formatUnitPrice } from '../portfolio/decimal';
import { safeDate, sourceName } from '../portfolio/labels';
import { SubjectError } from './common';
import { formatUpside } from './format';
import { numberToInput, parseNumberInput } from './query';

const MONEY_LABEL = /^(LPA|VPA|Dividendos|Média)/;

/** Premissa no idioma do app: dinheiro por ação na moeda, contagens com milhar, taxas com vírgula. */
export function formatAssumption(label: string, value: string, currency: string): string {
  const numeric = /^-?\d+(\.\d+)?$/.test(value);
  if (!numeric) return value.replace(/(\d)\.(\d)/g, '$1,$2');
  if (MONEY_LABEL.test(label)) return formatUnitPrice(value, currency, { maxDecimals: 4 });
  return formatDecimal(value, { maxDecimals: 4 });
}

function Upside({ value }: { value: number | null }) {
  if (value == null) return <span className="text-dim">—</span>;
  const up = value >= 0;
  return (
    <span className={cx('num', up ? 'text-positive' : 'text-negative')}>
      {up ? '▲' : '▼'} {formatUpside(value)}
    </span>
  );
}

function MethodCard({
  index,
  m,
  price,
}: {
  index: string;
  m: FairPriceMethod;
  price: string | null;
}) {
  return (
    <InstrumentCard
      index={index}
      title={t('pesquisa-preco-justo-metodo', { metodo: m.label })}
      headingLevel={3}
      footer={[{ label: t('pesquisa-formula'), value: m.formula }]}
    >
      <div className="flex flex-col gap-4">
        <div className="flex flex-wrap items-end gap-8">
          <Measure
            size="md"
            label={t('pesquisa-preco-justo-de', { metodo: m.label })}
            value={m.value ? formatUnitPrice(m.value, m.currency) : '—'}
            changeKey={m.value ?? ''}
          />
          <Measure
            size="md"
            label={t('pesquisa-margem-cotacao')}
            value={<Upside value={m.upside} />}
            changeKey={m.upside ?? 0}
            hint={
              price
                ? t('pesquisa-cotacao-valor', { valor: formatUnitPrice(price, m.currency) })
                : undefined
            }
          />
        </div>
        {m.unavailable_reason ? (
          <Notice tone="warning" title={t('pesquisa-indisponivel')}>
            {m.unavailable_reason}
          </Notice>
        ) : null}
        {m.assumptions.length ? (
          <TechTable
            dense
            label={t('pesquisa-premissas', { metodo: m.label })}
            prefsId={`Premissas · ${m.label}`}
            rows={m.assumptions}
            getRowId={(a) => a.label}
            columns={[
              {
                key: 'label',
                header: t('pesquisa-premissa'),
                cell: (a) => <span className="font-sans">{a.label}</span>,
              },
              {
                key: 'value',
                header: t('pesquisa-valor'),
                numeric: true,
                cell: (a) => formatAssumption(a.label, a.value, m.currency),
              },
              {
                key: 'source',
                header: t('pesquisa-fonte'),
                cell: (a) => (
                  <span className="font-sans text-[12px] text-dim">{a.source ?? '—'}</span>
                ),
              },
            ]}
          />
        ) : null}
      </div>
    </InstrumentCard>
  );
}

export function FairPricePanel({ subject }: { subject: SubjectArgs }) {
  const settings = useResearchSettings();
  const defaults = settings.data?.settings;
  // `null` = usar o padrão dos ajustes.
  const [rateText, setRateText] = useState<string | null>(null);
  const [years, setYears] = useState<number | null>(null);
  const rateInput = rateText ?? (defaults ? numberToInput(defaults.bazin_rate, true) : '6');
  const rate = parseNumberInput(rateInput, true);
  const rateOk = rate != null && rate > 0 && rate <= 1;
  const yearsValue = years ?? defaults?.bazin_years ?? 5;
  const q = useFairPrice(
    settings.isPending
      ? null
      : {
          ...subject,
          ...(rateOk ? { bazin_rate: rate } : {}),
          years: yearsValue,
        },
  );
  const custom = rateText != null || years != null;

  return (
    <div className="flex flex-col gap-4">
      <Notice tone="warning" title={t('pesquisa-nao-recomendacao')}>
        {q.data?.disclaimer ?? t('pesquisa-aviso-estimativas')}
      </Notice>
      {/* Topo alinhado: a dica embaixo da taxa não pode empurrar os vizinhos. */}
      <div className="flex flex-wrap items-start gap-3">
        <Field
          label={t('pesquisa-taxa-minima-bazin')}
          className="w-52"
          error={rateOk ? undefined : t('pesquisa-taxa-invalida')}
          hint={rateText == null ? t('pesquisa-padrao-ajustes') : undefined}
        >
          <Input
            numeric
            value={rateInput}
            onChange={(e) => setRateText(e.target.value)}
            suffix="%"
            inputMode="decimal"
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
        {custom ? (
          <Button
            className="mt-[22px]"
            variant="ghost"
            onClick={() => {
              setRateText(null);
              setYears(null);
            }}
          >
            {t('pesquisa-usar-padrao')}
          </Button>
        ) : null}
      </div>
      {q.isPending ? (
        <LoadingBlock label={t('pesquisa-calculando-preco-justo')} />
      ) : q.isError ? (
        <SubjectError error={q.error} onRetry={() => void q.refetch()} />
      ) : (
        <>
          <KeyValue
            items={[
              {
                label: t('pesquisa-cotacao'),
                value: q.data.price
                  ? t('pesquisa-cotacao-em', {
                      preco: formatUnitPrice(q.data.price.close, q.data.subject.currency),
                      data: safeDate(q.data.price.date),
                      fonte: sourceName(q.data.price.source),
                    })
                  : t('pesquisa-sem-cotacao'),
              },
            ]}
          />
          <div className="grid gap-4 @min-[1040px]:grid-cols-2">
            <MethodCard index="01" m={q.data.graham} price={q.data.price?.close ?? null} />
            <MethodCard index="02" m={q.data.bazin} price={q.data.price?.close ?? null} />
          </div>
        </>
      )}
    </div>
  );
}
