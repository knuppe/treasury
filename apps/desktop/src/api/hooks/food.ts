/**
 * Hooks de `food.*` (docs/consumption.md, docs/consumption.md): tela 06 · Consumo, compras (e a
 * compra cadastrada à mão) e categorias de produto.
 */
import type { BasketArgs } from '../generated/BasketArgs';
import type { BasketInflation } from '../generated/BasketInflation';
import type { FoodProduct } from '../generated/FoodProduct';
import type { FoodProductHistory } from '../generated/FoodProductHistory';
import type { FoodProductPatch } from '../generated/FoodProductPatch';
import type { FoodProductsArgs } from '../generated/FoodProductsArgs';
import type { FoodSummary } from '../generated/FoodSummary';
import type { FoodSummaryArgs } from '../generated/FoodSummaryArgs';
import type { FoodMergeSuggestion } from '../generated/FoodMergeSuggestion';
import type { HealthSummary } from '../generated/HealthSummary';
import type { OpenDataSync } from '../generated/OpenDataSync';
import type { ProductCategory } from '../generated/ProductCategory';
import type { ProductCategoryInput } from '../generated/ProductCategoryInput';
import type { ProductCategoryPatch } from '../generated/ProductCategoryPatch';
import type { Purchase } from '../generated/Purchase';
import type { PurchaseInput } from '../generated/PurchaseInput';
import type { PurchaseMatchArgs } from '../generated/PurchaseMatchArgs';
import type { PurchasesArgs } from '../generated/PurchasesArgs';
import type { Receipt } from '../generated/Receipt';
import type { TxnMatch } from '../generated/TxnMatch';
import { useCore, useCoreMutation } from './coreQuery';

export const useFoodSummary = (args: FoodSummaryArgs) =>
  useCore<FoodSummary>('food', 'summary', args);

export const useFoodProducts = (args: FoodProductsArgs) =>
  useCore<FoodProduct[]>('food', 'products', args);

export const useProductHistory = (id: string | null) =>
  useCore<FoodProductHistory>('food', 'product_history', { id }, { enabled: Boolean(id) });

export const useProductCategories = (enabled = true) =>
  useCore<ProductCategory[]>('food', 'categories', {}, { enabled });

export const useRelinkProducts = () =>
  useCoreMutation<Record<string, never>, { linked: number }>('food.relink', { affects: ['food'] });

export const useUpdateProduct = () =>
  useCoreMutation<FoodProductPatch, FoodProduct>('food.product_update', { affects: ['food'] });

export const useMergeProducts = () =>
  useCoreMutation<{ from_id: string; into_id: string }, FoodProduct>('food.product_merge', {
    affects: ['food'],
  });

export const useCreateProductCategory = () =>
  useCoreMutation<ProductCategoryInput, ProductCategory>('food.category_create', {
    affects: ['food'],
  });

export const useUpdateProductCategory = () =>
  useCoreMutation<ProductCategoryPatch, ProductCategory[]>('food.category_update', {
    affects: ['food'],
  });

export const useDeleteProductCategory = () =>
  useCoreMutation<{ id: string }, ProductCategory[]>('food.category_delete', {
    affects: ['food'],
  });

export const usePurchases = (args: PurchasesArgs) => useCore<Purchase[]>('food', 'purchases', args);

/** Despesas com o mesmo valor perto da data, para ligar à compra. */
export const usePurchaseCandidates = (args: PurchaseMatchArgs, enabled: boolean) =>
  useCore<TxnMatch[]>('food', 'purchase_candidates', args, { enabled });

export const useStores = () => useCore<string[]>('food', 'stores', {});

/** Grava a compra cadastrada à mão (pode criar ou ajustar o lançamento). */
export const useSavePurchase = () =>
  useCoreMutation<PurchaseInput, Receipt>('food.purchase_save', {
    affects: ['ledger', 'receipts'],
  });

export const useDeletePurchase = () =>
  useCoreMutation<{ id: string }, null>('food.purchase_delete', {
    affects: ['ledger', 'receipts'],
  });

/** Índice da sua cesta × índice oficial de alimentos (docs/gtin-inflation.md §5). */
export const useBasketInflation = (args: BasketArgs = {}) =>
  useCore<BasketInflation>('food', 'basket_inflation', args);

/** Saúde da cesta (Open Food Facts): NOVA, "alto em", proteína por real e alergias. */
export const useHealthSummary = () => useCore<HealthSummary>('food', 'health_summary', {});

/** Produtos com nomes parecidos que talvez sejam o mesmo (para juntar). */
export const useMergeSuggestions = () =>
  useCore<FoodMergeSuggestion[]>('food', 'merge_suggestions', {});

/** Busca no Open Food Facts os produtos ainda sem consulta (rede; só o código de barras sai). */
export const useOpenDataSync = () =>
  useCoreMutation<Record<string, never>, OpenDataSync>('food.open_data_sync', {
    affects: ['food'],
  });

/** Código de barras digitado ou lido da foto (o núcleo confere e devolve o produto). */
export const useAddProductGtin = () =>
  useCoreMutation<{ product_id: string; code: string }, FoodProduct>('food.product_gtin_add', {
    affects: ['food'],
  });

export const useRemoveProductGtin = () =>
  useCoreMutation<{ product_id: string; code: string }, null>('food.product_gtin_remove', {
    affects: ['food'],
  });
