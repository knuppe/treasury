/**
 * Hooks de `tax.*`: apuração mensal de IR em renda variável, relatório anual para a
 * declaração e DARFs (gerar, marcar pago, cancelar). Chaves `['tax', acao, args]`.
 */
import { keepPreviousData } from '@tanstack/react-query';

import type { AnnualDraft } from '../generated/AnnualDraft';
import type { AnnualInput } from '../generated/AnnualInput';
import type { AnnualResult } from '../generated/AnnualResult';
import type { CapitalGain } from '../generated/CapitalGain';
import type { CarneLeao } from '../generated/CarneLeao';
import type { CarneLeaoArgs } from '../generated/CarneLeaoArgs';
import type { CryptoCustody } from '../generated/CryptoCustody';
import type { Darf } from '../generated/Darf';
import type { LateCharges } from '../generated/LateCharges';
import type { MarkDarfPaidArgs } from '../generated/MarkDarfPaidArgs';
import type { PhysicalGainArgs } from '../generated/PhysicalGainArgs';
import type { SetCryptoCustodyArgs } from '../generated/SetCryptoCustodyArgs';
import type { TaxMonth } from '../generated/TaxMonth';
import type { TaxMonthArgs } from '../generated/TaxMonthArgs';
import type { TaxYear } from '../generated/TaxYear';
import type { TaxYearArgs } from '../generated/TaxYearArgs';
import { useCore, useCoreMutation } from './coreQuery';

/** Tira `member_id` vazio para que "família toda" tenha uma chave só. */
function clean<T extends { member_id?: string }>(args: T): T {
  const { member_id, ...rest } = args;
  return (member_id ? { ...rest, member_id } : rest) as T;
}

export const useTaxMonth = (args: TaxMonthArgs | null) =>
  useCore<TaxMonth>('tax', 'month', args ? clean(args) : {}, {
    enabled: Boolean(args?.month),
    placeholderData: keepPreviousData,
  });

export const useTaxYear = (args: TaxYearArgs) =>
  useCore<TaxYear>('tax', 'year', clean(args), { placeholderData: keepPreviousData });

export const useDarfs = (args: { year?: number; member_id?: string } = {}) =>
  useCore<Darf[]>('tax', 'list_darfs', clean(args));

const AFFECTS = { affects: ['tax'] } as const;

export const useGenerateDarf = () =>
  useCoreMutation<TaxMonthArgs, Darf>('tax.generate_darf', AFFECTS);
export const useMarkDarfPaid = () =>
  useCoreMutation<MarkDarfPaidArgs, Darf>('tax.mark_darf_paid', AFFECTS);
export const useCancelDarf = () =>
  useCoreMutation<{ id: string }, Darf>('tax.cancel_darf', AFFECTS);

/** Onde a cripto de cada conta está custodiada (Brasil: ganho mensal; exterior: Lei 14.754). */
export const useCryptoCustody = () => useCore<CryptoCustody[]>('tax', 'crypto_custody', {});
export const useSetCryptoCustody = () =>
  useCoreMutation<SetCryptoCustodyArgs, CryptoCustody[]>('tax.set_crypto_custody', AFFECTS);

/** Prévia da declaração de ajuste anual (ano-calendário) montada a partir do cofre. */
export const useAnnualDraft = (args: { year: number; member_id?: string }) =>
  useCore<AnnualDraft>('tax', 'annual_draft', clean(args));

/** Os dois modelos para uma entrada ajustada na tela (cálculo puro, sem gravar). */
export const useAnnualCompute = (input: AnnualInput | null) =>
  useCore<AnnualResult>('tax', 'annual_compute', input ? { input } : {}, {
    enabled: input != null,
    placeholderData: keepPreviousData,
  });

/** Multa e juros de um DARF vencido, pagando hoje (ou na data dada). */
export const useDarfCharges = (id: string, paidAt?: string) =>
  useCore<LateCharges>('tax', 'darf_charges', paidAt ? { id, paid_at: paidAt } : { id });

/** Ganho de capital de um bem do 02 · Bens (vendido, ou simulado com os dados da venda). */
export const usePhysicalGain = (args: PhysicalGainArgs | null) =>
  useCore<CapitalGain>('tax', 'physical_gain', args ?? {}, {
    enabled: args != null,
    placeholderData: keepPreviousData,
  });

/** Carnê-leão de um mês (cálculo puro). */
export const useCarneLeao = (args: CarneLeaoArgs | null) =>
  useCore<CarneLeao>('tax', 'carne_leao', args ?? {}, {
    enabled: args != null,
    placeholderData: keepPreviousData,
  });
