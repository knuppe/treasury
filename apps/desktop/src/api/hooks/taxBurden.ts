/**
 * Hooks da estimativa de impostos pagos (`tax_burden.*`, docs/taxes.md). Chaves
 * `['tax_burden', acao, args]`.
 *
 * A estimativa depende dos lançamentos, das categorias, dos bens, dos proventos e dos
 * titulares: o grupo `tax_burden` entra nos grupos de invalidação dessas escritas
 * (`AFFECTS` em `coreQuery.ts`). As escritas daqui (país e estado, grupo e alíquota das
 * categorias, informes de rendimentos) invalidam só o próprio módulo.
 */
import { keepPreviousData } from '@tanstack/react-query';

import type { City } from '../generated/City';
import type { CityState } from '../generated/CityState';
import type { IncomeReport } from '../generated/IncomeReport';
import type { IncomeReportInput } from '../generated/IncomeReportInput';
import type { TaxCategoryPatch } from '../generated/TaxCategoryPatch';
import type { TaxCategoryView } from '../generated/TaxCategoryView';
import type { TaxDataset } from '../generated/TaxDataset';
import type { TaxEstimate } from '../generated/TaxEstimate';
import type { TaxSettings } from '../generated/TaxSettings';
import type { TaxSettingsPatch } from '../generated/TaxSettingsPatch';
import { useCore, useCoreMutation } from './coreQuery';

/** Dados de referência de todos os países (fontes, alíquotas, estados, regras). */
export const useTaxDatasets = () =>
  useCore<TaxDataset[]>('tax_burden', 'datasets', {}, { staleTime: Infinity });

/** Dados de um país (`BR`, `US`, `EU`). */
export const useTaxDataset = (country: string | null | undefined) =>
  useCore<TaxDataset>(
    'tax_burden',
    'dataset',
    { country: country ?? '' },
    { enabled: Boolean(country), staleTime: Infinity },
  );

export const useTaxSettings = () => useCore<TaxSettings>('tax_burden', 'settings');

/** As três estimativas (conservador, provável, elevado) do ano; padrão: o atual, até hoje. */
export const useTaxEstimate = (year: number | null) =>
  useCore<TaxEstimate>('tax_burden', 'estimate', year == null ? {} : { year }, {
    placeholderData: keepPreviousData,
  });

/** Categorias de despesa com o grupo de imposto que vale e a alíquota própria. */
export const useTaxCategories = (enabled = true) =>
  useCore<TaxCategoryView[]>('tax_burden', 'categories', {}, { enabled });

/** Informes de rendimentos (de um ano, ou todos). */
export const useIncomeReports = (year?: number) =>
  useCore<IncomeReport[]>('tax_burden', 'income_reports', year == null ? {} : { year });

/** País, estado e dependentes (só pela interface). */
export const useSetTaxSettings = () =>
  useCoreMutation<TaxSettingsPatch, TaxSettings>('tax_burden.set_settings', {
    affects: ['tax_burden'],
  });

/** Grupo e alíquota própria de uma categoria (`null` volta ao padrão). */
export const useSetTaxCategory = () =>
  useCoreMutation<TaxCategoryPatch, TaxCategoryView[]>('tax_burden.set_category', {
    affects: ['tax_burden'],
  });

export const useSaveIncomeReport = () =>
  useCoreMutation<IncomeReportInput, IncomeReport[]>('tax_burden.save_income_report', {
    affects: ['tax_burden'],
  });

export const useDeleteIncomeReport = () =>
  useCoreMutation<{ id: string }, null>('tax_burden.delete_income_report', {
    affects: ['tax_burden'],
  });

/** A cidade da família e a arrecadação guardada dela (só em cofre do Brasil). */
export const useTaxCity = (enabled = true) =>
  useCore<CityState>('tax_burden', 'city', {}, { enabled });

/** Municípios de uma UF (lista do IBGE). */
export const useTaxCities = (uf: string | null) =>
  useCore<City[]>(
    'tax_burden',
    'cities',
    { uf: uf ?? '' },
    { enabled: Boolean(uf), staleTime: Infinity },
  );

/** Escolhe (ou limpa) a cidade da família (só pela interface). */
export const useSetTaxCity = () =>
  useCoreMutation<{ city: City | null }, CityState>('tax_burden.set_city', {
    affects: ['tax_burden'],
  });

/** Busca no Tesouro Nacional a arrecadação da cidade escolhida e guarda o resultado. */
export const useRefreshTaxCity = () =>
  useCoreMutation<Record<string, never>, CityState>('tax_burden.refresh_city', {
    affects: ['tax_burden'],
  });
