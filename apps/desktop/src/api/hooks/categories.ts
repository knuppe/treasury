/** Hooks de `categories.*`: árvore, uso, criar, alterar, mesclar e apagar. */
import type { AddDefaultsArgs } from '../generated/AddDefaultsArgs';
import type { Category } from '../generated/Category';
import type { CategoryDeleteArgs } from '../generated/CategoryDeleteArgs';
import type { CategoryInput } from '../generated/CategoryInput';
import type { CategoryListArgs } from '../generated/CategoryListArgs';
import type { CategoryMergeArgs } from '../generated/CategoryMergeArgs';
import type { CategoryNode } from '../generated/CategoryNode';
import type { CategoryPatch } from '../generated/CategoryPatch';
import type { CategoryUsage } from '../generated/CategoryUsage';
import type { MissingDefault } from '../generated/MissingDefault';
import type { SetGroupArgs } from '../generated/SetGroupArgs';
import type { SetGroupResult } from '../generated/SetGroupResult';
import { useCore, useCoreMutation } from './coreQuery';

export const useCategoryTree = (args: CategoryListArgs = {}) =>
  useCore<CategoryNode[]>('categories', 'tree', args);

/** Lista plana na ordem da árvore, com `path` (`Alimentação › Mercado`). */
export const useCategories = (args: CategoryListArgs = {}) =>
  useCore<Category[]>('categories', 'list', args);

export const useCategoryUsage = (id: string | null) =>
  useCore<CategoryUsage>('categories', 'usage', { id }, { enabled: Boolean(id) });

export const useCreateCategory = () =>
  useCoreMutation<CategoryInput, Category>('categories.create', { affects: ['categories'] });

export const useUpdateCategory = () =>
  useCoreMutation<CategoryPatch, Category>('categories.update', { affects: ['categories'] });

export const useDeleteCategory = () =>
  useCoreMutation<CategoryDeleteArgs, CategoryUsage>('categories.delete', {
    affects: ['categories'],
  });

/** Categorias do conjunto inicial que faltam no cofre. */
export const useMissingDefaultCategories = () =>
  useCore<MissingDefault[]>('categories', 'missing_defaults', {});

export const useAddDefaultCategories = () =>
  useCoreMutation<AddDefaultsArgs, number>('categories.add_defaults', {
    affects: ['categories'],
  });

export const useMergeCategories = () =>
  useCoreMutation<CategoryMergeArgs, Category>('categories.merge', { affects: ['categories'] });

/** Grupo de gasto de várias categorias de despesa de uma vez (uma escrita, um desfazer). */
export const useSetCategoryGroup = () =>
  useCoreMutation<SetGroupArgs, SetGroupResult>('categories.set_group', {
    affects: ['categories'],
  });
