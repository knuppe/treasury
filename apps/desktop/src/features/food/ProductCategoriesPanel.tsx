/**
 * Categorias de produto em Ajustes › Categorias, aba "Produtos" (docs/consumption.md,
 * decisão Q4): a árvore da semente (os grupos do cupom e os subitens do IPCA) editável, com a
 * classe de cada nó (essencial, complementar, supérfluo ou herdada do pai) e se é alimento.
 */
import { useState } from 'react';

import type { FoodClass } from '@/api/generated/FoodClass';
import type { ProductCategory } from '@/api/generated/ProductCategory';
import {
  useCreateProductCategory,
  useDeleteProductCategory,
  useProductCategories,
  useUpdateProductCategory,
} from '@/api/hooks/food';
import { Button, Icon, Input, Select } from '@/design/components';
import { Checkbox, QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

import { classLabel } from './FoodPanels';

const INHERIT = '__herdar__';

export function ProductCategoriesPanel() {
  const q = useProductCategories();
  const create = useCreateProductCategory();
  const [adding, setAdding] = useState<{ parent: string | null; name: string } | null>(null);
  const list = q.data ?? [];
  const save = () => {
    if (!adding?.name.trim()) return;
    create.mutate(
      { name: adding.name, ...(adding.parent ? { parent_id: adding.parent } : {}) },
      { onSuccess: () => setAdding(null) },
    );
  };
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[12.5px] text-dim">{t('alimentacao-cat-explicacao')}</p>
      <div>
        <Button
          size="sm"
          variant="secondary"
          icon={<Icon name="plus" size={12} />}
          onClick={() => setAdding({ parent: null, name: '' })}
        >
          {t('alimentacao-cat-nova')}
        </Button>
      </div>
      {adding ? (
        <div className="flex flex-wrap items-center gap-2">
          <Input
            aria-label={t('alimentacao-cat-nome')}
            autoFocus
            value={adding.name}
            onChange={(e) => setAdding({ ...adding, name: e.target.value })}
            onKeyDown={(e) => e.key === 'Enter' && save()}
          />
          <Button size="sm" loading={create.isPending} onClick={save}>
            {t('alimentacao-cat-criar')}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => setAdding(null)}>
            {t('comum-cancelar')}
          </Button>
        </div>
      ) : null}
      <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
        <ul className="flex flex-col divide-y divide-line" aria-label={t('alimentacao-cat-lista')}>
          {list.map((c) => (
            <CategoryRow key={c.id} c={c} onAdd={() => setAdding({ parent: c.id, name: '' })} />
          ))}
        </ul>
      </QueryState>
    </div>
  );
}

function CategoryRow({ c, onAdd }: { c: ProductCategory; onAdd: () => void }) {
  const update = useUpdateProductCategory();
  const remove = useDeleteProductCategory();
  const [name, setName] = useState<string | null>(null);
  return (
    <li
      className="grid items-center gap-2 py-1.5 text-[13px] sm:grid-cols-[1fr_190px_110px_auto]"
      style={{ paddingLeft: `${c.depth * 18}px` }}
    >
      {name != null ? (
        <Input
          aria-label={t('alimentacao-cat-nome')}
          autoFocus
          value={name}
          onChange={(e) => setName(e.target.value)}
          onBlur={() => {
            if (name.trim() && name !== c.name) update.mutate({ id: c.id, name });
            setName(null);
          }}
          onKeyDown={(e) => e.key === 'Enter' && (e.target as HTMLInputElement).blur()}
        />
      ) : (
        <button
          type="button"
          className="text-left hover:text-accent"
          title={t('alimentacao-cat-renomear')}
          onClick={() => setName(c.name)}
        >
          {c.name}
          {c.products ? <span className="ml-2 text-dim">({c.products})</span> : null}
        </button>
      )}
      <Select
        aria-label={t('alimentacao-cat-classe-de', { nome: c.name })}
        value={c.class ?? INHERIT}
        onValueChange={(v) =>
          update.mutate({ id: c.id, class: v === INHERIT ? null : (v as FoodClass) })
        }
        options={[
          {
            value: INHERIT,
            label: c.parent_id
              ? t('alimentacao-cat-herdada', { classe: classLabel(c.effective_class) })
              : t('alimentacao-classe-sem'),
          },
          ...(['essential', 'complementary', 'superfluous'] as const).map((k) => ({
            value: k,
            label: classLabel(k),
          })),
        ]}
      />
      <Checkbox
        checked={c.is_food}
        onChange={(v) => update.mutate({ id: c.id, is_food: v })}
        label={t('alimentacao-cat-alimento')}
      />
      <span className="flex gap-1">
        <Button size="sm" variant="ghost" onClick={onAdd} title={t('alimentacao-cat-sub')}>
          <Icon name="plus" size={12} />
        </Button>
        <Button
          size="sm"
          variant="ghost"
          title={t('alimentacao-cat-apagar', { nome: c.name })}
          aria-label={t('alimentacao-cat-apagar', { nome: c.name })}
          onClick={() => remove.mutate({ id: c.id })}
        >
          <Icon name="x" size={12} />
        </Button>
      </span>
    </li>
  );
}
