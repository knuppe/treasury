/**
 * Card "Resultado" (página do ativo e Carteira) com a perspectiva escolhida no título: nominal ou
 * real por IPCA, IGP-M, inflação do país, US CPI ou M2 (docs/real-return.md). O seletor é o
 * mesmo deflator da carteira (`usePortfolioUi`), então troca também o retorno anual real.
 *
 * Nominal: não realizado + realizado + proventos. Real: cada compra, venda e provento corrigido
 * até a data pelo índice; o rodapé mostra o nominal e o que o índice levou.
 */
import type { ReactNode } from 'react';

import type { Deflator } from '@/api/generated/Deflator';
import { InstrumentCard, Measure, Money, Select } from '@/design/components';
import { t } from '@/i18n';

import { DEFLATOR_OPTIONS } from '../labels';
import { usePortfolioUi } from '../state';
import { useDeflatorLabel } from '../useDeflatorLabel';
import { Pct } from './Pct';

/** Seletor compacto da perspectiva, para o título dos cards de resultado. */
export function ResultPerspective() {
  const deflator = usePortfolioUi((s) => s.deflator);
  const setDeflator = usePortfolioUi((s) => s.setDeflator);
  const label = useDeflatorLabel();
  return (
    <Select
      aria-label={t('carteira-resultado-perspectiva')}
      value={deflator}
      onValueChange={(v) => setDeflator(v as Deflator)}
      className="!h-7 !w-auto min-w-[8.5rem] !gap-2 !px-2 !text-[12px]"
      options={DEFLATOR_OPTIONS.map((o) => ({
        value: o.value,
        label: o.value === 'none' ? t('carteira-resultado-nominal') : label(o.value),
      }))}
    />
  );
}

export interface ResultCardProps {
  index: string;
  title: string;
  /** Deflator com que o núcleo calculou (o efetivo, fora do Brasil o IPCA vira o do país). */
  deflator: Deflator;
  /** Nominal: não realizado + realizado + proventos; `null` sem cotação. */
  nominal: number | null;
  /** Real pelo deflator; `null` sem dado do índice. */
  real: number | null;
  /** Nominal − real: o que o índice levou (positivo = perda), como nas Vendas. */
  effect: number | null;
  /** Base da porcentagem (custo). */
  cost: number;
  estimated?: boolean;
  /** Linhas do rodapé (não realizado, realizado, proventos), sempre nominais. */
  parts: readonly { label: string; value: ReactNode }[];
  /**
   * Seletor do índice no título. Na Carteira ele fica de fora: a barra "Resultado real
   * descontado por" logo acima já escolhe o índice, e o seletor apertava o título ("Resul…").
   */
  perspective?: boolean;
}

export function ResultCard({
  index,
  title,
  deflator,
  nominal,
  real,
  effect,
  cost,
  estimated,
  parts,
  perspective = true,
}: ResultCardProps) {
  const label = useDeflatorLabel();
  const isReal = deflator !== 'none';
  const shown = isReal ? real : nominal;
  const pct = shown != null && cost > 0 ? shown / cost : null;
  const footer = isReal
    ? [
        {
          label: t('carteira-resultado-nominal'),
          value: nominal == null ? '—' : <Money cents={nominal} tone="auto" />,
        },
        {
          label: t('carteira-resultado-efeito', { indice: label(deflator) }),
          value: effect == null ? '—' : <Money cents={-effect} tone="auto" signed />,
        },
        ...parts,
      ]
    : parts;
  return (
    <InstrumentCard
      index={index}
      title={title}
      actions={perspective ? <ResultPerspective /> : undefined}
      footer={footer}
    >
      <Measure
        size="md"
        label={
          isReal
            ? t('carteira-resultado-real-de', { indice: label(deflator) })
            : t('carteira-resultado-total')
        }
        value={shown == null ? '—' : <Money cents={shown} tone="auto" estimated={estimated} />}
        changeKey={`${deflator}:${shown ?? 0}`}
        hint={
          isReal && real == null && nominal != null ? (
            t('carteira-resultado-sem-indice', { indice: label(deflator) })
          ) : pct != null ? (
            <Pct value={pct} signed tone="auto" />
          ) : undefined
        }
      />
    </InstrumentCard>
  );
}
